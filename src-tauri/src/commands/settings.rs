use tauri::{AppHandle, Emitter, Runtime, State};

use crate::{
    app_state::{AppState, BootStatus},
    autostart,
    error::AppError,
    events::AppEvent,
    settings::{self, AppSettings, SettingsInput},
    tray::service::request_tray_refresh,
    watcher::{self, WatcherStatus},
};

const SETTINGS_CHANGED_EVENT: &str = "settings-changed";

/// IPC command: returns the app boot status.
#[tauri::command]
pub async fn get_boot_status(state: State<'_, AppState>) -> Result<BootStatus, AppError> {
    get_boot_status_from_state(&state)
}

/// IPC command: returns current app settings.
#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, AppError> {
    get_settings_from_state(&state).await
}

/// IPC command: returns the filesystem watcher status.
#[tauri::command]
pub async fn get_watcher_status(state: State<'_, AppState>) -> Result<WatcherStatus, AppError> {
    get_watcher_status_from_state(&state).await
}

/// IPC command: validates and persists updated settings.
#[tauri::command]
pub async fn save_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    input: SettingsInput,
) -> Result<AppSettings, AppError> {
    save_settings_for_app(&app, &state, input).await
}

/// Returns the boot status from app state.
pub fn get_boot_status_from_state(state: &AppState) -> Result<BootStatus, AppError> {
    Ok(state.boot_status.clone())
}

/// Loads settings from the database via app state.
pub async fn get_settings_from_state(state: &AppState) -> Result<AppSettings, AppError> {
    settings::load_or_initialize(&state.pool, &state.home_dir).await
}

/// Returns the current watcher status from the runtime.
pub async fn get_watcher_status_from_state(state: &AppState) -> Result<WatcherStatus, AppError> {
    Ok(state.watcher_runtime.status())
}

/// Saves settings with Tauri autostart backend integration.
pub async fn save_settings_for_app<R: Runtime>(
    app: &AppHandle<R>,
    state: &AppState,
    input: SettingsInput,
) -> Result<AppSettings, AppError> {
    let backend = autostart::TauriAutostartBackend::new(app);
    save_settings_with_autostart_backend(app, state, input, backend).await
}

/// Saves settings with a pluggable autostart backend and transactional rollback.
pub async fn save_settings_with_autostart_backend<
    R: Runtime,
    B: autostart::AutostartBackend + Send + 'static,
>(
    app: &AppHandle<R>,
    state: &AppState,
    input: SettingsInput,
    backend: B,
) -> Result<AppSettings, AppError> {
    let _settings_guard = state.settings_lock.lock().await;
    let current_settings = settings::load_or_initialize(&state.pool, &state.home_dir).await?;
    let saved_settings = settings::save(&state.pool, &state.home_dir, input).await?;
    if current_settings.autostart_enabled != saved_settings.autostart_enabled {
        let autostart_enabled = saved_settings.autostart_enabled;
        let backend_result = match tokio::task::spawn_blocking(move || {
            if autostart_enabled {
                backend.enable()
            } else {
                backend.disable()
            }
        })
        .await
        {
            Ok(result) => result,
            Err(error) => {
                settings::save(&state.pool, &state.home_dir, current_settings.into()).await?;
                return Err(AppError::settings(format!("autostart task failed: {error}")));
            }
        };

        if let Err(error) = backend_result {
            settings::save(&state.pool, &state.home_dir, current_settings.into()).await?;
            return Err(error);
        }
    }

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
