use gsd_dashboard::{
    bootstrap,
    commands::projects::get_project_chart_data_for_app,
    sessions::{repo as session_repo, IndexedSession, SessionIndexState, SessionSource},
    store::project_repo::{self, StoredPhasePlan, StoredProjectSnapshot},
};

const DAY_MS: i64 = 86_400_000;

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

fn phase(plan_number: &str) -> StoredPhasePlan {
    StoredPhasePlan {
        project_id: "project-1".to_string(),
        phase_number: "05".to_string(),
        phase_name: Some("Charts".to_string()),
        plan_number: Some(plan_number.to_string()),
        plan_path: format!(".planning/phases/05/05-{plan_number}-PLAN.md"),
        completed_at: None,
        checklist_json: "[]".to_string(),
        updated_at: 0,
    }
}

fn session(
    id: &str,
    started_at: i64,
    duration_ms: i64,
    tokens_in: i64,
    tokens_out: i64,
) -> IndexedSession {
    IndexedSession {
        id: id.to_string(),
        source: SessionSource::Codex,
        source_path: format!("/tmp/{id}.jsonl"),
        source_session_id: Some(id.to_string()),
        project_id: Some("project-1".to_string()),
        cwd: Some("/tmp/project-one".to_string()),
        started_at: Some(started_at),
        ended_at: Some(started_at + duration_ms),
        duration_ms: Some(duration_ms),
        message_count: 2,
        tokens_in: Some(tokens_in),
        tokens_out: Some(tokens_out),
        model: Some("test-model".to_string()),
        attribution_method: "cwd".to_string(),
        index_error: None,
    }
}

#[tokio::test]
async fn project_chart_data_returns_four_series_aggregates() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let state = bootstrap::bootstrap_from_paths(
        temp_dir.path().join("app-data"),
        temp_dir.path().join("home"),
    )
    .await
    .expect("bootstrap should succeed");
    let day_one = 1_777_000_000_000_i64;
    let day_two = day_one + DAY_MS;

    let connection = state
        .pool
        .get()
        .await
        .expect("connection should be available");
    connection
        .interact(move |connection| {
            project_repo::upsert_project_snapshot(
                connection,
                snapshot(),
                vec![phase("01"), phase("02")],
                1,
            )?;
            connection.execute(
                "UPDATE phase_plans SET completed_at = ?1 WHERE project_id = ?2 AND plan_path = ?3",
                (day_one, "project-1", ".planning/phases/05/05-01-PLAN.md"),
            )?;
            connection.execute(
                "UPDATE phase_plans SET completed_at = ?1 WHERE project_id = ?2 AND plan_path = ?3",
                (day_two, "project-1", ".planning/phases/05/05-02-PLAN.md"),
            )?;
            session_repo::persist_indexed_file_result(
                connection,
                &[
                    session("one", day_one, 1_000, 10, 5),
                    session("two", day_one + 1_000, 3_000, 20, 7),
                    session("three", day_two, 2_000, 1, 2),
                ],
                &SessionIndexState {
                    source_path: "/tmp/source.jsonl".to_string(),
                    source: SessionSource::Codex,
                    file_size: 10,
                    file_mtime: Some(1),
                    last_parsed_byte_offset: 10,
                    live_partial: false,
                    last_error: None,
                },
                1,
            )?;
            connection.execute(
                "UPDATE sessions
                 SET cache_read_tokens = ?1, cache_creation_tokens = ?2
                 WHERE id = ?3",
                (3_i64, 4_i64, "three"),
            )?;

            Ok::<_, gsd_dashboard::error::AppError>(())
        })
        .await
        .expect("interaction should complete")
        .expect("fixtures should insert");

    let chart_data = get_project_chart_data_for_app(&state, "project-1", Some("all"))
        .await
        .expect("chart data should load");

    assert_eq!(
        chart_data
            .sessions_per_day
            .iter()
            .map(|row| row.count)
            .sum::<i64>(),
        3
    );
    assert_eq!(
        chart_data
            .tokens_per_day
            .iter()
            .map(|row| row.tokens)
            .sum::<i64>(),
        52
    );
    assert!(chart_data
        .average_duration_per_day
        .iter()
        .any(|row| (row.average_duration_ms - 2_000.0).abs() < f64::EPSILON));
    assert_eq!(
        chart_data
            .milestone_velocity
            .iter()
            .map(|row| row.completed_plans)
            .sum::<i64>(),
        2
    );
}
