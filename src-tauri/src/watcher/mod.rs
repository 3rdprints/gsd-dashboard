pub mod refresh;
pub mod roots;
pub mod service;

pub use roots::{derive_polling_scan_roots, derive_watcher_roots};
pub use service::{
    refresh_session_file_for_app, start_watcher_service, PendingSessionFile, ProjectDebouncer,
    SessionFileDebouncer, WatcherMode, WatcherReasonCategory, WatcherRootStatus, WatcherRuntime,
    WatcherStatus, POLLING_INTERVAL_SECONDS, PROJECT_DEBOUNCE_MS,
};
