use std::collections::BTreeMap;

use rusqlite::{params, OptionalExtension};
use serde::Serialize;

use crate::{
    error::AppError,
    milestone_match::milestone_names_match,
    parser::{roadmap::RoadmapPhase, ProjectSnapshot},
    store::project_repo::{self, StoredProjectSnapshot},
};

const DEFAULT_PAGE_SIZE: i64 = 50;
const MAX_PAGE_SIZE: i64 = 200;
const TOKEN_TOTAL_EXPR: &str = "COALESCE(tokens_in, 0)
                + COALESCE(tokens_out, 0)
                + COALESCE(cache_read_tokens, 0)
                + COALESCE(cache_creation_tokens, 0)";

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMilestoneDto {
    pub name: Option<String>,
    pub progress_pct: f64,
    pub phase_count: i64,
    pub completed_phase_count: i64,
    pub phases: Vec<ProjectMilestonePhaseDto>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMilestonePhaseDto {
    pub number: String,
    pub name: Option<String>,
    pub is_current: bool,
    pub completed_at: Option<i64>,
    pub completed_plan_count: i64,
    pub total_plan_count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectPhasePanelDto {
    pub phase_number: Option<String>,
    pub phase_name: Option<String>,
    pub plan_path: Option<String>,
    pub state_path: String,
    pub state_excerpt: Option<String>,
    pub completed_item_count: i64,
    pub total_item_count: i64,
    pub items: Vec<ProjectPlanItemDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectPlanItemDto {
    pub plan_path: String,
    pub ord: i64,
    pub text: String,
    pub checked: bool,
    pub line_no: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSessionsPageDto {
    pub rows: Vec<ProjectSessionRowDto>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSessionRowDto {
    pub id: String,
    pub source: String,
    pub source_path: String,
    pub started_at: Option<i64>,
    pub ended_at: Option<i64>,
    pub duration_ms: Option<i64>,
    pub message_count: i64,
    pub tokens_in: i64,
    pub tokens_out: i64,
    pub token_total: i64,
    pub model: Option<String>,
}

/// Loads milestone phases with completion progress for a project.
pub fn load_project_milestones(
    connection: &mut rusqlite::Connection,
    project_id: &str,
) -> Result<Vec<ProjectMilestoneDto>, AppError> {
    let snapshot = load_project(connection, project_id)?;
    let db_phases = load_phase_progress(connection, project_id)?;
    let parsed_snapshot = serde_json::from_str::<ProjectSnapshot>(&snapshot.parsed_blob)?;
    let roadmap_phases = parsed_snapshot.roadmap_phases;
    let groups = if roadmap_phases.is_empty() {
        match db_phases.is_empty() {
            true => Vec::new(),
            false => vec![MilestonePhaseGroup {
                name: snapshot.current_milestone_name.clone(),
                phases: db_phases,
            }],
        }
    } else {
        group_milestone_phases(&snapshot, db_phases, roadmap_phases)
    };

    Ok(groups
        .into_iter()
        .filter(|group| !group.phases.is_empty())
        .map(|group| build_project_milestone(group.name, group.phases))
        .collect())
}

struct MilestonePhaseGroup {
    name: Option<String>,
    phases: Vec<ProjectMilestonePhaseDto>,
}

fn build_project_milestone(
    name: Option<String>,
    phases: Vec<ProjectMilestonePhaseDto>,
) -> ProjectMilestoneDto {
    let phase_count = phases.len() as i64;
    if phase_count == 0 {
        return ProjectMilestoneDto {
            name,
            progress_pct: 0.0,
            phase_count,
            completed_phase_count: 0,
            phases,
        };
    }

    let completed_phase_count = phases
        .iter()
        .filter(|phase| phase_is_complete(phase) && !phase.is_current)
        .count() as i64;
    let current_fraction = phases
        .iter()
        .find(|phase| phase.is_current)
        .map(|phase| {
            if phase.total_plan_count == 0 {
                0.0
            } else {
                phase.completed_plan_count as f64 / phase.total_plan_count as f64
            }
        })
        .unwrap_or(0.0);
    let progress_pct = ((completed_phase_count as f64 + current_fraction) / phase_count as f64)
        .clamp(0.0, 1.0)
        * 100.0;

    ProjectMilestoneDto {
        name,
        progress_pct,
        phase_count,
        completed_phase_count,
        phases,
    }
}

fn phase_is_complete(phase: &ProjectMilestonePhaseDto) -> bool {
    phase.completed_at.is_some()
        || (phase.total_plan_count > 0 && phase.completed_plan_count >= phase.total_plan_count)
}

fn group_milestone_phases(
    snapshot: &StoredProjectSnapshot,
    db_phases: Vec<ProjectMilestonePhaseDto>,
    roadmap_phases: Vec<RoadmapPhase>,
) -> Vec<MilestonePhaseGroup> {
    let current_phase_number = snapshot.current_phase_number.as_deref();
    let mut groups = Vec::new();
    let mut db_phase_by_number = db_phases
        .into_iter()
        .map(|phase| (phase.number.clone(), phase))
        .collect::<BTreeMap<_, _>>();

    for roadmap_phase in roadmap_phases {
        let group_name = roadmap_phase
            .milestone_name
            .clone()
            .or_else(|| snapshot.current_milestone_name.clone());
        let phase = db_phase_by_number
            .remove(&roadmap_phase.number)
            .map(|mut phase| {
                phase.is_current = Some(phase.number.as_str()) == current_phase_number;
                phase
            })
            .unwrap_or_else(|| {
                roadmap_phase_to_milestone_phase(&roadmap_phase, current_phase_number)
            });
        push_group_phase(&mut groups, group_name, phase);
    }

    for (_phase_number, phase) in db_phase_by_number {
        push_group_phase(&mut groups, snapshot.current_milestone_name.clone(), phase);
    }

    groups
}

fn push_group_phase(
    groups: &mut Vec<MilestonePhaseGroup>,
    name: Option<String>,
    phase: ProjectMilestonePhaseDto,
) {
    if let Some(group) = groups
        .iter_mut()
        .find(|group| milestone_group_names_match(group.name.as_deref(), name.as_deref()))
    {
        group.phases.push(phase);
        return;
    }

    groups.push(MilestonePhaseGroup {
        name,
        phases: vec![phase],
    });
}

fn milestone_group_names_match(left: Option<&str>, right: Option<&str>) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => milestone_names_match(left, right),
        (None, None) => true,
        _ => false,
    }
}

/// Loads the current phase's plan items and state excerpt.
pub fn load_project_phase_panel(
    connection: &mut rusqlite::Connection,
    project_id: &str,
) -> Result<ProjectPhasePanelDto, AppError> {
    let snapshot = load_project(connection, project_id)?;
    let current_phase_number = snapshot.current_phase_number.clone();
    let items = {
        let mut statement = connection
            .prepare(
                "SELECT plan_items.plan_path,
                        plan_items.ord,
                        plan_items.text,
                        plan_items.checked,
                        plan_items.line_no
                 FROM plan_items
                 JOIN phase_plans
                   ON phase_plans.project_id = plan_items.project_id
                  AND phase_plans.plan_path = plan_items.plan_path
                 WHERE plan_items.project_id = ?1
                   AND (?2 IS NOT NULL AND phase_plans.phase_number = ?2)
                 ORDER BY phase_plans.plan_number, plan_items.ord",
            )
            .map_err(AppError::from)?;
        let rows = statement
            .query_map(
                params![project_id, current_phase_number.as_deref()],
                |row| {
                    Ok(ProjectPlanItemDto {
                        plan_path: row.get(0)?,
                        ord: row.get(1)?,
                        text: row.get(2)?,
                        checked: row.get::<_, i64>(3)? != 0,
                        line_no: row.get(4)?,
                    })
                },
            )
            .map_err(AppError::from)?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(AppError::from)?
    };
    let (completed_item_count, total_item_count) = if items.is_empty() {
        current_phase_plan_counts(
            connection,
            project_id,
            snapshot.current_phase_number.as_deref(),
        )?
    } else {
        (
            items.iter().filter(|item| item.checked).count() as i64,
            items.len() as i64,
        )
    };
    let plan_path = if let Some(item) = items.first() {
        Some(item.plan_path.clone())
    } else {
        first_phase_plan_path(
            connection,
            project_id,
            snapshot.current_phase_number.as_deref(),
        )?
    };
    let parsed_snapshot = serde_json::from_str::<ProjectSnapshot>(&snapshot.parsed_blob)?;
    let state_excerpt = parsed_snapshot.state_excerpt;

    Ok(ProjectPhasePanelDto {
        phase_number: snapshot.current_phase_number,
        phase_name: snapshot.current_phase_name,
        plan_path,
        state_path: format!("{}/STATE.md", snapshot.planning_path),
        state_excerpt,
        completed_item_count,
        total_item_count,
        items,
    })
}

/// Returns a paginated, sortable list of sessions for a project.
pub fn list_project_sessions(
    connection: &mut rusqlite::Connection,
    project_id: &str,
    sort: Option<&str>,
    direction: Option<&str>,
    page: Option<i64>,
    page_size: Option<i64>,
) -> Result<ProjectSessionsPageDto, AppError> {
    let sort_column = sort_column(sort.unwrap_or("startedAt"))?;
    let sort_direction = sort_direction(direction.unwrap_or("desc"))?;
    let page = page.unwrap_or(1).max(1);
    let page_size = page_size
        .unwrap_or(DEFAULT_PAGE_SIZE)
        .clamp(1, MAX_PAGE_SIZE);
    let offset = page.saturating_sub(1).saturating_mul(page_size);
    let total = connection
        .query_row(
            "SELECT COUNT(*) FROM sessions WHERE project_id = ?1",
            [project_id],
            |row| row.get(0),
        )
        .map_err(AppError::from)?;
    let sql = format!(
        "SELECT id,
                source,
                source_path,
                started_at,
                ended_at,
                duration_ms,
                message_count,
                COALESCE(tokens_in, 0),
                COALESCE(tokens_out, 0),
                {TOKEN_TOTAL_EXPR},
                model
         FROM sessions
         WHERE project_id = ?1
         ORDER BY {sort_column} {sort_direction}, id ASC
         LIMIT ?2 OFFSET ?3"
    );
    let mut statement = connection.prepare(&sql).map_err(AppError::from)?;
    let rows = statement
        .query_map(params![project_id, page_size, offset], |row| {
            Ok(ProjectSessionRowDto {
                id: row.get(0)?,
                source: row.get(1)?,
                source_path: row.get(2)?,
                started_at: row.get(3)?,
                ended_at: row.get(4)?,
                duration_ms: row.get(5)?,
                message_count: row.get(6)?,
                tokens_in: row.get(7)?,
                tokens_out: row.get(8)?,
                token_total: row.get(9)?,
                model: row.get(10)?,
            })
        })
        .map_err(AppError::from)?;

    Ok(ProjectSessionsPageDto {
        rows: rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(AppError::from)?,
        total,
        page,
        page_size,
    })
}

fn load_project(
    connection: &mut rusqlite::Connection,
    project_id: &str,
) -> Result<StoredProjectSnapshot, AppError> {
    project_repo::load_project_by_id(connection, project_id)?
        .ok_or_else(|| AppError::store("project not found"))
}

fn first_phase_plan_path(
    connection: &mut rusqlite::Connection,
    project_id: &str,
    phase_number: Option<&str>,
) -> Result<Option<String>, AppError> {
    connection
        .query_row(
            "SELECT plan_path
             FROM phase_plans
             WHERE project_id = ?1
               AND (?2 IS NOT NULL AND phase_number = ?2)
             ORDER BY plan_number, plan_path
             LIMIT 1",
            params![project_id, phase_number],
            |row| row.get(0),
        )
        .optional()
        .map_err(AppError::from)
}

fn load_phase_progress(
    connection: &mut rusqlite::Connection,
    project_id: &str,
) -> Result<Vec<ProjectMilestonePhaseDto>, AppError> {
    let mut statement = connection
        .prepare(
            "SELECT phase_plans.phase_number,
                    MAX(phase_plans.phase_name),
                    CASE
                        WHEN COUNT(DISTINCT phase_plans.plan_path) > 0
                         AND COUNT(DISTINCT phase_plans.plan_path)
                           = COUNT(DISTINCT CASE
                               WHEN phase_plans.completed_at IS NOT NULL THEN phase_plans.plan_path
                               ELSE NULL
                             END)
                        THEN MAX(phase_plans.completed_at)
                        ELSE NULL
                    END,
                    COUNT(DISTINCT CASE
                        WHEN phase_plans.completed_at IS NOT NULL THEN phase_plans.plan_path
                        ELSE NULL
                    END),
                    COUNT(DISTINCT phase_plans.plan_path)
             FROM phase_plans
             WHERE phase_plans.project_id = ?1
             GROUP BY phase_plans.phase_number
             ORDER BY phase_plans.phase_number",
        )
        .map_err(AppError::from)?;
    let rows = statement
        .query_map([project_id], |row| {
            Ok(ProjectMilestonePhaseDto {
                number: row.get(0)?,
                name: row.get(1)?,
                is_current: false,
                completed_at: row.get(2)?,
                completed_plan_count: row.get(3)?,
                total_plan_count: row.get(4)?,
            })
        })
        .map_err(AppError::from)?;
    let mut phases = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(AppError::from)?;
    let current_phase_number = connection
        .query_row(
            "SELECT current_phase_number FROM projects WHERE id = ?1",
            [project_id],
            |row| row.get::<_, Option<String>>(0),
        )
        .map_err(AppError::from)?;
    for phase in &mut phases {
        phase.is_current = Some(phase.number.as_str()) == current_phase_number.as_deref();
    }

    Ok(phases)
}

fn roadmap_phase_to_milestone_phase(
    phase: &RoadmapPhase,
    current_phase_number: Option<&str>,
) -> ProjectMilestonePhaseDto {
    let total_plan_count = phase
        .total_plan_count
        .map(|count| count.max(1))
        .unwrap_or(1) as i64;
    let completed_plan_count = phase
        .completed_plan_count
        .unwrap_or_else(|| usize::from(phase.completed))
        .min(total_plan_count as usize) as i64;

    ProjectMilestonePhaseDto {
        number: phase.number.clone(),
        name: Some(phase.name.clone()),
        is_current: Some(phase.number.as_str()) == current_phase_number,
        completed_at: None,
        completed_plan_count,
        total_plan_count,
    }
}

fn current_phase_plan_counts(
    connection: &mut rusqlite::Connection,
    project_id: &str,
    phase_number: Option<&str>,
) -> Result<(i64, i64), AppError> {
    connection
        .query_row(
            "SELECT COALESCE(SUM(CASE WHEN completed_at IS NOT NULL THEN 1 ELSE 0 END), 0),
                    COUNT(*)
             FROM phase_plans
             WHERE project_id = ?1
               AND (?2 IS NOT NULL AND phase_number = ?2)",
            params![project_id, phase_number],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(AppError::from)
}

fn sort_column(value: &str) -> Result<&'static str, AppError> {
    match value {
        "startedAt" => Ok("COALESCE(started_at, 0)"),
        "source" => Ok("source"),
        "durationMs" => Ok("COALESCE(duration_ms, 0)"),
        "messageCount" => Ok("message_count"),
        "tokensIn" => Ok("COALESCE(tokens_in, 0)"),
        "tokensOut" => Ok("COALESCE(tokens_out, 0)"),
        "tokenTotal" => Ok(TOKEN_TOTAL_EXPR),
        _ => Err(AppError::store("invalid session sort")),
    }
}

fn sort_direction(value: &str) -> Result<&'static str, AppError> {
    match value {
        "asc" => Ok("ASC"),
        "desc" => Ok("DESC"),
        _ => Err(AppError::store("invalid session sort direction")),
    }
}
