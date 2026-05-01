use std::path::Path;

use crate::{
    app_state::AppState,
    error::AppError,
    events::AppEvent,
    scan_refresh::{self, ProjectRefreshOutcome},
    scanner::PlanningProjectCandidate,
    tray::service::request_tray_refresh,
};

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
    request_tray_refresh(state).await?;

    Ok(outcome)
}
