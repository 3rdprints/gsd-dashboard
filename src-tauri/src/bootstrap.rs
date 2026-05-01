use std::path::PathBuf;

use tauri::Manager;

use crate::{
    app_state::{AppState, BootStatus},
    error::AppError,
    settings, store, watcher,
};

pub async fn bootstrap_app<R: tauri::Runtime>(app: &tauri::App<R>) -> Result<AppState, AppError> {
    let app_data_dir = app.handle().path().app_data_dir()?;
    let home_dir = app.handle().path().home_dir()?;

    let state = bootstrap_from_paths(app_data_dir, home_dir).await?;
    watcher::start_watcher_service_for_app(app.handle().clone(), &state).await?;
    Ok(state)
}

pub async fn bootstrap_from_paths(
    app_data_dir: PathBuf,
    home_dir: PathBuf,
) -> Result<AppState, AppError> {
    let app_data_dir_for_create = app_data_dir.clone();
    tokio::task::spawn_blocking(move || std::fs::create_dir_all(&app_data_dir_for_create))
        .await
        .map_err(AppError::io)??;
    let cache_path = app_data_dir.join("cache.db");
    let pool = store::open_pool(&cache_path).await?;
    store::run_migrations(&pool).await?;
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
