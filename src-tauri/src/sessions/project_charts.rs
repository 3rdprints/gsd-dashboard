use rusqlite::params;
use serde::Serialize;

use crate::error::AppError;

const DAY_MS: i64 = 86_400_000;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectChartDataDto {
    pub sessions_per_day: Vec<ProjectDailyCountDto>,
    pub tokens_per_day: Vec<ProjectDailyTokensDto>,
    pub average_duration_per_day: Vec<ProjectDailyAverageDurationDto>,
    pub milestone_velocity: Vec<ProjectMilestoneVelocityDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDailyCountDto {
    pub date: String,
    pub count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDailyTokensDto {
    pub date: String,
    pub tokens: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDailyAverageDurationDto {
    pub date: String,
    pub average_duration_ms: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMilestoneVelocityDto {
    pub week: String,
    pub completed_plans: i64,
}

/// Loads daily session, token, duration, and velocity chart data for a project.
pub fn load_project_chart_data(
    connection: &mut rusqlite::Connection,
    project_id: &str,
    range: Option<&str>,
) -> Result<ProjectChartDataDto, AppError> {
    let lower_bound = range_lower_bound(connection, project_id, range.unwrap_or("30d"))?;

    Ok(ProjectChartDataDto {
        sessions_per_day: load_sessions_per_day(connection, project_id, lower_bound)?,
        tokens_per_day: load_tokens_per_day(connection, project_id, lower_bound)?,
        average_duration_per_day: load_average_duration_per_day(
            connection,
            project_id,
            lower_bound,
        )?,
        milestone_velocity: load_milestone_velocity(connection, project_id, lower_bound)?,
    })
}

fn range_lower_bound(
    connection: &mut rusqlite::Connection,
    project_id: &str,
    range: &str,
) -> Result<Option<i64>, AppError> {
    if range == "all" {
        return Ok(None);
    }
    let days = match range {
        "7d" => 7,
        "30d" => 30,
        "90d" => 90,
        _ => return Err(AppError::store("invalid chart range")),
    };
    let latest = connection
        .query_row(
            "SELECT MAX(started_at) FROM sessions WHERE project_id = ?1",
            [project_id],
            |row| row.get::<_, Option<i64>>(0),
        )
        .map_err(AppError::from)?;

    Ok(latest.map(|started_at| started_at - ((days - 1) * DAY_MS)))
}

fn load_sessions_per_day(
    connection: &mut rusqlite::Connection,
    project_id: &str,
    lower_bound: Option<i64>,
) -> Result<Vec<ProjectDailyCountDto>, AppError> {
    let mut statement = connection
        .prepare(
            "SELECT date(started_at / 1000, 'unixepoch', 'localtime'), COUNT(*)
             FROM sessions
             WHERE project_id = ?1
               AND started_at IS NOT NULL
               AND (?2 IS NULL OR started_at >= ?2)
             GROUP BY 1
             ORDER BY 1",
        )
        .map_err(AppError::from)?;
    let rows = statement
        .query_map(params![project_id, lower_bound], |row| {
            Ok(ProjectDailyCountDto {
                date: row.get(0)?,
                count: row.get(1)?,
            })
        })
        .map_err(AppError::from)?;

    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

fn load_tokens_per_day(
    connection: &mut rusqlite::Connection,
    project_id: &str,
    lower_bound: Option<i64>,
) -> Result<Vec<ProjectDailyTokensDto>, AppError> {
    let mut statement = connection
        .prepare(
            "SELECT date(started_at / 1000, 'unixepoch', 'localtime'),
                    COALESCE(SUM(
                        COALESCE(tokens_in, 0)
                        + COALESCE(tokens_out, 0)
                        + COALESCE(cache_read_tokens, 0)
                        + COALESCE(cache_creation_tokens, 0)
                    ), 0)
             FROM sessions
             WHERE project_id = ?1
               AND started_at IS NOT NULL
               AND (?2 IS NULL OR started_at >= ?2)
             GROUP BY 1
             ORDER BY 1",
        )
        .map_err(AppError::from)?;
    let rows = statement
        .query_map(params![project_id, lower_bound], |row| {
            Ok(ProjectDailyTokensDto {
                date: row.get(0)?,
                tokens: row.get(1)?,
            })
        })
        .map_err(AppError::from)?;

    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

fn load_average_duration_per_day(
    connection: &mut rusqlite::Connection,
    project_id: &str,
    lower_bound: Option<i64>,
) -> Result<Vec<ProjectDailyAverageDurationDto>, AppError> {
    let mut statement = connection
        .prepare(
            "SELECT date(started_at / 1000, 'unixepoch', 'localtime'),
                    AVG(duration_ms)
             FROM sessions
             WHERE project_id = ?1
               AND started_at IS NOT NULL
               AND duration_ms IS NOT NULL
               AND (?2 IS NULL OR started_at >= ?2)
             GROUP BY 1
             ORDER BY 1",
        )
        .map_err(AppError::from)?;
    let rows = statement
        .query_map(params![project_id, lower_bound], |row| {
            Ok(ProjectDailyAverageDurationDto {
                date: row.get(0)?,
                average_duration_ms: row.get(1)?,
            })
        })
        .map_err(AppError::from)?;

    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

fn load_milestone_velocity(
    connection: &mut rusqlite::Connection,
    project_id: &str,
    lower_bound: Option<i64>,
) -> Result<Vec<ProjectMilestoneVelocityDto>, AppError> {
    let mut statement = connection
        .prepare(
            "SELECT strftime('%Y-W%W', completed_at / 1000, 'unixepoch', 'localtime'),
                    COUNT(*)
             FROM phase_plans
             WHERE project_id = ?1
               AND completed_at IS NOT NULL
               AND (?2 IS NULL OR completed_at >= ?2)
             GROUP BY 1
             ORDER BY 1",
        )
        .map_err(AppError::from)?;
    let rows = statement
        .query_map(params![project_id, lower_bound], |row| {
            Ok(ProjectMilestoneVelocityDto {
                week: row.get(0)?,
                completed_plans: row.get(1)?,
            })
        })
        .map_err(AppError::from)?;

    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}
