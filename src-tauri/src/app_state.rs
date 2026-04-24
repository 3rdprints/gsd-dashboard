use std::path::PathBuf;

use deadpool_sqlite::Pool;
use serde::Serialize;

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool,
    pub home_dir: PathBuf,
    pub app_data_dir: PathBuf,
    pub cache_path: PathBuf,
    pub boot_status: BootStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BootStatus {
    pub app_data_dir: String,
    pub cache_path: String,
    pub cache_ready: bool,
    pub wal_enabled: bool,
    pub migrations_applied: u32,
    pub settings_initialized: bool,
}

impl AppState {
    pub fn new(
        pool: Pool,
        home_dir: PathBuf,
        app_data_dir: PathBuf,
        cache_path: PathBuf,
        boot_status: BootStatus,
    ) -> Self {
        Self {
            pool,
            home_dir,
            app_data_dir,
            cache_path,
            boot_status,
        }
    }
}
