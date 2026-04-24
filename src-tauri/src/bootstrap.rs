use std::path::PathBuf;

use tauri::Manager;

use crate::{
    app_state::{AppState, BootStatus},
    error::AppError,
    settings, store,
};

pub async fn bootstrap_app<R: tauri::Runtime>(app: &tauri::App<R>) -> Result<AppState, AppError> {
    let app_data_dir = app.handle().path().app_data_dir()?;
    let home_dir = app.handle().path().home_dir()?;

    bootstrap_from_paths(app_data_dir, home_dir).await
}

pub async fn bootstrap_from_paths(
    app_data_dir: PathBuf,
    home_dir: PathBuf,
) -> Result<AppState, AppError> {
    std::fs::create_dir_all(&app_data_dir)?;
    let cache_path = app_data_dir.join("cache.db");
    let pool = store::open_pool(&cache_path).await?;
    store::run_migrations(&pool).await?;
    let settings_initialized = settings::load_or_initialize(&pool, &home_dir).await.is_ok();
    if !settings_initialized {
        settings::load_or_initialize(&pool, &home_dir).await?;
    }

    let boot_status = BootStatus {
        app_data_dir: app_data_dir.display().to_string(),
        cache_path: cache_path.display().to_string(),
        cache_ready: cache_path.exists(),
        wal_enabled: store::wal_enabled(&pool).await?,
        migrations_applied: store::migration_version(&pool).await?,
        settings_initialized,
    };

    Ok(AppState::new(
        pool,
        home_dir,
        app_data_dir,
        cache_path,
        boot_status,
    ))
}
