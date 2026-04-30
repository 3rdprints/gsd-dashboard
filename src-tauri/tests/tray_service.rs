use gsd_dashboard::{
    bootstrap,
    settings::TrayBarSort,
    store::project_repo::StoredProjectSnapshot,
    tray::{
        menu::TrayMenuAction,
        service::{
            build_tray_state_for_app, build_tray_state_from_parts, request_tray_refresh,
            resolve_menu_action, TRAY_REFRESH_DEBOUNCE_MS,
        },
    },
};

fn snapshot(
    id: &str,
    name: &str,
    progress: f64,
    command: &str,
    activity: i64,
) -> StoredProjectSnapshot {
    StoredProjectSnapshot {
        id: id.to_string(),
        name: name.to_string(),
        root_path: format!("/tmp/{id}"),
        planning_path: format!("/tmp/{id}/.planning"),
        current_milestone_name: Some("v1".to_string()),
        current_milestone_index: Some(1),
        current_phase_number: Some("06".to_string()),
        current_phase_name: Some("Tray".to_string()),
        milestone_progress_pct: progress,
        next_command: command.to_string(),
        parsed_blob: "{}".to_string(),
        parse_error: None,
        last_activity_at: Some(activity),
        last_scanned_at: activity,
        created_at: activity,
        updated_at: activity,
    }
}

#[test]
fn tray_refresh_state_uses_same_visible_set_for_icon_menu_and_commands() {
    let tray_state = build_tray_state_from_parts(
        vec![
            snapshot("alpha", "Alpha", 10.0, "/gsd-next alpha", 30),
            snapshot("bravo", "Bravo", 60.0, "/gsd-next bravo", 20),
            snapshot("charlie", "Charlie", 90.0, "/gsd-next charlie", 10),
        ],
        &["alpha".to_string()],
        &["charlie".to_string()],
        TrayBarSort::Progress,
        8,
    )
    .expect("tray state should build");

    assert_eq!(tray_state.projects.len(), 1);
    assert_eq!(tray_state.projects[0].id, "bravo");
    assert_eq!(
        tray_state.commands_by_project_id.get("bravo"),
        Some(&"/gsd-next bravo".to_string())
    );
    assert!(!tray_state.commands_by_project_id.contains_key("alpha"));
    assert!(tray_state.tooltip.contains("1 active projects"));
    assert!(tray_state.icon_png.starts_with(b"\x89PNG\r\n\x1a\n"));
}

#[test]
fn menu_action_resolution_accepts_fixed_ids_and_visible_project_scoped_ids_only() {
    let tray_state = build_tray_state_from_parts(
        vec![snapshot("alpha", "Alpha", 10.0, "/gsd-next alpha", 30)],
        &[],
        &[],
        TrayBarSort::Name,
        8,
    )
    .expect("tray state should build");

    assert_eq!(
        resolve_menu_action("show_dashboard", &tray_state),
        Some(TrayMenuAction::ShowDashboard)
    );
    assert_eq!(
        resolve_menu_action("preferences", &tray_state),
        Some(TrayMenuAction::Preferences)
    );
    assert_eq!(
        resolve_menu_action("quit", &tray_state),
        Some(TrayMenuAction::Quit)
    );
    assert_eq!(
        resolve_menu_action("project:alpha", &tray_state),
        Some(TrayMenuAction::OpenProject {
            project_id: "alpha".to_string()
        })
    );
    assert_eq!(
        resolve_menu_action("copy_next:alpha", &tray_state),
        Some(TrayMenuAction::CopyNextCommand {
            project_id: "alpha".to_string()
        })
    );
    assert_eq!(resolve_menu_action("project:missing", &tray_state), None);
    assert_eq!(resolve_menu_action("copy_next:missing", &tray_state), None);
}

#[tokio::test]
async fn build_tray_state_for_app_loads_settings_and_project_snapshots() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let state = bootstrap::bootstrap_from_paths(
        temp_dir.path().join("app-data"),
        temp_dir.path().join("home"),
    )
    .await
    .expect("bootstrap should succeed");

    let tray_state = build_tray_state_for_app(&state)
        .await
        .expect("empty tray state should build");

    assert!(tray_state.projects.is_empty());
    assert!(tray_state.icon_png.starts_with(b"\x89PNG\r\n\x1a\n"));
}

#[tokio::test]
async fn refresh_request_api_schedules_without_awaiting_db_work() {
    let app = tauri::test::mock_builder()
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .expect("app should build");

    request_tray_refresh(app.handle());

    assert_eq!(TRAY_REFRESH_DEBOUNCE_MS, 250);
}
