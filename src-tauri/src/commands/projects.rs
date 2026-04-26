use std::collections::HashSet;

use serde::Serialize;
use tauri::State;

use crate::{
    app_state::AppState,
    error::AppError,
    sessions::{self, repo::UnmatchedSessionSummary, SessionSource},
    settings,
    store::project_repo::{self, StoredProjectSnapshot},
};

const DAY_MS: i64 = 86_400_000;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioDto {
    pub stats: PortfolioStatsDto,
    pub projects: Vec<PortfolioProjectCardDto>,
    pub hidden_projects: Vec<HiddenProjectDto>,
    pub unmatched_sessions: UnmatchedSessionsDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioStatsDto {
    pub projects_tracked: usize,
    pub active_milestones: usize,
    pub sessions_today: u64,
    pub tokens_today: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioProjectCardDto {
    pub id: String,
    pub name: String,
    pub root_path: String,
    pub planning_path: String,
    pub current_milestone_name: Option<String>,
    pub current_phase_number: Option<String>,
    pub current_phase_name: Option<String>,
    pub milestone_progress_pct: f64,
    pub next_command: String,
    pub parse_error: Option<String>,
    pub last_activity_at: Option<i64>,
    pub last_scanned_at: i64,
    pub session_sparkline_7d: Vec<SessionSparklineDayDto>,
    pub sessions_last_7d: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSparklineDayDto {
    pub date: String,
    pub count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HiddenProjectDto {
    pub id: String,
    pub name: String,
    pub root_path: String,
    pub next_command: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnmatchedSessionsDto {
    pub count: u64,
    pub label: String,
    pub claude_count: i64,
    pub codex_count: i64,
    pub recent: Vec<RecentUnmatchedSessionDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentUnmatchedSessionDto {
    pub id: String,
    pub source: String,
    pub source_path: String,
    pub started_at: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDetailDto {
    pub id: String,
    pub name: String,
    pub root_path: String,
    pub planning_path: String,
    pub current_milestone_name: Option<String>,
    pub current_phase_number: Option<String>,
    pub current_phase_name: Option<String>,
    pub milestone_progress_pct: f64,
    pub next_command: String,
    pub parse_error: Option<String>,
    pub last_activity_at: Option<i64>,
    pub last_scanned_at: i64,
}

#[tauri::command]
pub async fn get_portfolio(state: State<'_, AppState>) -> Result<PortfolioDto, AppError> {
    get_portfolio_for_app(&state).await
}

#[tauri::command]
pub async fn get_project(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<ProjectDetailDto, AppError> {
    get_project_for_app(&state, &project_id).await
}

pub async fn get_portfolio_for_app(state: &AppState) -> Result<PortfolioDto, AppError> {
    let app_settings = settings::load_or_initialize(&state.pool, &state.home_dir).await?;
    let hidden_project_ids = app_settings
        .hidden_project_ids
        .into_iter()
        .collect::<HashSet<_>>();
    let connection = state.pool.get().await.map_err(AppError::store)?;
    let snapshots = connection
        .interact(project_repo::list_project_snapshots)
        .await
        .map_err(AppError::store)??;

    let mut projects = Vec::new();
    let mut hidden_projects = Vec::new();

    for snapshot in snapshots {
        if hidden_project_ids.contains(&snapshot.id) {
            hidden_projects.push(HiddenProjectDto::from(snapshot));
        } else {
            projects.push(PortfolioProjectCardDto::from(snapshot));
        }
    }

    let visible_project_ids = projects
        .iter()
        .map(|project| project.id.clone())
        .collect::<Vec<_>>();
    let now_ms = current_epoch_ms();
    let today_start_ms = now_ms - (now_ms % DAY_MS);
    let seven_days_start_ms = today_start_ms - (6 * DAY_MS);
    let session_summary = connection
        .interact(move |connection| {
            sessions::repo::load_portfolio_session_summary(
                connection,
                &visible_project_ids,
                today_start_ms,
                seven_days_start_ms,
            )
        })
        .await
        .map_err(AppError::store)??;

    for project in &mut projects {
        let buckets = session_summary
            .sparkline_by_project
            .get(&project.id)
            .copied()
            .unwrap_or([0; 7]);
        project.sessions_last_7d = buckets.iter().sum();
        project.session_sparkline_7d = buckets
            .iter()
            .enumerate()
            .map(|(index, count)| SessionSparklineDayDto {
                date: (seven_days_start_ms + (index as i64 * DAY_MS)).to_string(),
                count: *count,
            })
            .collect();
    }

    let stats = PortfolioStatsDto {
        projects_tracked: projects.len(),
        active_milestones: projects
            .iter()
            .filter(|project| project.current_milestone_name.is_some())
            .count(),
        sessions_today: session_summary.sessions_today.max(0) as u64,
        tokens_today: session_summary.tokens_today.max(0) as u64,
    };
    let unmatched_count = session_summary.unmatched_count.max(0) as u64;

    Ok(PortfolioDto {
        stats,
        projects,
        hidden_projects,
        unmatched_sessions: UnmatchedSessionsDto {
            count: unmatched_count,
            label: if unmatched_count == 0 {
                "No unmatched sessions".to_string()
            } else {
                format!("{unmatched_count} unmatched sessions")
            },
            claude_count: session_summary.unmatched_claude_count,
            codex_count: session_summary.unmatched_codex_count,
            recent: session_summary
                .recent_unmatched
                .into_iter()
                .map(RecentUnmatchedSessionDto::from)
                .collect(),
        },
    })
}

pub async fn get_project_for_app(
    state: &AppState,
    project_id: &str,
) -> Result<ProjectDetailDto, AppError> {
    let project_id = project_id.to_string();
    let connection = state.pool.get().await.map_err(AppError::store)?;
    let snapshot = connection
        .interact(move |connection| project_repo::load_project_by_id(connection, &project_id))
        .await
        .map_err(AppError::store)??
        .ok_or_else(|| AppError::store("project not found"))?;

    Ok(ProjectDetailDto::from(snapshot))
}

impl From<StoredProjectSnapshot> for PortfolioProjectCardDto {
    fn from(snapshot: StoredProjectSnapshot) -> Self {
        Self {
            id: snapshot.id,
            name: snapshot.name,
            root_path: snapshot.root_path,
            planning_path: snapshot.planning_path,
            current_milestone_name: snapshot.current_milestone_name,
            current_phase_number: snapshot.current_phase_number,
            current_phase_name: snapshot.current_phase_name,
            milestone_progress_pct: snapshot.milestone_progress_pct,
            next_command: snapshot.next_command,
            parse_error: snapshot.parse_error,
            last_activity_at: snapshot.last_activity_at,
            last_scanned_at: snapshot.last_scanned_at,
            session_sparkline_7d: Vec::new(),
            sessions_last_7d: 0,
        }
    }
}

impl From<UnmatchedSessionSummary> for RecentUnmatchedSessionDto {
    fn from(session: UnmatchedSessionSummary) -> Self {
        Self {
            id: session.id,
            source: match session.source {
                SessionSource::Claude => "claude".to_string(),
                SessionSource::Codex => "codex".to_string(),
            },
            source_path: session.source_path,
            started_at: session.started_at,
        }
    }
}

fn current_epoch_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}

impl From<StoredProjectSnapshot> for HiddenProjectDto {
    fn from(snapshot: StoredProjectSnapshot) -> Self {
        Self {
            id: snapshot.id,
            name: snapshot.name,
            root_path: snapshot.root_path,
            next_command: snapshot.next_command,
        }
    }
}

impl From<StoredProjectSnapshot> for ProjectDetailDto {
    fn from(snapshot: StoredProjectSnapshot) -> Self {
        Self {
            id: snapshot.id,
            name: snapshot.name,
            root_path: snapshot.root_path,
            planning_path: snapshot.planning_path,
            current_milestone_name: snapshot.current_milestone_name,
            current_phase_number: snapshot.current_phase_number,
            current_phase_name: snapshot.current_phase_name,
            milestone_progress_pct: snapshot.milestone_progress_pct,
            next_command: snapshot.next_command,
            parse_error: snapshot.parse_error,
            last_activity_at: snapshot.last_activity_at,
            last_scanned_at: snapshot.last_scanned_at,
        }
    }
}
