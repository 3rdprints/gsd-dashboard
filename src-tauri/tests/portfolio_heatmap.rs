use gsd_dashboard::{
    bootstrap,
    commands::projects::load_portfolio_heatmap_for_app,
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

fn indexed_session(id: &str, project_id: &str, started_at: i64) -> IndexedSession {
    IndexedSession {
        id: id.to_string(),
        source: SessionSource::Claude,
        source_path: format!("/tmp/{id}.jsonl"),
        source_session_id: Some(id.to_string()),
        project_id: Some(project_id.to_string()),
        cwd: None,
        started_at: Some(started_at),
        ended_at: Some(started_at + 1_000),
        duration_ms: Some(1_000),
        message_count: 1,
        tokens_in: Some(4),
        tokens_out: Some(6),
        model: Some("test-model".to_string()),
        attribution_method: "cwd".to_string(),
        index_error: None,
    }
}

#[tokio::test]
async fn portfolio_heatmap_zero_fills_ninety_days_and_clamps_range() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let state = bootstrap::bootstrap_from_paths(
        temp_dir.path().join("app-data"),
        temp_dir.path().join("home"),
    )
    .await
    .expect("bootstrap should succeed");
    let now_ms = 1_777_132_245_000_i64;
    let today_start_ms = now_ms - now_ms.rem_euclid(DAY_MS);
    let active_day_ms = today_start_ms - (3 * DAY_MS) + 3_600_000;

    state
        .pool
        .get()
        .await
        .expect("connection should be available")
        .interact(move |connection| {
            gsd_dashboard::store::project_repo::upsert_project_snapshot(
                connection,
                project_snapshot("alpha", "Alpha"),
                Vec::new(),
                1,
            )?;
            let sessions = vec![indexed_session("active-session", "alpha", active_day_ms)];
            let index_state = SessionIndexState {
                source_path: "/tmp/heatmap-source.jsonl".to_string(),
                source: SessionSource::Claude,
                file_size: 100,
                file_mtime: Some(1),
                last_parsed_byte_offset: 100,
                live_partial: false,
                last_error: None,
            };
            session_repo::persist_indexed_file_result(connection, &sessions, &index_state, 1)?;
            daily_activity::rebuild_window(connection, 90, now_ms)
        })
        .await
        .expect("interaction should complete")
        .expect("daily activity should rebuild");

    let default_rows = load_portfolio_heatmap_for_app(&state, None)
        .await
        .expect("default heatmap should load");
    assert_eq!(default_rows.len(), 90);
    assert_eq!(
        default_rows
            .iter()
            .filter(|row| row.session_count == 0)
            .count(),
        89
    );

    let active_row = default_rows
        .iter()
        .find(|row| row.session_count == 1)
        .expect("active day should be present");
    assert_eq!(active_row.token_total, 10);
    assert_eq!(active_row.top_project_id.as_deref(), Some("alpha"));
    assert_eq!(active_row.top_project_name.as_deref(), Some("Alpha"));

    let min_rows = load_portfolio_heatmap_for_app(&state, Some(0))
        .await
        .expect("minimum clamp should load");
    assert_eq!(min_rows.len(), 1);

    let max_rows = load_portfolio_heatmap_for_app(&state, Some(500))
        .await
        .expect("maximum clamp should load");
    assert_eq!(max_rows.len(), 365);
}
