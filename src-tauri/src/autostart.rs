use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

use crate::error::AppError;

pub const AUTOSTART_ARG: &str = "--autostart";

/// Returns true if the CLI arguments contain the autostart flag.
pub fn is_autostart_launch<I, S>(args: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    args.into_iter()
        .any(|argument| argument.as_ref() == AUTOSTART_ARG)
}

/// Registers the Tauri autostart plugin configured with macOS LaunchAgent.
pub fn register_autostart_plugin<R: tauri::Runtime>(
    builder: tauri::Builder<R>,
) -> tauri::Builder<R> {
    builder.plugin(tauri_plugin_autostart::init(
        tauri_plugin_autostart::MacosLauncher::LaunchAgent,
        Some(vec![AUTOSTART_ARG]),
    ))
}

pub trait AutostartBackend {
    fn enable(&self) -> Result<(), AppError>;

    fn disable(&self) -> Result<(), AppError>;
}

impl<T: AutostartBackend + ?Sized> AutostartBackend for std::sync::Arc<T> {
    fn enable(&self) -> Result<(), AppError> {
        (**self).enable()
    }

    fn disable(&self) -> Result<(), AppError> {
        (**self).disable()
    }
}

pub struct TauriAutostartBackend<R: tauri::Runtime> {
    app: AppHandle<R>,
}

impl<R: tauri::Runtime> TauriAutostartBackend<R> {
    /// Creates a new autostart backend wrapping the given Tauri app handle.
    pub fn new(app: &AppHandle<R>) -> Self {
        Self { app: app.clone() }
    }
}

impl<R: tauri::Runtime> AutostartBackend for TauriAutostartBackend<R> {
    fn enable(&self) -> Result<(), AppError> {
        self.app.autolaunch().enable().map_err(AppError::settings)
    }

    fn disable(&self) -> Result<(), AppError> {
        self.app.autolaunch().disable().map_err(AppError::settings)
    }
}
