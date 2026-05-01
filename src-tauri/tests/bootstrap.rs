use gsd_dashboard::{
    app_state::{AppState, BootStatus},
    bootstrap,
    error::AppError,
    events::AppEvent,
};
use tauri::Manager;

const EXPECTED_MIGRATIONS_APPLIED: u32 = gsd_dashboard::store::migrations::MIGRATION_COUNT;

#[test]
fn app_error_invalid_scan_root_serializes_stable_fields() {
    let error = AppError::InvalidScanRoot {
        path: "/".to_string(),
        reason: "too broad".to_string(),
    };
    let value = serde_json::to_value(error).expect("error should serialize");

    assert_eq!(value["kind"], "invalidScanRoot");
    assert_eq!(value["path"], "/");
    assert_eq!(value["reason"], "too broad");
    assert_eq!(value["message"], "too broad");
}

#[test]
fn app_event_settings_changed_uses_tagged_contract() {
    let value = serde_json::to_value(AppEvent::SettingsChanged).expect("event should serialize");

    assert_eq!(value["event"], "settingsChanged");
    assert!(value.get("data").is_none());

    let boot_value = serde_json::to_value(AppEvent::BootReady {
        cache_path: "/tmp/cache.db".to_string(),
    })
    .expect("event should serialize");

    assert_eq!(boot_value["event"], "bootReady");
    assert_eq!(boot_value["data"]["cache_path"], "/tmp/cache.db");
}

#[test]
fn daily_activity_updated_serializes() {
    let value =
        serde_json::to_value(AppEvent::DailyActivityUpdated).expect("event should serialize");

    assert_eq!(value["event"], "daily_activity_updated");
    assert!(value.get("data").is_none());
}

#[test]
fn live_update_events_serialize_tiny_payloads() {
    let project_value = serde_json::to_value(AppEvent::ProjectUpdated {
        id: "project-1".to_string(),
    })
    .expect("event should serialize");
    assert_eq!(project_value["event"], "project:updated");
    assert_eq!(project_value["data"]["id"], "project-1");
    assert_eq!(
        project_value["data"]
            .as_object()
            .expect("payload should be an object")
            .len(),
        1
    );

    let session_value = serde_json::to_value(AppEvent::SessionNew {
        id: "session-1".to_string(),
        project_id: Some("project-1".to_string()),
    })
    .expect("event should serialize");
    assert_eq!(session_value["event"], "session:new");
    assert_eq!(session_value["data"]["id"], "session-1");
    assert_eq!(session_value["data"]["project_id"], "project-1");
    assert_eq!(
        session_value["data"]
            .as_object()
            .expect("payload should be an object")
            .len(),
        2
    );

    let watcher_value =
        serde_json::to_value(AppEvent::WatcherStatusChanged).expect("event should serialize");
    assert_eq!(watcher_value["event"], "watcher:status-changed");
    assert!(watcher_value.get("data").is_none());
}

#[tokio::test]
async fn bootstrap_initializes_owned_watcher_runtime_status() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let state = bootstrap::bootstrap_from_paths(
        temp_dir.path().join("app-data"),
        temp_dir.path().join("home"),
    )
    .await
    .expect("bootstrap should succeed");

    let status = state.watcher_runtime.status();

    assert!(status.roots.is_empty());
}

#[tokio::test]
async fn bootstrap_paths_create_cache_and_ready_boot_status() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let app_data_dir = temp_dir.path().join("app-data");
    let home_dir = temp_dir.path().join("home");
    std::fs::create_dir_all(&home_dir).expect("home dir should be created");

    let state = bootstrap::bootstrap_from_paths(app_data_dir.clone(), home_dir.clone())
        .await
        .expect("bootstrap should succeed");

    assert_eq!(state.home_dir, home_dir);
    assert_eq!(state.app_data_dir, app_data_dir);
    assert_eq!(state.cache_path, state.app_data_dir.join("cache.db"));
    assert!(state.cache_path.exists());
    assert_eq!(
        state.boot_status,
        BootStatus {
            app_data_dir: state.app_data_dir.display().to_string(),
            cache_path: state.cache_path.display().to_string(),
            cache_ready: true,
            wal_enabled: true,
            migrations_applied: EXPECTED_MIGRATIONS_APPLIED,
            settings_initialized: true,
        }
    );
}

#[test]
fn tauri_setup_manages_app_state_before_commands_run() {
    let mut app = tauri::test::mock_builder()
        .setup(|app| {
            let temp_dir = tempfile::tempdir().expect("temp dir should be created");
            let app_data_dir = temp_dir.path().join("app-data");
            let home_dir = temp_dir.path().join("home");
            std::fs::create_dir_all(&home_dir).expect("home dir should be created");
            let handle = tauri::async_runtime::handle();
            let state = handle.block_on(bootstrap::bootstrap_from_paths(app_data_dir, home_dir))?;
            app.manage(state);
            Ok(())
        })
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .expect("app should build");

    #[allow(deprecated)]
    app.run_iteration(|_, _| {});

    let state = app.state::<AppState>();
    assert!(state.boot_status.cache_ready);
    assert!(state.boot_status.wal_enabled);
    assert_eq!(
        state.boot_status.migrations_applied,
        EXPECTED_MIGRATIONS_APPLIED
    );
    assert!(state.boot_status.settings_initialized);
}
