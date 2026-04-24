use rusqlite::{params, OptionalExtension};

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct StoredSettings {
    pub scan_roots_json: String,
    pub hidden_project_ids_json: String,
    pub autostart_enabled: bool,
    pub tray_bar_max_projects: u8,
    pub tray_bar_sort: String,
}

pub fn load_settings(
    connection: &mut rusqlite::Connection,
) -> Result<Option<StoredSettings>, AppError> {
    connection
        .query_row(
            "SELECT scan_roots_json,
                    hidden_project_ids_json,
                    autostart_enabled,
                    tray_bar_max_projects,
                    tray_bar_sort
             FROM settings
             WHERE id = 1",
            [],
            |row| {
                let autostart_enabled: i64 = row.get(2)?;
                Ok(StoredSettings {
                    scan_roots_json: row.get(0)?,
                    hidden_project_ids_json: row.get(1)?,
                    autostart_enabled: autostart_enabled != 0,
                    tray_bar_max_projects: row.get::<_, u8>(3)?,
                    tray_bar_sort: row.get(4)?,
                })
            },
        )
        .optional()
        .map_err(AppError::from)
}

pub fn save_settings(
    connection: &mut rusqlite::Connection,
    settings: StoredSettings,
    now: i64,
) -> Result<(), AppError> {
    connection
        .execute(
            "INSERT INTO settings (
                id,
                scan_roots_json,
                hidden_project_ids_json,
                autostart_enabled,
                tray_bar_max_projects,
                tray_bar_sort,
                created_at,
                updated_at
            )
            VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?6)
            ON CONFLICT(id) DO UPDATE SET
                scan_roots_json = excluded.scan_roots_json,
                hidden_project_ids_json = excluded.hidden_project_ids_json,
                autostart_enabled = excluded.autostart_enabled,
                tray_bar_max_projects = excluded.tray_bar_max_projects,
                tray_bar_sort = excluded.tray_bar_sort,
                updated_at = excluded.updated_at",
            params![
                settings.scan_roots_json,
                settings.hidden_project_ids_json,
                i64::from(settings.autostart_enabled),
                i64::from(settings.tray_bar_max_projects),
                settings.tray_bar_sort,
                now,
            ],
        )
        .map(|_| ())
        .map_err(AppError::from)
}
