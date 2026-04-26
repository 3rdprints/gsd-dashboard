use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use gsd_dashboard::{
    bootstrap,
    commands::sessions::index_sessions_for_app,
    error::AppError,
    events::SessionIndexEvent,
    sessions::{
        indexer::{stream_session_file, StreamFileStatus},
        matcher::match_project,
        repo::load_index_state,
        IndexedSession, ProjectRoot, SessionIndexState, SessionSource,
    },
};

#[derive(Debug, PartialEq, Eq)]
struct StoredSessionStats {
    started_at: Option<i64>,
    ended_at: Option<i64>,
    message_count: i64,
    tokens_in: Option<i64>,
    tokens_out: Option<i64>,
}

fn fixture_path(name: &str) -> &'static str {
    match name {
        "claude-basic" => concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/fixtures/sessions/claude-basic.jsonl"
        ),
        "claude-partial" => concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/fixtures/sessions/claude-partial.jsonl"
        ),
        "codex-basic" => concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/fixtures/sessions/codex-basic.jsonl"
        ),
        _ => panic!("unknown fixture"),
    }
}

async fn test_state() -> (tempfile::TempDir, gsd_dashboard::app_state::AppState) {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let state = bootstrap::bootstrap_from_paths(
        temp_dir.path().join("app-data"),
        temp_dir.path().join("home"),
    )
    .await
    .expect("bootstrap should succeed");

    (temp_dir, state)
}

fn copy_fixture_roots(home_dir: &Path) -> (PathBuf, PathBuf) {
    let claude_dir = home_dir.join(".claude/projects/-tmp-gsd-dashboard-fixture");
    let codex_dir = home_dir.join(".codex/sessions/2024/05/27");
    fs::create_dir_all(&claude_dir).expect("claude fixture dir should be created");
    fs::create_dir_all(&codex_dir).expect("codex fixture dir should be created");
    let claude_path = claude_dir.join("claude-session-1.jsonl");
    let codex_path = codex_dir.join("codex-session-1.jsonl");
    fs::copy(fixture_path("claude-basic"), &claude_path).expect("claude fixture should copy");
    fs::copy(fixture_path("codex-basic"), &codex_path).expect("codex fixture should copy");

    (claude_path, codex_path)
}

fn collect_session_events() -> (
    Arc<Mutex<Vec<SessionIndexEvent>>>,
    impl Fn(SessionIndexEvent) -> Result<(), AppError> + Send + Sync + 'static,
) {
    let events = Arc::new(Mutex::new(Vec::new()));
    let recorded_events = Arc::clone(&events);

    (events, move |event| {
        recorded_events
            .lock()
            .expect("events lock should not be poisoned")
            .push(event);
        Ok(())
    })
}

async fn load_session_stats(
    state: &gsd_dashboard::app_state::AppState,
    session_id: &str,
) -> StoredSessionStats {
    let session_id = session_id.to_string();
    state
        .pool
        .get()
        .await
        .expect("connection should be available")
        .interact(move |connection| {
            connection
                .query_row(
                    "SELECT started_at, ended_at, message_count, tokens_in, tokens_out
                     FROM sessions
                     WHERE id = ?1",
                    [session_id],
                    |row| {
                        Ok(StoredSessionStats {
                            started_at: row.get(0)?,
                            ended_at: row.get(1)?,
                            message_count: row.get(2)?,
                            tokens_in: row.get(3)?,
                            tokens_out: row.get(4)?,
                        })
                    },
                )
                .map_err(AppError::from)
        })
        .await
        .expect("interaction should complete")
        .expect("session stats should load")
}

async fn count_sessions_for_source_path(
    state: &gsd_dashboard::app_state::AppState,
    source_path: &Path,
) -> i64 {
    let source_path = source_path.display().to_string();
    state
        .pool
        .get()
        .await
        .expect("connection should be available")
        .interact(move |connection| {
            connection
                .query_row(
                    "SELECT COUNT(*) FROM sessions WHERE source_path = ?1",
                    [source_path],
                    |row| row.get(0),
                )
                .map_err(AppError::from)
        })
        .await
        .expect("interaction should complete")
        .expect("session count should load")
}

fn empty_session(source: SessionSource, source_path: &str) -> IndexedSession {
    IndexedSession {
        id: "test-session".to_string(),
        source,
        source_path: source_path.to_string(),
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

#[test]
fn claude_session_fixture_extracts_metadata() {
    let source_path = fixture_path("claude-basic");
    let (accumulator, status) =
        stream_session_file(SessionSource::Claude, Path::new(source_path), None)
            .expect("claude fixture should stream");

    assert!(matches!(status, StreamFileStatus::Complete { .. }));
    assert_eq!(accumulator.session.source, SessionSource::Claude);
    assert_eq!(
        accumulator.session.source_session_id.as_deref(),
        Some("claude-session-1")
    );
    assert_eq!(accumulator.session.started_at, Some(1_716_811_200_000));
    assert_eq!(accumulator.session.ended_at, Some(1_716_811_215_000));
    assert_eq!(accumulator.session.duration_ms, Some(15_000));
    assert_eq!(accumulator.session.message_count, 2);
    assert_eq!(accumulator.session.tokens_in, Some(120));
    assert_eq!(accumulator.session.tokens_out, Some(45));
    assert_eq!(
        accumulator.session.model.as_deref(),
        Some("claude-3-5-sonnet")
    );
    assert_eq!(
        accumulator.session.cwd.as_deref(),
        Some("/tmp/gsd-dashboard-fixture")
    );
}

#[test]
fn codex_session_fixture_extracts_best_effort_metadata() {
    let source_path = fixture_path("codex-basic");
    let (accumulator, status) =
        stream_session_file(SessionSource::Codex, Path::new(source_path), None)
            .expect("codex fixture should stream");

    assert!(matches!(status, StreamFileStatus::Complete { .. }));
    assert_eq!(accumulator.session.source, SessionSource::Codex);
    assert_eq!(
        accumulator.session.source_session_id.as_deref(),
        Some("codex-session-1")
    );
    assert_eq!(accumulator.session.started_at, Some(1_716_814_800_000));
    assert_eq!(accumulator.session.ended_at, Some(1_716_814_812_500));
    assert_eq!(accumulator.session.duration_ms, Some(12_500));
    assert_eq!(
        accumulator.session.cwd.as_deref(),
        Some("/tmp/gsd-dashboard-fixture")
    );
}

#[test]
fn partial_trailing_line_keeps_offset_before_partial() {
    let source_path = fixture_path("claude-partial");
    let bytes = fs::read(source_path).expect("fixture should be readable");
    let partial_line_start = bytes
        .iter()
        .rposition(|byte| *byte == b'\n')
        .map(|position| position + 1)
        .expect("fixture should contain a trailing partial line");

    let (accumulator, status) =
        stream_session_file(SessionSource::Claude, Path::new(source_path), None)
            .expect("partial fixture should stream");

    assert_eq!(
        accumulator.live_partial_message.as_deref(),
        Some("Live session still writing")
    );
    assert_eq!(
        accumulator.session.source_session_id.as_deref(),
        Some("claude-session-1")
    );
    assert_eq!(accumulator.session.id, "claude:claude-session-1");
    assert_eq!(
        status,
        StreamFileStatus::LivePartial {
            committed_offset: partial_line_start as i64,
            message: "Live session still writing".to_string()
        }
    );
}

#[test]
fn matcher_prefers_cwd_and_retains_unmatched() {
    let known_projects = vec![ProjectRoot {
        id: "project-1".to_string(),
        root_path: "/tmp/gsd-dashboard-fixture".to_string(),
    }];
    let mut cwd_session = empty_session(SessionSource::Codex, "/tmp/codex.jsonl");
    cwd_session.cwd = Some("/tmp/gsd-dashboard-fixture/subdir".to_string());

    match_project(&mut cwd_session, &known_projects);

    assert_eq!(cwd_session.project_id.as_deref(), Some("project-1"));
    assert_eq!(cwd_session.attribution_method, "cwd");

    let mut unmatched = empty_session(SessionSource::Codex, "/tmp/codex-unmatched.jsonl");
    unmatched.cwd = Some("/tmp/other-project".to_string());

    match_project(&mut unmatched, &known_projects);

    assert_eq!(unmatched.project_id, None);
    assert_eq!(unmatched.attribution_method, "unmatched");
}

#[test]
fn claude_path_fallback_decodes_directory_encoding_against_known_roots() {
    let known_projects = vec![ProjectRoot {
        id: "gsd-dashboard".to_string(),
        root_path: "/Users/smacdonald/homegit/gsd-dashboard".to_string(),
    }];
    let mut session = empty_session(
        SessionSource::Claude,
        "~/.claude/projects/-Users-smacdonald-homegit-gsd-dashboard/claude-session-1.jsonl",
    );

    match_project(&mut session, &known_projects);

    assert_eq!(session.project_id.as_deref(), Some("gsd-dashboard"));
    assert_eq!(session.attribution_method, "claude_path");
}

#[test]
fn incremental_state_starts_at_previous_committed_offset() {
    let source_path = fixture_path("claude-basic");
    let bytes = fs::read(source_path).expect("fixture should be readable");
    let first_line_end = bytes
        .iter()
        .position(|byte| *byte == b'\n')
        .map(|position| position + 1)
        .expect("fixture should contain multiple lines");
    let previous_state = SessionIndexState {
        source_path: source_path.to_string(),
        source: SessionSource::Claude,
        file_size: bytes.len() as i64,
        file_mtime: None,
        last_parsed_byte_offset: first_line_end as i64,
        live_partial: false,
        last_error: None,
    };

    let (accumulator, status) = stream_session_file(
        SessionSource::Claude,
        Path::new(source_path),
        Some(&previous_state),
    )
    .expect("fixture should stream from previous offset");

    assert_eq!(accumulator.session.message_count, 1);
    assert_eq!(accumulator.session.started_at, Some(1_716_811_215_000));
    assert_eq!(
        status,
        StreamFileStatus::Complete {
            committed_offset: bytes.len() as i64
        }
    );
}

#[tokio::test]
async fn index_sessions_for_app_persists_fixture_roots() {
    let (_temp_dir, state) = test_state().await;
    let (_claude_path, _codex_path) = copy_fixture_roots(&state.home_dir);
    let (events, on_event) = collect_session_events();

    let summary = index_sessions_for_app(&state, on_event)
        .await
        .expect("session index should complete");
    let events = events.lock().expect("events should be readable").clone();

    assert_eq!(summary.root_count, 2);
    assert_eq!(summary.files_processed, 2);
    assert_eq!(summary.sessions_persisted, 2);
    assert_eq!(summary.unmatched_count, 2);
    assert_eq!(summary.error_count, 0);
    assert!(events
        .iter()
        .any(|event| matches!(event, SessionIndexEvent::Started { root_count: 2 })));
    assert!(events.iter().any(|event| {
        matches!(
            event,
            SessionIndexEvent::Finished {
                files_processed: 2,
                sessions_persisted: 2,
                unmatched_count: 2,
                error_count: 0
            }
        )
    }));
}

#[tokio::test]
async fn index_sessions_for_app_reuses_offsets_incrementally() {
    let (_temp_dir, state) = test_state().await;
    let (claude_path, _codex_path) = copy_fixture_roots(&state.home_dir);

    let first_summary = index_sessions_for_app(&state, |_| Ok(()))
        .await
        .expect("first session index should complete");
    let second_summary = index_sessions_for_app(&state, |_| Ok(()))
        .await
        .expect("second session index should complete");
    let source_path = claude_path.display().to_string();
    let stored_offset = state
        .pool
        .get()
        .await
        .expect("connection should be available")
        .interact(move |connection| {
            load_index_state(connection, &source_path)
                .map(|state| state.expect("claude index state should exist"))
        })
        .await
        .expect("interaction should complete")
        .expect("state should load")
        .last_parsed_byte_offset;

    assert_eq!(first_summary.sessions_persisted, 2);
    assert_eq!(second_summary.sessions_persisted, 0);
    assert_eq!(second_summary.files_processed, 2);
    assert_eq!(
        stored_offset,
        fs::metadata(&claude_path)
            .expect("claude fixture should exist")
            .len() as i64
    );
}

#[tokio::test]
async fn index_sessions_for_app_persists_cumulative_metadata_after_append() {
    let (_temp_dir, state) = test_state().await;
    let (claude_path, _codex_path) = copy_fixture_roots(&state.home_dir);
    index_sessions_for_app(&state, |_| Ok(()))
        .await
        .expect("first session index should complete");

    fs::write(
        &claude_path,
        format!(
            "{}{}",
            fs::read_to_string(fixture_path("claude-basic")).expect("fixture should read"),
            "{\"type\":\"assistant\",\"timestamp\":\"2024-05-27T12:01:00Z\",\"cwd\":\"/tmp/gsd-dashboard-fixture\",\"sessionId\":\"claude-session-1\",\"message\":{\"usage\":{\"input_tokens\":10,\"output_tokens\":5}}}\n"
        ),
    )
    .expect("fixture should be extended");

    let summary = index_sessions_for_app(&state, |_| Ok(()))
        .await
        .expect("second session index should complete");
    let stats = load_session_stats(&state, "claude:claude-session-1").await;

    assert_eq!(summary.sessions_persisted, 1);
    assert_eq!(
        stats,
        StoredSessionStats {
            started_at: Some(1_716_811_200_000),
            ended_at: Some(1_716_811_260_000),
            message_count: 3,
            tokens_in: Some(130),
            tokens_out: Some(50),
        }
    );
}

#[tokio::test]
async fn index_sessions_for_app_keeps_live_partial_session_id_stable_after_completion() {
    let (_temp_dir, state) = test_state().await;
    let claude_dir = state
        .home_dir
        .join(".claude/projects/-tmp-gsd-dashboard-fixture");
    fs::create_dir_all(&claude_dir).expect("claude fixture dir should be created");
    let claude_path = claude_dir.join("claude-session-1.jsonl");
    fs::copy(fixture_path("claude-partial"), &claude_path).expect("partial fixture should copy");

    let first_summary = index_sessions_for_app(&state, |_| Ok(()))
        .await
        .expect("partial session index should complete");
    fs::write(
        &claude_path,
        format!(
            "{}{}\n",
            fs::read_to_string(fixture_path("claude-basic")).expect("fixture should read"),
            "{\"type\":\"assistant\",\"timestamp\":\"2024-05-27T12:00:16Z\",\"cwd\":\"/tmp/gsd-dashboard-fixture\",\"sessionId\":\"claude-session-1\"}"
        ),
    )
    .expect("partial fixture should be completed");

    let second_summary = index_sessions_for_app(&state, |_| Ok(()))
        .await
        .expect("completed session index should complete");
    let session_count = count_sessions_for_source_path(&state, &claude_path).await;

    assert_eq!(first_summary.sessions_persisted, 1);
    assert_eq!(second_summary.sessions_persisted, 1);
    assert_eq!(session_count, 1);
}

#[tokio::test]
async fn index_sessions_for_app_does_not_advance_offset_when_persistence_fails() {
    let (_temp_dir, state) = test_state().await;
    let (claude_path, _codex_path) = copy_fixture_roots(&state.home_dir);
    index_sessions_for_app(&state, |_| Ok(()))
        .await
        .expect("first session index should complete");
    let previous_offset = fs::metadata(&claude_path)
        .expect("claude fixture should exist")
        .len() as i64;
    fs::write(
        &claude_path,
        format!(
            "{}{}",
            fs::read_to_string(fixture_path("claude-basic")).expect("fixture should read"),
            "{\"type\":\"assistant\",\"timestamp\":\"2024-05-27T12:01:00Z\",\"cwd\":\"/tmp/gsd-dashboard-fixture\",\"sessionId\":\"claude-session-1\"}\n"
        ),
    )
    .expect("fixture should be extended");

    state
        .pool
        .get()
        .await
        .expect("connection should be available")
        .interact(|connection| {
            connection.execute(
                "CREATE TRIGGER fail_session_update
                 BEFORE UPDATE ON sessions
                 BEGIN
                     SELECT RAISE(ABORT, 'forced persistence failure');
                 END;",
                [],
            )?;

            Ok::<_, AppError>(())
        })
        .await
        .expect("interaction should complete")
        .expect("failure trigger should be installed");

    let summary = index_sessions_for_app(&state, |_| Ok(()))
        .await
        .expect("per-file failures should not abort the whole index");
    let source_path = claude_path.display().to_string();
    let stored_offset = state
        .pool
        .get()
        .await
        .expect("connection should be available")
        .interact(move |connection| {
            load_index_state(connection, &source_path)
                .map(|state| state.expect("claude index state should exist"))
        })
        .await
        .expect("interaction should complete")
        .expect("state should load")
        .last_parsed_byte_offset;

    assert_eq!(summary.error_count, 1);
    assert_eq!(stored_offset, previous_offset);
}

#[test]
fn index_sessions_command_is_registered_for_release() {
    let build_script = include_str!("../build.rs");
    let default_capability = include_str!("../capabilities/default.json");

    assert!(build_script.contains("\"index_sessions\""));
    assert!(default_capability.contains("\"allow-index-sessions\""));
    assert!(!default_capability.contains("fs:allow-read"));
}
