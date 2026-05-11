use std::path::Path;

use crate::{
    app_state::AppState,
    error::AppError,
    events::AppEvent,
    scan_refresh::{self, ProjectRefreshOutcome},
    scanner::PlanningProjectCandidate,
    sessions::{
        file_indexer::{index_session_file, load_known_project_roots, IndexedFileResult},
        SessionSource,
    },
    store::daily_activity,
    tray::service::record_tray_refresh_request,
};

/// Re-scans a project's `.planning/` directory and emits update events.
pub async fn refresh_project_planning_dir_for_app(
    state: &AppState,
    planning_path: &Path,
    emit_event: impl Fn(AppEvent) -> Result<(), AppError>,
) -> Result<ProjectRefreshOutcome, AppError> {
    let project_root = planning_path
        .parent()
        .ok_or_else(|| AppError::io("Planning path must have a project root"))?
        .to_path_buf();
    let candidate = PlanningProjectCandidate {
        project_root,
        planning_path: planning_path.to_path_buf(),
    };
    let outcome = scan_refresh::scan_single_project_candidate(&state.pool, candidate).await?;

    emit_event(AppEvent::ProjectUpdated {
        id: outcome.project_id.clone(),
    })?;
    record_tray_refresh_request(state).await?;

    Ok(outcome)
}

/// Re-indexes a single session JSONL file and rebuilds daily activity if sessions changed.
pub async fn refresh_session_file(
    state: &AppState,
    source: SessionSource,
    source_path: &Path,
    emit_event: impl Fn(AppEvent) -> Result<(), AppError>,
) -> Result<IndexedFileResult, AppError> {
    let known_projects = load_known_project_roots(&state.pool).await?;
    let result = index_session_file(
        &state.pool,
        source,
        source_path.to_path_buf(),
        &known_projects,
    )
    .await?;

    for session in &result.session_changes {
        emit_event(AppEvent::SessionNew {
            id: session.id.clone(),
            project_id: session.project_id.clone(),
        })?;
    }

    if !result.session_changes.is_empty() {
        rebuild_daily_activity(&state.pool).await?;
        emit_event(AppEvent::DailyActivityUpdated)?;
    }

    Ok(result)
}

async fn rebuild_daily_activity(pool: &deadpool_sqlite::Pool) -> Result<(), AppError> {
    let now_ms = current_epoch_ms();
    let connection = pool.get().await.map_err(AppError::store)?;
    connection
        .interact(move |connection| daily_activity::rebuild_window(connection, 90, now_ms))
        .await
        .map_err(AppError::store)?
}

fn current_epoch_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis().try_into().unwrap_or(0))
        .unwrap_or(0)
}
