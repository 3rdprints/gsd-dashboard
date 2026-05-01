use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, RwLock},
    time::Duration,
};

use notify_debouncer_full::{
    new_debouncer,
    notify::{RecommendedWatcher, RecursiveMode},
    DebounceEventResult, Debouncer, RecommendedCache,
};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Runtime};
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
    let mut debouncer = new_debouncer(
        Duration::from_millis(PROJECT_DEBOUNCE_MS),
        None,
        move |result: DebounceEventResult| {
            let _ = event_sender.blocking_send(result);
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

    let changed = config.changed || state.watcher_runtime.set_roots(statuses);
    let event_task = tokio::spawn(process_watcher_events(
        app.clone(),
        state.clone(),
        config.home_dir.clone(),
        native_roots,
        event_receiver,
    ));
    let polling_task = tokio::spawn(poll_degraded_roots(
        app,
        state.clone(),
        config.home_dir,
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
    changed: bool,
}

async fn configure_watcher_roots(state: &AppState) -> Result<WatcherConfig, AppError> {
    let settings = settings::load_or_initialize(&state.pool, &state.home_dir).await?;
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
            retry_enabled: true,
        }
    }
}

async fn process_watcher_events<R: Runtime>(
    app: AppHandle<R>,
    state: AppState,
    home_dir: PathBuf,
    roots: Vec<PathBuf>,
    mut events: mpsc::Receiver<DebounceEventResult>,
) {
    while let Some(result) = events.recv().await {
        match result {
            Ok(events) => {
                for event in events {
                    for path in &event.paths {
                        refresh_changed_path(&app, &state, &home_dir, &roots, path).await;
                    }
                }
            }
            Err(errors) => {
                for error in errors {
                    let category = WatcherReasonCategory::from_error_message(&error.to_string());
                    for path in error.paths {
                        if let Some(root) = roots.iter().find(|root| path.starts_with(root)) {
                            let changed =
                                state
                                    .watcher_runtime
                                    .set_root_status(WatcherRootStatus::polling(
                                        root.display().to_string(),
                                        category,
                                    ));
                            if changed {
                                emit_app_event(&app, AppEvent::WatcherStatusChanged);
                            }
                        }
                    }
                }
            }
        }
    }
}

async fn poll_degraded_roots<R: Runtime>(
    app: AppHandle<R>,
    state: AppState,
    home_dir: PathBuf,
    roots: Vec<PathBuf>,
) {
    let mut interval = tokio::time::interval(Duration::from_secs(POLLING_INTERVAL_SECONDS));
    loop {
        interval.tick().await;
        for root in &roots {
            refresh_root(&app, &state, &home_dir, root).await;
        }
    }
}

async fn refresh_changed_path<R: Runtime>(
    app: &AppHandle<R>,
    state: &AppState,
    home_dir: &Path,
    roots: &[PathBuf],
    path: &Path,
) {
    if let Some(root) = roots.iter().find(|root| path.starts_with(root)) {
        refresh_root(app, state, home_dir, root).await;
    }
}

async fn refresh_root<R: Runtime>(
    app: &AppHandle<R>,
    state: &AppState,
    home_dir: &Path,
    root: &Path,
) {
    if root.file_name().and_then(|name| name.to_str()) == Some(".planning") {
        let _ =
            crate::watcher::refresh::refresh_project_planning_dir_for_app(state, root, |event| {
                emit_app_event(app, event);
                Ok(())
            })
            .await;
        return;
    }

    if let Some(source) = session_source_for_root(home_dir, root) {
        for source_path in collect_jsonl_files(root).await {
            let _ = refresh_session_file_for_app(state, source, &source_path, |event| {
                emit_app_event(app, event);
                Ok(())
            })
            .await;
        }
    }
}

fn session_source_for_root(home_dir: &Path, root: &Path) -> Option<SessionSource> {
    if root == home_dir.join(".claude/projects") {
        Some(SessionSource::Claude)
    } else if root == home_dir.join(".codex/sessions") {
        Some(SessionSource::Codex)
    } else {
        None
    }
}

async fn collect_jsonl_files(root: &Path) -> Vec<PathBuf> {
    let root = root.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let mut files = Vec::new();
        collect_jsonl_files_sync(&root, &mut files);
        files
    })
    .await
    .unwrap_or_default()
}

fn collect_jsonl_files_sync(root: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_jsonl_files_sync(&path, files);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("jsonl") {
            files.push(path);
        }
    }
}

fn emit_app_event<R: Runtime>(app: &AppHandle<R>, event: AppEvent) {
    match event {
        AppEvent::BootReady { cache_path } => {
            let _ = app.emit("boot-ready", serde_json::json!({ "cachePath": cache_path }));
        }
        AppEvent::SettingsChanged => {
            let _ = app.emit("settings-changed", ());
        }
        AppEvent::DailyActivityUpdated => {
            let _ = app.emit("daily_activity_updated", ());
        }
        AppEvent::ProjectUpdated { id } => {
            let _ = app.emit("project:updated", serde_json::json!({ "id": id }));
        }
        AppEvent::SessionNew { id, project_id } => {
            let _ = app.emit(
                "session:new",
                serde_json::json!({ "id": id, "projectId": project_id }),
            );
        }
        AppEvent::WatcherStatusChanged => {
            let _ = app.emit("watcher:status-changed", ());
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
                "Check folder permissions, then leave Settings open for the next retry."
            }
            Self::WatchLimit => "Increase inotify watch limits, then wait for automatic retry.",
            Self::Filesystem => "Move the project to a local folder or keep polling enabled.",
            Self::Unknown => "No action needed unless updates feel stale.",
        }
    }
}
