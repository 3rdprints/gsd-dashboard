use std::{
    fs,
    path::{Path, PathBuf},
};

use gsd_dashboard::{
    bootstrap,
    settings::{SettingsInput, TrayBarSort},
    store::project_repo::{self, StoredProjectSnapshot},
    watcher::{
        derive_watcher_roots, ProjectDebouncer, WatcherMode, WatcherReasonCategory,
        WatcherRootStatus, WatcherRuntime, POLLING_INTERVAL_SECONDS, PROJECT_DEBOUNCE_MS,
    },
};

fn settings_input(scan_roots: Vec<String>) -> SettingsInput {
    SettingsInput {
        scan_roots,
        hidden_project_ids: Vec::new(),
        autostart_enabled: false,
        tray_bar_max_projects: 8,
        tray_bar_sort: TrayBarSort::RecentActivity,
        global_sessions_default_range: "7d".to_string(),
    }
}

fn project_snapshot(root_path: &Path) -> StoredProjectSnapshot {
    StoredProjectSnapshot {
        id: "project-1".to_string(),
        name: "Project One".to_string(),
        root_path: root_path.display().to_string(),
        planning_path: root_path.join(".planning").display().to_string(),
        current_milestone_name: Some("v1.0".to_string()),
        current_milestone_index: Some(1),
        current_phase_number: Some("07".to_string()),
        current_phase_name: Some("Live Updates".to_string()),
        milestone_progress_pct: 40.0,
        next_command: "/gsd-next".to_string(),
        parsed_blob: "{}".to_string(),
        parse_error: None,
        last_activity_at: Some(1_777_000_100),
        last_scanned_at: 1_777_000_200,
        created_at: 0,
        updated_at: 0,
    }
}

#[tokio::test]
async fn live_updates_watcher_registers_only_discovered_planning_and_existing_session_roots() {
    // LIVE-01, T-07-01: watcher roots must be limited to discovered `.planning`
    // directories and existing supported session roots, excluding `/`, bare `$HOME`,
    // broad scan-root recursive watching, Phase 10 Codex roots, and archived roots.
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let home_dir = temp_dir.path().join("home");
    let app_data_dir = temp_dir.path().join("app-data");
    let project_root = home_dir.join("work/project-one");
    let planning_dir = project_root.join(".planning");
    fs::create_dir_all(&planning_dir).expect("planning dir should be created");
    fs::create_dir_all(home_dir.join(".claude/projects")).expect("claude root should exist");
    fs::create_dir_all(home_dir.join(".codex/sessions")).expect("codex root should exist");
    fs::create_dir_all(home_dir.join(".codex/archived_sessions"))
        .expect("archived codex root should exist");

    let state = bootstrap::bootstrap_from_paths(app_data_dir, home_dir.clone())
        .await
        .expect("bootstrap should succeed");
    gsd_dashboard::settings::save(
        &state.pool,
        &home_dir,
        settings_input(vec!["~/work".into()]),
    )
    .await
    .expect("settings should save");
    let connection = state
        .pool
        .get()
        .await
        .expect("connection should be available");
    let snapshot = project_snapshot(&project_root);
    connection
        .interact(move |connection| {
            project_repo::upsert_project_snapshot(connection, snapshot, Vec::new(), 1)
        })
        .await
        .expect("interaction should complete")
        .expect("project should persist");
    let settings = gsd_dashboard::settings::load_or_initialize(&state.pool, &home_dir)
        .await
        .expect("settings should load");

    let roots = derive_watcher_roots(&state.pool, &home_dir, &settings)
        .await
        .expect("watcher roots should derive");
    let root_strings = roots
        .iter()
        .map(|root| root.display().to_string())
        .collect::<Vec<_>>();

    assert_eq!(
        root_strings,
        vec![
            planning_dir.display().to_string(),
            home_dir.join(".claude/projects").display().to_string(),
            home_dir.join(".codex/sessions").display().to_string(),
        ]
    );
    assert!(!root_strings
        .iter()
        .any(|root| root.contains("archived_sessions")));
}

#[tokio::test]
async fn live_updates_watcher_starts_native_supervisor_for_registered_roots() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let home_dir = temp_dir.path().join("home");
    let app_data_dir = temp_dir.path().join("app-data");
    let project_root = home_dir.join("work/project-one");
    let planning_dir = project_root.join(".planning");
    fs::create_dir_all(&planning_dir).expect("planning dir should be created");

    let state = bootstrap::bootstrap_from_paths(app_data_dir, home_dir.clone())
        .await
        .expect("bootstrap should succeed");
    let connection = state
        .pool
        .get()
        .await
        .expect("connection should be available");
    let snapshot = project_snapshot(&project_root);
    connection
        .interact(move |connection| {
            project_repo::upsert_project_snapshot(connection, snapshot, Vec::new(), 1)
        })
        .await
        .expect("interaction should complete")
        .expect("project should persist");

    let mut app = tauri::test::mock_builder()
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .expect("app should build");
    #[allow(deprecated)]
    app.run_iteration(|_, _| {});

    gsd_dashboard::watcher::start_watcher_service_for_app(app.handle().clone(), &state)
        .await
        .expect("watcher should start");

    assert!(state.watcher_runtime.is_running());
    assert_eq!(state.watcher_runtime.status().roots.len(), 1);
    assert_eq!(
        state.watcher_runtime.status().roots[0].root,
        planning_dir.display().to_string()
    );
    assert!(matches!(
        state.watcher_runtime.status().roots[0].mode,
        WatcherMode::Native | WatcherMode::Polling
    ));
}

#[tokio::test]
async fn live_updates_polling_discovers_new_project_under_scan_root() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let home_dir = temp_dir.path().join("home");
    let app_data_dir = temp_dir.path().join("app-data");
    let scan_root = home_dir.join("work");
    let project_root = scan_root.join("project-two");
    let planning_dir = project_root.join(".planning");
    fs::create_dir_all(&planning_dir).expect("planning dir should be created");
    fs::write(
        planning_dir.join("ROADMAP.md"),
        "# Roadmap\n\n## Phases\n\n- [ ] 01 - Foundation\n",
    )
    .expect("roadmap should be written");
    fs::write(
        planning_dir.join("STATE.md"),
        "# State\n\nCurrent Phase: 01 - Foundation\n",
    )
    .expect("state should be written");

    let state = bootstrap::bootstrap_from_paths(app_data_dir, home_dir.clone())
        .await
        .expect("bootstrap should succeed");
    let mut app = tauri::test::mock_builder()
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .expect("app should build");
    #[allow(deprecated)]
    app.run_iteration(|_, _| {});

    gsd_dashboard::watcher::service::poll_scan_roots_once_for_app(
        app.handle(),
        &state,
        &home_dir,
        &[scan_root],
    )
    .await;

    let connection = state
        .pool
        .get()
        .await
        .expect("connection should be available");
    let projects = connection
        .interact(project_repo::list_project_snapshots)
        .await
        .expect("interaction should complete")
        .expect("projects should load");

    assert_eq!(projects.len(), 1);
    assert_eq!(
        projects[0].planning_path,
        planning_dir.display().to_string()
    );
}

#[test]
fn live_updates_watcher_debounces_project_changes_at_500ms() {
    // LIVE-01, T-07-01: project `.planning` changes must use injected watcher/time
    // seams so debounce assertions do not depend on real OS watcher timing.
    assert_eq!(PROJECT_DEBOUNCE_MS, 500);

    let mut debouncer = ProjectDebouncer::new(PROJECT_DEBOUNCE_MS);
    let first_project = PathBuf::from("/workspace/project-one/.planning");
    let second_project = PathBuf::from("/workspace/project-two/.planning");

    debouncer.record_event(&first_project, first_project.join("STATE.md"), 0);
    debouncer.record_event(&first_project, first_project.join("ROADMAP.md"), 100);
    debouncer.record_event(&second_project, second_project.join("STATE.md"), 200);

    assert!(debouncer.take_due(599).is_empty());
    assert_eq!(debouncer.take_due(600), vec![first_project]);
    assert_eq!(debouncer.take_due(700), vec![second_project]);
    assert!(debouncer.take_due(1_000).is_empty());
}

#[test]
fn live_updates_watcher_enters_60s_polling_fallback_for_failed_root() {
    // LIVE-03, T-07-03: failed roots must expose explicit polling fallback status
    // with root, reason category, fix hint, 60s cadence, and retry state.
    let runtime = WatcherRuntime::new();
    runtime.set_root_status(WatcherRootStatus::polling(
        "/tmp/project/.planning".to_string(),
        WatcherReasonCategory::WatchLimit,
    ));

    let status = runtime.status();
    assert_eq!(status.roots.len(), 1);
    assert_eq!(status.roots[0].root, "/tmp/project/.planning");
    assert_eq!(status.roots[0].mode, WatcherMode::Polling);
    assert_eq!(
        status.roots[0].reason_category,
        Some(WatcherReasonCategory::WatchLimit)
    );
    assert_eq!(status.roots[0].polling_interval_seconds, Some(60));
    assert!(status.roots[0].retry_enabled);
    assert_eq!(POLLING_INTERVAL_SECONDS, 60);
    assert_eq!(
        status.roots[0].fix_hint.as_deref(),
        Some("Increase inotify watch limits, then wait for automatic retry.")
    );
}
