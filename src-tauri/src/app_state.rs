use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use deadpool_sqlite::Pool;
use serde::Serialize;

use crate::watcher::WatcherRuntime;

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool,
    pub home_dir: PathBuf,
    pub app_data_dir: PathBuf,
    pub cache_path: PathBuf,
    pub boot_status: BootStatus,
    pub watcher_runtime: WatcherRuntime,
    tray_refresh_requests: Arc<AtomicU64>,
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
            watcher_runtime: WatcherRuntime::new(),
            tray_refresh_requests: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn request_tray_refresh(&self) -> u64 {
        self.tray_refresh_requests.fetch_add(1, Ordering::SeqCst) + 1
    }

    pub fn tray_refresh_request_count(&self) -> u64 {
        self.tray_refresh_requests.load(Ordering::SeqCst)
    }
}
