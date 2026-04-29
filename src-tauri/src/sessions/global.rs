use rusqlite::{params_from_iter, types::Value};
use serde::{Deserialize, Serialize};

use crate::error::AppError;

const DEFAULT_PAGE_SIZE: i64 = 100;
const MAX_PAGE_SIZE: i64 = 200;

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalSessionFilters {
    pub source: Option<String>,
    pub project_id: Option<String>,
    pub started_after: Option<i64>,
    pub started_before: Option<i64>,
    pub duration_min_ms: Option<i64>,
    pub duration_max_ms: Option<i64>,
    pub tokens_min: Option<i64>,
    pub tokens_max: Option<i64>,
    pub unmatched_only: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalSessionsPageDto {
    pub rows: Vec<GlobalSessionRowDto>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalSessionRowDto {
    pub id: String,
    pub source: String,
    pub project_id: Option<String>,
    pub project_name: Option<String>,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalChartDataDto {
    pub sessions_per_day_by_source: Vec<GlobalSessionsBySourceDayDto>,
    pub tokens_per_day_by_project: Vec<GlobalTokensByProjectDayDto>,
    pub time_of_day_histogram: Vec<GlobalHistogramBucketDto>,
    pub day_of_week_distribution: Vec<GlobalDayOfWeekBucketDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalSessionsBySourceDayDto {
    pub date: String,
    pub claude: i64,
    pub codex: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalTokensByProjectDayDto {
    pub date: String,
    pub project_id: Option<String>,
    pub project_name: String,
    pub tokens: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalHistogramBucketDto {
    pub hour: i64,
    pub count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalDayOfWeekBucketDto {
    pub day: i64,
    pub count: i64,
}

pub fn list_global_sessions(
    connection: &mut rusqlite::Connection,
    filters: &GlobalSessionFilters,
    page: Option<i64>,
    page_size: Option<i64>,
) -> Result<GlobalSessionsPageDto, AppError> {
    let source = validated_source(filters.source.as_deref())?;
    let unmatched_only = filters.unmatched_only.unwrap_or(false) as i64;
    let page = page.unwrap_or(1).max(1);
    let page_size = page_size
        .unwrap_or(DEFAULT_PAGE_SIZE)
        .clamp(1, MAX_PAGE_SIZE);
    let offset = (page - 1) * page_size;
    let filter = build_filter_sql(filters, source, unmatched_only);

    let total = connection
        .query_row(
            &format!(
                "SELECT COUNT(*) FROM {} {}",
                filter.table_sql, filter.where_sql
            ),
            params_from_iter(filter.values.clone()),
            |row| row.get(0),
        )
        .map_err(AppError::from)?;

    let mut values = filter.values;
    values.push(Value::Integer(page_size));
    values.push(Value::Integer(offset));
    let mut statement = connection
        .prepare(&format!(
            "SELECT s.id,
                    s.source,
                    s.project_id,
                    projects.name,
                    s.source_path,
                    s.started_at,
                    s.ended_at,
                    s.duration_ms,
                    COALESCE(s.message_count, 0),
                    COALESCE(s.tokens_in, 0),
                    COALESCE(s.tokens_out, 0),
                    {},
                    s.model
             FROM {}
             LEFT JOIN projects ON projects.id = s.project_id
             {}
             ORDER BY s.started_at DESC NULLS LAST, s.id ASC
             LIMIT ? OFFSET ?",
            token_total_sql(),
            filter.table_sql,
            filter.where_sql,
        ))
        .map_err(AppError::from)?;
    let rows = statement
        .query_map(params_from_iter(values), |row| {
            Ok(GlobalSessionRowDto {
                id: row.get(0)?,
                source: row.get(1)?,
                project_id: row.get(2)?,
                project_name: row.get(3)?,
                source_path: row.get(4)?,
                started_at: row.get(5)?,
                ended_at: row.get(6)?,
                duration_ms: row.get(7)?,
                message_count: row.get(8)?,
                tokens_in: row.get(9)?,
                tokens_out: row.get(10)?,
                token_total: row.get(11)?,
                model: row.get(12)?,
            })
        })
        .map_err(AppError::from)?;

    Ok(GlobalSessionsPageDto {
        rows: rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(AppError::from)?,
        total,
        page,
        page_size,
    })
}

pub fn load_global_chart_data(
    connection: &mut rusqlite::Connection,
    filters: &GlobalSessionFilters,
) -> Result<GlobalChartDataDto, AppError> {
    let source = validated_source(filters.source.as_deref())?;
    let unmatched_only = filters.unmatched_only.unwrap_or(false) as i64;

    Ok(GlobalChartDataDto {
        sessions_per_day_by_source: load_sessions_per_day_by_source(
            connection,
            filters,
            source,
            unmatched_only,
        )?,
        tokens_per_day_by_project: load_tokens_per_day_by_project(
            connection,
            filters,
            source,
            unmatched_only,
        )?,
        time_of_day_histogram: load_time_of_day_histogram(
            connection,
            filters,
            source,
            unmatched_only,
        )?,
        day_of_week_distribution: load_day_of_week_distribution(
            connection,
            filters,
            source,
            unmatched_only,
        )?,
    })
}

fn load_sessions_per_day_by_source(
    connection: &mut rusqlite::Connection,
    filters: &GlobalSessionFilters,
    source: Option<&str>,
    unmatched_only: i64,
) -> Result<Vec<GlobalSessionsBySourceDayDto>, AppError> {
    let filter = build_filter_sql(filters, source, unmatched_only);
    let mut statement = connection
        .prepare(&format!(
            "SELECT date(s.started_at / 1000, 'unixepoch', 'localtime'),
                    COALESCE(SUM(CASE WHEN s.source = 'claude' THEN 1 ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN s.source = 'codex' THEN 1 ELSE 0 END), 0)
             FROM {}
             {}
               AND s.started_at IS NOT NULL
             GROUP BY 1
             ORDER BY 1",
            filter.table_sql, filter.where_sql,
        ))
        .map_err(AppError::from)?;
    let rows = statement
        .query_map(params_from_iter(filter.values), |row| {
            Ok(GlobalSessionsBySourceDayDto {
                date: row.get(0)?,
                claude: row.get(1)?,
                codex: row.get(2)?,
            })
        })
        .map_err(AppError::from)?;

    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

fn load_tokens_per_day_by_project(
    connection: &mut rusqlite::Connection,
    filters: &GlobalSessionFilters,
    source: Option<&str>,
    unmatched_only: i64,
) -> Result<Vec<GlobalTokensByProjectDayDto>, AppError> {
    let filter = build_filter_sql(filters, source, unmatched_only);
    let mut statement = connection
        .prepare(&format!(
            "WITH filtered AS (
                 SELECT s.id,
                        s.project_id,
                        s.started_at,
                        {} AS token_total
                 FROM {}
                 {}
                   AND s.started_at IS NOT NULL
             ),
             top_projects AS (
                 SELECT project_id
                 FROM filtered
                 WHERE project_id IS NOT NULL
                 GROUP BY project_id
                 ORDER BY SUM(token_total) DESC, project_id ASC
                 LIMIT 5
             )
             SELECT date(filtered.started_at / 1000, 'unixepoch', 'localtime'),
                    CASE
                        WHEN filtered.project_id IN (SELECT project_id FROM top_projects)
                        THEN filtered.project_id
                        ELSE NULL
                    END AS bucket_project_id,
                    CASE
                        WHEN filtered.project_id IN (SELECT project_id FROM top_projects)
                        THEN COALESCE(projects.name, filtered.project_id)
                        ELSE 'Other'
                    END AS bucket_project_name,
                    COALESCE(SUM(filtered.token_total), 0)
             FROM filtered
             LEFT JOIN projects ON projects.id = filtered.project_id
             GROUP BY 1, 2, 3
             ORDER BY 1, 3",
            token_total_sql(),
            filter.table_sql,
            filter.where_sql,
        ))
        .map_err(AppError::from)?;
    let rows = statement
        .query_map(params_from_iter(filter.values), |row| {
            Ok(GlobalTokensByProjectDayDto {
                date: row.get(0)?,
                project_id: row.get(1)?,
                project_name: row.get(2)?,
                tokens: row.get(3)?,
            })
        })
        .map_err(AppError::from)?;

    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

fn load_time_of_day_histogram(
    connection: &mut rusqlite::Connection,
    filters: &GlobalSessionFilters,
    source: Option<&str>,
    unmatched_only: i64,
) -> Result<Vec<GlobalHistogramBucketDto>, AppError> {
    let filter = build_filter_sql(filters, source, unmatched_only);
    let mut counts = [0_i64; 24];
    let mut statement = connection
        .prepare(&format!(
            "SELECT CAST(strftime('%H', s.started_at / 1000, 'unixepoch', 'localtime') AS INTEGER),
                    COUNT(*)
             FROM {}
             {}
               AND s.started_at IS NOT NULL
             GROUP BY 1",
            filter.table_sql, filter.where_sql,
        ))
        .map_err(AppError::from)?;
    let rows = statement
        .query_map(params_from_iter(filter.values), |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
        })
        .map_err(AppError::from)?;

    for row in rows {
        let (hour, count) = row.map_err(AppError::from)?;
        if (0..24).contains(&hour) {
            counts[hour as usize] = count;
        }
    }

    Ok(counts
        .iter()
        .enumerate()
        .map(|(hour, count)| GlobalHistogramBucketDto {
            hour: hour as i64,
            count: *count,
        })
        .collect())
}

fn load_day_of_week_distribution(
    connection: &mut rusqlite::Connection,
    filters: &GlobalSessionFilters,
    source: Option<&str>,
    unmatched_only: i64,
) -> Result<Vec<GlobalDayOfWeekBucketDto>, AppError> {
    let filter = build_filter_sql(filters, source, unmatched_only);
    let mut counts = [0_i64; 7];
    let mut statement = connection
        .prepare(&format!(
            "SELECT CAST(strftime('%w', s.started_at / 1000, 'unixepoch', 'localtime') AS INTEGER),
                    COUNT(*)
             FROM {}
             {}
               AND s.started_at IS NOT NULL
             GROUP BY 1",
            filter.table_sql, filter.where_sql,
        ))
        .map_err(AppError::from)?;
    let rows = statement
        .query_map(params_from_iter(filter.values), |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
        })
        .map_err(AppError::from)?;

    for row in rows {
        let (day, count) = row.map_err(AppError::from)?;
        if (0..7).contains(&day) {
            counts[day as usize] = count;
        }
    }

    Ok(counts
        .iter()
        .enumerate()
        .map(|(day, count)| GlobalDayOfWeekBucketDto {
            day: day as i64,
            count: *count,
        })
        .collect())
}

fn validated_source(source: Option<&str>) -> Result<Option<&str>, AppError> {
    match source {
        Some("claude" | "codex") | None => Ok(source),
        Some(_) => Err(AppError::store("invalid session source")),
    }
}

fn token_total_sql() -> &'static str {
    "COALESCE(s.tokens_in, 0)
        + COALESCE(s.tokens_out, 0)
        + COALESCE(s.cache_read_tokens, 0)
        + COALESCE(s.cache_creation_tokens, 0)"
}

#[derive(Debug, Clone)]
struct FilterSql {
    table_sql: &'static str,
    where_sql: String,
    values: Vec<Value>,
}

fn build_filter_sql(
    filters: &GlobalSessionFilters,
    source: Option<&str>,
    unmatched_only: i64,
) -> FilterSql {
    let mut where_sql = String::from("WHERE 1 = 1");
    let mut values = Vec::new();
    let table_sql = if unmatched_only != 0 {
        "sessions s INDEXED BY idx_sessions_unmatched_started"
    } else {
        "sessions s"
    };

    if let Some(source) = source {
        where_sql.push_str(" AND s.source = ?");
        values.push(Value::Text(source.to_string()));
    }
    if let Some(project_id) = &filters.project_id {
        where_sql.push_str(" AND s.project_id = ?");
        values.push(Value::Text(project_id.clone()));
    }
    if let Some(started_after) = filters.started_after {
        where_sql.push_str(" AND s.started_at >= ?");
        values.push(Value::Integer(started_after));
    }
    if let Some(started_before) = filters.started_before {
        where_sql.push_str(" AND s.started_at < ?");
        values.push(Value::Integer(started_before));
    }
    if let Some(duration_min_ms) = filters.duration_min_ms {
        where_sql.push_str(" AND s.duration_ms >= ?");
        values.push(Value::Integer(duration_min_ms));
    }
    if let Some(duration_max_ms) = filters.duration_max_ms {
        where_sql.push_str(" AND s.duration_ms <= ?");
        values.push(Value::Integer(duration_max_ms));
    }
    if let Some(tokens_min) = filters.tokens_min {
        where_sql.push_str(" AND (");
        where_sql.push_str(token_total_sql());
        where_sql.push_str(") >= ?");
        values.push(Value::Integer(tokens_min));
    }
    if let Some(tokens_max) = filters.tokens_max {
        where_sql.push_str(" AND (");
        where_sql.push_str(token_total_sql());
        where_sql.push_str(") <= ?");
        values.push(Value::Integer(tokens_max));
    }
    if unmatched_only != 0 {
        where_sql.push_str(" AND s.project_id IS NULL");
    }

    FilterSql {
        table_sql,
        where_sql,
        values,
    }
}
