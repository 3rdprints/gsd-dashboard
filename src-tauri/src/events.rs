use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "event", content = "data", rename_all = "camelCase")]
pub enum AppEvent {
    BootReady { cache_path: String },
    SettingsChanged,
}
