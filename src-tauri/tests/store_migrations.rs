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
