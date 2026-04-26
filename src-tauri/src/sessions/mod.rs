pub mod repo;

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
