use gsd_dashboard::{
    bootstrap,
    commands::sessions::{get_global_chart_data_for_app, GlobalSessionFilters},
    sessions::{repo as session_repo, IndexedSession, SessionIndexState, SessionSource},
    store::project_repo::{self, StoredProjectSnapshot},
};

const DAY_MS: i64 = 86_400_000;
const HOUR_MS: i64 = 3_600_000;

fn snapshot(id: &str, name: &str) -> StoredProjectSnapshot {
    StoredProjectSnapshot {
        id: id.to_string(),
        name: name.to_string(),
        root_path: format!("/tmp/{id}"),
        planning_path: format!("/tmp/{id}/.planning"),
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
    project_id: &str,
    started_at: i64,
    tokens_in: i64,
    tokens_out: i64,
) -> IndexedSession {
    IndexedSession {
        id: id.to_string(),
        source,
        source_path: format!("/tmp/{id}.jsonl"),
        source_session_id: Some(id.to_string()),
        project_id: Some(project_id.to_string()),
        cwd: Some(format!("/tmp/{project_id}")),
        started_at: Some(started_at),
        ended_at: Some(started_at + 1_000),
        duration_ms: Some(1_000),
        message_count: 2,
        tokens_in: Some(tokens_in),
        tokens_out: Some(tokens_out),
        model: Some("test-model".to_string()),
        attribution_method: "cwd".to_string(),
        index_error: None,
    }
}

#[tokio::test]
async fn global_chart_data_returns_four_aggregate_series() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let state = bootstrap::bootstrap_from_paths(
        temp_dir.path().join("app-data"),
        temp_dir.path().join("home"),
    )
    .await
    .expect("bootstrap should succeed");
    let day_one = 1_777_000_000_000_i64;
    let day_two = day_one + DAY_MS;

    let connection = state.pool.get().await.expect("connection should be available");
    connection
        .interact(move |connection| {
            for id in 1..=6 {
                project_repo::upsert_project_snapshot(
                    connection,
                    snapshot(&format!("project-{id}"), &format!("Project {id}")),
                    Vec::new(),
                    1,
                )?;
            }
            session_repo::persist_indexed_file_result(
                connection,
                &[
                    session("claude-one", SessionSource::Claude, "project-1", day_one + HOUR_MS, 10, 5),
                    session("codex-one", SessionSource::Codex, "project-2", day_one + (2 * HOUR_MS), 20, 5),
                    session("claude-two", SessionSource::Claude, "project-1", day_two + HOUR_MS, 1, 1),
                    session("project-three", SessionSource::Codex, "project-3", day_two + (3 * HOUR_MS), 30, 1),
                    session("project-four", SessionSource::Codex, "project-4", day_two + (4 * HOUR_MS), 40, 1),
                    session("project-five", SessionSource::Codex, "project-5", day_two + (5 * HOUR_MS), 50, 1),
                    session("project-six", SessionSource::Codex, "project-6", day_two + (6 * HOUR_MS), 60, 1),
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
                (3_i64, 4_i64, "claude-two"),
            )?;

            Ok::<_, gsd_dashboard::error::AppError>(())
        })
        .await
        .expect("interaction should complete")
        .expect("fixtures should insert");

    let chart_data = get_global_chart_data_for_app(
        &state,
        GlobalSessionFilters {
            started_after: Some(day_one),
            started_before: Some(day_two + DAY_MS),
            ..GlobalSessionFilters::default()
        },
    )
    .await
    .expect("chart data should load");

    assert_eq!(
        chart_data
            .sessions_per_day_by_source
            .iter()
            .map(|row| row.claude + row.codex)
            .sum::<i64>(),
        7
    );
    assert!(chart_data
        .sessions_per_day_by_source
        .iter()
        .any(|row| row.claude == 1 && row.codex == 1));
    assert_eq!(
        chart_data
            .tokens_per_day_by_project
            .iter()
            .map(|row| row.tokens)
            .sum::<i64>(),
        224
    );
    assert!(chart_data
        .tokens_per_day_by_project
        .iter()
        .any(|row| row.project_id.is_none() && row.project_name == "Other"));
    assert_eq!(chart_data.time_of_day_histogram.len(), 24);
    assert_eq!(
        chart_data
            .time_of_day_histogram
            .iter()
            .map(|row| row.count)
            .sum::<i64>(),
        7
    );
    assert_eq!(chart_data.day_of_week_distribution.len(), 7);
    assert_eq!(
        chart_data
            .day_of_week_distribution
            .iter()
            .map(|row| row.count)
            .sum::<i64>(),
        7
    );
}
