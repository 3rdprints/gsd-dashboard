use std::path::PathBuf;

use tauri::{ipc::Channel, State};

use crate::{
    app_state::AppState, error::AppError, events::ScanEvent, scan_roots, scan_service,
    scanner::ScanSummary, settings,
    store::project_repo,
};

#[tauri::command]
pub async fn scan_projects(
    state: State<'_, AppState>,
    on_event: Channel<ScanEvent>,
) -> Result<ScanSummary, AppError> {
    scan_projects_for_app(&state, move |event| {
        on_event.send(event).map_err(AppError::from)
    })
    .await
}

#[tauri::command]
pub async fn rebuild_cache(
    state: State<'_, AppState>,
    on_event: Channel<ScanEvent>,
) -> Result<ScanSummary, AppError> {
    rebuild_cache_for_app(&state, move |event| {
        on_event.send(event).map_err(AppError::from)
    })
    .await
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

    scan_projects_for_app(state, on_event).await
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
