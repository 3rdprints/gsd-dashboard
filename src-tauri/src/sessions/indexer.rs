use std::{
    fs::File,
    io::{BufRead, BufReader, Seek},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use deadpool_sqlite::Pool;
use serde::Serialize;
use serde_json::Value;

use crate::{
    error::AppError,
    events::SessionIndexEvent,
    sessions::{
        claude::parse_claude_record,
        codex::parse_codex_record,
        matcher::match_project,
        repo::{load_indexed_session, persist_indexed_file_result},
        IndexedSession, ProjectRoot, SessionIndexState, SessionParseAccumulator, SessionSource,
    },
    store::project_repo,
};

pub use crate::sessions::StreamFileStatus;

const LIVE_PARTIAL_MESSAGE: &str = "Live session still writing";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SessionIndexSummary {
    pub root_count: usize,
    pub files_processed: usize,
    pub sessions_persisted: usize,
    pub unmatched_count: usize,
    pub error_count: usize,
}

#[derive(Debug, Clone)]
struct SessionRoot {
    source: SessionSource,
    path: PathBuf,
}

pub async fn index_session_roots(
    pool: Pool,
    home_dir: PathBuf,
    on_event: impl Fn(SessionIndexEvent) -> Result<(), AppError> + Send + Sync + 'static,
) -> Result<SessionIndexSummary, AppError> {
    let roots = discover_existing_roots(home_dir).await?;
    on_event(SessionIndexEvent::Started {
        root_count: roots.len(),
    })?;

    let known_projects = load_known_project_roots(&pool).await?;
    let mut summary = SessionIndexSummary {
        root_count: roots.len(),
        files_processed: 0,
        sessions_persisted: 0,
        unmatched_count: 0,
        error_count: 0,
    };

    for root in roots {
        on_event(SessionIndexEvent::SourceStarted {
            source: root.source.as_str().to_string(),
            root_path: root.path.display().to_string(),
        })?;

        let files = discover_jsonl_files(root.path.clone()).await?;
        for source_path in files {
            summary.files_processed += 1;
            match index_session_file(&pool, root.source, source_path.clone(), &known_projects).await
            {
                Ok(result) => {
                    summary.sessions_persisted += result.sessions_persisted;
                    on_event(SessionIndexEvent::FileIndexed {
                        source: root.source.as_str().to_string(),
                        source_path: source_path.display().to_string(),
                        sessions_persisted: result.sessions_persisted,
                        live_partial: result.live_partial,
                    })?;
                }
                Err(error) => {
                    summary.error_count += 1;
                    on_event(SessionIndexEvent::FileIndexError {
                        source: root.source.as_str().to_string(),
                        source_path: source_path.display().to_string(),
                        message: error.to_string(),
                    })?;
                }
            }
        }
    }

    summary.unmatched_count = load_unmatched_count(&pool).await?;
    on_event(SessionIndexEvent::Finished {
        files_processed: summary.files_processed,
        sessions_persisted: summary.sessions_persisted,
        unmatched_count: summary.unmatched_count,
        error_count: summary.error_count,
    })?;

    Ok(summary)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct IndexedFileResult {
    sessions_persisted: usize,
    live_partial: bool,
}

pub fn stream_session_file(
    source: SessionSource,
    path: &Path,
    state: Option<&SessionIndexState>,
) -> Result<(SessionParseAccumulator, StreamFileStatus), AppError> {
    let source_path = path.display().to_string();
    let mut file = File::open(path).map_err(AppError::io)?;
    let metadata = file.metadata().map_err(AppError::io)?;
    let starting_offset = state
        .map(|state| state.last_parsed_byte_offset.max(0) as u64)
        .unwrap_or(0);

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

    let file_size = metadata.len() as i64;
    Ok((
        accumulator,
        StreamFileStatus::Complete {
            committed_offset: committed_offset.min(file_size),
        },
    ))
}

async fn discover_existing_roots(home_dir: PathBuf) -> Result<Vec<SessionRoot>, AppError> {
    tokio::task::spawn_blocking(move || {
        let candidates = [
            SessionRoot {
                source: SessionSource::Claude,
                path: home_dir.join(".claude/projects"),
            },
            SessionRoot {
                source: SessionSource::Codex,
                path: home_dir.join(".codex/sessions"),
            },
        ];

        candidates
            .into_iter()
            .filter(|root| root.path.is_dir())
            .collect::<Vec<_>>()
    })
    .await
    .map_err(AppError::io)
}

async fn discover_jsonl_files(root: PathBuf) -> Result<Vec<PathBuf>, AppError> {
    tokio::task::spawn_blocking(move || {
        let mut files = Vec::new();
        collect_jsonl_files(&root, &mut files)?;
        files.sort();

        Ok::<_, AppError>(files)
    })
    .await
    .map_err(AppError::io)?
}

fn collect_jsonl_files(path: &Path, files: &mut Vec<PathBuf>) -> Result<(), AppError> {
    for entry in std::fs::read_dir(path).map_err(AppError::io)? {
        let entry = entry.map_err(AppError::io)?;
        let entry_path = entry.path();
        let file_type = entry.file_type().map_err(AppError::io)?;

        if file_type.is_dir() {
            collect_jsonl_files(&entry_path, files)?;
        } else if file_type.is_file()
            && entry_path
                .extension()
                .and_then(|extension| extension.to_str())
                == Some("jsonl")
        {
            files.push(entry_path);
        }
    }

    Ok(())
}

async fn load_known_project_roots(pool: &Pool) -> Result<Vec<ProjectRoot>, AppError> {
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

async fn load_unmatched_count(pool: &Pool) -> Result<usize, AppError> {
    let connection = pool.get().await.map_err(AppError::store)?;
    connection
        .interact(|connection| {
            connection
                .query_row(
                    "SELECT COUNT(*) FROM sessions WHERE project_id IS NULL",
                    [],
                    |row| row.get::<_, i64>(0),
                )
                .map(|count| count as usize)
                .map_err(AppError::from)
        })
        .await
        .map_err(AppError::store)?
}

async fn index_session_file(
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
    let (mut accumulator, status) = tokio::task::spawn_blocking({
        let source_path = source_path.clone();
        let previous_state = previous_state.clone();
        move || stream_session_file(source, &source_path, previous_state.as_ref())
    })
    .await
    .map_err(AppError::io)??;
    let committed_offset = match &status {
        StreamFileStatus::Complete { committed_offset } => *committed_offset,
        StreamFileStatus::LivePartial {
            committed_offset, ..
        } => *committed_offset,
    };
    let live_partial = matches!(status, StreamFileStatus::LivePartial { .. });
    let file_state = build_index_state(source, &source_path, committed_offset, live_partial)?;
    let should_persist_session =
        committed_offset > previous_offset && accumulator.session.message_count > 0;
    let sessions = if should_persist_session {
        if previous_offset > 0 {
            if let Some(previous_session) =
                load_previous_session(pool, accumulator.session.id.clone()).await?
            {
                accumulator.session =
                    merge_incremental_session(previous_session, accumulator.session);
            }
        }
        match_project(&mut accumulator.session, known_projects);
        vec![accumulator.session]
    } else {
        Vec::new()
    };

    if committed_offset == previous_offset && sessions.is_empty() {
        return Ok(IndexedFileResult {
            sessions_persisted: 0,
            live_partial,
        });
    }

    let now = current_unix_seconds();
    let sessions_persisted = sessions.len();
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
    })
}

fn build_index_state(
    source: SessionSource,
    source_path: &Path,
    committed_offset: i64,
    live_partial: bool,
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
        last_error: None,
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
