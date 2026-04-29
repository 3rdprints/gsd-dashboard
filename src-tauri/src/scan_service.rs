use std::path::{Path, PathBuf};

use deadpool_sqlite::Pool;

use crate::{
    error::AppError,
    events::ScanEvent,
    parser::{
        self,
        config::parse_config,
        plan::parse_plan,
        roadmap,
        roadmap::parse_roadmap,
        state::{extract_state_excerpt, parse_state},
        ParseIssue, PhaseIdentity, PlanDocument, ProjectSnapshot,
    },
    scan_persistence,
    scanner::{self, PlanningProjectCandidate, ScanSummary},
};

pub async fn scan_roots(
    pool: Pool,
    roots: Vec<PathBuf>,
    home_dir: PathBuf,
    on_event: impl Fn(ScanEvent) -> Result<(), AppError> + Send + Sync + 'static,
) -> Result<ScanSummary, AppError> {
    on_event(ScanEvent::Started {
        root_count: roots.len(),
    })?;

    let mut summary = ScanSummary::default();

    for root in roots {
        on_event(ScanEvent::RootStarted {
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

            let identity_candidate = candidate.clone();
            let identity =
                tokio::task::spawn_blocking(move || infer_project_identity(&identity_candidate))
                    .await
                    .map_err(AppError::io)?;
            on_event(ScanEvent::ProjectFound {
                project_id: identity.id.clone(),
                project_name: identity.name.clone(),
                root_path: candidate.project_root.display().to_string(),
            })?;

            let project_scan = read_and_parse_candidate(candidate.clone()).await?;
            let has_errors = !project_scan.parse_issues.is_empty();

            scan_persistence::persist_project_scan(&pool, &candidate, &identity, project_scan)
                .await?;

            if has_errors {
                summary.error_count += 1;
                on_event(ScanEvent::ProjectParseError {
                    project_id: identity.id,
                    project_name: identity.name,
                    file_path: candidate.planning_path.display().to_string(),
                    message: "One or more planning files could not be parsed".to_string(),
                })?;
            } else {
                summary.parsed_count += 1;
                on_event(ScanEvent::ProjectParsed {
                    project_id: identity.id,
                    project_name: identity.name,
                })?;
            }
        }
    }

    on_event(ScanEvent::Finished {
        discovered_count: summary.discovered_count,
        parsed_count: summary.parsed_count,
        error_count: summary.error_count,
    })?;

    Ok(summary)
}

#[derive(Debug, Clone)]
pub(crate) struct ProjectIdentity {
    pub(crate) id: String,
    pub(crate) name: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ProjectScan {
    pub(crate) snapshot: ProjectSnapshot,
    pub(crate) parse_issues: Vec<ParseIssue>,
}

#[derive(Debug, Clone, Copy, Default)]
struct PhaseFileProgress {
    plan_count: usize,
    summary_count: usize,
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

    let milestones = read_optional_or_issue(&milestones_path, &mut parse_issues)
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

    let state_bytes = read_optional_or_issue(&state_path, &mut parse_issues);
    let state_excerpt = state_bytes
        .as_ref()
        .and_then(|bytes| std::str::from_utf8(bytes).ok())
        .and_then(|body| extract_state_excerpt(body, 20, 2048).ok());
    let state = state_bytes
        .map(|bytes| parse_state(&bytes).map_err(|error| error.issue(display_path(&state_path))))
        .transpose()
        .map_err(|issue| {
            parse_issues.push(issue);
        })
        .ok()
        .flatten();

    let config = read_optional_or_issue(&config_path, &mut parse_issues)
        .map(|bytes| parse_config(&bytes).map_err(|error| error.issue(display_path(&config_path))))
        .transpose()
        .map_err(|issue| {
            parse_issues.push(issue);
        })
        .ok()
        .flatten();

    let mut phase_file_progress = PhaseFileProgress::default();
    let plans = parse_plan_files(
        &candidate.planning_path,
        &mut parse_issues,
        &mut phase_file_progress,
    )?;
    let roadmap = roadmap.unwrap_or_else(|| empty_roadmap(milestones));
    let progress = derive_project_progress(&roadmap, &plans, phase_file_progress);
    let milestone_progress_pct = state
        .as_ref()
        .and_then(|state| {
            state
                .status
                .as_deref()
                .is_some_and(is_completed_status)
                .then_some(100)
        })
        .unwrap_or(progress.percent);
    let current_milestone = state
        .as_ref()
        .and_then(|state| state.current_milestone.as_ref())
        .and_then(|state_milestone| {
            roadmap
                .milestones
                .iter()
                .find(|milestone| milestone_names_match(&milestone.name, &state_milestone.name))
                .cloned()
                .or_else(|| Some(state_milestone.clone()))
        })
        .or_else(|| roadmap.milestones.first().cloned());
    let current_phase = state
        .as_ref()
        .and_then(|state| state.current_phase.clone())
        .or_else(|| {
            if state
                .as_ref()
                .and_then(|state| state.status.as_deref())
                .is_some_and(is_completed_status)
            {
                None
            } else {
                roadmap
                    .phases
                    .iter()
                    .find(|phase| !phase.completed)
                    .map(|phase| PhaseIdentity {
                        number: phase.number.clone(),
                        name: phase.name.clone(),
                    })
            }
        });

    Ok(ProjectScan {
        snapshot: ProjectSnapshot {
            project_id: identity.id,
            project_name: identity.name,
            root_path: display_path(&candidate.project_root),
            planning_path: display_path(&candidate.planning_path),
            current_milestone,
            current_phase,
            milestone_progress_pct,
            roadmap_phases: roadmap.phases.clone(),
            phase_plans: plans
                .iter()
                .filter_map(project_phase_plan)
                .collect::<Vec<_>>(),
            state_excerpt,
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
    phase_file_progress: &mut PhaseFileProgress,
) -> Result<Vec<PlanDocument>, AppError> {
    let phases_path = planning_path.join("phases");
    let mut plans = Vec::new();

    if !phases_path.exists() {
        return Ok(plans);
    }

    collect_plan_files_recursive(&phases_path, parse_issues, &mut plans, phase_file_progress);
    Ok(plans)
}

fn collect_plan_files_recursive(
    directory: &Path,
    parse_issues: &mut Vec<ParseIssue>,
    plans: &mut Vec<PlanDocument>,
    phase_file_progress: &mut PhaseFileProgress,
) {
    let entries = match std::fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) => {
            parse_issues.push(ParseIssue {
                file_path: display_path(directory),
                kind: "io".to_string(),
                message: error.to_string(),
            });
            return;
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                parse_issues.push(ParseIssue {
                    file_path: display_path(directory),
                    kind: "io".to_string(),
                    message: error.to_string(),
                });
                continue;
            }
        };
        let entry_path = entry.path();
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(error) => {
                parse_issues.push(ParseIssue {
                    file_path: display_path(&entry_path),
                    kind: "io".to_string(),
                    message: error.to_string(),
                });
                continue;
            }
        };
        if file_type.is_dir() {
            collect_plan_files_recursive(&entry_path, parse_issues, plans, phase_file_progress);
            continue;
        }

        let Some(file_name) = entry_path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if file_name == "SUMMARY.md" || file_name.ends_with("-SUMMARY.md") {
            phase_file_progress.summary_count += 1;
            continue;
        }

        let is_plan = file_name == "PLAN.md" || file_name.ends_with("-PLAN.md");
        if !is_plan {
            continue;
        }
        phase_file_progress.plan_count += 1;

        match std::fs::read(&entry_path) {
            Ok(bytes) => match parse_plan(&bytes) {
                Ok(mut plan) => {
                    plan.source_path = Some(display_path(&entry_path));
                    plans.push(plan);
                }
                Err(error) => parse_issues.push(error.issue(display_path(&entry_path))),
            },
            Err(error) => parse_issues.push(ParseIssue {
                file_path: display_path(&entry_path),
                kind: "io".to_string(),
                message: error.to_string(),
            }),
        }
    }
}

fn derive_project_progress(
    roadmap: &roadmap::RoadmapDocument,
    plans: &[PlanDocument],
    phase_file_progress: PhaseFileProgress,
) -> parser::ProgressSummary {
    if phase_file_progress.plan_count > 0 {
        return parser::ProgressSummary {
            percent: percent(
                phase_file_progress
                    .summary_count
                    .min(phase_file_progress.plan_count),
                phase_file_progress.plan_count,
            ),
            source: "planSummaryCompletion".to_string(),
        };
    }

    parser::derive_progress(roadmap, plans)
}

fn is_completed_status(status: &str) -> bool {
    let normalized = status.trim().to_ascii_lowercase();
    normalized == "completed"
        || normalized == "complete"
        || normalized.contains("milestone achieved")
        || normalized.contains("milestone archived")
        || normalized.contains("shipped")
}

fn milestone_names_match(left: &str, right: &str) -> bool {
    let left = normalize_milestone_name(left);
    let right = normalize_milestone_name(right);

    left == right || left.contains(&right) || right.contains(&left)
}

fn normalize_milestone_name(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn percent(completed: usize, total: usize) -> u8 {
    if total == 0 {
        return 0;
    }

    ((completed * 100) / total).min(100) as u8
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

fn read_optional_or_issue(path: &Path, parse_issues: &mut Vec<ParseIssue>) -> Option<Vec<u8>> {
    match read_optional(path) {
        Ok(bytes) => bytes,
        Err(error) => {
            parse_issues.push(ParseIssue {
                file_path: display_path(path),
                kind: "io".to_string(),
                message: error.to_string(),
            });
            None
        }
    }
}

fn project_phase_plan(plan: &PlanDocument) -> Option<parser::PhasePlan> {
    Some(parser::PhasePlan {
        phase: PhaseIdentity {
            number: plan.phase.clone()?,
            name: String::new(),
        },
        plan: plan.plan.clone().unwrap_or_default(),
        plan_type: plan.plan_type.clone().unwrap_or_default(),
        plan_path: plan.source_path.clone().unwrap_or_default(),
        checklist: plan.checklist.clone(),
        items: plan.items.clone(),
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
    let slug = if id.is_empty() {
        fallback.to_string()
    } else {
        id
    };
    let canonical_root = candidate
        .project_root
        .canonicalize()
        .unwrap_or_else(|_| candidate.project_root.clone());
    let root_hash = stable_short_hash(canonical_root.to_string_lossy().as_ref());

    ProjectIdentity {
        id: format!("{slug}-{root_hash}"),
        name: raw_name.replace(['-', '_'], " "),
    }
}

fn stable_short_hash(value: &str) -> String {
    let mut hash = 0xcbf29ce484222325_u64;

    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }

    format!("{:08x}", hash as u32)
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
