use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, RwLock},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use notify_debouncer_full::{
    new_debouncer,
    notify::{RecommendedWatcher, RecursiveMode},
    DebounceEventResult, Debouncer, RecommendedCache,
};
use serde::Serialize;
use tauri::{AppHandle, Runtime};
use tokio::{sync::mpsc, task::JoinHandle};

use crate::{
    app_state::AppState, error::AppError, events::AppEvent, sessions::SessionSource, settings,
};

pub const PROJECT_DEBOUNCE_MS: u64 = 500;
pub const POLLING_INTERVAL_SECONDS: u64 = 60;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WatcherStatus {
    pub roots: Vec<WatcherRootStatus>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WatcherRootStatus {
    pub root: String,
    pub mode: WatcherMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason_category: Option<WatcherReasonCategory>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix_hint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub polling_interval_seconds: Option<u64>,
    pub retry_enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WatcherMode {
    Native,
    Polling,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WatcherReasonCategory {
    Permission,
    WatchLimit,
    Filesystem,
    Unknown,
}

#[derive(Clone)]
pub struct WatcherRuntime {
    status: Arc<RwLock<WatcherStatus>>,
    supervisor: Arc<Mutex<Option<WatcherSupervisor>>>,
}

struct WatcherSupervisor {
    debouncer: Option<Debouncer<RecommendedWatcher, RecommendedCache>>,
    event_task: JoinHandle<()>,
    polling_task: JoinHandle<()>,
}

#[derive(Debug, Clone)]
pub struct ProjectDebouncer {
    debounce_ms: u64,
    pending: BTreeMap<PathBuf, u64>,
}

#[derive(Debug, Clone)]
pub struct SessionFileDebouncer {
    debounce_ms: u64,
    pending: BTreeMap<PathBuf, PendingSessionFile>,
}

#[derive(Debug, Clone)]
pub struct PendingSessionFile {
    pub source: SessionSource,
    pub last_event_ms: u64,
}

#[derive(Debug)]
pub enum WatcherDebounceEvent {
    Project(PathBuf),
    SessionFile {
        source: SessionSource,
        path: PathBuf,
    },
    Errors(Vec<notify_debouncer_full::notify::Error>),
}

impl Default for WatcherRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl WatcherRuntime {
    pub fn new() -> Self {
        Self {
            status: Arc::new(RwLock::new(WatcherStatus { roots: Vec::new() })),
            supervisor: Arc::new(Mutex::new(None)),
        }
    }

    pub fn status(&self) -> WatcherStatus {
        self.status
            .read()
            .expect("watcher status lock should not be poisoned")
            .clone()
    }

    pub fn set_roots(&self, roots: Vec<WatcherRootStatus>) -> bool {
        let mut status = self
            .status
            .write()
            .expect("watcher status lock should not be poisoned");
        if status.roots == roots {
            return false;
        }

        status.roots = roots;
        true
    }

    pub fn set_root_status(&self, root_status: WatcherRootStatus) -> bool {
        let mut next_roots = self.status().roots;
        if let Some(existing) = next_roots
            .iter_mut()
            .find(|existing| existing.root == root_status.root)
        {
            if *existing == root_status {
                return false;
            }
            *existing = root_status;
        } else {
            next_roots.push(root_status);
        }

        self.set_roots(next_roots)
    }

    pub fn is_running(&self) -> bool {
        self.supervisor
            .lock()
            .expect("watcher supervisor lock should not be poisoned")
            .is_some()
    }

    fn replace_supervisor(&self, supervisor: WatcherSupervisor) {
        let previous = self
            .supervisor
            .lock()
            .expect("watcher supervisor lock should not be poisoned")
            .replace(supervisor);
        drop(previous);
    }
}

impl Drop for WatcherSupervisor {
    fn drop(&mut self) {
        self.event_task.abort();
        self.polling_task.abort();
        if let Some(debouncer) = self.debouncer.take() {
            debouncer.stop_nonblocking();
        }
    }
}

impl ProjectDebouncer {
    pub fn new(debounce_ms: u64) -> Self {
        Self {
            debounce_ms,
            pending: BTreeMap::new(),
        }
    }

    pub fn record_event(
        &mut self,
        planning_root: &Path,
        _event_path: impl AsRef<Path>,
        now_ms: u64,
    ) {
        self.pending.insert(planning_root.to_path_buf(), now_ms);
    }

    pub fn take_due(&mut self, now_ms: u64) -> Vec<PathBuf> {
        let due_roots = self
            .pending
            .iter()
            .filter_map(|(root, last_event_ms)| {
                last_event_ms
                    .saturating_add(self.debounce_ms)
                    .le(&now_ms)
                    .then(|| root.clone())
            })
            .collect::<Vec<_>>();

        for root in &due_roots {
            self.pending.remove(root);
        }

        due_roots
    }
}

impl SessionFileDebouncer {
    pub fn new(debounce_ms: u64) -> Self {
        Self {
            debounce_ms,
            pending: BTreeMap::new(),
        }
    }

    pub fn record_event(
        &mut self,
        source: SessionSource,
        source_root: &Path,
        event_path: impl AsRef<Path>,
        now_ms: u64,
    ) {
        let event_path = event_path.as_ref();
        if !event_path.starts_with(source_root)
            || event_path
                .extension()
                .and_then(|extension| extension.to_str())
                != Some("jsonl")
        {
            return;
        }

        self.pending.insert(
            event_path.to_path_buf(),
            PendingSessionFile {
                source,
                last_event_ms: now_ms,
            },
        );
    }

    pub fn take_due(&mut self, now_ms: u64) -> Vec<(SessionSource, PathBuf)> {
        let due_files = self
            .pending
            .iter()
            .filter_map(|(path, pending)| {
                pending
                    .last_event_ms
                    .saturating_add(self.debounce_ms)
                    .le(&now_ms)
                    .then(|| (pending.source, path.clone()))
            })
            .collect::<Vec<_>>();

        for (_, path) in &due_files {
            self.pending.remove(path);
        }

        due_files
    }
}

pub async fn refresh_session_file_for_app(
    state: &AppState,
    source: SessionSource,
    source_path: &Path,
    emit_event: impl Fn(AppEvent) -> Result<(), AppError>,
) -> Result<(), AppError> {
    crate::watcher::refresh::refresh_session_file(state, source, source_path, emit_event)
        .await
        .map(|_| ())
}

pub async fn start_watcher_service(state: &AppState) -> Result<bool, AppError> {
    let changed = configure_watcher_roots(state).await?.changed;
    Ok(changed)
}

pub async fn start_watcher_service_for_app<R: Runtime>(
    app: AppHandle<R>,
    state: &AppState,
) -> Result<bool, AppError> {
    let state = state.clone();
    let config = configure_watcher_roots(&state).await?;
    let (event_sender, event_receiver) = mpsc::channel(128);
    let debounce_roots = config.roots.clone();
    let project_debouncer = Arc::new(Mutex::new(ProjectDebouncer::new(PROJECT_DEBOUNCE_MS)));
    let session_debouncer = Arc::new(Mutex::new(SessionFileDebouncer::new(PROJECT_DEBOUNCE_MS)));
    let mut debouncer = new_debouncer(
        Duration::from_millis(PROJECT_DEBOUNCE_MS),
        None,
        move |result: DebounceEventResult| {
            let now_ms = current_unix_ms();
            match result {
                Ok(events) => {
                    for event in events {
                        for path in &event.paths {
                            let Some(root) =
                                debounce_roots.iter().find(|root| path.starts_with(root))
                            else {
                                continue;
                            };
                            if root.file_name().and_then(|name| name.to_str()) == Some(".planning")
                            {
                                project_debouncer
                                    .lock()
                                    .expect("project debouncer lock should not be poisoned")
                                    .record_event(root, &path, now_ms);
                            } else if let Some(source) = session_source_for_root(root) {
                                session_debouncer
                                    .lock()
                                    .expect("session debouncer lock should not be poisoned")
                                    .record_event(source, root, &path, now_ms);
                            }
                        }
                    }

                    let due_ms = now_ms.saturating_add(PROJECT_DEBOUNCE_MS);
                    for root in project_debouncer
                        .lock()
                        .expect("project debouncer lock should not be poisoned")
                        .take_due(due_ms)
                    {
                        let _ = event_sender.blocking_send(WatcherDebounceEvent::Project(root));
                    }
                    for (source, path) in session_debouncer
                        .lock()
                        .expect("session debouncer lock should not be poisoned")
                        .take_due(due_ms)
                    {
                        let _ = event_sender
                            .blocking_send(WatcherDebounceEvent::SessionFile { source, path });
                    }
                }
                Err(errors) => {
                    let _ = event_sender.blocking_send(WatcherDebounceEvent::Errors(errors));
                }
            }
        },
    )
    .map_err(AppError::io)?;

    let mut statuses = Vec::with_capacity(config.roots.len());
    let mut native_roots = Vec::new();
    let mut polling_roots = Vec::new();

    for root in config.roots {
        match debouncer.watch(&root, RecursiveMode::Recursive) {
            Ok(()) => {
                statuses.push(WatcherRootStatus::native(root.display().to_string()));
                native_roots.push(root);
            }
            Err(error) => {
                let category = WatcherReasonCategory::from_error_message(&error.to_string());
                statuses.push(WatcherRootStatus::polling(
                    root.display().to_string(),
                    category,
                ));
                polling_roots.push(root);
            }
        }
    }

    let status_changed = state.watcher_runtime.set_roots(statuses);
    let changed = config.changed || status_changed;
    let polling_roots = Arc::new(RwLock::new(polling_roots));
    let event_task = tokio::spawn(crate::watcher::runtime::process_watcher_events(
        app.clone(),
        state.clone(),
        config.home_dir.clone(),
        native_roots,
        Arc::clone(&polling_roots),
        event_receiver,
    ));
    let polling_task = tokio::spawn(crate::watcher::runtime::poll_degraded_roots(
        app,
        state.clone(),
        config.home_dir,
        config.scan_roots,
        polling_roots,
    ));

    state.watcher_runtime.replace_supervisor(WatcherSupervisor {
        debouncer: Some(debouncer),
        event_task,
        polling_task,
    });

    Ok(changed)
}

struct WatcherConfig {
    home_dir: PathBuf,
    roots: Vec<PathBuf>,
    scan_roots: Vec<PathBuf>,
    changed: bool,
}

async fn configure_watcher_roots(state: &AppState) -> Result<WatcherConfig, AppError> {
    let settings = settings::load_or_initialize(&state.pool, &state.home_dir).await?;
    let scan_roots = crate::watcher::derive_polling_scan_roots(&state.home_dir, &settings)?;
    let roots =
        crate::watcher::derive_watcher_roots(&state.pool, &state.home_dir, &settings).await?;
    let statuses = roots
        .iter()
        .map(|root| WatcherRootStatus::native(root.display().to_string()))
        .collect::<Vec<_>>();
    let changed = state.watcher_runtime.set_roots(statuses);

    Ok(WatcherConfig {
        home_dir: state.home_dir.clone(),
        roots,
        scan_roots,
        changed,
    })
}

impl WatcherRootStatus {
    pub fn native(root: String) -> Self {
        Self {
            root,
            mode: WatcherMode::Native,
            reason_category: None,
            reason: None,
            fix_hint: None,
            polling_interval_seconds: None,
            retry_enabled: false,
        }
    }

    pub fn polling(root: String, reason_category: WatcherReasonCategory) -> Self {
        Self {
            root,
            mode: WatcherMode::Polling,
            reason_category: Some(reason_category),
            reason: Some(reason_category.reason().to_string()),
            fix_hint: Some(reason_category.fix_hint().to_string()),
            polling_interval_seconds: Some(POLLING_INTERVAL_SECONDS),
            retry_enabled: false,
        }
    }
}

impl WatcherReasonCategory {
    pub fn from_error_message(message: &str) -> Self {
        let normalized = message.to_ascii_lowercase();
        if normalized.contains("permission") || normalized.contains("denied") {
            Self::Permission
        } else if normalized.contains("inotify")
            || normalized.contains("watch limit")
            || normalized.contains("too many open files")
            || normalized.contains("no space left on device")
            || normalized.contains("enospc")
        {
            Self::WatchLimit
        } else if normalized.contains("filesystem") || normalized.contains("not supported") {
            Self::Filesystem
        } else {
            Self::Unknown
        }
    }

    pub fn reason(self) -> &'static str {
        match self {
            Self::Permission => "Permission denied",
            Self::WatchLimit => "System watch limit reached",
            Self::Filesystem => "Filesystem does not support native watching",
            Self::Unknown => "Native watcher unavailable",
        }
    }

    pub fn fix_hint(self) -> &'static str {
        match self {
            Self::Permission => {
                "Check folder permissions, then restart the app to restore native watching."
            }
            Self::WatchLimit => "Increase inotify watch limits, then restart the app.",
            Self::Filesystem => "Move the project to a local folder or keep polling enabled.",
            Self::Unknown => "No action needed unless updates feel stale.",
        }
    }
}

fn session_source_for_root(root: &Path) -> Option<SessionSource> {
    let root = root.to_string_lossy();
    if root.contains(".claude/projects") {
        Some(SessionSource::Claude)
    } else if root.contains(".codex/sessions") {
        Some(SessionSource::Codex)
    } else {
        None
    }
}

fn current_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
