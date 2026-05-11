use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use deadpool_sqlite::Pool;
use serde::{Deserialize, Serialize};

use crate::{
    error::AppError,
    scan_roots::validate_scan_root,
    store::settings_repo::{self, StoredSettings},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub scan_roots: Vec<String>,
    pub hidden_project_ids: Vec<String>,
    pub tray_hidden_project_ids: Vec<String>,
    pub autostart_enabled: bool,
    pub tray_bar_max_projects: u8,
    pub tray_bar_sort: TrayBarSort,
    pub global_sessions_default_range: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsInput {
    pub scan_roots: Vec<String>,
    pub hidden_project_ids: Vec<String>,
    pub tray_hidden_project_ids: Vec<String>,
    pub autostart_enabled: bool,
    pub tray_bar_max_projects: u8,
    pub tray_bar_sort: TrayBarSort,
    pub global_sessions_default_range: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TrayBarSort {
    Name,
    Progress,
    #[serde(rename = "recent_activity", alias = "recentActivity")]
    RecentActivity,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            scan_roots: vec!["~/Documents".to_string()],
            hidden_project_ids: Vec::new(),
            tray_hidden_project_ids: Vec::new(),
            autostart_enabled: false,
            tray_bar_max_projects: 8,
            tray_bar_sort: TrayBarSort::RecentActivity,
            global_sessions_default_range: default_global_sessions_range().to_string(),
        }
    }
}

impl From<AppSettings> for SettingsInput {
    fn from(settings: AppSettings) -> Self {
        Self {
            scan_roots: settings.scan_roots,
            hidden_project_ids: settings.hidden_project_ids,
            tray_hidden_project_ids: settings.tray_hidden_project_ids,
            autostart_enabled: settings.autostart_enabled,
            tray_bar_max_projects: settings.tray_bar_max_projects,
            tray_bar_sort: settings.tray_bar_sort,
            global_sessions_default_range: settings.global_sessions_default_range,
        }
    }
}

/// Loads settings from the database or initializes with defaults if none exist.
pub async fn load_or_initialize(pool: &Pool, home_dir: &Path) -> Result<AppSettings, AppError> {
    let stored = {
        let connection = pool.get().await.map_err(AppError::store)?;
        connection
            .interact(settings_repo::load_settings)
            .await
            .map_err(AppError::store)??
    };

    match stored {
        Some(stored_settings) => {
            let settings = AppSettings::try_from(stored_settings)?;
            validate_settings(&settings.scan_roots, home_dir)?;
            Ok(settings)
        }
        None => {
            let defaults = AppSettings::default();
            save(pool, home_dir, defaults.clone().into()).await
        }
    }
}

/// Validates and persists updated settings to the database.
pub async fn save(
    pool: &Pool,
    home_dir: &Path,
    input: SettingsInput,
) -> Result<AppSettings, AppError> {
    validate_settings(&input.scan_roots, home_dir)?;
    let settings = AppSettings::from(input);
    let stored = StoredSettings::try_from(settings.clone())?;
    let now = unix_timestamp();
    let connection = pool.get().await.map_err(AppError::store)?;
    connection
        .interact(move |connection| settings_repo::save_settings(connection, stored, now))
        .await
        .map_err(AppError::store)??;

    Ok(settings)
}

fn validate_settings(scan_roots: &[String], home_dir: &Path) -> Result<(), AppError> {
    for scan_root in normalize_scan_roots(scan_roots.to_vec()) {
        validate_scan_root(Path::new(&scan_root), home_dir)?;
    }

    Ok(())
}

fn normalize_scan_roots(scan_roots: Vec<String>) -> Vec<String> {
    let roots = scan_roots
        .into_iter()
        .map(|root| root.trim().to_string())
        .filter(|root| !root.is_empty())
        .collect::<Vec<_>>();

    if roots.is_empty() {
        AppSettings::default().scan_roots
    } else {
        roots
    }
}

impl From<SettingsInput> for AppSettings {
    fn from(input: SettingsInput) -> Self {
        Self {
            scan_roots: normalize_scan_roots(input.scan_roots),
            hidden_project_ids: input.hidden_project_ids,
            tray_hidden_project_ids: input.tray_hidden_project_ids,
            autostart_enabled: input.autostart_enabled,
            tray_bar_max_projects: input.tray_bar_max_projects,
            tray_bar_sort: input.tray_bar_sort,
            global_sessions_default_range: coerce_global_sessions_range(
                &input.global_sessions_default_range,
            )
            .to_string(),
        }
    }
}

impl TryFrom<StoredSettings> for AppSettings {
    type Error = AppError;

    fn try_from(stored: StoredSettings) -> Result<Self, Self::Error> {
        Ok(Self {
            scan_roots: normalize_scan_roots(serde_json::from_str(&stored.scan_roots_json)?),
            hidden_project_ids: serde_json::from_str(&stored.hidden_project_ids_json)?,
            tray_hidden_project_ids: serde_json::from_str(&stored.tray_hidden_project_ids_json)?,
            autostart_enabled: stored.autostart_enabled,
            tray_bar_max_projects: stored.tray_bar_max_projects,
            tray_bar_sort: TrayBarSort::from_db_value(&stored.tray_bar_sort)?,
            global_sessions_default_range: coerce_global_sessions_range(
                &stored.global_sessions_default_range,
            )
            .to_string(),
        })
    }
}

impl TryFrom<AppSettings> for StoredSettings {
    type Error = AppError;

    fn try_from(settings: AppSettings) -> Result<Self, Self::Error> {
        Ok(Self {
            scan_roots_json: serde_json::to_string(&settings.scan_roots)?,
            hidden_project_ids_json: serde_json::to_string(&settings.hidden_project_ids)?,
            tray_hidden_project_ids_json: serde_json::to_string(&settings.tray_hidden_project_ids)?,
            autostart_enabled: settings.autostart_enabled,
            tray_bar_max_projects: settings.tray_bar_max_projects,
            tray_bar_sort: settings.tray_bar_sort.as_db_value().to_string(),
            global_sessions_default_range: coerce_global_sessions_range(
                &settings.global_sessions_default_range,
            )
            .to_string(),
        })
    }
}

impl TrayBarSort {
    fn as_db_value(self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::Progress => "progress",
            Self::RecentActivity => "recent_activity",
        }
    }

    fn from_db_value(value: &str) -> Result<Self, AppError> {
        match value {
            "name" => Ok(Self::Name),
            "progress" => Ok(Self::Progress),
            "recent_activity" => Ok(Self::RecentActivity),
            other => Err(AppError::store(format!("unknown tray sort value: {other}"))),
        }
    }
}

fn unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or_default()
}

fn default_global_sessions_range() -> &'static str {
    "7d"
}

fn coerce_global_sessions_range(value: &str) -> &str {
    match value {
        "7d" | "30d" | "90d" | "all" => value,
        _ => default_global_sessions_range(),
    }
}
