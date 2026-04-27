#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let handle = tauri::async_runtime::handle();
            let state = handle.block_on(gsd_dashboard::bootstrap::bootstrap_app(app))?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            gsd_dashboard::commands::projects::get_portfolio,
            gsd_dashboard::commands::projects::get_project,
            gsd_dashboard::commands::projects::get_project_chart_data,
            gsd_dashboard::commands::projects::get_project_milestones,
            gsd_dashboard::commands::projects::get_project_phase_panel,
            gsd_dashboard::commands::projects::list_project_sessions,
            gsd_dashboard::commands::scan::rebuild_cache,
            gsd_dashboard::commands::scan::scan_projects,
            gsd_dashboard::commands::sessions::index_sessions,
            gsd_dashboard::commands::settings::get_boot_status,
            gsd_dashboard::commands::settings::get_settings,
            gsd_dashboard::commands::settings::save_settings
        ])
        .run(tauri::generate_context!())
        .expect("failed to run GSD Dashboard");
}
