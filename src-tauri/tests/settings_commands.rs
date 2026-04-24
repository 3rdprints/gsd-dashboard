use gsd_dashboard::{
    bootstrap,
    commands::settings::{
        get_boot_status_from_state, get_settings_from_state, save_settings_for_app,
    },
    error::AppError,
    settings::{SettingsInput, TrayBarSort},
};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tauri::Listener;

fn settings_input(scan_roots: Vec<String>) -> SettingsInput {
    SettingsInput {
        scan_roots,
        hidden_project_ids: Vec::new(),
        autostart_enabled: false,
        tray_bar_max_projects: 8,
        tray_bar_sort: TrayBarSort::RecentActivity,
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
    assert_eq!(status.migrations_applied, 1);
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
