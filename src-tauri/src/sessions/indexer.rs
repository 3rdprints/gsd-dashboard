use std::{
    fs::File,
    io::{BufRead, BufReader, Seek},
    path::Path,
};

use serde_json::Value;

use crate::{
    error::AppError,
    sessions::{
        claude::parse_claude_record, codex::parse_codex_record, IndexedSession, SessionIndexState,
        SessionParseAccumulator, SessionSource,
    },
};

pub use crate::sessions::StreamFileStatus;

const LIVE_PARTIAL_MESSAGE: &str = "Live session still writing";

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

    let file_size = metadata.len() as i64;
    Ok((
        accumulator,
        StreamFileStatus::Complete {
            committed_offset: committed_offset.min(file_size),
        },
    ))
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

fn trim_jsonl_newline(line: &[u8]) -> &[u8] {
    let without_lf = line.strip_suffix(b"\n").unwrap_or(line);
    without_lf.strip_suffix(b"\r").unwrap_or(without_lf)
}
