use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use deadpool_sqlite::Pool;

use crate::{
    error::AppError,
    parser::{ParseIssue, ProjectSnapshot},
    scan_service::{ProjectIdentity, ProjectScan},
    scanner::PlanningProjectCandidate,
    store::project_repo::{self, StoredPhasePlan, StoredPlanItem, StoredProjectSnapshot},
};

pub(crate) async fn persist_project_scan(
    pool: &Pool,
    candidate: &PlanningProjectCandidate,
    identity: &ProjectIdentity,
    project_scan: ProjectScan,
) -> Result<(), AppError> {
    let now = unix_timestamp();
    let first_issue = project_scan.parse_issues.first().cloned();
    let scan_log_issues = project_scan.parse_issues.clone();
    let errors_json = serde_json::to_string(&project_scan.parse_issues)?;
    let stored_snapshot = stored_snapshot(project_scan.snapshot, first_issue.as_ref(), now)?;
    let phase_plans = stored_phase_plans(&stored_snapshot.id, &stored_snapshot.parsed_blob)?;
    let root_path = display_path(&candidate.project_root);
    let planning_path = display_path(&candidate.planning_path);
    let project_id = identity.id.clone();

    let connection = pool.get().await.map_err(AppError::store)?;
    connection
        .interact(move |connection| {
            let plan_items = stored_plan_items(&stored_snapshot.id, &stored_snapshot.parsed_blob)?;
            project_repo::upsert_project_snapshot(connection, stored_snapshot, phase_plans, now)?;
            for (plan_path, items) in plan_items {
                if let Err(error) =
                    project_repo::replace_plan_items(connection, &project_id, &plan_path, items)
                        .and_then(|_| {
                            project_repo::set_plan_completed_at_if_all_checked(
                                connection,
                                &project_id,
                                &plan_path,
                                now,
                            )
                        })
                {
                    project_repo::append_scan_log(
                        connection,
                        project_repo::StoredScanLogEntry {
                            project_id: Some(project_id.clone()),
                            root_path: Some(root_path.clone()),
                            planning_path: Some(planning_path.clone()),
                            file_path: Some(plan_path),
                            status: "parseError".to_string(),
                            message: Some(error.to_string()),
                            errors_json: errors_json.clone(),
                            created_at: 0,
                        },
                        now,
                    )?;
                }
            }

            for issue in scan_log_issues {
                project_repo::append_scan_log(
                    connection,
                    project_repo::StoredScanLogEntry {
                        project_id: Some(project_id.clone()),
                        root_path: Some(root_path.clone()),
                        planning_path: Some(planning_path.clone()),
                        file_path: Some(issue.file_path),
                        status: "parseError".to_string(),
                        message: Some(issue.message),
                        errors_json: errors_json.clone(),
                        created_at: 0,
                    },
                    now,
                )?;
            }

            Ok::<_, AppError>(())
        })
        .await
        .map_err(AppError::store)?
}

fn stored_snapshot(
    snapshot: ProjectSnapshot,
    first_issue: Option<&ParseIssue>,
    now: i64,
) -> Result<StoredProjectSnapshot, AppError> {
    Ok(StoredProjectSnapshot {
        id: snapshot.project_id.clone(),
        name: snapshot.project_name.clone(),
        root_path: snapshot.root_path.clone(),
        planning_path: snapshot.planning_path.clone(),
        current_milestone_name: snapshot
            .current_milestone
            .as_ref()
            .map(|milestone| milestone.name.clone()),
        current_milestone_index: snapshot
            .current_milestone
            .as_ref()
            .map(|milestone| milestone.index as i64),
        current_phase_number: snapshot
            .current_phase
            .as_ref()
            .map(|phase| phase.number.clone()),
        current_phase_name: snapshot
            .current_phase
            .as_ref()
            .map(|phase| phase.name.clone()),
        milestone_progress_pct: f64::from(snapshot.milestone_progress_pct),
        next_command: snapshot.next_command.clone(),
        parsed_blob: serde_json::to_string(&snapshot)?,
        parse_error: first_issue.map(|issue| issue.message.clone()),
        last_activity_at: None,
        last_scanned_at: now,
        created_at: 0,
        updated_at: 0,
    })
}

fn stored_phase_plans(
    project_id: &str,
    parsed_blob: &str,
) -> Result<Vec<StoredPhasePlan>, AppError> {
    let snapshot: ProjectSnapshot = serde_json::from_str(parsed_blob)?;

    Ok(snapshot
        .phase_plans
        .into_iter()
        .enumerate()
        .map(|(index, plan)| {
            let phase_number = plan.phase.number;
            let plan_number = plan.plan;
            let plan_path = if plan.plan_path.is_empty() {
                format!(
                    "phase-{phase_number}/plan-{}-{}",
                    if plan_number.is_empty() {
                        "unknown"
                    } else {
                        plan_number.as_str()
                    },
                    index + 1
                )
            } else {
                plan.plan_path
            };

            StoredPhasePlan {
                project_id: project_id.to_string(),
                phase_number,
                phase_name: Some(plan.phase.name),
                plan_number: Some(plan_number),
                plan_path,
                completed_at: plan.completed.then_some(0),
                checklist_json: serde_json::to_string(&plan.checklist)
                    .unwrap_or_else(|_| "[]".to_string()),
                updated_at: 0,
            }
        })
        .collect())
}

fn stored_plan_items(
    project_id: &str,
    parsed_blob: &str,
) -> Result<Vec<(String, Vec<StoredPlanItem>)>, AppError> {
    let snapshot: ProjectSnapshot = serde_json::from_str(parsed_blob)?;

    Ok(snapshot
        .phase_plans
        .into_iter()
        .filter(|plan| !plan.plan_path.is_empty())
        .map(|plan| {
            let plan_path = plan.plan_path;
            let items = plan
                .items
                .into_iter()
                .map(|item| StoredPlanItem {
                    project_id: project_id.to_string(),
                    plan_path: plan_path.clone(),
                    ord: item.ord as i64,
                    text: item.text,
                    checked: item.checked,
                    line_no: item.line_no as i64,
                })
                .collect::<Vec<_>>();
            (plan_path, items)
        })
        .collect())
}

fn display_path(path: &Path) -> String {
    path.display().to_string()
}

fn unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or_default()
}
