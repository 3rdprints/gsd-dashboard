use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use deadpool_sqlite::Pool;

use crate::{
    error::AppError,
    parser::{
        self, config::parse_config, plan::parse_plan, roadmap, roadmap::parse_roadmap,
        state::parse_state, ParseIssue, PhaseIdentity, PlanDocument, ProjectSnapshot,
    },
    scanner::{self, PlanningProjectCandidate, ScanSummary},
    store::project_repo::{self, StoredPhasePlan, StoredProjectSnapshot},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScanProgressEvent {
    Started {
        root_count: usize,
    },
    RootStarted {
        root_path: String,
    },
    ProjectFound {
        project_id: String,
        project_name: String,
        root_path: String,
    },
    ProjectParsed {
        project_id: String,
        project_name: String,
    },
    ProjectParseError {
        project_id: String,
        project_name: String,
        file_path: String,
        message: String,
    },
    Finished {
        discovered_count: usize,
        parsed_count: usize,
        error_count: usize,
    },
}

pub async fn scan_roots(
    pool: Pool,
    roots: Vec<PathBuf>,
    home_dir: PathBuf,
    on_event: impl Fn(ScanProgressEvent) -> Result<(), AppError> + Send + Sync + 'static,
) -> Result<ScanSummary, AppError> {
    on_event(ScanProgressEvent::Started {
        root_count: roots.len(),
    })?;

    let mut summary = ScanSummary::default();

    for root in roots {
        on_event(ScanProgressEvent::RootStarted {
            root_path: root.display().to_string(),
        })?;

        let root_for_discovery = root.clone();
        let home_for_discovery = home_dir.clone();
        let candidates = tokio::task::spawn_blocking(move || {
            scanner::discover_planning_dirs(&root_for_discovery, &home_for_discovery)
        })
        .await
        .map_err(AppError::io)??;

        for candidate in candidates {
            summary.discovered_count += 1;

            let identity = infer_project_identity(&candidate);
            on_event(ScanProgressEvent::ProjectFound {
                project_id: identity.id.clone(),
                project_name: identity.name.clone(),
                root_path: candidate.project_root.display().to_string(),
            })?;

            let project_scan = read_and_parse_candidate(candidate.clone()).await?;
            let has_errors = !project_scan.parse_issues.is_empty();

            persist_project_scan(&pool, &candidate, &identity, project_scan).await?;

            if has_errors {
                summary.error_count += 1;
                on_event(ScanProgressEvent::ProjectParseError {
                    project_id: identity.id,
                    project_name: identity.name,
                    file_path: candidate.planning_path.display().to_string(),
                    message: "One or more planning files could not be parsed".to_string(),
                })?;
            } else {
                summary.parsed_count += 1;
                on_event(ScanProgressEvent::ProjectParsed {
                    project_id: identity.id,
                    project_name: identity.name,
                })?;
            }
        }
    }

    on_event(ScanProgressEvent::Finished {
        discovered_count: summary.discovered_count,
        parsed_count: summary.parsed_count,
        error_count: summary.error_count,
    })?;

    Ok(summary)
}

#[derive(Debug, Clone)]
struct ProjectIdentity {
    id: String,
    name: String,
}

#[derive(Debug, Clone)]
struct ProjectScan {
    snapshot: ProjectSnapshot,
    parse_issues: Vec<ParseIssue>,
}

async fn read_and_parse_candidate(
    candidate: PlanningProjectCandidate,
) -> Result<ProjectScan, AppError> {
    tokio::task::spawn_blocking(move || parse_candidate_files(&candidate))
        .await
        .map_err(AppError::io)?
}

fn parse_candidate_files(candidate: &PlanningProjectCandidate) -> Result<ProjectScan, AppError> {
    let identity = infer_project_identity(candidate);
    let roadmap_path = candidate.planning_path.join("ROADMAP.md");
    let state_path = candidate.planning_path.join("STATE.md");
    let milestones_path = candidate.planning_path.join("MILESTONES.md");
    let config_path = candidate.planning_path.join("config.json");
    let mut parse_issues = Vec::new();

    let roadmap = read_required(&roadmap_path)
        .and_then(|bytes| {
            parse_roadmap(&bytes).map_err(|error| error.issue(display_path(&roadmap_path)))
        })
        .map_err(|issue| {
            parse_issues.push(issue);
        })
        .ok();

    let milestones = read_optional(&milestones_path)?
        .map(|bytes| {
            roadmap::parse_milestones(&bytes)
                .map_err(|error| error.issue(display_path(&milestones_path)))
        })
        .transpose()
        .map_err(|issue| {
            parse_issues.push(issue);
        })
        .ok()
        .flatten()
        .unwrap_or_default();

    let state = read_optional(&state_path)?
        .map(|bytes| parse_state(&bytes).map_err(|error| error.issue(display_path(&state_path))))
        .transpose()
        .map_err(|issue| {
            parse_issues.push(issue);
        })
        .ok()
        .flatten();

    let config = read_optional(&config_path)?
        .map(|bytes| parse_config(&bytes).map_err(|error| error.issue(display_path(&config_path))))
        .transpose()
        .map_err(|issue| {
            parse_issues.push(issue);
        })
        .ok()
        .flatten();

    let plans = parse_plan_files(&candidate.planning_path, &mut parse_issues)?;
    let roadmap = roadmap.unwrap_or_else(|| empty_roadmap(milestones));
    let progress = parser::derive_progress(&roadmap, &plans);
    let current_milestone = state
        .as_ref()
        .and_then(|state| state.current_milestone.clone())
        .or_else(|| roadmap.milestones.first().cloned());
    let current_phase = state
        .as_ref()
        .and_then(|state| state.current_phase.clone())
        .or_else(|| {
            roadmap
                .phases
                .iter()
                .find(|phase| !phase.completed)
                .map(|phase| PhaseIdentity {
                    number: phase.number.clone(),
                    name: phase.name.clone(),
                })
        });

    Ok(ProjectScan {
        snapshot: ProjectSnapshot {
            project_id: identity.id,
            project_name: identity.name,
            root_path: display_path(&candidate.project_root),
            planning_path: display_path(&candidate.planning_path),
            current_milestone,
            current_phase,
            milestone_progress_pct: progress.percent,
            phase_plans: plans
                .iter()
                .filter_map(project_phase_plan)
                .collect::<Vec<_>>(),
            next_command: state
                .as_ref()
                .map(|state| state.next_command.clone())
                .unwrap_or_else(|| "/gsd-next".to_string()),
            config,
            parse_issues: parse_issues.clone(),
        },
        parse_issues,
    })
}

fn parse_plan_files(
    planning_path: &Path,
    parse_issues: &mut Vec<ParseIssue>,
) -> Result<Vec<PlanDocument>, AppError> {
    let phases_path = planning_path.join("phases");
    let mut plans = Vec::new();

    if !phases_path.exists() {
        return Ok(plans);
    }

    for phase_entry in std::fs::read_dir(phases_path)? {
        let phase_entry = phase_entry?;
        if !phase_entry.file_type()?.is_dir() {
            continue;
        }

        for plan_entry in std::fs::read_dir(phase_entry.path())? {
            let plan_entry = plan_entry?;
            let plan_path = plan_entry.path();
            let is_plan = plan_path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with("-PLAN.md"));
            if !is_plan {
                continue;
            }

            match std::fs::read(&plan_path) {
                Ok(bytes) => match parse_plan(&bytes) {
                    Ok(plan) => plans.push(plan),
                    Err(error) => parse_issues.push(error.issue(display_path(&plan_path))),
                },
                Err(error) => parse_issues.push(ParseIssue {
                    file_path: display_path(&plan_path),
                    kind: "io".to_string(),
                    message: error.to_string(),
                }),
            }
        }
    }

    Ok(plans)
}

fn read_required(path: &Path) -> Result<Vec<u8>, ParseIssue> {
    std::fs::read(path).map_err(|error| ParseIssue {
        file_path: display_path(path),
        kind: "io".to_string(),
        message: error.to_string(),
    })
}

fn read_optional(path: &Path) -> Result<Option<Vec<u8>>, AppError> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(AppError::io(error)),
    }
}

async fn persist_project_scan(
    pool: &Pool,
    candidate: &PlanningProjectCandidate,
    identity: &ProjectIdentity,
    project_scan: ProjectScan,
) -> Result<(), AppError> {
    let now = unix_timestamp();
    let first_issue = project_scan.parse_issues.first().cloned();
    let errors_json = serde_json::to_string(&project_scan.parse_issues)?;
    let stored_snapshot = stored_snapshot(project_scan.snapshot, first_issue.as_ref(), now)?;
    let phase_plans = stored_phase_plans(&stored_snapshot.id, &stored_snapshot.parsed_blob)?;
    let root_path = display_path(&candidate.project_root);
    let planning_path = display_path(&candidate.planning_path);
    let project_id = identity.id.clone();

    let connection = pool.get().await.map_err(AppError::store)?;
    connection
        .interact(move |connection| {
            project_repo::upsert_project_snapshot(connection, stored_snapshot, phase_plans, now)?;

            if let Some(issue) = first_issue {
                project_repo::append_scan_log(
                    connection,
                    project_repo::StoredScanLogEntry {
                        project_id: Some(project_id),
                        root_path: Some(root_path),
                        planning_path: Some(planning_path),
                        file_path: Some(issue.file_path),
                        status: "parseError".to_string(),
                        message: Some(issue.message),
                        errors_json,
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
        .map(|plan| StoredPhasePlan {
            project_id: project_id.to_string(),
            phase_number: plan.phase.number,
            phase_name: Some(plan.phase.name),
            plan_number: Some(plan.plan),
            plan_path: String::new(),
            checklist_json: serde_json::to_string(&plan.checklist)
                .unwrap_or_else(|_| "[]".to_string()),
            updated_at: 0,
        })
        .collect())
}

fn project_phase_plan(plan: &PlanDocument) -> Option<parser::PhasePlan> {
    Some(parser::PhasePlan {
        phase: PhaseIdentity {
            number: plan.phase.clone()?,
            name: String::new(),
        },
        plan: plan.plan.clone().unwrap_or_default(),
        plan_type: plan.plan_type.clone().unwrap_or_default(),
        checklist: plan.checklist.clone(),
    })
}

fn infer_project_identity(candidate: &PlanningProjectCandidate) -> ProjectIdentity {
    let fallback = "project";
    let raw_name = candidate
        .project_root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(fallback);
    let id = raw_name
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();

    ProjectIdentity {
        id: if id.is_empty() {
            fallback.to_string()
        } else {
            id
        },
        name: raw_name.replace(['-', '_'], " "),
    }
}

fn empty_roadmap(milestones: Vec<parser::MilestoneIdentity>) -> roadmap::RoadmapDocument {
    roadmap::RoadmapDocument {
        milestones,
        phases: Vec::new(),
        milestone_progress_pct: 0,
        progress_source: "missingRoadmap".to_string(),
        phase_checkbox_total: 0,
        phase_checkbox_completed: 0,
    }
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
