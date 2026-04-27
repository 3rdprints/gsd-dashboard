use gsd_dashboard::{
    bootstrap,
    commands::sessions::{list_global_sessions_for_app, GlobalSessionFilters},
    sessions::{repo as session_repo, IndexedSession, SessionIndexState, SessionSource},
    store::project_repo::{self, StoredProjectSnapshot},
};

fn snapshot(id: &str, name: &str, root_path: &str) -> StoredProjectSnapshot {
    StoredProjectSnapshot {
        id: id.to_string(),
        name: name.to_string(),
        root_path: root_path.to_string(),
        planning_path: format!("{root_path}/.planning"),
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

fn session(
    id: &str,
    source: SessionSource,
    project_id: Option<&str>,
    started_at: i64,
    duration_ms: i64,
    tokens_in: i64,
    tokens_out: i64,
) -> IndexedSession {
    IndexedSession {
        id: id.to_string(),
        source,
        source_path: format!("/tmp/{id}.jsonl"),
        source_session_id: Some(id.to_string()),
        project_id: project_id.map(str::to_string),
        cwd: project_id.map(|id| format!("/tmp/{id}")),
        started_at: Some(started_at),
        ended_at: Some(started_at + duration_ms),
        duration_ms: Some(duration_ms),
        message_count: 2,
        tokens_in: Some(tokens_in),
        tokens_out: Some(tokens_out),
        model: Some("test-model".to_string()),
        attribution_method: if project_id.is_some() { "cwd" } else { "unmatched" }.to_string(),
        index_error: None,
    }
}

#[tokio::test]
async fn global_sessions_query_combines_filters() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let state = bootstrap::bootstrap_from_paths(
        temp_dir.path().join("app-data"),
        temp_dir.path().join("home"),
    )
    .await
    .expect("bootstrap should succeed");

    let connection = state.pool.get().await.expect("connection should be available");
    connection
        .interact(|connection| {
            project_repo::upsert_project_snapshot(
                connection,
                snapshot("project-1", "Project One", "/tmp/project-1"),
                Vec::new(),
                1,
            )?;
            project_repo::upsert_project_snapshot(
                connection,
                snapshot("project-2", "Project Two", "/tmp/project-2"),
                Vec::new(),
                1,
            )?;
            session_repo::persist_indexed_file_result(
                connection,
                &[
                    session("match-new", SessionSource::Claude, Some("project-1"), 1_777_003_000, 4_000, 80, 10),
                    session("match-old", SessionSource::Claude, Some("project-1"), 1_777_001_000, 3_000, 30, 10),
                    session("codex-project", SessionSource::Codex, Some("project-1"), 1_777_002_000, 4_000, 90, 10),
                    session("other-project", SessionSource::Claude, Some("project-2"), 1_777_004_000, 4_000, 90, 10),
                    session("too-short", SessionSource::Claude, Some("project-1"), 1_777_005_000, 500, 90, 10),
                    session("too-few-tokens", SessionSource::Claude, Some("project-1"), 1_777_006_000, 4_000, 1, 1),
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

    let page = list_global_sessions_for_app(
        &state,
        GlobalSessionFilters {
            source: Some("claude".to_string()),
            project_id: Some("project-1".to_string()),
            started_after: Some(1_777_000_500),
            started_before: Some(1_777_004_000),
            duration_min_ms: Some(1_000),
            duration_max_ms: Some(5_000),
            tokens_min: Some(40),
            tokens_max: Some(100),
            unmatched_only: Some(false),
        },
        None,
        Some(500),
    )
    .await
    .expect("global sessions should load");

    assert_eq!(page.page, 1);
    assert_eq!(page.page_size, 200);
    assert_eq!(page.total, 2);
    assert_eq!(
        page.rows.iter().map(|row| row.id.as_str()).collect::<Vec<_>>(),
        vec!["match-new", "match-old"]
    );
    assert!(page.rows[0].started_at >= page.rows[1].started_at);

    let error = list_global_sessions_for_app(
        &state,
        GlobalSessionFilters {
            source: Some("claude'; DROP TABLE sessions; --".to_string()),
            ..GlobalSessionFilters::default()
        },
        None,
        None,
    )
    .await
    .expect_err("injected source should be rejected");
    assert!(error.to_string().contains("invalid session source"));

    let connection = state.pool.get().await.expect("connection should be available");
    let count = connection
        .interact(|connection| {
            connection
                .query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get::<_, i64>(0))
                .map_err(gsd_dashboard::error::AppError::from)
        })
        .await
        .expect("interaction should complete")
        .expect("sessions table should remain queryable");
    assert_eq!(count, 6);
}
