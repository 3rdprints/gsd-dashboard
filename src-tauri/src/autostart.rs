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
