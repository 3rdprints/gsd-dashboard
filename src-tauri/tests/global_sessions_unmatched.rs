use gsd_dashboard::{
    bootstrap,
    commands::sessions::{list_global_sessions_for_app, GlobalSessionFilters},
    sessions::{repo as session_repo, IndexedSession, SessionIndexState, SessionSource},
    store::project_repo::{self, StoredProjectSnapshot},
};

fn snapshot() -> StoredProjectSnapshot {
    StoredProjectSnapshot {
        id: "project-1".to_string(),
        name: "Project One".to_string(),
        root_path: "/tmp/project-1".to_string(),
        planning_path: "/tmp/project-1/.planning".to_string(),
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

fn session(id: &str, project_id: Option<&str>, started_at: i64) -> IndexedSession {
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
        message_count: 2,
        tokens_in: Some(1),
        tokens_out: Some(2),
        model: None,
        attribution_method: if project_id.is_some() {
            "cwd"
        } else {
            "unmatched"
        }
        .to_string(),
        index_error: None,
    }
}

#[tokio::test]
async fn global_sessions_unmatched_uses_partial_index() {
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
                    session("matched", Some("project-1"), 1_777_001_000),
                    session("unmatched-new", None, 1_777_003_000),
                    session("unmatched-old", None, 1_777_002_000),
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
            unmatched_only: Some(true),
            ..GlobalSessionFilters::default()
        },
        None,
        None,
        Some(1),
        Some(100),
    )
    .await
    .expect("unmatched sessions should load");

    assert_eq!(page.total, 2);
    assert!(page.rows.iter().all(|row| row.project_id.is_none()));
    assert_eq!(
        page.rows
            .iter()
            .map(|row| row.id.as_str())
            .collect::<Vec<_>>(),
        vec!["unmatched-new", "unmatched-old"]
    );

    let connection = state
        .pool
        .get()
        .await
        .expect("connection should be available");
    let plan = connection
        .interact(|connection| {
            connection
                .query_row(
                    "EXPLAIN QUERY PLAN
                     SELECT s.id FROM sessions AS s INDEXED BY idx_sessions_unmatched_started
                     WHERE s.project_id IS NULL
                     ORDER BY COALESCE(s.started_at, 0) DESC, s.id ASC
                     LIMIT 100",
                    [],
                    |row| row.get::<_, String>(3),
                )
                .map_err(gsd_dashboard::error::AppError::from)
        })
        .await
        .expect("interaction should complete")
        .expect("query plan should load");
    assert!(
        plan.contains("idx_sessions_unmatched_started"),
        "expected partial unmatched index in query plan, got {plan}"
    );
}
