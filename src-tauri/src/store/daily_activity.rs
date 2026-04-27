use rusqlite::{params, OptionalExtension};

use crate::error::AppError;

const DAY_MS: i64 = 86_400_000;
const MIN_DAYS: i64 = 1;
const MAX_DAYS: i64 = 365;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DailyActivityRow {
    pub date: String,
    pub session_count: i64,
    pub token_total: i64,
    pub top_project_id: Option<String>,
    pub top_project_name: Option<String>,
}

pub fn rebuild_window(
    connection: &mut rusqlite::Connection,
    days: i64,
    now_ms: i64,
) -> Result<(), AppError> {
    let days = clamp_days(days);
    let today_start_ms = now_ms - now_ms.rem_euclid(DAY_MS);
    let window_start_ms = today_start_ms - ((days - 1) * DAY_MS);
    let updated_at = now_ms / 1_000;
    let transaction = connection.transaction().map_err(AppError::from)?;

    transaction
        .execute(
            "DELETE FROM daily_activity
             WHERE date >= date(?1 / 1000, 'unixepoch', 'localtime')
               AND date <= date(?2 / 1000, 'unixepoch', 'localtime')",
            params![window_start_ms, today_start_ms],
        )
        .map_err(AppError::from)?;

    transaction
        .execute(
            "INSERT INTO daily_activity (date, session_count, token_total, top_project_id, updated_at)
             WITH session_days AS (
                 SELECT date(started_at / 1000, 'unixepoch', 'localtime') AS date,
                        project_id,
                        COALESCE(tokens_in, 0)
                            + COALESCE(tokens_out, 0)
                            + COALESCE(cache_read_tokens, 0)
                            + COALESCE(cache_creation_tokens, 0) AS token_total
                 FROM sessions
                 WHERE started_at >= ?1
                   AND started_at < ?2 + ?3
             ),
             day_totals AS (
                 SELECT date,
                        COUNT(*) AS session_count,
                        COALESCE(SUM(token_total), 0) AS token_total
                 FROM session_days
                 GROUP BY date
             ),
             project_rank AS (
                 SELECT date,
                        project_id,
                        COUNT(*) AS project_session_count,
                        ROW_NUMBER() OVER (
                            PARTITION BY date
                            ORDER BY COUNT(*) DESC, project_id ASC
                        ) AS row_number
                 FROM session_days
                 WHERE project_id IS NOT NULL
                 GROUP BY date, project_id
             )
             SELECT day_totals.date,
                    day_totals.session_count,
                    day_totals.token_total,
                    project_rank.project_id,
                    ?4
             FROM day_totals
             LEFT JOIN project_rank
               ON project_rank.date = day_totals.date
              AND project_rank.row_number = 1",
            params![window_start_ms, today_start_ms, DAY_MS, updated_at],
        )
        .map_err(AppError::from)?;

    transaction.commit().map_err(AppError::from)
}

pub fn load_window(
    connection: &mut rusqlite::Connection,
    days: i64,
) -> Result<Vec<DailyActivityRow>, AppError> {
    let days = clamp_days(days);
    let end_date = latest_activity_date(connection)?.unwrap_or_else(current_local_date);
    let start_date = end_date - time::Duration::days(days - 1);
    let mut rows = Vec::with_capacity(days as usize);

    let mut statement = connection
        .prepare(
            "SELECT daily_activity.session_count,
                    daily_activity.token_total,
                    daily_activity.top_project_id,
                    projects.name
             FROM daily_activity
             LEFT JOIN projects ON projects.id = daily_activity.top_project_id
             WHERE daily_activity.date = ?1",
        )
        .map_err(AppError::from)?;

    for offset in 0..days {
        let date = start_date + time::Duration::days(offset);
        let date_key = date.to_string();
        let loaded = statement
            .query_row([date_key.as_str()], |row| {
                Ok(DailyActivityRow {
                    date: date_key.clone(),
                    session_count: row.get(0)?,
                    token_total: row.get(1)?,
                    top_project_id: row.get(2)?,
                    top_project_name: row.get(3)?,
                })
            })
            .optional()
            .map_err(AppError::from)?;

        rows.push(loaded.unwrap_or(DailyActivityRow {
            date: date_key,
            session_count: 0,
            token_total: 0,
            top_project_id: None,
            top_project_name: None,
        }));
    }

    Ok(rows)
}

fn latest_activity_date(
    connection: &mut rusqlite::Connection,
) -> Result<Option<time::Date>, AppError> {
    let latest = connection
        .query_row("SELECT MAX(date) FROM daily_activity", [], |row| {
            row.get::<_, Option<String>>(0)
        })
        .map_err(AppError::from)?;

    latest.map(|date| parse_date_key(&date)).transpose()
}

fn current_local_date() -> time::Date {
    time::OffsetDateTime::now_utc().date()
}

fn parse_date_key(date: &str) -> Result<time::Date, AppError> {
    let mut parts = date.split('-');
    let year = parts
        .next()
        .ok_or_else(|| AppError::store("missing year"))?
        .parse::<i32>()
        .map_err(AppError::store)?;
    let month = parts
        .next()
        .ok_or_else(|| AppError::store("missing month"))?
        .parse::<u8>()
        .map_err(AppError::store)?;
    let day = parts
        .next()
        .ok_or_else(|| AppError::store("missing day"))?
        .parse::<u8>()
        .map_err(AppError::store)?;
    if parts.next().is_some() {
        return Err(AppError::store("invalid date key"));
    }

    let month = time::Month::try_from(month).map_err(AppError::store)?;
    time::Date::from_calendar_date(year, month, day).map_err(AppError::store)
}

fn clamp_days(days: i64) -> i64 {
    days.clamp(MIN_DAYS, MAX_DAYS)
}
