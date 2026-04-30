use std::path::PathBuf;

use tauri::{ipc::Channel, AppHandle, State};

use crate::{
    app_state::AppState, error::AppError, events::ScanEvent, scan_roots, scan_service,
    scanner::ScanSummary, sessions, settings, store::project_repo,
    tray::service::request_tray_refresh,
};

#[tauri::command]
pub async fn scan_projects(
    app: AppHandle,
    state: State<'_, AppState>,
    on_event: Channel<ScanEvent>,
) -> Result<ScanSummary, AppError> {
    let summary = scan_projects_for_app(&state, move |event| {
        on_event.send(event).map_err(AppError::from)
    })
    .await?;
    request_tray_refresh(&app);
    Ok(summary)
}

#[tauri::command]
pub async fn rebuild_cache(
    app: AppHandle,
    state: State<'_, AppState>,
    on_event: Channel<ScanEvent>,
) -> Result<ScanSummary, AppError> {
    let summary = rebuild_cache_for_app(&state, move |event| {
        on_event.send(event).map_err(AppError::from)
    })
    .await?;
    request_tray_refresh(&app);
    Ok(summary)
}

pub async fn rebuild_cache_for_app(
    state: &AppState,
    on_event: impl Fn(ScanEvent) -> Result<(), AppError> + Send + Sync + 'static,
) -> Result<ScanSummary, AppError> {
    let connection = state.pool.get().await.map_err(AppError::store)?;
    connection
        .interact(project_repo::clear_project_cache)
        .await
        .map_err(AppError::store)??;

    let summary = scan_projects_for_app(state, on_event).await?;
    rematch_unmatched_sessions(&state.pool).await?;

    Ok(summary)
}

pub async fn scan_projects_for_app(
    state: &AppState,
    on_event: impl Fn(ScanEvent) -> Result<(), AppError> + Send + Sync + 'static,
) -> Result<ScanSummary, AppError> {
    let app_settings = settings::load_or_initialize(&state.pool, &state.home_dir).await?;
    let roots = app_settings
        .scan_roots
        .into_iter()
        .map(|root| scan_roots::normalize_scan_root(&PathBuf::from(root), &state.home_dir))
        .collect::<Vec<_>>();

    scan_service::scan_roots(state.pool.clone(), roots, state.home_dir.clone(), on_event).await
}

async fn rematch_unmatched_sessions(pool: &deadpool_sqlite::Pool) -> Result<(), AppError> {
    let now = std::time::UNIX_EPOCH
        .elapsed()
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0);
    let connection = pool.get().await.map_err(AppError::store)?;
    connection
        .interact(move |connection| {
            let known_projects = project_repo::list_project_snapshots(connection)?
                .into_iter()
                .map(|project| sessions::ProjectRoot {
                    id: project.id,
                    root_path: project.root_path,
                })
                .collect::<Vec<_>>();
            sessions::repo::rematch_unmatched_sessions_against_projects(
                connection,
                &known_projects,
                now,
            )
            .map(|_| ())
        })
        .await
        .map_err(AppError::store)?
}
