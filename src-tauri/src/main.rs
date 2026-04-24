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
        .run(tauri::generate_context!())
        .expect("failed to run GSD Dashboard");
}
