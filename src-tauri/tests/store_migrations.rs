use gsd_dashboard::store;
use rusqlite::OptionalExtension;

const EXPECTED_MIGRATION_COUNT: u32 = gsd_dashboard::store::migrations::MIGRATION_COUNT;

async fn migrated_pool(db_path: &std::path::Path) -> deadpool_sqlite::Pool {
    let pool = store::open_pool(db_path).await.expect("pool should open");
    store::run_migrations(&pool)
        .await
        .expect("migrations should run");
    pool
}

#[tokio::test]
async fn opening_pool_runs_migrations_and_creates_settings_table() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let db_path = temp_dir.path().join("cache.db");

    let pool = migrated_pool(&db_path).await;
    let conn = pool.get().await.expect("connection should be available");
    let table_name = conn
        .interact(|conn| {
            conn.query_row(
                "SELECT name FROM sqlite_master WHERE type = 'table' AND name = 'settings'",
                [],
                |row| row.get::<_, String>(0),
            )
            .optional()
        })
        .await
        .expect("interaction should complete")
        .expect("query should run");

    assert_eq!(table_name.as_deref(), Some("settings"));
}

#[tokio::test]
async fn pool_initialization_enables_wal_mode() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let db_path = temp_dir.path().join("cache.db");

    let pool = migrated_pool(&db_path).await;
    let conn = pool.get().await.expect("connection should be available");
    let journal_mode = conn
        .interact(|conn| {
            conn.pragma_query_value(None, "journal_mode", |row| row.get::<_, String>(0))
        })
        .await
        .expect("interaction should complete")
        .expect("journal mode should be readable");

    assert_eq!(journal_mode.to_lowercase(), "wal");
}

#[tokio::test]
async fn migrated_schema_survives_pool_reopen() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let db_path = temp_dir.path().join("cache.db");

    drop(migrated_pool(&db_path).await);

    let pool = migrated_pool(&db_path).await;
    let conn = pool.get().await.expect("connection should be available");
    let settings_columns = conn
        .interact(|conn| {
            let mut statement = conn.prepare("PRAGMA table_info(settings)")?;
            let rows = statement.query_map([], |row| row.get::<_, String>(1))?;
            rows.collect::<Result<Vec<_>, _>>()
        })
        .await
        .expect("interaction should complete")
        .expect("table info should be readable");

    assert!(settings_columns.contains(&"scan_roots_json".to_string()));
    assert!(settings_columns.contains(&"tray_bar_sort".to_string()));
}

#[tokio::test]
async fn project_cache_schema_exists_after_reopen() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let db_path = temp_dir.path().join("cache.db");

    drop(migrated_pool(&db_path).await);

    let pool = migrated_pool(&db_path).await;
    let version = store::migration_version(&pool)
        .await
        .expect("migration version should be readable");
    assert!(version >= EXPECTED_MIGRATION_COUNT);

    let conn = pool.get().await.expect("connection should be available");
    let tables = conn
        .interact(|conn| {
            let mut statement = conn.prepare(
                "SELECT name FROM sqlite_master
                 WHERE type = 'table' AND name IN ('projects', 'phase_plans', 'scan_log')
                 ORDER BY name",
            )?;
            let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
            rows.collect::<Result<Vec<_>, _>>()
        })
        .await
        .expect("interaction should complete")
        .expect("table names should be readable");

    assert_eq!(tables, vec!["phase_plans", "projects", "scan_log"]);

    let project_columns = conn
        .interact(|conn| {
            let mut statement = conn.prepare("PRAGMA table_info(projects)")?;
            let rows = statement.query_map([], |row| {
                Ok((row.get::<_, String>(1)?, row.get::<_, String>(2)?))
            })?;
            rows.collect::<Result<Vec<_>, _>>()
        })
        .await
        .expect("interaction should complete")
        .expect("project columns should be readable");

    assert!(project_columns
        .iter()
        .any(|(name, data_type)| name == "current_phase_number" && data_type == "TEXT"));
    assert!(project_columns
        .iter()
        .any(|(name, data_type)| name == "parse_error" && data_type == "TEXT"));
}

#[tokio::test]
async fn migration_4_adds_plan_items_completed_at_and_cache_tokens() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let db_path = temp_dir.path().join("cache.db");

    let pool = migrated_pool(&db_path).await;
    let conn = pool.get().await.expect("connection should be available");
    conn.interact(|conn| {
        let phase_plan_columns = table_column_names(conn, "phase_plans")?;
        assert!(phase_plan_columns.contains(&"completed_at".to_string()));

        let session_columns = table_column_names(conn, "sessions")?;
        assert!(session_columns.contains(&"cache_read_tokens".to_string()));
        assert!(session_columns.contains(&"cache_creation_tokens".to_string()));

        let plan_item_columns = table_column_names(conn, "plan_items")?;
        assert_eq!(
            plan_item_columns,
            vec!["project_id", "plan_path", "ord", "text", "checked", "line_no"]
        );

        conn.execute(
            "INSERT INTO projects (
                id, name, root_path, planning_path, parsed_blob, last_scanned_at, created_at, updated_at
            ) VALUES ('project-1', 'Project', '/tmp/project', '/tmp/project/.planning', '{}', 1, 1, 1)",
            [],
        )?;
        conn.execute(
            "INSERT INTO phase_plans (
                project_id, phase_number, plan_number, plan_path, checklist_json, updated_at
            ) VALUES ('project-1', '05', '01', '/tmp/project/.planning/05-01-PLAN.md', '[]', 1)",
            [],
        )?;
        conn.execute(
            "INSERT INTO plan_items (
                project_id, plan_path, ord, text, checked, line_no
            ) VALUES ('project-1', '/tmp/project/.planning/05-01-PLAN.md', 1, 'Do work', 0, 42)",
            [],
        )?;
        conn.execute(
            "DELETE FROM phase_plans WHERE project_id = 'project-1' AND plan_path = '/tmp/project/.planning/05-01-PLAN.md'",
            [],
        )?;

        let plan_item_count: i64 =
            conn.query_row("SELECT COUNT(*) FROM plan_items", [], |row| row.get(0))?;
        assert_eq!(plan_item_count, 0);

        Ok::<_, rusqlite::Error>(())
    })
    .await
    .expect("interaction should complete")
    .expect("schema contract should hold");
}

#[tokio::test]
async fn migration_5_adds_daily_activity() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let db_path = temp_dir.path().join("cache.db");

    let pool = migrated_pool(&db_path).await;
    let conn = pool.get().await.expect("connection should be available");
    conn.interact(|conn| {
        let daily_activity_columns = table_column_names(conn, "daily_activity")?;
        assert_eq!(
            daily_activity_columns,
            vec![
                "date",
                "session_count",
                "token_total",
                "top_project_id",
                "updated_at"
            ]
        );

        conn.execute(
            "INSERT INTO projects (
                id, name, root_path, planning_path, parsed_blob, last_scanned_at, created_at, updated_at
            ) VALUES ('project-1', 'Project', '/tmp/project', '/tmp/project/.planning', '{}', 1, 1, 1)",
            [],
        )?;
        conn.execute(
            "INSERT INTO daily_activity (
                date, session_count, token_total, top_project_id, updated_at
            ) VALUES ('2026-04-27', 2, 300, 'project-1', 1)",
            [],
        )?;
        conn.execute("DELETE FROM projects WHERE id = 'project-1'", [])?;

        let top_project_id: Option<String> = conn.query_row(
            "SELECT top_project_id FROM daily_activity WHERE date = '2026-04-27'",
            [],
            |row| row.get(0),
        )?;
        assert_eq!(top_project_id, None);

        Ok::<_, rusqlite::Error>(())
    })
    .await
    .expect("interaction should complete")
    .expect("daily activity schema should hold");
}

#[tokio::test]
async fn settings_table_has_tray_visibility_column() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let db_path = temp_dir.path().join("cache.db");

    let pool = migrated_pool(&db_path).await;
    let conn = pool.get().await.expect("connection should be available");
    conn.interact(|conn| {
        let settings_columns = table_column_names(conn, "settings")?;
        assert!(settings_columns.contains(&"tray_hidden_project_ids_json".to_string()));

        Ok::<_, rusqlite::Error>(())
    })
    .await
    .expect("interaction should complete")
    .expect("settings schema should hold");
}

fn table_column_names(
    conn: &rusqlite::Connection,
    table_name: &str,
) -> Result<Vec<String>, rusqlite::Error> {
    let mut statement = conn.prepare(&format!("PRAGMA table_info({table_name})"))?;
    let rows = statement.query_map([], |row| row.get::<_, String>(1))?;
    rows.collect::<Result<Vec<_>, _>>()
}
