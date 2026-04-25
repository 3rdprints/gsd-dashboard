use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use gsd_dashboard::{
    app_state::{AppState, BootStatus},
    commands::scan::rebuild_cache_for_app,
    error::AppError,
    events::ScanEvent,
    settings::{AppSettings, SettingsInput, TrayBarSort},
    store::{self, project_repo},
};

fn snapshot(id: &str, root_path: &str) -> project_repo::StoredProjectSnapshot {
    project_repo::StoredProjectSnapshot {
        id: id.to_string(),
        name: "Deck Pilot".to_string(),
        root_path: root_path.to_string(),
        planning_path: format!("{root_path}/.planning"),
        current_milestone_name: Some("v1.0".to_string()),
        current_milestone_index: Some(1),
        current_phase_number: Some("03".to_string()),
        current_phase_name: Some("Portfolio".to_string()),
        milestone_progress_pct: 42.0,
        next_command: "/gsd-next".to_string(),
        parsed_blob: r#"{"phase":"03"}"#.to_string(),
        parse_error: None,
        last_activity_at: Some(1_777_000_100),
        last_scanned_at: 1_777_000_200,
        created_at: 0,
        updated_at: 0,
    }
}

fn phase_plan(project_id: &str) -> project_repo::StoredPhasePlan {
    project_repo::StoredPhasePlan {
        project_id: project_id.to_string(),
        phase_number: "03".to_string(),
        phase_name: Some("Portfolio Vertical Slice".to_string()),
        plan_number: Some("02".to_string()),
        plan_path: ".planning/phases/03-portfolio-vertical-slice/03-02-PLAN.md".to_string(),
        checklist_json: r#"[{"label":"rebuild cache","done":false}]"#.to_string(),
        updated_at: 0,
    }
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
        SettingsInput {
            scan_roots: vec![scan_root.display().to_string()],
            hidden_project_ids: vec!["hidden-project".to_string()],
            autostart_enabled: true,
            tray_bar_max_projects: 5,
            tray_bar_sort: TrayBarSort::Name,
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
    fs::create_dir_all(planning_dir.join("phases/03-portfolio-vertical-slice"))
        .expect("planning phase dir should be created");
    fs::write(
        planning_dir.join("ROADMAP.md"),
        format!(
            r#"# Roadmap

**Milestone:** v1.0 MVP

- [x] **Phase 1: Foundation**
- [ ] **Phase 3: {project_name}**
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
**Phase:** 3 (Portfolio Vertical Slice)

## Next Command

```
/gsd-execute-phase 3
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
        planning_dir.join("phases/03-portfolio-vertical-slice/03-02-PLAN.md"),
        r#"---
phase: 03-portfolio-vertical-slice
plan: 02
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

async fn load_settings(pool: &deadpool_sqlite::Pool) -> AppSettings {
    gsd_dashboard::settings::load_or_initialize(pool, Path::new("/tmp"))
        .await
        .expect("settings should load")
}

#[tokio::test]
async fn clear_project_cache_removes_only_derived_rows() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let home_dir = temp_dir.path();
    let scan_root = home_dir.join("workspace");
    let pool = migrated_pool(&temp_dir.path().join("cache.db")).await;
    gsd_dashboard::settings::save(
        &pool,
        home_dir,
        SettingsInput {
            scan_roots: vec![scan_root.display().to_string()],
            hidden_project_ids: vec!["hidden-project".to_string()],
            autostart_enabled: true,
            tray_bar_max_projects: 5,
            tray_bar_sort: TrayBarSort::Name,
        },
    )
    .await
    .expect("settings should be saved");

    let connection = pool.get().await.expect("connection should be available");
    connection
        .interact(|connection| {
            project_repo::upsert_project_snapshot(
                connection,
                snapshot("project-1", "/tmp/deckpilot"),
                vec![phase_plan("project-1")],
                1_777_000_300,
            )?;
            project_repo::append_scan_log(
                connection,
                project_repo::StoredScanLogEntry {
                    project_id: Some("project-1".to_string()),
                    root_path: Some("/tmp/deckpilot".to_string()),
                    planning_path: Some("/tmp/deckpilot/.planning".to_string()),
                    file_path: None,
                    status: "parsed".to_string(),
                    message: None,
                    errors_json: "[]".to_string(),
                    created_at: 0,
                },
                1_777_000_400,
            )?;

            project_repo::clear_project_cache(connection)?;

            let settings_count =
                connection.query_row("SELECT COUNT(*) FROM settings", [], |row| {
                    row.get::<_, i64>(0)
                })?;
            let project_count =
                connection.query_row("SELECT COUNT(*) FROM projects", [], |row| {
                    row.get::<_, i64>(0)
                })?;
            let phase_plan_count =
                connection.query_row("SELECT COUNT(*) FROM phase_plans", [], |row| {
                    row.get::<_, i64>(0)
                })?;
            let scan_log_count =
                connection.query_row("SELECT COUNT(*) FROM scan_log", [], |row| {
                    row.get::<_, i64>(0)
                })?;

            assert_eq!(settings_count, 1);
            assert_eq!(project_count, 0);
            assert_eq!(phase_plan_count, 0);
            assert_eq!(scan_log_count, 0);

            Ok::<_, AppError>(())
        })
        .await
        .expect("interaction should complete")
        .expect("cache clear should preserve settings only");
}

#[tokio::test]
async fn rebuild_cache_preserves_settings_and_emits_scan_events() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let home_dir = temp_dir.path().to_path_buf();
    let scan_root = home_dir.join("workspace");
    let project_root = scan_root.join("good-project");
    write_valid_planning_project(&project_root, "Good Project");
    let state = test_app_state(home_dir, &scan_root).await;
    let before_settings = load_settings(&state.pool).await;
    let events = Arc::new(Mutex::new(Vec::new()));
    let recorded_events = Arc::clone(&events);

    let summary = rebuild_cache_for_app(&state, move |event| {
        recorded_events
            .lock()
            .expect("events lock should not be poisoned")
            .push(event);
        Ok(())
    })
    .await
    .expect("rebuild should scan configured roots");

    let after_settings = load_settings(&state.pool).await;
    let events = events.lock().expect("events should be readable").clone();

    assert_eq!(summary.discovered_count, 1);
    assert_eq!(summary.parsed_count, 1);
    assert_eq!(before_settings, after_settings);
    assert!(events
        .iter()
        .any(|event| matches!(event, ScanEvent::Started { .. })));
    assert!(events
        .iter()
        .any(|event| matches!(event, ScanEvent::Finished { .. })));
}

#[tokio::test]
async fn rebuild_cache_reuses_scan_root_guardrails() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let home_dir = temp_dir.path().to_path_buf();
    let scan_root = home_dir.join("workspace");
    let state = test_app_state(home_dir.clone(), &scan_root).await;

    let connection = state
        .pool
        .get()
        .await
        .expect("connection should be available");
    connection
        .interact(|connection| {
            connection.execute(
                "UPDATE settings SET scan_roots_json = ?1 WHERE id = 1",
                [r#"["/"]"#],
            )?;

            Ok::<_, AppError>(())
        })
        .await
        .expect("interaction should complete")
        .expect("settings should be overwritten for guardrail test");

    let error = rebuild_cache_for_app(&state, |_| Ok(()))
        .await
        .expect_err("broad roots should be rejected");

    assert!(matches!(error, AppError::InvalidScanRoot { .. }));
}
