use gsd_dashboard::{
    autostart::{is_autostart_launch, AutostartBackend},
    bootstrap,
    commands::settings::save_settings_with_autostart_backend,
    error::AppError,
    settings::{self, AppSettings, SettingsInput, TrayBarSort},
};
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};
use tauri::Listener;

#[test]
fn autostart_launch_arg_matches_exact_flag_only() {
    assert!(is_autostart_launch(["/app/GSD Dashboard", "--autostart",]));
    assert!(!is_autostart_launch([
        "/app/GSD Dashboard",
        "--autostart=1",
    ]));
    assert!(!is_autostart_launch(["/app/GSD Dashboard", "--startup",]));
}

#[derive(Default)]
struct FakeAutostartBackend {
    enable_calls: AtomicUsize,
    disable_calls: AtomicUsize,
    fail: AtomicBool,
}

impl FakeAutostartBackend {
    fn fail() -> Self {
        Self {
            fail: AtomicBool::new(true),
            ..Self::default()
        }
    }

    fn enable_calls(&self) -> usize {
        self.enable_calls.load(Ordering::SeqCst)
    }

    fn disable_calls(&self) -> usize {
        self.disable_calls.load(Ordering::SeqCst)
    }
}

impl AutostartBackend for FakeAutostartBackend {
    fn enable(&self) -> Result<(), AppError> {
        self.enable_calls.fetch_add(1, Ordering::SeqCst);
        if self.fail.load(Ordering::SeqCst) {
            Err(AppError::settings("autostart enable failed"))
        } else {
            Ok(())
        }
    }

    fn disable(&self) -> Result<(), AppError> {
        self.disable_calls.fetch_add(1, Ordering::SeqCst);
        if self.fail.load(Ordering::SeqCst) {
            Err(AppError::settings("autostart disable failed"))
        } else {
            Ok(())
        }
    }
}

fn settings_input(settings: &AppSettings, autostart_enabled: bool) -> SettingsInput {
    SettingsInput {
        scan_roots: settings.scan_roots.clone(),
        hidden_project_ids: settings.hidden_project_ids.clone(),
        tray_hidden_project_ids: settings.tray_hidden_project_ids.clone(),
        autostart_enabled,
        tray_bar_max_projects: settings.tray_bar_max_projects,
        tray_bar_sort: settings.tray_bar_sort,
        global_sessions_default_range: settings.global_sessions_default_range.clone(),
    }
}

async fn app_and_state() -> (
    tauri::App<tauri::test::MockRuntime>,
    gsd_dashboard::app_state::AppState,
) {
    let temp_dir = tempfile::tempdir()
        .expect("temp dir should be created")
        .into_path();
    let mut app = tauri::test::mock_builder()
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .expect("app should build");
    #[allow(deprecated)]
    app.run_iteration(|_, _| {});

    let state = bootstrap::bootstrap_from_paths(temp_dir.join("app-data"), temp_dir.join("home"))
        .await
        .expect("bootstrap should succeed");

    (app, state)
}

#[tokio::test]
async fn save_settings_applies_autostart_before_persisting_enabled_intent() {
    let (app, state) = app_and_state().await;
    let current = settings::load_or_initialize(&state.pool, &state.home_dir)
        .await
        .expect("settings should load");
    assert!(!current.autostart_enabled);

    let backend = FakeAutostartBackend::default();
    let saved = save_settings_with_autostart_backend(
        app.handle(),
        &state,
        settings_input(&current, true),
        &backend,
    )
    .await
    .expect("settings should save");

    assert!(saved.autostart_enabled);
    assert_eq!(backend.enable_calls(), 1);
    assert_eq!(backend.disable_calls(), 0);

    let persisted = settings::load_or_initialize(&state.pool, &state.home_dir)
        .await
        .expect("settings should reload");
    assert!(persisted.autostart_enabled);
}

#[tokio::test]
async fn save_settings_applies_autostart_before_persisting_disabled_intent() {
    let (app, state) = app_and_state().await;
    let enabled = settings::save(
        &state.pool,
        &state.home_dir,
        SettingsInput {
            scan_roots: vec!["~/Documents".to_string()],
            hidden_project_ids: Vec::new(),
            tray_hidden_project_ids: Vec::new(),
            autostart_enabled: true,
            tray_bar_max_projects: 8,
            tray_bar_sort: TrayBarSort::RecentActivity,
            global_sessions_default_range: "7d".to_string(),
        },
    )
    .await
    .expect("enabled settings should save");

    let backend = FakeAutostartBackend::default();
    let saved = save_settings_with_autostart_backend(
        app.handle(),
        &state,
        settings_input(&enabled, false),
        &backend,
    )
    .await
    .expect("settings should save");

    assert!(!saved.autostart_enabled);
    assert_eq!(backend.enable_calls(), 0);
    assert_eq!(backend.disable_calls(), 1);

    let persisted = settings::load_or_initialize(&state.pool, &state.home_dir)
        .await
        .expect("settings should reload");
    assert!(!persisted.autostart_enabled);
}

#[tokio::test]
async fn save_settings_does_not_persist_or_emit_when_autostart_backend_fails() {
    let (app, state) = app_and_state().await;
    let current = settings::load_or_initialize(&state.pool, &state.home_dir)
        .await
        .expect("settings should load");
    assert!(!current.autostart_enabled);

    let event_seen = Arc::new(AtomicBool::new(false));
    let event_seen_clone = Arc::clone(&event_seen);
    app.listen("settings-changed", move |_| {
        event_seen_clone.store(true, Ordering::SeqCst);
    });

    let backend = FakeAutostartBackend::fail();
    let error = save_settings_with_autostart_backend(
        app.handle(),
        &state,
        settings_input(&current, true),
        &backend,
    )
    .await
    .expect_err("backend failure should reject settings save");

    assert!(error.to_string().contains("autostart enable failed"));
    assert_eq!(backend.enable_calls(), 1);
    assert_eq!(backend.disable_calls(), 0);
    assert!(!event_seen.load(Ordering::SeqCst));

    let persisted = settings::load_or_initialize(&state.pool, &state.home_dir)
        .await
        .expect("settings should reload");
    assert!(!persisted.autostart_enabled);
}
