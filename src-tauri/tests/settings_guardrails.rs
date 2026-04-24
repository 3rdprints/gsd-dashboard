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

#[test]
fn default_settings_serialize_recent_activity_wire_value() {
    let serialized =
        serde_json::to_value(AppSettings::default()).expect("settings should serialize");

    assert_eq!(serialized["trayBarSort"], "recent_activity");
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

#[test]
fn validate_scan_root_rejects_broad_roots_and_accepts_specific_folders() {
    let home = Path::new("/Users/smacdonald");

    assert_invalid_root(scan_roots::validate_scan_root(Path::new("/"), home), "/");
    assert_invalid_root(
        scan_roots::validate_scan_root(home, home),
        "/Users/smacdonald",
    );
    assert_invalid_root(
        scan_roots::validate_scan_root(Path::new("~"), home),
        "/Users/smacdonald",
    );
    assert_invalid_root(
        scan_roots::validate_scan_root(Path::new("/Users"), home),
        "/Users",
    );
    assert_invalid_root(
        scan_roots::validate_scan_root(Path::new("/tmp"), home),
        "/tmp",
    );
    assert_invalid_root(
        scan_roots::validate_scan_root(Path::new("/Volumes"), home),
        "/Volumes",
    );

    scan_roots::validate_scan_root(Path::new("~/Documents"), home)
        .expect("specific folders under home should be accepted");
    scan_roots::validate_scan_root(Path::new("~/Documents/Work"), home)
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
