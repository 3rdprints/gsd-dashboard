use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};

const MIGRATION_SLICE: &[M<'_>] = &[
    M::up(
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
    ),
    M::up(
        "CREATE TABLE IF NOT EXISTS projects (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        root_path TEXT NOT NULL UNIQUE,
        planning_path TEXT NOT NULL,
        current_milestone_name TEXT,
        current_milestone_index INTEGER,
        current_phase_number TEXT,
        current_phase_name TEXT,
        milestone_progress_pct REAL NOT NULL DEFAULT 0,
        next_command TEXT NOT NULL DEFAULT '/gsd-next',
        parsed_blob TEXT NOT NULL,
        parse_error TEXT,
        last_activity_at INTEGER,
        last_scanned_at INTEGER NOT NULL,
        created_at INTEGER NOT NULL,
        updated_at INTEGER NOT NULL
    );

    CREATE TABLE IF NOT EXISTS phase_plans (
        project_id TEXT NOT NULL,
        phase_number TEXT NOT NULL,
        phase_name TEXT,
        plan_number TEXT,
        plan_path TEXT NOT NULL,
        checklist_json TEXT NOT NULL,
        updated_at INTEGER NOT NULL,
        FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
        UNIQUE (project_id, plan_path)
    );

    CREATE INDEX IF NOT EXISTS idx_phase_plans_project_id
        ON phase_plans(project_id);

    CREATE TABLE IF NOT EXISTS scan_log (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        project_id TEXT,
        root_path TEXT,
        planning_path TEXT,
        file_path TEXT,
        status TEXT NOT NULL,
        message TEXT,
        errors_json TEXT NOT NULL DEFAULT '[]',
        created_at INTEGER NOT NULL,
        FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE SET NULL
    );

    CREATE INDEX IF NOT EXISTS idx_scan_log_project_id_created_at
        ON scan_log(project_id, created_at);

    CREATE INDEX IF NOT EXISTS idx_scan_log_root_path_created_at
        ON scan_log(root_path, created_at);",
    ),
];
const MIGRATIONS: Migrations<'_> = Migrations::from_slice(MIGRATION_SLICE);

pub fn run(connection: &mut Connection) -> Result<(), rusqlite_migration::Error> {
    MIGRATIONS.to_latest(connection)
}
