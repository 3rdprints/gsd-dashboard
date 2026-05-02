use tauri::{AppHandle, Emitter, Runtime, State};

use crate::{
    app_state::{AppState, BootStatus},
    error::AppError,
    events::AppEvent,
    settings::{self, AppSettings, SettingsInput},
    tray::service::request_tray_refresh,
    watcher::{self, WatcherStatus},
};

const SETTINGS_CHANGED_EVENT: &str = "settings-changed";

#[tauri::command]
pub async fn get_boot_status(state: State<'_, AppState>) -> Result<BootStatus, AppError> {
    get_boot_status_from_state(&state)
}

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, AppError> {
    get_settings_from_state(&state).await
}

#[tauri::command]
pub async fn get_watcher_status(state: State<'_, AppState>) -> Result<WatcherStatus, AppError> {
    get_watcher_status_from_state(&state).await
}

#[tauri::command]
pub async fn save_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    input: SettingsInput,
) -> Result<AppSettings, AppError> {
    save_settings_for_app(&app, &state, input).await
}

pub fn get_boot_status_from_state(state: &AppState) -> Result<BootStatus, AppError> {
    Ok(state.boot_status.clone())
}

pub async fn get_settings_from_state(state: &AppState) -> Result<AppSettings, AppError> {
    settings::load_or_initialize(&state.pool, &state.home_dir).await
}

pub async fn get_watcher_status_from_state(state: &AppState) -> Result<WatcherStatus, AppError> {
    Ok(state.watcher_runtime.status())
}

pub async fn save_settings_for_app<R: Runtime>(
    app: &AppHandle<R>,
    state: &AppState,
    input: SettingsInput,
) -> Result<AppSettings, AppError> {
    let saved_settings = settings::save(&state.pool, &state.home_dir, input).await?;
    let watcher_changed = match watcher::start_watcher_service_for_app(app.clone(), state).await {
        Ok(changed) => changed,
        Err(error) => {
            eprintln!("watcher restart failed after settings save: {error}");
            false
        }
    };
    app.emit(SETTINGS_CHANGED_EVENT, AppEvent::SettingsChanged)?;
    if watcher_changed {
        app.emit("watcher:status-changed", AppEvent::WatcherStatusChanged)?;
    }
    request_tray_refresh(app);
    Ok(saved_settings)
}
