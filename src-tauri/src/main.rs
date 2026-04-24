#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let handle = tauri::async_runtime::handle();
            let state = handle.block_on(gsd_dashboard::bootstrap::bootstrap_app(app))?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            gsd_dashboard::commands::settings::get_boot_status,
            gsd_dashboard::commands::settings::get_settings,
            gsd_dashboard::commands::settings::save_settings
        ])
        .run(tauri::generate_context!())
        .expect("failed to run GSD Dashboard");
}
