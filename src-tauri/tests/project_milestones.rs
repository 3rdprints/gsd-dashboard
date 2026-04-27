use gsd_dashboard::{
    bootstrap,
    commands::projects::get_project_milestones_for_app,
    store::project_repo::{self, StoredPhasePlan, StoredPlanItem, StoredProjectSnapshot},
};

fn snapshot() -> StoredProjectSnapshot {
    StoredProjectSnapshot {
        id: "project-1".to_string(),
        name: "Project One".to_string(),
        root_path: "/tmp/project-one".to_string(),
        planning_path: "/tmp/project-one/.planning".to_string(),
        current_milestone_name: Some("v1.0".to_string()),
        current_milestone_index: Some(0),
        current_phase_number: Some("02".to_string()),
        current_phase_name: Some("Build Detail".to_string()),
        milestone_progress_pct: 0.0,
        next_command: "/gsd-next".to_string(),
        parsed_blob: "{\"state_excerpt\":\"## Current Position\\nPhase 02\"}".to_string(),
        parse_error: None,
        last_activity_at: None,
        last_scanned_at: 1_777_000_000,
        created_at: 0,
        updated_at: 0,
    }
}

fn phase(number: &str) -> StoredPhasePlan {
    StoredPhasePlan {
        project_id: "project-1".to_string(),
        phase_number: number.to_string(),
        phase_name: Some(format!("Phase {number}")),
        plan_number: Some("01".to_string()),
        plan_path: format!(".planning/phases/{number}/PLAN.md"),
        checklist_json: "[]".to_string(),
        updated_at: 0,
    }
}

#[tokio::test]
async fn hybrid_progress_math_uses_roadmap_and_plan_fallbacks() {
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
                snapshot(),
                vec![phase("01"), phase("02"), phase("03")],
                1_777_000_001,
            )?;
            connection.execute(
                "UPDATE phase_plans SET completed_at = ?1 WHERE project_id = ?2 AND phase_number = ?3",
                (1_777_000_100_i64, "project-1", "01"),
            )?;
            project_repo::replace_plan_items(
                connection,
                "project-1",
                ".planning/phases/02/PLAN.md",
                vec![
                    item(0, true),
                    item(1, true),
                    item(2, false),
                    item(3, false),
                ],
            )?;

            Ok::<_, gsd_dashboard::error::AppError>(())
        })
        .await
        .expect("interaction should complete")
        .expect("fixtures should insert");

    let milestones = get_project_milestones_for_app(&state, "project-1")
        .await
        .expect("milestones should load");

    assert_eq!(milestones.len(), 1);
    assert_eq!(milestones[0].name.as_deref(), Some("v1.0"));
    assert_eq!(milestones[0].phase_count, 3);
    assert_eq!(milestones[0].completed_phase_count, 1);
    assert!((milestones[0].progress_pct - 50.0).abs() < f64::EPSILON);
    assert_eq!(milestones[0].phases[1].completed_plan_count, 2);
    assert_eq!(milestones[0].phases[1].total_plan_count, 4);
    assert!(milestones[0].phases[1].is_current);
}

fn item(ord: i64, checked: bool) -> StoredPlanItem {
    StoredPlanItem {
        project_id: "project-1".to_string(),
        plan_path: ".planning/phases/02/PLAN.md".to_string(),
        ord,
        text: format!("Plan item {ord}"),
        checked,
        line_no: ord + 1,
    }
}
