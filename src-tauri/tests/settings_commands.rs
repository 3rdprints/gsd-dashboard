use gsd_dashboard::{
    bootstrap,
    commands::settings::{
        get_boot_status_from_state, get_settings_from_state, get_watcher_status_from_state,
        save_settings_for_app,
    },
    error::AppError,
    settings::{SettingsInput, TrayBarSort},
    store::project_repo::{self, StoredProjectSnapshot},
    watcher::{WatcherReasonCategory, WatcherRootStatus},
};
use std::{
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tauri::Listener;

fn settings_input(scan_roots: Vec<String>) -> SettingsInput {
    SettingsInput {
        scan_roots,
        hidden_project_ids: Vec::new(),
        tray_hidden_project_ids: Vec::new(),
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
async fn get_boot_status_returns_cache_ready_state() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let state = bootstrap::bootstrap_from_paths(
        temp_dir.path().join("app-data"),
        temp_dir.path().join("home"),
    )
    .await
    .expect("bootstrap should succeed");

    let status = get_boot_status_from_state(&state).expect("boot status should return");

    assert!(status.cache_ready);
    assert!(status.wal_enabled);
    assert_eq!(
        status.migrations_applied,
        gsd_dashboard::store::migrations::MIGRATION_COUNT
    );
    assert!(status.settings_initialized);
}

#[tokio::test]
async fn get_settings_returns_initialized_defaults_after_boot() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let state = bootstrap::bootstrap_from_paths(
        temp_dir.path().join("app-data"),
        temp_dir.path().join("home"),
    )
    .await
    .expect("bootstrap should succeed");

    let settings = get_settings_from_state(&state)
        .await
        .expect("settings should load");

    assert_eq!(settings.scan_roots, vec!["~/Documents"]);
    assert!(settings.hidden_project_ids.is_empty());
    assert!(!settings.autostart_enabled);
}

#[tokio::test]
async fn settings_commands_get_watcher_status_returns_runtime_status_without_persisting_settings() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let state = bootstrap::bootstrap_from_paths(
        temp_dir.path().join("app-data"),
        temp_dir.path().join("home"),
    )
    .await
    .expect("bootstrap should succeed");
    state
        .watcher_runtime
        .set_root_status(WatcherRootStatus::polling(
            "/tmp/project/.planning".to_string(),
            WatcherReasonCategory::Filesystem,
        ));

    let status = get_watcher_status_from_state(&state)
        .await
        .expect("watcher status should return");

    assert_eq!(status.roots.len(), 1);
    assert_eq!(status.roots[0].root, "/tmp/project/.planning");
    assert_eq!(
        status.roots[0].reason.as_deref(),
        Some("Filesystem does not support native watching")
    );
}

#[tokio::test]
async fn save_settings_delegates_validation_and_emits_invalidation() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let mut app = tauri::test::mock_builder()
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .expect("app should build");
    #[allow(deprecated)]
    app.run_iteration(|_, _| {});

    let state = bootstrap::bootstrap_from_paths(
        temp_dir.path().join("app-data"),
        temp_dir.path().join("home"),
    )
    .await
    .expect("bootstrap should succeed");

    let event_seen = Arc::new(AtomicBool::new(false));
    let event_seen_clone = Arc::clone(&event_seen);
    app.listen("settings-changed", move |_| {
        event_seen_clone.store(true, Ordering::SeqCst);
    });

    let saved = save_settings_for_app(
        app.handle(),
        &state,
        settings_input(vec!["~/Documents".to_string(), "~/homegit".to_string()]),
    )
    .await
    .expect("valid settings should save");

    assert_eq!(saved.scan_roots.len(), 2);
    assert!(event_seen.load(Ordering::SeqCst));

    let rejected =
        save_settings_for_app(app.handle(), &state, settings_input(vec!["/".to_string()]))
            .await
            .expect_err("broad root should be rejected");

    match rejected {
        AppError::InvalidScanRoot { path, .. } => assert_eq!(path, "/"),
        other => panic!("expected invalid scan root, got {other:?}"),
    }
}

#[tokio::test]
async fn save_settings_restarts_watcher_status_for_current_roots() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let home_dir = temp_dir.path().join("home");
    let project_root = home_dir.join("work/project-one");
    std::fs::create_dir_all(project_root.join(".planning")).expect("planning dir should exist");

    let mut app = tauri::test::mock_builder()
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .expect("app should build");
    #[allow(deprecated)]
    app.run_iteration(|_, _| {});

    let state = bootstrap::bootstrap_from_paths(temp_dir.path().join("app-data"), home_dir.clone())
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

    let watcher_event_seen = Arc::new(AtomicBool::new(false));
    let watcher_event_seen_clone = Arc::clone(&watcher_event_seen);
    app.listen("watcher:status-changed", move |_| {
        watcher_event_seen_clone.store(true, Ordering::SeqCst);
    });

    save_settings_for_app(
        app.handle(),
        &state,
        settings_input(vec!["~/work".to_string()]),
    )
    .await
    .expect("settings should save and watcher should restart");

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
    while std::time::Instant::now() < deadline {
        if watcher_event_seen.load(Ordering::SeqCst) {
            break;
        }
        #[allow(deprecated)]
        app.run_iteration(|_, _| {});
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }

    let status = get_watcher_status_from_state(&state)
        .await
        .expect("watcher status should return");
    assert!(state.watcher_runtime.is_running());
    assert_eq!(status.roots.len(), 1);
    assert_eq!(
        status.roots[0].root,
        project_root.join(".planning").display().to_string()
    );
    assert!(watcher_event_seen.load(Ordering::SeqCst));
}
