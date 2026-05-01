use tauri::{AppHandle, Emitter, Runtime, State};

use crate::{
    app_state::{AppState, BootStatus},
    error::AppError,
    events::AppEvent,
    settings::{self, AppSettings, SettingsInput},
    watcher::WatcherStatus,
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
    app.emit(SETTINGS_CHANGED_EVENT, AppEvent::SettingsChanged)?;
    Ok(saved_settings)
}
