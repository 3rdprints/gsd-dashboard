use gsd_dashboard::{
    bootstrap,
    commands::projects::list_project_sessions_for_app,
    sessions::{repo as session_repo, IndexedSession, SessionIndexState, SessionSource},
    store::project_repo::{self, StoredProjectSnapshot},
};

fn snapshot() -> StoredProjectSnapshot {
    StoredProjectSnapshot {
        id: "project-1".to_string(),
        name: "Project One".to_string(),
        root_path: "/tmp/project-one".to_string(),
        planning_path: "/tmp/project-one/.planning".to_string(),
        current_milestone_name: Some("v1.0".to_string()),
        current_milestone_index: Some(0),
        current_phase_number: Some("05".to_string()),
        current_phase_name: Some("Charts".to_string()),
        milestone_progress_pct: 0.0,
        next_command: "/gsd-next".to_string(),
        parsed_blob: "{}".to_string(),
        parse_error: None,
        last_activity_at: None,
        last_scanned_at: 1_777_000_000,
        created_at: 0,
        updated_at: 0,
    }
}

fn session(id: &str, started_at: i64, duration_ms: i64, tokens_in: i64) -> IndexedSession {
    IndexedSession {
        id: id.to_string(),
        source: SessionSource::Claude,
        source_path: format!("/tmp/{id}.jsonl"),
        source_session_id: Some(id.to_string()),
        project_id: Some("project-1".to_string()),
        cwd: Some("/tmp/project-one".to_string()),
        started_at: Some(started_at),
        ended_at: Some(started_at + duration_ms),
        duration_ms: Some(duration_ms),
        message_count: tokens_in / 10,
        tokens_in: Some(tokens_in),
        tokens_out: Some(1),
        model: Some("test-model".to_string()),
        attribution_method: "cwd".to_string(),
        index_error: None,
    }
}

#[tokio::test]
async fn project_sessions_support_paging_sorting_and_reject_injection() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let state = bootstrap::bootstrap_from_paths(
        temp_dir.path().join("app-data"),
        temp_dir.path().join("home"),
    )
    .await
    .expect("bootstrap should succeed");
    let connection = state
        .pool
        .get()
        .await
        .expect("connection should be available");
    connection
        .interact(|connection| {
            project_repo::upsert_project_snapshot(connection, snapshot(), Vec::new(), 1)?;
            session_repo::persist_indexed_file_result(
                connection,
                &[
                    session("old-low", 1_777_000_000, 1_000, 10),
                    session("new-high", 1_777_002_000, 4_000, 80),
                    session("mid", 1_777_001_000, 2_000, 30),
                ],
                &SessionIndexState {
                    source_path: "/tmp/source.jsonl".to_string(),
                    source: SessionSource::Claude,
                    file_size: 10,
                    file_mtime: Some(1),
                    last_parsed_byte_offset: 10,
                    live_partial: false,
                    last_error: None,
                },
                1,
            )?;

            Ok::<_, gsd_dashboard::error::AppError>(())
        })
        .await
        .expect("interaction should complete")
        .expect("fixtures should insert");

    let first_page = list_project_sessions_for_app(
        &state,
        "project-1",
        Some("tokensIn"),
        Some("desc"),
        None,
        None,
    )
    .await
    .expect("sessions should load");
    assert_eq!(first_page.page_size, 50);
    assert_eq!(first_page.total, 3);
    assert_eq!(first_page.rows[0].id, "new-high");

    let second_page = list_project_sessions_for_app(
        &state,
        "project-1",
        Some("durationMs"),
        Some("asc"),
        Some(2),
        Some(2),
    )
    .await
    .expect("sessions should load");
    assert_eq!(second_page.page, 2);
    assert_eq!(second_page.page_size, 2);
    assert_eq!(second_page.rows.len(), 1);
    assert_eq!(second_page.rows[0].id, "new-high");

    let source_sorted =
        list_project_sessions_for_app(&state, "project-1", Some("source"), Some("asc"), None, None)
            .await
            .expect("source sort should load");
    assert_eq!(source_sorted.total, 3);
    assert_eq!(source_sorted.rows[0].source, "claude");

    let token_total_sorted = list_project_sessions_for_app(
        &state,
        "project-1",
        Some("tokenTotal"),
        Some("desc"),
        None,
        None,
    )
    .await
    .expect("token total sort should load");
    assert_eq!(token_total_sorted.rows[0].id, "new-high");

    let error = list_project_sessions_for_app(
        &state,
        "project-1",
        Some("startedAt; DROP TABLE sessions; --"),
        Some("desc"),
        None,
        None,
    )
    .await
    .expect_err("injected sort should be rejected");
    assert!(error.to_string().contains("invalid session sort"));

    let connection = state
        .pool
        .get()
        .await
        .expect("connection should be available");
    let count = connection
        .interact(|connection| {
            connection
                .query_row("SELECT COUNT(*) FROM sessions", [], |row| {
                    row.get::<_, i64>(0)
                })
                .map_err(gsd_dashboard::error::AppError::from)
        })
        .await
        .expect("interaction should complete")
        .expect("sessions table should remain queryable");
    assert_eq!(count, 3);
}
