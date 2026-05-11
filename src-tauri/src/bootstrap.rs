use std::path::PathBuf;

use tauri::{Manager, Runtime};

use crate::{
    app_state::{AppState, BootStatus},
    autostart::is_autostart_launch,
    error::AppError,
    settings, store,
    tray::service::{setup_tray, show_dashboard_window},
    watcher,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupVisibilityAction {
    ShowDashboard,
    KeepHidden,
}

/// Determines whether to show or hide the dashboard window on startup.
pub fn startup_visibility_action(
    is_autostart_launch: bool,
    tray_setup_succeeded: bool,
) -> StartupVisibilityAction {
    if is_autostart_launch && tray_setup_succeeded {
        StartupVisibilityAction::KeepHidden
    } else {
        StartupVisibilityAction::ShowDashboard
    }
}

/// Initializes the application state from Tauri app paths and starts the file watcher.
pub async fn bootstrap_app<R: tauri::Runtime>(app: &tauri::App<R>) -> Result<AppState, AppError> {
    let app_data_dir = app.handle().path().app_data_dir()?;
    let home_dir = app.handle().path().home_dir()?;

    let state = bootstrap_from_paths(app_data_dir, home_dir).await?;
    watcher::start_watcher_service_for_app(app.handle().clone(), &state).await?;
    Ok(state)
}

/// Registers the app state in Tauri's managed state and sets up the system tray.
pub fn manage_app_state_and_tray<R: Runtime>(
    app: &mut tauri::App<R>,
    state: AppState,
) -> Result<(), AppError> {
    app.manage(state);
    let is_autostart_launch = is_autostart_launch(std::env::args());
    let tray_setup_succeeded = match setup_tray(app.handle()) {
        Ok(()) => true,
        Err(error) => {
            eprintln!("tray setup failed: {error}");
            false
        }
    };

    match startup_visibility_action(is_autostart_launch, tray_setup_succeeded) {
        StartupVisibilityAction::ShowDashboard => show_dashboard_window(app.handle(), None)?,
        StartupVisibilityAction::KeepHidden => {}
    }

    Ok(())
}

/// Creates app state from explicit directory paths, opening the SQLite cache and running migrations.
pub async fn bootstrap_from_paths(
    app_data_dir: PathBuf,
    home_dir: PathBuf,
) -> Result<AppState, AppError> {
    let app_data_dir_for_create = app_data_dir.clone();
    tokio::task::spawn_blocking(move || std::fs::create_dir_all(&app_data_dir_for_create))
        .await
        .map_err(AppError::io)??;
    let cache_path = app_data_dir.join("cache.db");
    let pool = open_migrated_cache_pool(&cache_path).await?;
    settings::load_or_initialize(&pool, &home_dir).await?;
    let cache_path_for_ready = cache_path.clone();
    let cache_ready = tokio::task::spawn_blocking(move || cache_path_for_ready.exists())
        .await
        .map_err(AppError::io)?;

    let boot_status = BootStatus {
        app_data_dir: app_data_dir.display().to_string(),
        cache_path: cache_path.display().to_string(),
        cache_ready,
        wal_enabled: store::wal_enabled(&pool).await?,
        migrations_applied: store::migration_version(&pool).await?,
        settings_initialized: true,
    };

    let state = AppState::new(pool, home_dir, app_data_dir, cache_path, boot_status);
    watcher::start_watcher_service(&state).await?;

    Ok(state)
}

async fn open_migrated_cache_pool(cache_path: &PathBuf) -> Result<deadpool_sqlite::Pool, AppError> {
    let pool = store::open_pool(cache_path).await?;
    match store::run_migrations(&pool).await {
        Ok(()) => Ok(pool),
        Err(error) if is_newer_cache_schema_error(&error) => {
            drop(pool);
            remove_cache_files(cache_path).await?;
            let rebuilt_pool = store::open_pool(cache_path).await?;
            store::run_migrations(&rebuilt_pool).await?;
            Ok(rebuilt_pool)
        }
        Err(error) => Err(error),
    }
}

fn is_newer_cache_schema_error(error: &AppError) -> bool {
    error
        .to_string()
        .contains("migration number that is too high")
}

async fn remove_cache_files(cache_path: &PathBuf) -> Result<(), AppError> {
    let paths = [
        cache_path.clone(),
        cache_path.with_extension("db-wal"),
        cache_path.with_extension("db-shm"),
    ];

    tokio::task::spawn_blocking(move || {
        for path in paths {
            match std::fs::remove_file(&path) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error),
            }
        }
        Ok(())
    })
    .await
    .map_err(AppError::io)?
    .map_err(AppError::io)
}
