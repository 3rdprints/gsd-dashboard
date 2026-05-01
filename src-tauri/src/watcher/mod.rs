pub mod roots;
pub mod service;

pub use roots::{derive_polling_scan_roots, derive_watcher_roots};
pub use service::{
    start_watcher_service, WatcherMode, WatcherReasonCategory, WatcherRootStatus, WatcherRuntime,
    WatcherStatus, POLLING_INTERVAL_SECONDS, PROJECT_DEBOUNCE_MS, SESSION_INDEX_WORKER_LIMIT,
};
