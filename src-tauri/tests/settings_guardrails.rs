use gsd_dashboard::{
    error::AppError,
    scan_roots,
    settings::{self, AppSettings, SettingsInput, TrayBarSort},
    store,
};
use std::path::Path;

async fn open_migrated_pool(db_path: &std::path::Path) -> deadpool_sqlite::Pool {
    let pool = store::open_pool(db_path).await.expect("pool should open");
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
        global_sessions_default_range: settings.global_sessions_default_range.clone(),
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
    assert_eq!(settings.global_sessions_default_range, "7d");
}

#[test]
fn default_settings_serialize_recent_activity_wire_value() {
    let serialized =
        serde_json::to_value(AppSettings::default()).expect("settings should serialize");

    assert_eq!(serialized["trayBarSort"], "recent_activity");
    assert_eq!(serialized["globalSessionsDefaultRange"], "7d");
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
            global_sessions_default_range: "30d".to_string(),
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
async fn invalid_global_sessions_default_range_coerces_to_seven_days() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let db_path = temp_dir.path().join("cache.db");
    let pool = open_migrated_pool(&db_path).await;
    let defaults = settings::load_or_initialize(&pool, temp_dir.path())
        .await
        .expect("settings should initialize");
    settings::save(&pool, temp_dir.path(), input_from(&defaults))
        .await
        .expect("settings should save");

    let conn = pool.get().await.expect("connection should be available");
    conn.interact(|conn| {
        conn.execute(
            "UPDATE settings SET global_sessions_default_range = 'forever' WHERE id = 1",
            [],
        )
    })
    .await
    .expect("interaction should complete")
    .expect("corrupt update should run");

    let loaded = settings::load_or_initialize(&pool, temp_dir.path())
        .await
        .expect("invalid global sessions range should coerce");
    assert_eq!(loaded.global_sessions_default_range, "7d");
}

#[tokio::test]
async fn empty_scan_roots_coerce_to_default() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let db_path = temp_dir.path().join("cache.db");
    let pool = open_migrated_pool(&db_path).await;

    let saved = settings::save(
        &pool,
        temp_dir.path(),
        SettingsInput {
            scan_roots: vec![" ".to_string()],
            hidden_project_ids: Vec::new(),
            autostart_enabled: false,
            tray_bar_max_projects: 8,
            tray_bar_sort: TrayBarSort::RecentActivity,
            global_sessions_default_range: "7d".to_string(),
        },
    )
    .await
    .expect("empty scan roots should save as defaults");

    assert_eq!(saved.scan_roots, vec!["~/Documents"]);

    let conn = pool.get().await.expect("connection should be available");
    conn.interact(|conn| {
        conn.execute("UPDATE settings SET scan_roots_json = '[]' WHERE id = 1", [])
    })
    .await
    .expect("interaction should complete")
    .expect("empty roots update should run");

    let loaded = settings::load_or_initialize(&pool, temp_dir.path())
        .await
        .expect("empty stored scan roots should coerce");
    assert_eq!(loaded.scan_roots, vec!["~/Documents"]);
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

#[test]
fn validate_scan_root_rejects_broad_roots_and_accepts_specific_folders() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let home = temp_dir.path().join("home");
    let outside_home = temp_dir.path().join("outside");
    let documents = home.join("Documents");
    let work = home.join("Documents").join("Work");

    assert_invalid_root(
        scan_roots::validate_scan_root(&home, &home),
        &home.display().to_string(),
    );
    assert_invalid_root(
        scan_roots::validate_scan_root(Path::new("~"), &home),
        &home.display().to_string(),
    );
    assert_invalid_root(
        scan_roots::validate_scan_root(temp_dir.path(), &home),
        &temp_dir.path().display().to_string(),
    );
    assert_invalid_root(
        scan_roots::validate_scan_root(&outside_home, &home),
        &outside_home.display().to_string(),
    );

    scan_roots::validate_scan_root(&documents, &home)
        .expect("specific folders under home should be accepted");
    scan_roots::validate_scan_root(&work, &home)
        .expect("nested folders under home should be accepted");
    scan_roots::validate_scan_root(Path::new("~/Documents"), &home)
        .expect("tilde-specific folders under home should be accepted");
    scan_roots::validate_scan_root(Path::new("~/Documents/Work"), &home)
        .expect("nested folders under home should be accepted");
}

#[tokio::test]
async fn invalid_roots_do_not_persist() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let db_path = temp_dir.path().join("cache.db");
    let home = temp_dir.path().join("home");
    let pool = open_migrated_pool(&db_path).await;

    let saved = settings::save(
        &pool,
        &home,
        SettingsInput {
            scan_roots: vec!["~/Documents".to_string()],
            hidden_project_ids: Vec::new(),
            autostart_enabled: false,
            tray_bar_max_projects: 8,
            tray_bar_sort: TrayBarSort::RecentActivity,
            global_sessions_default_range: "7d".to_string(),
        },
    )
    .await
    .expect("valid settings should save");

    let rejected = settings::save(
        &pool,
        &home,
        SettingsInput {
            scan_roots: vec!["/".to_string(), home.display().to_string()],
            hidden_project_ids: vec!["should-not-persist".to_string()],
            autostart_enabled: true,
            tray_bar_max_projects: 2,
            tray_bar_sort: TrayBarSort::Progress,
            global_sessions_default_range: "90d".to_string(),
        },
    )
    .await
    .expect_err("broad roots should be rejected");

    assert_invalid_root(Err(rejected), "/");

    let after_rejection = settings::load_or_initialize(&pool, &home)
        .await
        .expect("previous settings should still load");
    assert_eq!(after_rejection, saved);
    assert!(!after_rejection.scan_roots.iter().any(|root| root == "/"));
    assert!(!after_rejection
        .scan_roots
        .iter()
        .any(|root| root == &home.display().to_string()));
}

fn assert_invalid_root(result: Result<(), AppError>, expected_path: &str) {
    let error = result.expect_err("root should be rejected");
    match error {
        AppError::InvalidScanRoot { path, reason } => {
            assert_eq!(path, expected_path);
            assert_eq!(
                reason,
                "This scan root is too broad. Choose a specific folder inside your home directory, such as ~/Documents or a project workspace."
            );
        }
        other => panic!("expected invalid scan root, got {other:?}"),
    }
}
