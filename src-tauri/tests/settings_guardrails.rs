use gsd_dashboard::{
    settings::{self, AppSettings, SettingsInput, TrayBarSort},
    store,
};

async fn open_migrated_pool(db_path: &std::path::Path) -> deadpool_sqlite::Pool {
    let pool = store::open_pool(db_path)
        .await
        .expect("pool should open");
    store::run_migrations(&pool)
        .await
        .expect("migrations should run");
    pool
}

fn input_from(settings: &AppSettings) -> SettingsInput {
    SettingsInput {
        scan_roots: settings.scan_roots.clone(),
        hidden_project_ids: settings.hidden_project_ids.clone(),
        autostart_enabled: settings.autostart_enabled,
        tray_bar_max_projects: settings.tray_bar_max_projects,
        tray_bar_sort: settings.tray_bar_sort,
    }
}

#[tokio::test]
async fn missing_settings_row_initializes_phase_one_defaults() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let db_path = temp_dir.path().join("cache.db");
    let pool = open_migrated_pool(&db_path).await;

    let settings = settings::load_or_initialize(&pool, temp_dir.path())
        .await
        .expect("settings should initialize");

    assert_eq!(settings.scan_roots, vec!["~/Documents"]);
    assert!(settings.hidden_project_ids.is_empty());
    assert!(!settings.autostart_enabled);
    assert_eq!(settings.tray_bar_max_projects, 8);
    assert_eq!(settings.tray_bar_sort, TrayBarSort::RecentActivity);
}

#[tokio::test]
async fn settings_round_trip_survives_database_reopen() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let db_path = temp_dir.path().join("cache.db");
    let pool = open_migrated_pool(&db_path).await;

    let saved = settings::save(
        &pool,
        temp_dir.path(),
        SettingsInput {
            scan_roots: vec!["~/Documents".to_string(), "~/homegit".to_string()],
            hidden_project_ids: vec!["project-a".to_string()],
            autostart_enabled: true,
            tray_bar_max_projects: 4,
            tray_bar_sort: TrayBarSort::Name,
        },
    )
    .await
    .expect("settings should save");
    drop(pool);

    let reopened_pool = open_migrated_pool(&db_path).await;
    let reopened = settings::load_or_initialize(&reopened_pool, temp_dir.path())
        .await
        .expect("settings should reload");

    assert_eq!(reopened, saved);
}

#[tokio::test]
async fn invalid_saved_json_returns_app_error_instead_of_panicking() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let db_path = temp_dir.path().join("cache.db");
    let pool = open_migrated_pool(&db_path).await;
    let defaults = settings::load_or_initialize(&pool, temp_dir.path())
        .await
        .expect("settings should initialize");
    let input = input_from(&defaults);
    settings::save(&pool, temp_dir.path(), input)
        .await
        .expect("settings should save");

    let conn = pool.get().await.expect("connection should be available");
    conn.interact(|conn| {
        conn.execute(
            "UPDATE settings SET scan_roots_json = 'not valid json' WHERE id = 1",
            [],
        )
    })
    .await
    .expect("interaction should complete")
    .expect("corrupt update should run");

    let error = settings::load_or_initialize(&pool, temp_dir.path())
        .await
        .expect_err("invalid JSON should be reported");
    let serialized = serde_json::to_value(&error).expect("error should serialize");

    assert_eq!(serialized["kind"], "store");
    assert!(serialized["message"].as_str().is_some());
}
