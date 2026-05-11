use deadpool_sqlite::Pool;

use crate::{
    error::AppError,
    scan_persistence,
    scan_service::{self, ProjectIdentity},
    scanner::PlanningProjectCandidate,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectRefreshOutcome {
    pub project_id: String,
    pub project_name: String,
    pub had_parse_errors: bool,
}

/// Re-scans and persists a single project from its planning candidate path.
pub async fn scan_single_project_candidate(
    pool: &Pool,
    candidate: PlanningProjectCandidate,
) -> Result<ProjectRefreshOutcome, AppError> {
    let project_scan = scan_service::read_and_parse_candidate(candidate.clone()).await?;
    let identity = ProjectIdentity {
        id: project_scan.snapshot.project_id.clone(),
        name: project_scan.snapshot.project_name.clone(),
    };
    let had_parse_errors = !project_scan.parse_issues.is_empty();
    scan_persistence::persist_project_scan(pool, &candidate, &identity, project_scan).await?;

    Ok(ProjectRefreshOutcome {
        project_id: identity.id,
        project_name: identity.name,
        had_parse_errors,
    })
}
