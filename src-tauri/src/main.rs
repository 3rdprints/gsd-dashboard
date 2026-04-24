#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use gsd_dashboard::store;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let db_path = data_dir.join("cache.db");
            let handle = tauri::async_runtime::handle();
            let pool = handle.block_on(async {
                let pool = store::open_pool(&db_path).await?;
                store::run_migrations(&pool).await?;
                Ok::<_, gsd_dashboard::error::AppError>(pool)
            })?;
            app.manage(gsd_dashboard::app_state::AppState::new(pool));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to run GSD Dashboard");
}
