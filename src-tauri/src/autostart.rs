use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

use crate::error::AppError;

pub const AUTOSTART_ARG: &str = "--autostart";

pub fn is_autostart_launch<I, S>(args: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    args.into_iter()
        .any(|argument| argument.as_ref() == AUTOSTART_ARG)
}

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

pub struct TauriAutostartBackend<'a, R: tauri::Runtime> {
    app: &'a AppHandle<R>,
}

impl<'a, R: tauri::Runtime> TauriAutostartBackend<'a, R> {
    pub fn new(app: &'a AppHandle<R>) -> Self {
        Self { app }
    }
}

impl<R: tauri::Runtime> AutostartBackend for TauriAutostartBackend<'_, R> {
    fn enable(&self) -> Result<(), AppError> {
        self.app.autolaunch().enable().map_err(AppError::settings)
    }

    fn disable(&self) -> Result<(), AppError> {
        self.app.autolaunch().disable().map_err(AppError::settings)
    }
}
