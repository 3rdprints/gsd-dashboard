use gsd_dashboard::{
    autostart::{is_autostart_launch, AutostartBackend},
    bootstrap,
    commands::settings::save_settings_with_autostart_backend,
    error::AppError,
    settings::{self, AppSettings, SettingsInput, TrayBarSort},
};
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    mpsc,
    Arc,
    Condvar,
    Mutex,
};
use std::time::Duration;
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
    panic: AtomicBool,
}

impl FakeAutostartBackend {
    fn fail() -> Self {
        Self {
            fail: AtomicBool::new(true),
            ..Self::default()
        }
    }

    fn panic() -> Self {
        Self {
            panic: AtomicBool::new(true),
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
        assert!(
            !self.panic.load(Ordering::SeqCst),
            "autostart enable panicked"
        );
        if self.fail.load(Ordering::SeqCst) {
            Err(AppError::settings("autostart enable failed"))
        } else {
            Ok(())
        }
    }

    fn disable(&self) -> Result<(), AppError> {
        self.disable_calls.fetch_add(1, Ordering::SeqCst);
        assert!(
            !self.panic.load(Ordering::SeqCst),
            "autostart disable panicked"
        );
        if self.fail.load(Ordering::SeqCst) {
            Err(AppError::settings("autostart disable failed"))
        } else {
            Ok(())
        }
    }
}

struct BlockingFailAutostartBackend {
    entered: Mutex<Option<mpsc::Sender<()>>>,
    released: Arc<(Mutex<bool>, Condvar)>,
}

impl BlockingFailAutostartBackend {
    fn new(entered: mpsc::Sender<()>) -> Self {
        Self {
            entered: Mutex::new(Some(entered)),
            released: Arc::new((Mutex::new(false), Condvar::new())),
        }
    }

    fn release(&self) {
        let (lock, cvar) = &*self.released;
        let mut released = lock.lock().expect("release lock should not be poisoned");
        *released = true;
        cvar.notify_all();
    }

    fn wait_until_released(&self) {
        let (lock, cvar) = &*self.released;
        let mut released = lock.lock().expect("release lock should not be poisoned");
        while !*released {
            released = cvar
                .wait(released)
                .expect("release lock should not be poisoned");
        }
    }
}

impl AutostartBackend for BlockingFailAutostartBackend {
    fn enable(&self) -> Result<(), AppError> {
        if let Some(entered) = self
            .entered
            .lock()
            .expect("entered lock should not be poisoned")
            .take()
        {
            let _ = entered.send(());
        }
        self.wait_until_released();
        Err(AppError::settings("autostart enable failed"))
    }

    fn disable(&self) -> Result<(), AppError> {
        Ok(())
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
    tempfile::TempDir,
    tauri::App<tauri::test::MockRuntime>,
    gsd_dashboard::app_state::AppState,
) {
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

    (temp_dir, app, state)
}

#[tokio::test]
async fn autostart_settings_save_settings_applies_autostart_after_persisting_enabled_intent() {
    let (_temp_dir, app, state) = app_and_state().await;
    let current = settings::load_or_initialize(&state.pool, &state.home_dir)
        .await
        .expect("settings should load");
    assert!(!current.autostart_enabled);

    let backend = Arc::new(FakeAutostartBackend::default());
    let saved = save_settings_with_autostart_backend(
        app.handle(),
        &state,
        settings_input(&current, true),
        backend.clone(),
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
async fn autostart_settings_save_settings_applies_autostart_after_persisting_disabled_intent() {
    let (_temp_dir, app, state) = app_and_state().await;
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

    let backend = Arc::new(FakeAutostartBackend::default());
    let saved = save_settings_with_autostart_backend(
        app.handle(),
        &state,
        settings_input(&enabled, false),
        backend.clone(),
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
async fn autostart_settings_save_does_not_mutate_backend_when_validation_fails() {
    let (_temp_dir, app, state) = app_and_state().await;
    let current = settings::load_or_initialize(&state.pool, &state.home_dir)
        .await
        .expect("settings should load");
    assert!(!current.autostart_enabled);

    let backend = Arc::new(FakeAutostartBackend::default());
    let mut input = settings_input(&current, true);
    input.scan_roots = vec!["/".to_string()];

    let error = save_settings_with_autostart_backend(app.handle(), &state, input, backend.clone())
        .await
        .expect_err("invalid settings should reject before autostart mutation");

    assert!(matches!(error, AppError::InvalidScanRoot { .. }));
    assert_eq!(backend.enable_calls(), 0);
    assert_eq!(backend.disable_calls(), 0);

    let persisted = settings::load_or_initialize(&state.pool, &state.home_dir)
        .await
        .expect("settings should reload");
    assert!(!persisted.autostart_enabled);
}

#[tokio::test]
async fn autostart_settings_save_does_not_persist_or_emit_when_backend_fails() {
    let (_temp_dir, app, state) = app_and_state().await;
    let current = settings::load_or_initialize(&state.pool, &state.home_dir)
        .await
        .expect("settings should load");
    assert!(!current.autostart_enabled);

    let event_seen = Arc::new(AtomicBool::new(false));
    let event_seen_clone = Arc::clone(&event_seen);
    app.listen("settings-changed", move |_| {
        event_seen_clone.store(true, Ordering::SeqCst);
    });

    let backend = Arc::new(FakeAutostartBackend::fail());
    let error = save_settings_with_autostart_backend(
        app.handle(),
        &state,
        settings_input(&current, true),
        backend.clone(),
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

#[tokio::test]
async fn autostart_settings_save_rolls_back_when_blocking_backend_task_panics() {
    let (_temp_dir, app, state) = app_and_state().await;
    let current = settings::load_or_initialize(&state.pool, &state.home_dir)
        .await
        .expect("settings should load");
    assert!(!current.autostart_enabled);

    let event_seen = Arc::new(AtomicBool::new(false));
    let event_seen_clone = Arc::clone(&event_seen);
    app.listen("settings-changed", move |_| {
        event_seen_clone.store(true, Ordering::SeqCst);
    });

    let backend = Arc::new(FakeAutostartBackend::panic());
    let error = save_settings_with_autostart_backend(
        app.handle(),
        &state,
        settings_input(&current, true),
        backend.clone(),
    )
    .await
    .expect_err("backend task panic should reject settings save");

    assert!(error.to_string().contains("autostart task failed"));
    assert_eq!(backend.enable_calls(), 1);
    assert_eq!(backend.disable_calls(), 0);
    assert!(!event_seen.load(Ordering::SeqCst));

    let persisted = settings::load_or_initialize(&state.pool, &state.home_dir)
        .await
        .expect("settings should reload");
    assert!(!persisted.autostart_enabled);
}

#[tokio::test]
async fn autostart_settings_save_serializes_rollback_against_newer_save() {
    let (_temp_dir, app, state) = app_and_state().await;
    let current = settings::load_or_initialize(&state.pool, &state.home_dir)
        .await
        .expect("settings should load");
    assert!(!current.autostart_enabled);

    let (entered_tx, entered_rx) = mpsc::channel();
    let blocking_backend = Arc::new(BlockingFailAutostartBackend::new(entered_tx));
    let first_handle = app.handle().clone();
    let first_state = state.clone();
    let first_input = settings_input(&current, true);
    let first_backend = blocking_backend.clone();
    let first_save = tokio::spawn(async move {
        save_settings_with_autostart_backend(
            &first_handle,
            &first_state,
            first_input,
            first_backend,
        )
        .await
    });

    tokio::task::spawn_blocking(move || entered_rx.recv())
        .await
        .expect("entered wait should not panic")
        .expect("first backend should be entered");

    let second_handle = app.handle().clone();
    let second_state = state.clone();
    let second_input = settings_input(&current, true);
    let second_backend = Arc::new(FakeAutostartBackend::default());
    let second_save = tokio::spawn(async move {
        save_settings_with_autostart_backend(
            &second_handle,
            &second_state,
            second_input,
            second_backend,
        )
        .await
    });

    tokio::time::sleep(Duration::from_millis(50)).await;
    blocking_backend.release();

    let first_error = first_save
        .await
        .expect("first save task should not panic")
        .expect_err("first save should fail");
    assert!(first_error.to_string().contains("autostart enable failed"));

    let second_saved = second_save
        .await
        .expect("second save task should not panic")
        .expect("second save should succeed");
    assert!(second_saved.autostart_enabled);

    let persisted = settings::load_or_initialize(&state.pool, &state.home_dir)
        .await
        .expect("settings should reload");
    assert!(persisted.autostart_enabled);
}
