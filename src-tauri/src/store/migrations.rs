use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};

const MIGRATION_SLICE: &[M<'_>] = &[M::up(
    "CREATE TABLE IF NOT EXISTS settings (
        id INTEGER PRIMARY KEY CHECK (id = 1),
        scan_roots_json TEXT NOT NULL,
        hidden_project_ids_json TEXT NOT NULL,
        autostart_enabled INTEGER NOT NULL DEFAULT 0,
        tray_bar_max_projects INTEGER NOT NULL DEFAULT 8,
        tray_bar_sort TEXT NOT NULL DEFAULT 'recent_activity',
        created_at INTEGER NOT NULL,
        updated_at INTEGER NOT NULL
    );",
)];
const MIGRATIONS: Migrations<'_> = Migrations::from_slice(MIGRATION_SLICE);

pub fn run(connection: &mut Connection) -> Result<(), rusqlite_migration::Error> {
    MIGRATIONS.to_latest(connection)
}
