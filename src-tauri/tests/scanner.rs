use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use gsd_dashboard::{
    app_state::{AppState, BootStatus},
    commands::scan::scan_projects_for_app,
    error::AppError,
    events::ScanEvent,
    scan_service,
    scanner::discover_planning_dirs,
    store::{self, project_repo},
};

fn create_planning_dir(project_root: &Path) {
    fs::create_dir_all(project_root.join(".planning")).expect("planning dir should be created");
}

async fn migrated_pool(db_path: &Path) -> deadpool_sqlite::Pool {
    let pool = store::open_pool(db_path).await.expect("pool should open");
    store::run_migrations(&pool)
        .await
        .expect("migrations should run");
    pool
}

async fn test_app_state(home_dir: PathBuf, scan_root: &Path) -> AppState {
    let app_data_dir = home_dir.join("app-data");
    let cache_path = app_data_dir.join("cache.db");
    fs::create_dir_all(&app_data_dir).expect("app data dir should be created");
    let pool = migrated_pool(&cache_path).await;
    gsd_dashboard::settings::save(
        &pool,
        &home_dir,
        gsd_dashboard::settings::SettingsInput {
            scan_roots: vec![scan_root.display().to_string()],
            hidden_project_ids: Vec::new(),
            autostart_enabled: false,
            tray_bar_max_projects: 8,
            tray_bar_sort: gsd_dashboard::settings::TrayBarSort::RecentActivity,
        },
    )
    .await
    .expect("settings should be saved");

    AppState::new(
        pool,
        home_dir,
        app_data_dir.clone(),
        cache_path.clone(),
        BootStatus {
            app_data_dir: app_data_dir.display().to_string(),
            cache_path: cache_path.display().to_string(),
            cache_ready: true,
            wal_enabled: true,
            migrations_applied: 2,
            settings_initialized: true,
        },
    )
}

fn write_valid_planning_project(project_root: &Path, project_name: &str) {
    let planning_dir = project_root.join(".planning");
    fs::create_dir_all(planning_dir.join("phases/02-planning-parser-scanner"))
        .expect("planning phase dir should be created");
    fs::write(
        planning_dir.join("ROADMAP.md"),
        format!(
            r#"# Roadmap

**Milestone:** v1.0 MVP

- [x] **Phase 1: Foundation**
- [ ] **Phase 2: {project_name}**
"#
        ),
    )
    .expect("roadmap should be written");
    fs::write(
        planning_dir.join("STATE.md"),
        r#"---
milestone: v1.0
milestone_name: v1.0 MVP
---

## Current Position

**Milestone:** v1.0 MVP
**Phase:** 2 (Planning Parser)

## Next Command

```
/gsd-execute-phase 2
```
"#,
    )
    .expect("state should be written");
    fs::write(
        planning_dir.join("config.json"),
        r#"{"workflow":{"auto_advance":true}}"#,
    )
    .expect("config should be written");
    fs::write(
        planning_dir.join("phases/02-planning-parser-scanner/02-01-PLAN.md"),
        r#"---
phase: 02-planning-parser-scanner
plan: 01
type: execute
---

<tasks>
<task type="auto">
  <name>Task 1</name>
  <done>Done.</done>
</task>
</tasks>
"#,
    )
    .expect("plan should be written");
}

#[test]
fn scanner_discovers_planning_dirs() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let home_dir = temp_dir.path();
    let scan_root = home_dir.join("workspace");
    let project_root = scan_root.join("project-a");

    create_planning_dir(&project_root);

    let candidates =
        discover_planning_dirs(&scan_root, home_dir).expect("scan root should be discoverable");

    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].project_root, project_root);
    assert_eq!(candidates[0].planning_path, project_root.join(".planning"));
}

#[test]
fn scanner_rejects_bare_home_root() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let home_dir = temp_dir.path();

    let error =
        discover_planning_dirs(home_dir, home_dir).expect_err("bare home root should be rejected");

    assert!(matches!(error, AppError::InvalidScanRoot { .. }));
}

#[test]
fn scanner_deduplicates_symlinked_planning_dirs() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let home_dir = temp_dir.path();
    let scan_root = home_dir.join("workspace");
    let project_root = scan_root.join("project-a");

    create_planning_dir(&project_root);

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&project_root, scan_root.join("project-link"))
            .expect("project symlink should be created");
    }

    let candidates =
        discover_planning_dirs(&scan_root, home_dir).expect("scan root should be discoverable");

    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].planning_path, project_root.join(".planning"));
}

#[test]
fn scanner_skips_unreadable_entries() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let home_dir = temp_dir.path();
    let scan_root = home_dir.join("workspace");
    let project_root = scan_root.join("project-a");
    let unreadable_dir = scan_root.join("unreadable");

    create_planning_dir(&project_root);
    fs::create_dir_all(&unreadable_dir).expect("unreadable dir should be created");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(&unreadable_dir, fs::Permissions::from_mode(0o000))
            .expect("permissions should be restricted");
    }

    let candidates =
        discover_planning_dirs(&scan_root, home_dir).expect("unreadable entries should be skipped");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(&unreadable_dir, fs::Permissions::from_mode(0o755))
            .expect("permissions should be restored for cleanup");
    }

    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].planning_path, project_root.join(".planning"));
}

#[test]
#[ignore]
fn scanner_discovers_real_homegit_projects() {
    let home_dir = Path::new("/Users/smacdonald");
    let candidates = discover_planning_dirs(Path::new("/Users/smacdonald/homegit"), home_dir)
        .expect("homegit scan should not abort");

    assert!(!candidates.is_empty());
}

#[tokio::test]
#[ignore]
async fn scan_service_scans_real_homegit_projects() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let pool = migrated_pool(&temp_dir.path().join("cache.db")).await;
    let summary = scan_service::scan_roots(
        pool,
        vec![PathBuf::from("/Users/smacdonald/homegit")],
        PathBuf::from("/Users/smacdonald"),
        |_| Ok(()),
    )
    .await
    .expect("homegit scan should not fail to start");

    assert!(summary.discovered_count > 0);
}

#[tokio::test]
async fn malformed_project_does_not_abort_scan() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let home_dir = temp_dir.path();
    let scan_root = home_dir.join("workspace");
    let bad_project = scan_root.join("bad-project");
    let good_project = scan_root.join("good-project");
    let pool = migrated_pool(&temp_dir.path().join("cache.db")).await;

    write_valid_planning_project(&bad_project, "Bad Project");
    write_valid_planning_project(&good_project, "Good Project");
    fs::write(
        bad_project.join(".planning/config.json"),
        br#"{"workflow":"not an object"}"#,
    )
    .expect("malformed config should be written");

    let summary = scan_service::scan_roots(
        pool.clone(),
        vec![scan_root],
        home_dir.to_path_buf(),
        |_| Ok(()),
    )
    .await
    .expect("scan should continue after per-project parse errors");

    assert_eq!(summary.discovered_count, 2);
    assert_eq!(summary.parsed_count, 1);
    assert_eq!(summary.error_count, 1);

    let connection = pool.get().await.expect("connection should be available");
    connection
        .interact(move |connection| {
            let bad = project_repo::load_project_by_root(
                connection,
                bad_project.to_string_lossy().as_ref(),
            )?
            .expect("degraded malformed project should be persisted");
            let good = project_repo::load_project_by_root(
                connection,
                good_project.to_string_lossy().as_ref(),
            )?
            .expect("later valid project should be persisted");

            assert!(bad.id.starts_with("bad-project-"));
            assert!(bad.parse_error.is_some());
            assert!(good.id.starts_with("good-project-"));
            assert_eq!(good.parse_error, None);

            Ok::<_, AppError>(())
        })
        .await
        .expect("interaction should complete")
        .expect("repository reads should pass");
}

#[tokio::test]
async fn scanner_records_parse_error_in_scan_log() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let home_dir = temp_dir.path();
    let scan_root = home_dir.join("workspace");
    let bad_project = scan_root.join("bad-project");
    let pool = migrated_pool(&temp_dir.path().join("cache.db")).await;

    write_valid_planning_project(&bad_project, "Bad Project");
    fs::write(
        bad_project.join(".planning/config.json"),
        br#"{"workflow":"not an object"}"#,
    )
    .expect("malformed config should be written");

    scan_service::scan_roots(
        pool.clone(),
        vec![scan_root],
        home_dir.to_path_buf(),
        |_| Ok(()),
    )
    .await
    .expect("scan should record parse error");

    let connection = pool.get().await.expect("connection should be available");
    connection
        .interact(|connection| {
            let status = connection.query_row(
                "SELECT status FROM scan_log WHERE status = ?1",
                ["parseError"],
                |row| row.get::<_, String>(0),
            )?;

            assert_eq!(status, "parseError");

            Ok::<_, AppError>(())
        })
        .await
        .expect("interaction should complete")
        .expect("scan log should be readable");
}

#[tokio::test]
async fn same_named_projects_get_distinct_cache_ids() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let home_dir = temp_dir.path();
    let scan_root = home_dir.join("workspace");
    let first_project = scan_root.join("client-a/app");
    let second_project = scan_root.join("client-b/app");
    let pool = migrated_pool(&temp_dir.path().join("cache.db")).await;

    write_valid_planning_project(&first_project, "First App");
    write_valid_planning_project(&second_project, "Second App");

    let summary = scan_service::scan_roots(
        pool.clone(),
        vec![scan_root],
        home_dir.to_path_buf(),
        |_| Ok(()),
    )
    .await
    .expect("same-named projects should scan");

    assert_eq!(summary.discovered_count, 2);
    assert_eq!(summary.parsed_count, 2);
    assert_eq!(summary.error_count, 0);

    let connection = pool.get().await.expect("connection should be available");
    connection
        .interact(move |connection| {
            let first = project_repo::load_project_by_root(
                connection,
                first_project.to_string_lossy().as_ref(),
            )?
            .expect("first same-named project should be persisted");
            let second = project_repo::load_project_by_root(
                connection,
                second_project.to_string_lossy().as_ref(),
            )?
            .expect("second same-named project should be persisted");

            assert_ne!(first.id, second.id);
            assert!(first.id.starts_with("app-"));
            assert!(second.id.starts_with("app-"));

            Ok::<_, AppError>(())
        })
        .await
        .expect("interaction should complete")
        .expect("repository reads should pass");
}

#[tokio::test]
async fn plan_directory_io_error_records_parse_error() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let home_dir = temp_dir.path();
    let scan_root = home_dir.join("workspace");
    let bad_project = scan_root.join("bad-project");
    let planning_dir = bad_project.join(".planning");
    let pool = migrated_pool(&temp_dir.path().join("cache.db")).await;

    fs::create_dir_all(&planning_dir).expect("planning dir should be created");
    fs::write(
        planning_dir.join("ROADMAP.md"),
        "# Roadmap\n\n- [ ] **Phase 1: Bad Project**\n",
    )
    .expect("roadmap should be written");
    fs::write(planning_dir.join("phases"), "not a directory")
        .expect("phases file should trigger read_dir error");

    let summary = scan_service::scan_roots(
        pool.clone(),
        vec![scan_root],
        home_dir.to_path_buf(),
        |_| Ok(()),
    )
    .await
    .expect("plan directory I/O errors should not abort scan");

    assert_eq!(summary.discovered_count, 1);
    assert_eq!(summary.parsed_count, 0);
    assert_eq!(summary.error_count, 1);

    let connection = pool.get().await.expect("connection should be available");
    connection
        .interact(move |connection| {
            let bad = project_repo::load_project_by_root(
                connection,
                bad_project.to_string_lossy().as_ref(),
            )?
            .expect("project with plan directory error should be persisted");

            assert!(bad.parse_error.is_some());

            Ok::<_, AppError>(())
        })
        .await
        .expect("interaction should complete")
        .expect("repository read should pass");
}

#[test]
fn scanner_remains_discovery_only() {
    let scanner_source =
        fs::read_to_string("src/scanner.rs").expect("scanner source should be readable");
    let forbidden_patterns = [
        "project_repo",
        "parse_roadmap",
        "parse_state",
        "parse_plan",
        "parse_config",
        "upsert_project_snapshot",
        "append_scan_log",
    ];

    for pattern in forbidden_patterns {
        assert!(
            !scanner_source.contains(pattern),
            "scanner.rs should not contain {pattern}"
        );
    }
}

#[tokio::test]
async fn scan_command_streams_progress() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let home_dir = temp_dir.path().to_path_buf();
    let scan_root = home_dir.join("workspace");
    let project_root = scan_root.join("good-project");
    write_valid_planning_project(&project_root, "Good Project");
    let state = test_app_state(home_dir, &scan_root).await;
    let events = Arc::new(Mutex::new(Vec::new()));
    let recorded_events = Arc::clone(&events);

    let summary = scan_projects_for_app(&state, move |event| {
        recorded_events
            .lock()
            .expect("events lock should not be poisoned")
            .push(event);
        Ok(())
    })
    .await
    .expect("scan command helper should complete");

    let events = events.lock().expect("events should be readable").clone();
    let event_names = events
        .iter()
        .map(|event| match event {
            ScanEvent::Started { .. } => "started",
            ScanEvent::RootStarted { .. } => "rootStarted",
            ScanEvent::ProjectFound { .. } => "projectFound",
            ScanEvent::ProjectParsed { .. } => "projectParsed",
            ScanEvent::ProjectParseError { .. } => "projectParseError",
            ScanEvent::Finished { .. } => "finished",
        })
        .collect::<Vec<_>>();

    assert_eq!(summary.discovered_count, 1);
    assert_eq!(
        event_names,
        vec![
            "started",
            "rootStarted",
            "projectFound",
            "projectParsed",
            "finished"
        ]
    );

    let serialized_events = serde_json::to_string(&events).expect("events should serialize");
    for forbidden in ["# Roadmap", "# State", "<task", r#""workflow":"#] {
        assert!(
            !serialized_events.contains(forbidden),
            "serialized scan events should not contain raw body marker {forbidden}"
        );
    }
}

#[tokio::test]
async fn scan_command_expands_tilde_scan_roots() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let home_dir = temp_dir.path().to_path_buf();
    let scan_root = home_dir.join("homegit");
    let project_root = scan_root.join("good-project");
    let state = test_app_state(home_dir, &scan_root).await;
    write_valid_planning_project(&project_root, "Good Project");

    gsd_dashboard::settings::save(
        &state.pool,
        &state.home_dir,
        gsd_dashboard::settings::SettingsInput {
            scan_roots: vec!["~/homegit".to_string()],
            hidden_project_ids: Vec::new(),
            autostart_enabled: false,
            tray_bar_max_projects: 8,
            tray_bar_sort: gsd_dashboard::settings::TrayBarSort::RecentActivity,
        },
    )
    .await
    .expect("tilde scan root should save");

    let summary = scan_projects_for_app(&state, |_| Ok(()))
        .await
        .expect("tilde scan root should scan");

    assert_eq!(summary.discovered_count, 1);
    assert_eq!(summary.parsed_count, 1);
    assert_eq!(summary.error_count, 0);
}
