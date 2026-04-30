use gsd_dashboard::store::{self, project_repo};

fn snapshot(id: &str, root_path: &str, plan_count: usize) -> project_repo::StoredProjectSnapshot {
    project_repo::StoredProjectSnapshot {
        id: id.to_string(),
        name: "Deck Pilot".to_string(),
        root_path: root_path.to_string(),
        planning_path: format!("{root_path}/.planning"),
        current_milestone_name: Some("v1.0".to_string()),
        current_milestone_index: Some(1),
        current_phase_number: Some("02".to_string()),
        current_phase_name: Some("Parser".to_string()),
        milestone_progress_pct: 37.5 + plan_count as f64,
        next_command: String::new(),
        parsed_blob: format!(r#"{{"plans":{plan_count}}}"#),
        parse_error: None,
        last_activity_at: Some(1_777_000_100),
        last_scanned_at: 1_777_000_200,
        created_at: 0,
        updated_at: 0,
    }
}

fn phase_plan(project_id: &str, plan_number: &str) -> project_repo::StoredPhasePlan {
    project_repo::StoredPhasePlan {
        project_id: project_id.to_string(),
        phase_number: "02".to_string(),
        phase_name: Some("Planning Parser".to_string()),
        plan_number: Some(plan_number.to_string()),
        plan_path: format!(".planning/phases/02-planning-parser-scanner/02-{plan_number}-PLAN.md"),
        completed_at: None,
        checklist_json: format!(r#"[{{"label":"plan {plan_number}","done":false}}]"#),
        updated_at: 0,
    }
}

async fn migrated_connection() -> (tempfile::TempDir, deadpool_sqlite::Object) {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let db_path = temp_dir.path().join("cache.db");
    let pool = store::open_pool(&db_path).await.expect("pool should open");
    store::run_migrations(&pool)
        .await
        .expect("migrations should run");
    let conn = pool.get().await.expect("connection should be available");
    (temp_dir, conn)
}

#[tokio::test]
async fn project_snapshot_round_trip_replaces_phase_plans() {
    let (_temp_dir, conn) = migrated_connection().await;

    conn.interact(|conn| {
        project_repo::upsert_project_snapshot(
            conn,
            snapshot("project-1", "/tmp/deckpilot", 2),
            vec![phase_plan("project-1", "01"), phase_plan("project-1", "02")],
            1_777_000_300,
        )?;
        project_repo::upsert_project_snapshot(
            conn,
            snapshot("project-1", "/tmp/deckpilot", 1),
            vec![phase_plan("project-1", "03")],
            1_777_000_400,
        )?;

        let loaded = project_repo::load_project_by_root(conn, "/tmp/deckpilot")?
            .expect("project should be loaded");
        let plans = project_repo::load_phase_plans(conn, "project-1")?;

        assert_eq!(loaded.id, "project-1");
        assert_eq!(loaded.next_command, "/gsd-next");
        assert_eq!(loaded.parse_error, None);
        assert_eq!(loaded.created_at, 1_777_000_300);
        assert_eq!(loaded.updated_at, 1_777_000_400);
        assert_eq!(plans.len(), 1);
        assert_eq!(plans[0].plan_number.as_deref(), Some("03"));
        assert!(plans[0].checklist_json.contains("plan 03"));

        Ok::<_, gsd_dashboard::error::AppError>(())
    })
    .await
    .expect("interaction should complete")
    .expect("repository operations should pass");
}

#[tokio::test]
async fn parse_error_and_scan_log_are_persisted() {
    let (_temp_dir, conn) = migrated_connection().await;

    conn.interact(|conn| {
        let mut parsed_with_error = snapshot("project-2", "/tmp/listingguru", 0);
        parsed_with_error.parse_error = Some("STATE.md malformed".to_string());
        project_repo::upsert_project_snapshot(conn, parsed_with_error, Vec::new(), 1_777_000_500)?;

        project_repo::append_scan_log(
            conn,
            project_repo::StoredScanLogEntry {
                project_id: Some("project-2".to_string()),
                root_path: Some("/tmp/listingguru".to_string()),
                planning_path: Some("/tmp/listingguru/.planning".to_string()),
                file_path: Some("/tmp/listingguru/.planning/STATE.md".to_string()),
                status: "parseError".to_string(),
                message: Some("STATE.md malformed".to_string()),
                errors_json: r#"[{"kind":"state"}]"#.to_string(),
                created_at: 0,
            },
            1_777_000_600,
        )?;

        let loaded = project_repo::load_project_by_root(conn, "/tmp/listingguru")?
            .expect("project should be loaded");
        let scan_status = conn.query_row(
            "SELECT status FROM scan_log WHERE project_id = ?1",
            ["project-2"],
            |row| row.get::<_, String>(0),
        )?;

        assert_eq!(loaded.parse_error.as_deref(), Some("STATE.md malformed"));
        assert_eq!(scan_status, "parseError");

        Ok::<_, gsd_dashboard::error::AppError>(())
    })
    .await
    .expect("interaction should complete")
    .expect("repository operations should pass");
}
