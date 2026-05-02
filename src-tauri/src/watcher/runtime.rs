use std::{
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
    time::Duration,
};

use tauri::{AppHandle, Emitter, Runtime};
use tokio::sync::mpsc;

use crate::{
    app_state::AppState,
    events::AppEvent,
    sessions::SessionSource,
    store::project_repo,
    tray::service::request_tray_refresh,
    watcher::service::WatcherDebounceEvent,
    watcher::{
        refresh_session_file_for_app, WatcherReasonCategory, WatcherRootStatus,
        POLLING_INTERVAL_SECONDS,
    },
};

pub async fn process_watcher_events<R: Runtime>(
    app: AppHandle<R>,
    state: AppState,
    home_dir: PathBuf,
    roots: Vec<PathBuf>,
    polling_roots: Arc<RwLock<Vec<PathBuf>>>,
    mut events: mpsc::Receiver<WatcherDebounceEvent>,
) {
    while let Some(event) = events.recv().await {
        match event {
            WatcherDebounceEvent::Project(root) => {
                refresh_root(&app, &state, &home_dir, &root).await;
            }
            WatcherDebounceEvent::SessionFile { source, path } => {
                refresh_session_file(&app, &state, source, &path).await;
            }
            WatcherDebounceEvent::Errors(errors) => {
                for error in errors {
                    let category = WatcherReasonCategory::from_error_message(&error.to_string());
                    for path in error.paths {
                        if let Some(root) = roots.iter().find(|root| path.starts_with(root)) {
                            add_polling_root(&polling_roots, root);
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

pub async fn poll_degraded_roots<R: Runtime>(
    app: AppHandle<R>,
    state: AppState,
    home_dir: PathBuf,
    scan_roots: Vec<PathBuf>,
    polling_roots: Arc<RwLock<Vec<PathBuf>>>,
) {
    let mut interval = tokio::time::interval(Duration::from_secs(POLLING_INTERVAL_SECONDS));
    loop {
        interval.tick().await;
        let roots = polling_roots
            .read()
            .expect("polling roots lock should not be poisoned")
            .clone();
        for root in &roots {
            refresh_root(&app, &state, &home_dir, root).await;
        }
        poll_scan_roots_once_for_app(&app, &state, &home_dir, &scan_roots).await;
    }
}

pub async fn poll_scan_roots_once_for_app<R: Runtime>(
    app: &AppHandle<R>,
    state: &AppState,
    home_dir: &Path,
    scan_roots: &[PathBuf],
) {
    let known_planning_paths = load_known_planning_paths(state).await;
    for scan_root in scan_roots {
        let candidates = discover_scan_root_candidates(scan_root, home_dir).await;
        for candidate in candidates {
            let planning_path = candidate.planning_path.display().to_string();
            if known_planning_paths.contains(&planning_path) {
                continue;
            }
            if crate::watcher::refresh::refresh_project_planning_dir_for_app(
                state,
                &candidate.planning_path,
                |event| {
                    emit_app_event(app, event);
                    Ok(())
                },
            )
            .await
            .is_ok()
            {
                request_tray_refresh(app);
            }
        }
    }
}

async fn refresh_root<R: Runtime>(
    app: &AppHandle<R>,
    state: &AppState,
    home_dir: &Path,
    root: &Path,
) {
    if root.file_name().and_then(|name| name.to_str()) == Some(".planning") {
        if crate::watcher::refresh::refresh_project_planning_dir_for_app(state, root, |event| {
            emit_app_event(app, event);
            Ok(())
        })
        .await
        .is_ok()
        {
            request_tray_refresh(app);
        }
        return;
    }

    if let Some(source) = session_source_for_root(home_dir, root) {
        for source_path in collect_jsonl_files(root).await {
            refresh_session_file(app, state, source, &source_path).await;
        }
    }
}

async fn refresh_session_file<R: Runtime>(
    app: &AppHandle<R>,
    state: &AppState,
    source: SessionSource,
    source_path: &Path,
) {
    let _ = refresh_session_file_for_app(state, source, source_path, |event| {
        emit_app_event(app, event);
        Ok(())
    })
    .await;
}

async fn load_known_planning_paths(state: &AppState) -> Vec<String> {
    let Ok(connection) = state.pool.get().await else {
        return Vec::new();
    };
    connection
        .interact(project_repo::list_project_snapshots)
        .await
        .ok()
        .and_then(Result::ok)
        .unwrap_or_default()
        .into_iter()
        .map(|project| project.planning_path)
        .collect()
}

async fn discover_scan_root_candidates(
    scan_root: &Path,
    home_dir: &Path,
) -> Vec<crate::scanner::PlanningProjectCandidate> {
    let scan_root = scan_root.to_path_buf();
    let home_dir = home_dir.to_path_buf();
    tokio::task::spawn_blocking(move || {
        crate::scanner::discover_planning_dirs(&scan_root, &home_dir)
    })
    .await
    .ok()
    .and_then(Result::ok)
    .unwrap_or_default()
}

fn add_polling_root(polling_roots: &Arc<RwLock<Vec<PathBuf>>>, root: &Path) {
    let mut roots = polling_roots
        .write()
        .expect("polling roots lock should not be poisoned");
    if !roots.iter().any(|existing| existing == root) {
        roots.push(root.to_path_buf());
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
        AppEvent::TrayNavigate { .. } => {}
    }
}
