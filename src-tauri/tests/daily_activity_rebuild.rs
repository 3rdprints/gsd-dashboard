use std::{
    fs,
    sync::{Arc, Mutex},
};

use gsd_dashboard::{
    bootstrap,
    commands::sessions::index_sessions_for_app,
    error::AppError,
    events::{AppEvent, SessionIndexEvent},
    sessions::{repo as session_repo, IndexedSession, SessionIndexState, SessionSource},
    store::{daily_activity, project_repo::StoredProjectSnapshot},
};

const DAY_MS: i64 = 86_400_000;

fn project_snapshot(id: &str, name: &str) -> StoredProjectSnapshot {
    StoredProjectSnapshot {
        id: id.to_string(),
        name: name.to_string(),
        root_path: format!("/tmp/{id}"),
        planning_path: format!("/tmp/{id}/.planning"),
        current_milestone_name: Some("v1.0".to_string()),
        current_milestone_index: Some(1),
        current_phase_number: Some("05".to_string()),
        current_phase_name: Some("Charts".to_string()),
        milestone_progress_pct: 50.0,
        next_command: "/gsd-next".to_string(),
        parsed_blob: "{}".to_string(),
        parse_error: None,
        last_activity_at: None,
        last_scanned_at: 1,
        created_at: 1,
        updated_at: 1,
    }
}

fn indexed_session(id: &str, project_id: Option<&str>, started_at: i64) -> IndexedSession {
    IndexedSession {
        id: id.to_string(),
        source: SessionSource::Claude,
        source_path: format!("/tmp/{id}.jsonl"),
        source_session_id: Some(id.to_string()),
        project_id: project_id.map(str::to_string),
        cwd: None,
        started_at: Some(started_at),
        ended_at: Some(started_at + 1_000),
        duration_ms: Some(1_000),
        message_count: 1,
        tokens_in: Some(10),
        tokens_out: Some(20),
        model: Some("test-model".to_string()),
        attribution_method: project_id.map_or("unmatched", |_| "cwd").to_string(),
        index_error: None,
    }
}

#[tokio::test]
async fn daily_activity_rebuild_is_idempotent_and_emits_event() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let state = bootstrap::bootstrap_from_paths(
        temp_dir.path().join("app-data"),
        temp_dir.path().join("home"),
    )
    .await
    .expect("bootstrap should succeed");
    let now_ms = 1_777_132_245_000_i64;
    let today_start_ms = now_ms - now_ms.rem_euclid(DAY_MS);
    let yesterday_ms = today_start_ms - DAY_MS + 3_600_000;

    let connection = state
        .pool
        .get()
        .await
        .expect("connection should be available");
    connection
        .interact(move |connection| {
            gsd_dashboard::store::project_repo::upsert_project_snapshot(
                connection,
                project_snapshot("alpha", "Alpha"),
                Vec::new(),
                1,
            )?;
            gsd_dashboard::store::project_repo::upsert_project_snapshot(
                connection,
                project_snapshot("beta", "Beta"),
                Vec::new(),
                1,
            )?;

            let sessions = vec![
                indexed_session("alpha-1", Some("alpha"), yesterday_ms),
                indexed_session("alpha-2", Some("alpha"), yesterday_ms + 1_000),
                indexed_session("beta-1", Some("beta"), yesterday_ms + 2_000),
                indexed_session("unmatched-1", None, yesterday_ms + 3_000),
            ];
            let index_state = SessionIndexState {
                source_path: "/tmp/source.jsonl".to_string(),
                source: SessionSource::Claude,
                file_size: 100,
                file_mtime: Some(1),
                last_parsed_byte_offset: 100,
                live_partial: false,
                last_error: None,
            };
            session_repo::persist_indexed_file_result(connection, &sessions, &index_state, 1)?;
            connection
                .execute(
                    "UPDATE sessions
                     SET cache_read_tokens = 3,
                         cache_creation_tokens = 7",
                    [],
                )
                .map_err(AppError::from)?;

            daily_activity::rebuild_window(connection, 90, now_ms)?;
            daily_activity::rebuild_window(connection, 90, now_ms)?;
            daily_activity::load_window(connection, 2)
        })
        .await
        .expect("interaction should complete")
        .expect("daily activity should rebuild");

    let rows = state
        .pool
        .get()
        .await
        .expect("connection should be available")
        .interact(|connection| daily_activity::load_window(connection, 2))
        .await
        .expect("interaction should complete")
        .expect("daily activity should load");

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].session_count, 0);
    assert_eq!(rows[1].session_count, 4);
    assert_eq!(rows[1].token_total, 160);
    assert_eq!(rows[1].top_project_id.as_deref(), Some("alpha"));
    assert_eq!(rows[1].top_project_name.as_deref(), Some("Alpha"));

    let claude_dir = state.home_dir.join(".claude/projects/-tmp-alpha");
    fs::create_dir_all(&claude_dir).expect("claude dir should be created");
    fs::write(
        claude_dir.join("claude-session-1.jsonl"),
        r#"{"type":"message","sessionId":"daily-event","timestamp":"2024-05-27T12:00:00Z","message":{"usage":{"input_tokens":1,"output_tokens":2},"model":"claude-test"},"cwd":"/tmp/alpha"}
"#,
    )
    .expect("fixture should be written");

    let events = Arc::new(Mutex::new(Vec::new()));
    let recorded_events = Arc::clone(&events);
    index_sessions_for_app(&state, move |event| {
        recorded_events
            .lock()
            .expect("events lock should not be poisoned")
            .push(event);
        Ok(())
    })
    .await
    .expect("session index should complete");

    let events = events.lock().expect("events should be readable");
    assert!(events.iter().any(|event| {
        matches!(
            event,
            SessionIndexEvent::App(AppEvent::DailyActivityUpdated)
        )
    }));
}
