use std::sync::{Arc, RwLock};

use serde::Serialize;

pub const PROJECT_DEBOUNCE_MS: u64 = 500;
pub const POLLING_INTERVAL_SECONDS: u64 = 60;
pub const SESSION_INDEX_WORKER_LIMIT: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WatcherStatus {
    pub roots: Vec<WatcherRootStatus>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WatcherRootStatus {
    pub root: String,
    pub mode: WatcherMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason_category: Option<WatcherReasonCategory>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix_hint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub polling_interval_seconds: Option<u64>,
    pub retry_enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WatcherMode {
    Native,
    Polling,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WatcherReasonCategory {
    Permission,
    WatchLimit,
    Filesystem,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct WatcherRuntime {
    status: Arc<RwLock<WatcherStatus>>,
}

impl Default for WatcherRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl WatcherRuntime {
    pub fn new() -> Self {
        Self {
            status: Arc::new(RwLock::new(WatcherStatus { roots: Vec::new() })),
        }
    }

    pub fn status(&self) -> WatcherStatus {
        self.status
            .read()
            .expect("watcher status lock should not be poisoned")
            .clone()
    }

    pub fn set_roots(&self, roots: Vec<WatcherRootStatus>) -> bool {
        let mut status = self
            .status
            .write()
            .expect("watcher status lock should not be poisoned");
        if status.roots == roots {
            return false;
        }

        status.roots = roots;
        true
    }

    pub fn set_root_status(&self, root_status: WatcherRootStatus) -> bool {
        let mut next_roots = self.status().roots;
        if let Some(existing) = next_roots
            .iter_mut()
            .find(|existing| existing.root == root_status.root)
        {
            if *existing == root_status {
                return false;
            }
            *existing = root_status;
        } else {
            next_roots.push(root_status);
        }

        self.set_roots(next_roots)
    }
}

impl WatcherRootStatus {
    pub fn native(root: String) -> Self {
        Self {
            root,
            mode: WatcherMode::Native,
            reason_category: None,
            reason: None,
            fix_hint: None,
            polling_interval_seconds: None,
            retry_enabled: false,
        }
    }

    pub fn polling(root: String, reason_category: WatcherReasonCategory) -> Self {
        Self {
            root,
            mode: WatcherMode::Polling,
            reason_category: Some(reason_category),
            reason: Some(reason_category.reason().to_string()),
            fix_hint: Some(reason_category.fix_hint().to_string()),
            polling_interval_seconds: Some(POLLING_INTERVAL_SECONDS),
            retry_enabled: true,
        }
    }
}

impl WatcherReasonCategory {
    pub fn from_error_message(message: &str) -> Self {
        let normalized = message.to_ascii_lowercase();
        if normalized.contains("permission") || normalized.contains("denied") {
            Self::Permission
        } else if normalized.contains("inotify")
            || normalized.contains("watch limit")
            || normalized.contains("too many open files")
        {
            Self::WatchLimit
        } else if normalized.contains("filesystem") || normalized.contains("not supported") {
            Self::Filesystem
        } else {
            Self::Unknown
        }
    }

    pub fn reason(self) -> &'static str {
        match self {
            Self::Permission => "Permission denied",
            Self::WatchLimit => "System watch limit reached",
            Self::Filesystem => "Filesystem does not support native watching",
            Self::Unknown => "Native watcher unavailable",
        }
    }

    pub fn fix_hint(self) -> &'static str {
        match self {
            Self::Permission => "Check folder permissions, then leave Settings open for the next retry.",
            Self::WatchLimit => "Increase inotify watch limits, then wait for automatic retry.",
            Self::Filesystem => "Move the project to a local folder or keep polling enabled.",
            Self::Unknown => "No action needed unless updates feel stale.",
        }
    }
}
