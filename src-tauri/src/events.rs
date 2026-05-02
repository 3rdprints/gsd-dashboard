use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "event", content = "data", rename_all = "camelCase")]
pub enum AppEvent {
    BootReady {
        cache_path: String,
    },
    SettingsChanged,
    TrayNavigate {
        route: String,
    },
    #[serde(rename = "daily_activity_updated")]
    DailyActivityUpdated,
    #[serde(rename = "project:updated")]
    ProjectUpdated {
        id: String,
    },
    #[serde(rename = "session:new")]
    SessionNew {
        id: String,
        project_id: Option<String>,
    },
    #[serde(rename = "watcher:status-changed")]
    WatcherStatusChanged,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "event", content = "data", rename_all = "camelCase")]
pub enum ScanEvent {
    Started {
        root_count: usize,
    },
    RootStarted {
        root_path: String,
    },
    ProjectFound {
        project_id: String,
        project_name: String,
        root_path: String,
    },
    ProjectParsed {
        project_id: String,
        project_name: String,
    },
    ProjectParseError {
        project_id: String,
        project_name: String,
        file_path: String,
        message: String,
    },
    Finished {
        discovered_count: usize,
        parsed_count: usize,
        error_count: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "event", content = "data", rename_all = "camelCase")]
pub enum SessionIndexEvent {
    App(AppEvent),
    Started {
        root_count: usize,
    },
    SourceStarted {
        source: String,
        root_path: String,
    },
    FileIndexed {
        source: String,
        source_path: String,
        sessions_persisted: usize,
        live_partial: bool,
    },
    FileIndexError {
        source: String,
        source_path: String,
        message: String,
    },
    Finished {
        files_processed: usize,
        sessions_persisted: usize,
        unmatched_count: usize,
        error_count: usize,
    },
}
