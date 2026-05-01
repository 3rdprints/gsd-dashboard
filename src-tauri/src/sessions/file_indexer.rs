use std::{
    fs::File,
    io::{BufRead, BufReader, Seek},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use deadpool_sqlite::Pool;
use serde_json::Value;

use crate::{
    error::AppError,
    sessions::{
        claude::parse_claude_record,
        codex::parse_codex_record,
        matcher::match_project,
        repo::{load_indexed_session, persist_indexed_file_result},
        IndexedSession, ProjectRoot, SessionIndexState, SessionParseAccumulator, SessionSource,
        StreamFileStatus,
    },
    store::project_repo,
};

const LIVE_PARTIAL_MESSAGE: &str = "Live session still writing";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedFileResult {
    pub(crate) sessions_persisted: usize,
    pub(crate) live_partial: bool,
    pub(crate) session_changes: Vec<IndexedSessionChange>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedSessionChange {
    pub(crate) id: String,
    pub(crate) project_id: Option<String>,
}

pub fn stream_session_file(
    source: SessionSource,
    path: &Path,
    state: Option<&SessionIndexState>,
) -> Result<(SessionParseAccumulator, StreamFileStatus), AppError> {
    let source_path = path.display().to_string();
    let mut file = File::open(path).map_err(AppError::io)?;
    let metadata = file.metadata().map_err(AppError::io)?;
    let file_size = metadata.len() as i64;
    let starting_offset = match state {
        Some(state)
            if state.last_parsed_byte_offset >= 0 && file_size >= state.last_parsed_byte_offset =>
        {
            state.last_parsed_byte_offset as u64
        }
        _ => 0,
    };

    file.seek(std::io::SeekFrom::Start(starting_offset))
        .map_err(AppError::io)?;

    let mut reader = BufReader::new(file);
    let mut committed_offset = starting_offset as i64;
    let mut accumulator = SessionParseAccumulator {
        session: empty_session(source, source_path),
        nonfatal_error_count: 0,
        live_partial_message: None,
    };
    let mut line = Vec::new();

    loop {
        line.clear();
        let bytes_read = reader.read_until(b'\n', &mut line).map_err(AppError::io)?;
        if bytes_read == 0 {
            break;
        }

        let has_newline = line.ends_with(b"\n");
        let record_bytes = trim_jsonl_newline(&line);
        if record_bytes.is_empty() {
            committed_offset += bytes_read as i64;
            continue;
        }

        if !has_newline {
            accumulator.live_partial_message = Some(LIVE_PARTIAL_MESSAGE.to_string());
            finalize_accumulator(source, path, &mut accumulator);
            return Ok((
                accumulator,
                StreamFileStatus::LivePartial {
                    committed_offset,
                    message: LIVE_PARTIAL_MESSAGE.to_string(),
                },
            ));
        }

        match serde_json::from_slice::<Value>(record_bytes) {
            Ok(value) => {
                match source {
                    SessionSource::Claude => parse_claude_record(&value, &mut accumulator),
                    SessionSource::Codex => parse_codex_record(&value, &mut accumulator),
                }
                committed_offset += bytes_read as i64;
            }
            Err(error) => {
                accumulator.nonfatal_error_count += 1;
                accumulator.session.index_error = Some(error.to_string());
                committed_offset += bytes_read as i64;
            }
        }
    }

    finalize_accumulator(source, path, &mut accumulator);

    Ok((
        accumulator,
        StreamFileStatus::Complete {
            committed_offset: committed_offset.min(file_size),
        },
    ))
}

pub(crate) async fn index_session_file(
    pool: &Pool,
    source: SessionSource,
    source_path: PathBuf,
    known_projects: &[ProjectRoot],
) -> Result<IndexedFileResult, AppError> {
    let source_path_string = source_path.display().to_string();
    let previous_state = load_previous_index_state(pool, source_path_string).await?;
    let previous_offset = previous_state
        .as_ref()
        .map(|state| state.last_parsed_byte_offset)
        .unwrap_or(0);
    let (mut accumulator, status, file_state) = tokio::task::spawn_blocking({
        let source_path = source_path.clone();
        let previous_state = previous_state.clone();
        move || {
            let (accumulator, status) =
                stream_session_file(source, &source_path, previous_state.as_ref())?;
            let committed_offset = committed_offset_from_status(&status);
            let live_partial = matches!(status, StreamFileStatus::LivePartial { .. });
            let file_state = build_index_state(
                source,
                &source_path,
                committed_offset,
                live_partial,
                accumulator.session.index_error.clone(),
            )?;

            Ok::<_, AppError>((accumulator, status, file_state))
        }
    })
    .await
    .map_err(AppError::io)??;
    let committed_offset = committed_offset_from_status(&status);
    let live_partial = matches!(status, StreamFileStatus::LivePartial { .. });
    let offset_was_reset = previous_offset > file_state.file_size;
    let should_persist_session = (offset_was_reset || committed_offset > previous_offset)
        && accumulator.session.message_count > 0;
    let mut skipped_unmatched_session = false;
    let sessions = if should_persist_session {
        if previous_offset > 0 && !offset_was_reset {
            if let Some(previous_session) =
                load_previous_session(pool, accumulator.session.id.clone()).await?
            {
                accumulator.session =
                    merge_incremental_session(previous_session, accumulator.session);
            }
        }
        let mut session = accumulator.session;
        let known_projects = known_projects.to_vec();
        let session = tokio::task::spawn_blocking(move || {
            match_project(&mut session, &known_projects);
            session
        })
        .await
        .map_err(AppError::io)?;
        if session.project_id.is_some() {
            vec![session]
        } else {
            skipped_unmatched_session = true;
            Vec::new()
        }
    } else {
        Vec::new()
    };

    if skipped_unmatched_session {
        return Ok(IndexedFileResult {
            sessions_persisted: 0,
            live_partial,
            session_changes: Vec::new(),
        });
    }

    if committed_offset == previous_offset && sessions.is_empty() {
        return Ok(IndexedFileResult {
            sessions_persisted: 0,
            live_partial,
            session_changes: Vec::new(),
        });
    }

    let now = current_unix_seconds();
    let sessions_persisted = sessions.len();
    let session_changes = sessions
        .iter()
        .map(|session| IndexedSessionChange {
            id: session.id.clone(),
            project_id: session.project_id.clone(),
        })
        .collect::<Vec<_>>();
    let connection = pool.get().await.map_err(AppError::store)?;
    connection
        .interact(move |connection| {
            persist_indexed_file_result(connection, &sessions, &file_state, now)
        })
        .await
        .map_err(AppError::store)??;

    Ok(IndexedFileResult {
        sessions_persisted,
        live_partial,
        session_changes,
    })
}

pub(crate) async fn load_known_project_roots(pool: &Pool) -> Result<Vec<ProjectRoot>, AppError> {
    let connection = pool.get().await.map_err(AppError::store)?;
    connection
        .interact(|connection| {
            project_repo::list_project_snapshots(connection).map(|projects| {
                projects
                    .into_iter()
                    .map(|project| ProjectRoot {
                        id: project.id,
                        root_path: project.root_path,
                    })
                    .collect::<Vec<_>>()
            })
        })
        .await
        .map_err(AppError::store)?
}

async fn load_previous_index_state(
    pool: &Pool,
    source_path: String,
) -> Result<Option<SessionIndexState>, AppError> {
    let connection = pool.get().await.map_err(AppError::store)?;
    connection
        .interact(move |connection| {
            crate::sessions::repo::load_index_state(connection, &source_path)
        })
        .await
        .map_err(AppError::store)?
}

async fn load_previous_session(
    pool: &Pool,
    session_id: String,
) -> Result<Option<IndexedSession>, AppError> {
    let connection = pool.get().await.map_err(AppError::store)?;
    connection
        .interact(move |connection| load_indexed_session(connection, &session_id))
        .await
        .map_err(AppError::store)?
}

fn committed_offset_from_status(status: &StreamFileStatus) -> i64 {
    match status {
        StreamFileStatus::Complete { committed_offset } => *committed_offset,
        StreamFileStatus::LivePartial {
            committed_offset, ..
        } => *committed_offset,
    }
}

fn build_index_state(
    source: SessionSource,
    source_path: &Path,
    committed_offset: i64,
    live_partial: bool,
    last_error: Option<String>,
) -> Result<SessionIndexState, AppError> {
    let metadata = std::fs::metadata(source_path).map_err(AppError::io)?;
    let file_mtime = metadata
        .modified()
        .ok()
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .and_then(|duration| duration.as_secs().try_into().ok());

    Ok(SessionIndexState {
        source_path: source_path.display().to_string(),
        source,
        file_size: metadata.len() as i64,
        file_mtime,
        last_parsed_byte_offset: committed_offset,
        live_partial,
        last_error,
    })
}

fn current_unix_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

fn merge_incremental_session(
    previous_session: IndexedSession,
    delta_session: IndexedSession,
) -> IndexedSession {
    let started_at = min_option(previous_session.started_at, delta_session.started_at);
    let ended_at = max_option(previous_session.ended_at, delta_session.ended_at);

    IndexedSession {
        id: delta_session.id,
        source: delta_session.source,
        source_path: delta_session.source_path,
        source_session_id: delta_session
            .source_session_id
            .or(previous_session.source_session_id),
        project_id: previous_session.project_id,
        cwd: delta_session.cwd.or(previous_session.cwd),
        started_at,
        ended_at,
        duration_ms: started_at.zip(ended_at).map(|(start, end)| end - start),
        message_count: previous_session.message_count + delta_session.message_count,
        tokens_in: sum_options(previous_session.tokens_in, delta_session.tokens_in),
        tokens_out: sum_options(previous_session.tokens_out, delta_session.tokens_out),
        model: delta_session.model.or(previous_session.model),
        attribution_method: previous_session.attribution_method,
        index_error: delta_session.index_error.or(previous_session.index_error),
    }
}

fn min_option(left: Option<i64>, right: Option<i64>) -> Option<i64> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.min(right)),
        (Some(value), None) | (None, Some(value)) => Some(value),
        (None, None) => None,
    }
}

fn max_option(left: Option<i64>, right: Option<i64>) -> Option<i64> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.max(right)),
        (Some(value), None) | (None, Some(value)) => Some(value),
        (None, None) => None,
    }
}

fn sum_options(left: Option<i64>, right: Option<i64>) -> Option<i64> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left + right),
        (Some(value), None) | (None, Some(value)) => Some(value),
        (None, None) => None,
    }
}

fn empty_session(source: SessionSource, source_path: String) -> IndexedSession {
    IndexedSession {
        id: session_id(source, &None, Path::new(&source_path)),
        source,
        source_path,
        source_session_id: None,
        project_id: None,
        cwd: None,
        started_at: None,
        ended_at: None,
        duration_ms: None,
        message_count: 0,
        tokens_in: None,
        tokens_out: None,
        model: None,
        attribution_method: "unmatched".to_string(),
        index_error: None,
    }
}

fn session_id(source: SessionSource, source_session_id: &Option<String>, path: &Path) -> String {
    source_session_id.as_ref().map_or_else(
        || format!("{}:{}", source.as_str(), path.display()),
        |source_session_id| format!("{}:{source_session_id}", source.as_str()),
    )
}

fn finalize_accumulator(
    source: SessionSource,
    path: &Path,
    accumulator: &mut SessionParseAccumulator,
) {
    if accumulator.session.source_session_id.is_none() {
        accumulator.session.source_session_id = path
            .file_stem()
            .and_then(|name| name.to_str())
            .map(str::to_string);
    }
    accumulator.session.id = session_id(source, &accumulator.session.source_session_id, path);

    if accumulator.session.index_error.is_none() && accumulator.nonfatal_error_count > 0 {
        accumulator.session.index_error = Some(format!(
            "{} nonfatal parse errors",
            accumulator.nonfatal_error_count
        ));
    }
}

fn trim_jsonl_newline(line: &[u8]) -> &[u8] {
    let without_lf = line.strip_suffix(b"\n").unwrap_or(line);
    without_lf.strip_suffix(b"\r").unwrap_or(without_lf)
}
