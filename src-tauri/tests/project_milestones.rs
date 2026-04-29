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
        parsed_blob: r###"{
            "projectId": "project-1",
            "projectName": "Project One",
            "rootPath": "/tmp/project-one",
            "planningPath": "/tmp/project-one/.planning",
            "currentMilestone": { "index": 0, "name": "v1.0" },
            "currentPhase": { "number": "02", "name": "Build Detail" },
            "milestoneProgressPct": 0,
            "roadmapPhases": [],
            "phasePlans": [],
            "stateExcerpt": "## Current Position\nPhase 02",
            "nextCommand": "/gsd-next",
            "config": null,
            "parseIssues": []
        }"###
            .to_string(),
        parse_error: None,
        last_activity_at: None,
        last_scanned_at: 1_777_000_000,
        created_at: 0,
        updated_at: 0,
    }
}

fn roadmap_snapshot() -> StoredProjectSnapshot {
    StoredProjectSnapshot {
        current_milestone_name: Some("Beta".to_string()),
        current_milestone_index: Some(1),
        current_phase_number: Some("03".to_string()),
        current_phase_name: Some("Phase 03".to_string()),
        parsed_blob: r#"{
            "projectId": "project-1",
            "projectName": "Project One",
            "rootPath": "/tmp/project-one",
            "planningPath": "/tmp/project-one/.planning",
            "currentMilestone": { "index": 1, "name": "Beta" },
            "currentPhase": { "number": "03", "name": "Phase 03" },
            "milestoneProgressPct": 0,
            "roadmapPhases": [
                { "number": "01", "name": "Phase 01", "completed": true, "milestoneName": "Alpha" },
                { "number": "02", "name": "Phase 02", "completed": false, "milestoneName": "Alpha" },
                { "number": "03", "name": "Phase 03", "completed": false, "milestoneName": "Beta" },
                { "number": "04", "name": "Phase 04", "completed": false, "milestoneName": "Beta" }
            ],
            "phasePlans": [],
            "stateExcerpt": null,
            "nextCommand": "/gsd-next",
            "config": null,
            "parseIssues": []
        }"#
        .to_string(),
        ..snapshot()
    }
}

fn phase(number: &str) -> StoredPhasePlan {
    StoredPhasePlan {
        project_id: "project-1".to_string(),
        phase_number: number.to_string(),
        phase_name: Some(format!("Phase {number}")),
        plan_number: Some("01".to_string()),
        plan_path: format!(".planning/phases/{number}/PLAN.md"),
        completed_at: None,
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

    let connection = state
        .pool
        .get()
        .await
        .expect("connection should be available");
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
    assert!((milestones[0].progress_pct - 33.33333333333333).abs() < 1e-6);
    assert_eq!(milestones[0].phases[1].completed_plan_count, 0);
    assert_eq!(milestones[0].phases[1].total_plan_count, 1);
    assert!(milestones[0].phases[1].is_current);
}

#[tokio::test]
async fn milestone_progress_returns_all_roadmap_milestones() {
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
            project_repo::upsert_project_snapshot(connection, roadmap_snapshot(), Vec::new(), 1)?;
            Ok::<_, gsd_dashboard::error::AppError>(())
        })
        .await
        .expect("interaction should complete")
        .expect("fixtures should insert");

    let milestones = get_project_milestones_for_app(&state, "project-1")
        .await
        .expect("milestones should load");

    assert_eq!(milestones.len(), 2);
    assert_eq!(milestones[0].name.as_deref(), Some("Alpha"));
    assert_eq!(milestones[0].phase_count, 2);
    assert_eq!(milestones[0].completed_phase_count, 1);
    assert!((milestones[0].progress_pct - 50.0).abs() < 1e-6);
    assert_eq!(milestones[1].name.as_deref(), Some("Beta"));
    assert_eq!(milestones[1].phase_count, 2);
    assert!(milestones[1].phases[0].is_current);
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
