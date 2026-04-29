use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use gsd_dashboard::{
    bootstrap,
    commands::sessions::{clear_session_index_for_app, index_sessions_for_app},
    error::AppError,
    events::SessionIndexEvent,
    sessions::{
        indexer::{stream_session_file, StreamFileStatus},
        matcher::match_project,
        repo::{load_index_state, persist_indexed_file_result},
        IndexedSession, ProjectRoot, SessionIndexState, SessionSource,
    },
    store::project_repo::{self, StoredProjectSnapshot},
};

#[derive(Debug, PartialEq, Eq)]
struct StoredSessionStats {
    started_at: Option<i64>,
    ended_at: Option<i64>,
    message_count: i64,
    tokens_in: Option<i64>,
    tokens_out: Option<i64>,
}

type SessionEventStore = Arc<Mutex<Vec<SessionIndexEvent>>>;
type SessionEventRecorder =
    Box<dyn Fn(SessionIndexEvent) -> Result<(), AppError> + Send + Sync + 'static>;

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
        "codex-sparse" => concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/fixtures/sessions/codex-sparse.jsonl"
        ),
        "codex-current" => concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/fixtures/sessions/codex-current.jsonl"
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
    seed_known_project(&state).await;

    (temp_dir, state)
}

async fn seed_known_project(state: &gsd_dashboard::app_state::AppState) {
    state
        .pool
        .get()
        .await
        .expect("connection should be available")
        .interact(|connection| {
            project_repo::upsert_project_snapshot(
                connection,
                StoredProjectSnapshot {
                    id: "gsd-dashboard-fixture".to_string(),
                    name: "GSD Dashboard Fixture".to_string(),
                    root_path: "/tmp/gsd-dashboard-fixture".to_string(),
                    planning_path: "/tmp/gsd-dashboard-fixture/.planning".to_string(),
                    current_milestone_name: None,
                    current_milestone_index: None,
                    current_phase_number: None,
                    current_phase_name: None,
                    milestone_progress_pct: 0.0,
                    next_command: "/gsd-next".to_string(),
                    parsed_blob: "{}".to_string(),
                    parse_error: None,
                    last_activity_at: None,
                    last_scanned_at: 0,
                    created_at: 0,
                    updated_at: 0,
                },
                Vec::new(),
                1,
            )
        })
        .await
        .expect("interaction should complete")
        .expect("known project should be seeded");
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

fn collect_session_events() -> (SessionEventStore, SessionEventRecorder) {
    let events = Arc::new(Mutex::new(Vec::new()));
    let recorded_events = Arc::clone(&events);

    (
        events,
        Box::new(move |event| {
            recorded_events
                .lock()
                .expect("events lock should not be poisoned")
                .push(event);
            Ok(())
        }),
    )
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
fn codex_sparse_session_fixture_uses_optional_field_fallbacks() {
    let source_path = fixture_path("codex-sparse");
    let (accumulator, status) =
        stream_session_file(SessionSource::Codex, Path::new(source_path), None)
            .expect("codex sparse fixture should stream");

    assert!(matches!(status, StreamFileStatus::Complete { .. }));
    assert_eq!(
        accumulator.session.source_session_id.as_deref(),
        Some("codex-session-sparse")
    );
    assert_eq!(
        accumulator.session.cwd.as_deref(),
        Some("/tmp/gsd-dashboard-fixture")
    );
    assert_eq!(accumulator.session.tokens_in, Some(21));
    assert_eq!(accumulator.session.tokens_out, Some(8));
}

#[test]
fn codex_current_session_fixture_uses_info_token_usage() {
    let source_path = fixture_path("codex-current");
    let (accumulator, status) =
        stream_session_file(SessionSource::Codex, Path::new(source_path), None)
            .expect("codex current fixture should stream");

    assert!(matches!(status, StreamFileStatus::Complete { .. }));
    assert_eq!(
        accumulator.session.source_session_id.as_deref(),
        Some("codex-current-session")
    );
    assert_eq!(
        accumulator.session.cwd.as_deref(),
        Some("/tmp/gsd-dashboard-fixture")
    );
    assert_eq!(accumulator.session.tokens_in, Some(26_940));
    assert_eq!(accumulator.session.tokens_out, Some(435));
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
fn matcher_attributes_worktree_cwd_to_base_project() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let base_project = temp_dir.path().join("deckpilot-web");
    let worktree = temp_dir.path().join("worktrees/agent-a485842780e148052");
    let worktree_subdir = worktree.join("src");
    fs::create_dir_all(base_project.join(".git/worktrees/agent-a485842780e148052"))
        .expect("base git metadata should be created");
    fs::create_dir_all(&worktree_subdir).expect("worktree subdir should be created");
    fs::write(
        worktree.join(".git"),
        format!(
            "gitdir: {}/.git/worktrees/agent-a485842780e148052\n",
            base_project.display()
        ),
    )
    .expect("worktree git file should be written");
    let known_projects = vec![ProjectRoot {
        id: "deckpilot-web".to_string(),
        root_path: base_project.display().to_string(),
    }];
    let mut session = empty_session(SessionSource::Codex, "/tmp/codex.jsonl");
    session.cwd = Some(worktree_subdir.display().to_string());

    match_project(&mut session, &known_projects);

    assert_eq!(session.project_id.as_deref(), Some("deckpilot-web"));
    assert_eq!(session.attribution_method, "worktree_cwd");
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
fn claude_path_fallback_prefers_most_specific_encoded_root() {
    let known_projects = vec![
        ProjectRoot {
            id: "homegit".to_string(),
            root_path: "/Users/smacdonald/homegit".to_string(),
        },
        ProjectRoot {
            id: "gsd-dashboard".to_string(),
            root_path: "/Users/smacdonald/homegit/gsd-dashboard".to_string(),
        },
    ];
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

#[test]
fn truncated_file_resets_incremental_offset_to_zero() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let session_path = temp_dir.path().join("rewritten.jsonl");
    let rewritten = "{\"type\":\"user\",\"timestamp\":\"2024-05-27T12:00:00Z\",\"cwd\":\"/tmp/rewritten\",\"sessionId\":\"rewritten-session\"}\n";
    fs::write(&session_path, rewritten).expect("rewritten session should be written");
    let previous_state = SessionIndexState {
        source_path: session_path.display().to_string(),
        source: SessionSource::Claude,
        file_size: 10_000,
        file_mtime: None,
        last_parsed_byte_offset: 10_000,
        live_partial: false,
        last_error: None,
    };

    let (accumulator, status) =
        stream_session_file(SessionSource::Claude, &session_path, Some(&previous_state))
            .expect("rewritten file should stream from the beginning");

    assert_eq!(accumulator.session.message_count, 1);
    assert_eq!(
        accumulator.session.source_session_id.as_deref(),
        Some("rewritten-session")
    );
    assert_eq!(
        status,
        StreamFileStatus::Complete {
            committed_offset: rewritten.len() as i64
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
    assert_eq!(summary.unmatched_count, 0);
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
                unmatched_count: 0,
                error_count: 0
            }
        )
    }));
}

#[tokio::test]
async fn index_sessions_for_app_prunes_and_skips_unmatched_sessions() {
    let (_temp_dir, state) = test_state().await;
    let codex_dir = state.home_dir.join(".codex/sessions/2026/04/27");
    fs::create_dir_all(&codex_dir).expect("codex fixture dir should be created");
    let unmatched_path = codex_dir.join("unmatched-session.jsonl");
    let unmatched_source_path = unmatched_path.display().to_string();
    fs::write(
        &unmatched_path,
        "{\"type\":\"event_msg\",\"timestamp\":\"2026-04-27T14:00:00Z\",\"payload\":{\"id\":\"unmatched-session\",\"cwd\":\"/tmp/not-a-known-project\"}}\n",
    )
    .expect("unmatched codex session should be written");
    let stale_session = IndexedSession {
        id: "codex:stale-unmatched".to_string(),
        source: SessionSource::Codex,
        source_path: "/tmp/stale-unmatched.jsonl".to_string(),
        source_session_id: Some("stale-unmatched".to_string()),
        project_id: None,
        cwd: Some("/tmp/not-a-known-project".to_string()),
        started_at: Some(1_777_000_000),
        ended_at: Some(1_777_000_000),
        duration_ms: Some(0),
        message_count: 1,
        tokens_in: Some(0),
        tokens_out: Some(0),
        model: None,
        attribution_method: "unmatched".to_string(),
        index_error: None,
    };
    let stale_state = SessionIndexState {
        source_path: stale_session.source_path.clone(),
        source: SessionSource::Codex,
        file_size: 10,
        file_mtime: Some(1),
        last_parsed_byte_offset: 10,
        live_partial: false,
        last_error: None,
    };
    state
        .pool
        .get()
        .await
        .expect("connection should be available")
        .interact(move |connection| {
            persist_indexed_file_result(connection, &[stale_session], &stale_state, 1)
        })
        .await
        .expect("interaction should complete")
        .expect("stale unmatched should persist");

    let summary = index_sessions_for_app(&state, |_| Ok(()))
        .await
        .expect("session index should complete");
    let (unmatched_count, unmatched_state_count, stale_state_count) = state
        .pool
        .get()
        .await
        .expect("connection should be available")
        .interact(move |connection| {
            let unmatched_count = connection
                .query_row(
                    "SELECT COUNT(*) FROM sessions WHERE project_id IS NULL",
                    [],
                    |row| row.get::<_, i64>(0),
                )
                .map_err(AppError::from)?;
            let unmatched_state_count = connection
                .query_row(
                    "SELECT COUNT(*) FROM session_index_state WHERE source_path = ?1",
                    [unmatched_source_path],
                    |row| row.get::<_, i64>(0),
                )
                .map_err(AppError::from)?;
            let stale_state_count = connection
                .query_row(
                    "SELECT COUNT(*) FROM session_index_state WHERE source_path = ?1",
                    ["/tmp/stale-unmatched.jsonl"],
                    |row| row.get::<_, i64>(0),
                )
                .map_err(AppError::from)?;

            Ok::<_, AppError>((unmatched_count, unmatched_state_count, stale_state_count))
        })
        .await
        .expect("interaction should complete")
        .expect("unmatched count should load");

    assert_eq!(summary.sessions_persisted, 0);
    assert_eq!(summary.unmatched_count, 0);
    assert_eq!(unmatched_count, 0);
    assert_eq!(unmatched_state_count, 0);
    assert_eq!(stale_state_count, 0);
}

#[tokio::test]
async fn index_sessions_for_app_reparses_stale_tokenless_codex_sessions() {
    let (_temp_dir, state) = test_state().await;
    let codex_dir = state.home_dir.join(".codex/sessions/2026/04/27");
    fs::create_dir_all(&codex_dir).expect("codex fixture dir should be created");
    let codex_path = codex_dir.join("codex-current.jsonl");
    fs::copy(fixture_path("codex-current"), &codex_path).expect("codex fixture should copy");
    let codex_source_path = codex_path.display().to_string();
    let codex_file_size = codex_path
        .metadata()
        .expect("codex fixture metadata should load")
        .len() as i64;
    let stale_session = IndexedSession {
        id: "codex:codex-current-session".to_string(),
        source: SessionSource::Codex,
        source_path: codex_source_path.clone(),
        source_session_id: Some("codex-current-session".to_string()),
        project_id: Some("gsd-dashboard-fixture".to_string()),
        cwd: Some("/tmp/gsd-dashboard-fixture".to_string()),
        started_at: Some(1_777_000_000_000),
        ended_at: Some(1_777_000_000_000),
        duration_ms: Some(0),
        message_count: 1,
        tokens_in: Some(0),
        tokens_out: Some(0),
        model: Some("gpt-5".to_string()),
        attribution_method: "cwd".to_string(),
        index_error: None,
    };
    let stale_state = SessionIndexState {
        source_path: codex_source_path,
        source: SessionSource::Codex,
        file_size: codex_file_size,
        file_mtime: Some(1),
        last_parsed_byte_offset: codex_file_size,
        live_partial: false,
        last_error: None,
    };
    state
        .pool
        .get()
        .await
        .expect("connection should be available")
        .interact(move |connection| {
            persist_indexed_file_result(connection, &[stale_session], &stale_state, 1)
        })
        .await
        .expect("interaction should complete")
        .expect("stale tokenless codex session should persist");

    index_sessions_for_app(&state, |_| Ok(()))
        .await
        .expect("session index should complete");
    let stats = load_session_stats(&state, "codex:codex-current-session").await;

    assert_eq!(stats.tokens_in, Some(26940));
    assert_eq!(stats.tokens_out, Some(435));
}

#[tokio::test]
async fn clear_session_index_removes_sessions_and_offsets() {
    let (_temp_dir, state) = test_state().await;
    let (_claude_path, _codex_path) = copy_fixture_roots(&state.home_dir);
    index_sessions_for_app(&state, |_| Ok(()))
        .await
        .expect("session index should complete");

    let summary = clear_session_index_for_app(&state)
        .await
        .expect("session index should clear");
    let counts = state
        .pool
        .get()
        .await
        .expect("connection should be available")
        .interact(|connection| {
            let session_count =
                connection.query_row("SELECT COUNT(*) FROM sessions", [], |row| {
                    row.get::<_, i64>(0)
                })?;
            let state_count =
                connection.query_row("SELECT COUNT(*) FROM session_index_state", [], |row| {
                    row.get::<_, i64>(0)
                })?;
            Ok::<_, AppError>((session_count, state_count))
        })
        .await
        .expect("interaction should complete")
        .expect("counts should load");

    assert_eq!(summary.sessions_cleared, 2);
    assert_eq!(summary.index_states_cleared, 2);
    assert_eq!(counts, (0, 0));
}

#[tokio::test]
async fn index_sessions_for_app_skips_codex_index_summaries() {
    let (_temp_dir, state) = test_state().await;
    let (_claude_path, _codex_path) = copy_fixture_roots(&state.home_dir);
    let codex_index_dir = state.home_dir.join(".codex/sessions/index/by-dir");
    fs::create_dir_all(&codex_index_dir).expect("codex index dir should be created");
    let codex_index_path = codex_index_dir.join("_tmp_gsd-dashboard-fixture.jsonl");
    fs::write(
        &codex_index_path,
        "{\"record_type\":\"summary\",\"cwd\":\"/tmp/gsd-dashboard-fixture\",\"message_count_delta\":99}\n",
    )
    .expect("codex index summary should be written");
    let codex_index_source_path = codex_index_path.display().to_string();
    let stale_session = IndexedSession {
        id: "codex:stale-index-summary".to_string(),
        source: SessionSource::Codex,
        source_path: codex_index_source_path.clone(),
        source_session_id: Some("stale-index-summary".to_string()),
        project_id: None,
        cwd: Some("/tmp/gsd-dashboard-fixture".to_string()),
        started_at: Some(1_716_814_800_000),
        ended_at: Some(1_716_814_800_000),
        duration_ms: Some(0),
        message_count: 99,
        tokens_in: Some(0),
        tokens_out: Some(0),
        model: None,
        attribution_method: "unmatched".to_string(),
        index_error: None,
    };
    let stale_state = SessionIndexState {
        source_path: codex_index_source_path.clone(),
        source: SessionSource::Codex,
        file_size: 10,
        file_mtime: Some(1),
        last_parsed_byte_offset: 10,
        live_partial: false,
        last_error: None,
    };
    state
        .pool
        .get()
        .await
        .expect("connection should be available")
        .interact(move |connection| {
            persist_indexed_file_result(connection, &[stale_session], &stale_state, 1)
        })
        .await
        .expect("interaction should complete")
        .expect("stale summary should persist");

    let summary = index_sessions_for_app(&state, |_| Ok(()))
        .await
        .expect("session index should complete");
    let stale_count = state
        .pool
        .get()
        .await
        .expect("connection should be available")
        .interact(move |connection| {
            connection
                .query_row(
                    "SELECT COUNT(*) FROM sessions WHERE source_path = ?1",
                    [codex_index_source_path],
                    |row| row.get::<_, i64>(0),
                )
                .map_err(AppError::from)
        })
        .await
        .expect("interaction should complete")
        .expect("stale count should load");

    assert_eq!(summary.files_processed, 2);
    assert_eq!(summary.sessions_persisted, 2);
    assert_eq!(stale_count, 0);
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
async fn index_sessions_for_app_reindexes_truncated_session_file() {
    let (_temp_dir, state) = test_state().await;
    let (claude_path, _codex_path) = copy_fixture_roots(&state.home_dir);
    index_sessions_for_app(&state, |_| Ok(()))
        .await
        .expect("first session index should complete");

    fs::write(
        &claude_path,
        "{\"type\":\"assistant\",\"timestamp\":\"2024-05-27T12:02:00Z\",\"cwd\":\"/tmp/gsd-dashboard-fixture\",\"sessionId\":\"claude-session-1\",\"message\":{\"usage\":{\"input_tokens\":7,\"output_tokens\":3}}}\n",
    )
    .expect("session file should be rewritten smaller");

    let summary = index_sessions_for_app(&state, |_| Ok(()))
        .await
        .expect("rewritten session index should complete");
    let stats = load_session_stats(&state, "claude:claude-session-1").await;

    assert_eq!(summary.sessions_persisted, 1);
    assert_eq!(
        stats,
        StoredSessionStats {
            started_at: Some(1_716_811_320_000),
            ended_at: Some(1_716_811_320_000),
            message_count: 1,
            tokens_in: Some(7),
            tokens_out: Some(3),
        }
    );
}

#[tokio::test]
async fn index_sessions_for_app_persists_nonfatal_parse_error_in_index_state() {
    let (_temp_dir, state) = test_state().await;
    let claude_dir = state.home_dir.join(".claude/projects/-tmp-bad-jsonl");
    fs::create_dir_all(&claude_dir).expect("claude fixture dir should be created");
    let bad_path = claude_dir.join("bad-session.jsonl");
    fs::write(&bad_path, "{not json}\n").expect("bad session should be written");

    let summary = index_sessions_for_app(&state, |_| Ok(()))
        .await
        .expect("bad session index should complete");
    let source_path = bad_path.display().to_string();
    let index_state = state
        .pool
        .get()
        .await
        .expect("connection should be available")
        .interact(move |connection| {
            load_index_state(connection, &source_path)
                .map(|state| state.expect("bad index state should exist"))
        })
        .await
        .expect("interaction should complete")
        .expect("state should load");

    assert_eq!(summary.sessions_persisted, 0);
    assert_eq!(
        index_state.last_parsed_byte_offset,
        "{not json}\n".len() as i64
    );
    assert!(index_state.last_error.is_some());
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
            fs::read_to_string(fixture_path("claude-basic"))
                .expect("fixture should read")
                .replace("\"input_tokens\":50", "\"input_tokens\":99"),
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
                "CREATE TRIGGER fail_session_replace
                 BEFORE DELETE ON sessions
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
    assert!(build_script.contains("\"clear_session_index\""));
    assert!(default_capability.contains("\"allow-index-sessions\""));
    assert!(default_capability.contains("\"allow-clear-session-index\""));
    assert!(!default_capability.contains("fs:allow-read"));
}
