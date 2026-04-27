pub mod claude;
pub mod codex;
pub mod global;
pub mod indexer;
pub mod matcher;
pub mod project_charts;
pub mod project_detail;
pub mod repo;

use serde_json::Value;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionSource {
    Claude,
    Codex,
}

impl SessionSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
        }
    }
}

impl TryFrom<&str> for SessionSource {
    type Error = crate::error::AppError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "claude" => Ok(Self::Claude),
            "codex" => Ok(Self::Codex),
            _ => Err(crate::error::AppError::store(format!(
                "unknown session source: {value}"
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedSession {
    pub id: String,
    pub source: SessionSource,
    pub source_path: String,
    pub source_session_id: Option<String>,
    pub project_id: Option<String>,
    pub cwd: Option<String>,
    pub started_at: Option<i64>,
    pub ended_at: Option<i64>,
    pub duration_ms: Option<i64>,
    pub message_count: i64,
    pub tokens_in: Option<i64>,
    pub tokens_out: Option<i64>,
    pub model: Option<String>,
    pub attribution_method: String,
    pub index_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionIndexState {
    pub source_path: String,
    pub source: SessionSource,
    pub file_size: i64,
    pub file_mtime: Option<i64>,
    pub last_parsed_byte_offset: i64,
    pub live_partial: bool,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectRoot {
    pub id: String,
    pub root_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamFileStatus {
    Complete {
        committed_offset: i64,
    },
    LivePartial {
        committed_offset: i64,
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionParseAccumulator {
    pub session: IndexedSession,
    pub nonfatal_error_count: i64,
    pub live_partial_message: Option<String>,
}

pub fn parse_timestamp_ms(value: &Value) -> Option<i64> {
    if let Some(timestamp) = value.as_str() {
        return OffsetDateTime::parse(timestamp, &Rfc3339)
            .ok()
            .and_then(|datetime| {
                (datetime.unix_timestamp_nanos() / 1_000_000)
                    .try_into()
                    .ok()
            });
    }

    let number = value.as_i64().or_else(|| value.as_u64()?.try_into().ok())?;
    if number > 10_000_000_000 {
        Some(number)
    } else {
        number.checked_mul(1_000)
    }
}

pub fn apply_record_timestamp(accumulator: &mut SessionParseAccumulator, timestamp_ms: i64) {
    if accumulator.session.started_at.is_none() {
        accumulator.session.started_at = Some(timestamp_ms);
    }
    accumulator.session.ended_at = Some(timestamp_ms);

    if let (Some(started_at), Some(ended_at)) =
        (accumulator.session.started_at, accumulator.session.ended_at)
    {
        accumulator.session.duration_ms = Some(ended_at - started_at);
    }
}

pub fn add_token_count(target: &mut Option<i64>, value: Option<i64>) {
    if let Some(value) = value {
        *target = Some(target.unwrap_or(0) + value);
    }
}

pub fn json_i64(value: Option<&Value>) -> Option<i64> {
    value.and_then(|value| value.as_i64().or_else(|| value.as_u64()?.try_into().ok()))
}

pub fn json_string(value: Option<&Value>) -> Option<String> {
    value
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}
