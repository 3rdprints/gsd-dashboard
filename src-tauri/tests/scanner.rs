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
    sessions::project_detail,
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
            global_sessions_default_range: "7d".to_string(),
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
fn scanner_skips_planning_dirs_under_hidden_workspaces() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let home_dir = temp_dir.path();
    let scan_root = home_dir.join("workspace");
    let real_project_root = scan_root.join("project-a");
    let hidden_worktree_root = scan_root.join(".claude/worktrees/agent-a485842780e148052");

    create_planning_dir(&real_project_root);
    create_planning_dir(&hidden_worktree_root);

    let candidates =
        discover_planning_dirs(&scan_root, home_dir).expect("scan root should be discoverable");

    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].project_root, real_project_root);
}

#[test]
fn scanner_skips_git_worktree_roots() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let home_dir = temp_dir.path();
    let scan_root = home_dir.join("workspace");
    let real_project_root = scan_root.join("project-a");
    let worktree_root = scan_root.join("project-a-worktree");

    create_planning_dir(&real_project_root);
    create_planning_dir(&worktree_root);
    fs::write(
        worktree_root.join(".git"),
        "gitdir: ../project-a/.git/worktrees/project-a-worktree\n",
    )
    .expect("worktree git file should be written");

    let candidates =
        discover_planning_dirs(&scan_root, home_dir).expect("scan root should be discoverable");

    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].project_root, real_project_root);
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
#[cfg(unix)]
fn scanner_deduplicates_symlinked_planning_dirs() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let home_dir = temp_dir.path();
    let scan_root = home_dir.join("workspace");
    let project_root = scan_root.join("project-a");

    create_planning_dir(&project_root);

    std::os::unix::fs::symlink(&project_root, scan_root.join("project-link"))
        .expect("project symlink should be created");

    let candidates =
        discover_planning_dirs(&scan_root, home_dir).expect("scan root should be discoverable");

    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].planning_path, project_root.join(".planning"));
}

#[test]
#[cfg(unix)]
fn scanner_skips_unreadable_entries() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let home_dir = temp_dir.path();
    let scan_root = home_dir.join("workspace");
    let project_root = scan_root.join("project-a");
    let unreadable_dir = scan_root.join("unreadable");

    create_planning_dir(&project_root);
    fs::create_dir_all(&unreadable_dir).expect("unreadable dir should be created");

    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(&unreadable_dir, fs::Permissions::from_mode(0o000))
        .expect("permissions should be restricted");

    let candidates =
        discover_planning_dirs(&scan_root, home_dir).expect("unreadable entries should be skipped");

    fs::set_permissions(&unreadable_dir, fs::Permissions::from_mode(0o755))
        .expect("permissions should be restored for cleanup");

    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].planning_path, project_root.join(".planning"));
}

#[test]
#[ignore]
fn scanner_discovers_real_homegit_projects() {
    let Some((home_dir, scan_root)) = manual_workstation_scan_paths() else {
        eprintln!("set GSD_DASHBOARD_MANUAL_HOME and GSD_DASHBOARD_MANUAL_SCAN_ROOT to run");
        return;
    };
    let candidates = discover_planning_dirs(&scan_root, &home_dir)
        .expect("manual workstation scan should not abort");

    assert!(!candidates.is_empty());
}

#[tokio::test]
#[ignore]
async fn scan_service_scans_real_homegit_projects() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let pool = migrated_pool(&temp_dir.path().join("cache.db")).await;
    let Some((home_dir, scan_root)) = manual_workstation_scan_paths() else {
        eprintln!("set GSD_DASHBOARD_MANUAL_HOME and GSD_DASHBOARD_MANUAL_SCAN_ROOT to run");
        return;
    };
    let summary = scan_service::scan_roots(pool, vec![scan_root], home_dir, |_| Ok(()))
        .await
        .expect("homegit scan should not fail to start");

    assert!(summary.discovered_count > 0);
}

fn manual_workstation_scan_paths() -> Option<(PathBuf, PathBuf)> {
    let home_dir = std::env::var_os("GSD_DASHBOARD_MANUAL_HOME").map(PathBuf::from)?;
    let scan_root = std::env::var_os("GSD_DASHBOARD_MANUAL_SCAN_ROOT").map(PathBuf::from)?;

    Some((home_dir, scan_root))
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
async fn scan_progress_uses_phase_plan_summary_files() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let home_dir = temp_dir.path();
    let scan_root = home_dir.join("workspace");
    let project_root = scan_root.join("summary-progress-project");
    let planning_dir = project_root.join(".planning");
    let phase_dir = planning_dir.join("phases/02-planning-parser-scanner");
    let pool = migrated_pool(&temp_dir.path().join("cache.db")).await;

    write_valid_planning_project(&project_root, "Summary Progress Project");
    fs::write(
        phase_dir.join("02-02-PLAN.md"),
        r#"---
phase: 02-planning-parser-scanner
plan: 02
type: execute
---

<tasks>
<task type="auto">
  <name>Task 2</name>
</task>
</tasks>
"#,
    )
    .expect("second plan should be written");
    fs::write(
        phase_dir.join("02-01-SUMMARY.md"),
        "# Summary\n\nPlan 1 complete.\n",
    )
    .expect("summary should be written");

    scan_service::scan_roots(
        pool.clone(),
        vec![scan_root],
        home_dir.to_path_buf(),
        |_| Ok(()),
    )
    .await
    .expect("scan should parse project");

    let connection = pool.get().await.expect("connection should be available");
    connection
        .interact(move |connection| {
            let project = project_repo::load_project_by_root(
                connection,
                project_root.to_string_lossy().as_ref(),
            )?
            .expect("project should be persisted");

            assert!((project.milestone_progress_pct - 50.0).abs() < f64::EPSILON);

            Ok::<_, AppError>(())
        })
        .await
        .expect("interaction should complete")
        .expect("project should load");
}

#[tokio::test]
async fn completed_archived_project_uses_state_progress_and_roadmap_timeline() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let home_dir = temp_dir.path();
    let scan_root = home_dir.join("workspace");
    let project_root = scan_root.join("listingguru");
    let planning_dir = project_root.join(".planning");
    let pool = migrated_pool(&temp_dir.path().join("cache.db")).await;

    fs::create_dir_all(&planning_dir).expect("planning dir should be created");
    fs::write(
        planning_dir.join("ROADMAP.md"),
        r#"# Roadmap: ListingGuru

## Milestones

- ✅ **v1.2 Optimizer Rework** — Phases 15-19 (shipped 2026-03-30)

## Phases

<details>
<summary>✅ v1.2 Optimizer Rework (Phases 15-19) — SHIPPED 2026-03-30</summary>

- [x] Phase 15: Pipeline & Etsy API Verification (3/3 plans) — completed 2026-03-28
- [x] Phase 16: Optimizer Modal UX (2/2 plans) — completed 2026-03-29
- [x] Phase 17: Admin Analysis Stats (2/2 plans) — completed 2026-03-29
- [x] Phase 18: Rename to Optimizer (2/2 plans) — completed 2026-03-29
- [x] Phase 19: Performance Validation (1/1 plan) — completed 2026-03-30

</details>
"#,
    )
    .expect("roadmap should be written");
    fs::write(
        planning_dir.join("STATE.md"),
        r#"---
gsd_state_version: 1.0
milestone: v1.2
milestone_name: Optimizer Rework
status: completed
progress:
  total_phases: 5
  completed_phases: 5
  total_plans: 10
  completed_plans: 10
  percent: 100
---

# Project State

## Current Position

Milestone: v1.2 Optimizer Rework — SHIPPED
Status: Milestone archived, preparing for next
"#,
    )
    .expect("state should be written");

    scan_service::scan_roots(
        pool.clone(),
        vec![scan_root],
        home_dir.to_path_buf(),
        |_| Ok(()),
    )
    .await
    .expect("scan should parse archived project");

    let connection = pool.get().await.expect("connection should be available");
    connection
        .interact(move |connection| {
            let project = project_repo::load_project_by_root(
                connection,
                project_root.to_string_lossy().as_ref(),
            )?
            .expect("project should be persisted");

            assert_eq!(
                project.current_milestone_name.as_deref(),
                Some("v1.2 Optimizer Rework")
            );
            assert_eq!(project.current_phase_number, None);
            assert!((project.milestone_progress_pct - 100.0).abs() < f64::EPSILON);

            let milestones = project_detail::load_project_milestones(connection, &project.id)?;

            assert_eq!(milestones.len(), 1);
            assert_eq!(milestones[0].phase_count, 5);
            assert_eq!(milestones[0].completed_phase_count, 5);
            assert!((milestones[0].progress_pct - 100.0).abs() < f64::EPSILON);

            Ok::<_, AppError>(())
        })
        .await
        .expect("interaction should complete")
        .expect("project should load");
}

#[tokio::test]
async fn active_project_uses_state_progress_and_summary_backed_plan_counts() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let home_dir = temp_dir.path();
    let scan_root = home_dir.join("workspace");
    let project_root = scan_root.join("deckpilot-web");
    let planning_dir = project_root.join(".planning");
    let phase_dir = planning_dir.join("phases/38-human-uat-platform-tech-debt-close-out");
    let pool = migrated_pool(&temp_dir.path().join("cache.db")).await;

    fs::create_dir_all(&phase_dir).expect("phase dir should be created");
    fs::write(
        planning_dir.join("ROADMAP.md"),
        r#"# Roadmap: Deckpilot

<details>
<summary>🔄 v2.0 Full Product Vision Rewrite (Phases 22-39)</summary>

- [x] Phase 22: platform-foundation (3/3 plans)
- [x] Phase 23: regional-code-system (2/2 plans)
- [ ] Phase 38: human-uat-platform-tech-debt-close-out (1/3 plans)
- [ ] Phase 39: multi-span-joist-planning (0/2 plans)

### Phase 22: platform-foundation

**Plans:** 3/3 plans complete

### Phase 23: regional-code-system

**Plans:** 2/2 plans complete

### Phase 38: human-uat-platform-tech-debt-close-out

**Plans:** 3 plans

### Phase 39: multi-span-joist-planning

**Plans:** 0/2 plans complete

</details>
"#,
    )
    .expect("roadmap should be written");
    fs::write(
        planning_dir.join("STATE.md"),
        r#"---
milestone: v2.0
milestone_name: Full Product Vision Rewrite
status: executing
progress:
  total_phases: 22
  completed_phases: 20
  total_plans: 112
  completed_plans: 110
  percent: 98
---

## Current Position

Milestone: v2.0 Full Product Vision Rewrite
Phase: 38 (human-uat-platform-tech-debt-close-out) — EXECUTING
Plan: 2 of 3 (Plan 01 complete pending operator branch-protection step)

```
v2.0 Progress [ ] 0% (0/11 phases)
```
"#,
    )
    .expect("state should be written");

    for plan_number in ["01", "02", "03"] {
        fs::write(
            phase_dir.join(format!("38-{plan_number}-PLAN.md")),
            format!(
                r#"---
phase: 38
plan: {plan_number}
type: execute
---

# Plan {plan_number}
"#
            ),
        )
        .expect("plan should be written");
    }
    fs::write(
        phase_dir.join("38-01-SUMMARY.md"),
        "# Summary\n\nPlan 01 complete.\n",
    )
    .expect("summary should be written");
    for (phase_number, plan_total, summary_total) in [("22", 3, 3), ("23", 2, 2), ("39", 2, 0)] {
        let sibling_phase_dir = planning_dir.join(format!("phases/{phase_number}-phase"));
        fs::create_dir_all(&sibling_phase_dir).expect("phase dir should be created");
        for plan_index in 1..=plan_total {
            fs::write(
                sibling_phase_dir.join(format!("{phase_number}-{plan_index:02}-PLAN.md")),
                format!(
                    r#"---
phase: {phase_number}
plan: {plan_index:02}
type: execute
---
"#
                ),
            )
            .expect("plan should be written");
        }
        for plan_index in 1..=summary_total {
            fs::write(
                sibling_phase_dir.join(format!("{phase_number}-{plan_index:02}-SUMMARY.md")),
                "# Summary\n",
            )
            .expect("summary should be written");
        }
    }

    scan_service::scan_roots(
        pool.clone(),
        vec![scan_root],
        home_dir.to_path_buf(),
        |_| Ok(()),
    )
    .await
    .expect("scan should parse active project");

    let connection = pool.get().await.expect("connection should be available");
    connection
        .interact(move |connection| {
            let project = project_repo::load_project_by_root(
                connection,
                project_root.to_string_lossy().as_ref(),
            )?
            .expect("project should be persisted");

            assert_eq!(project.current_phase_number.as_deref(), Some("38"));
            assert!((project.milestone_progress_pct - 98.0).abs() < f64::EPSILON);

            let panel = project_detail::load_project_phase_panel(connection, &project.id)?;
            assert_eq!(panel.completed_item_count, 1);
            assert_eq!(panel.total_item_count, 3);

            let milestones = project_detail::load_project_milestones(connection, &project.id)?;
            assert_eq!(milestones[0].phase_count, 4);
            assert_eq!(milestones[0].completed_phase_count, 2);
            let phase_38 = milestones[0]
                .phases
                .iter()
                .find(|phase| phase.number == "38")
                .expect("phase 38 should be included");
            assert_eq!(phase_38.completed_plan_count, 1);
            assert_eq!(phase_38.total_plan_count, 3);

            Ok::<_, AppError>(())
        })
        .await
        .expect("interaction should complete")
        .expect("project should load");
}

#[tokio::test]
async fn state_milestone_index_resolves_against_roadmap() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let home_dir = temp_dir.path();
    let scan_root = home_dir.join("workspace");
    let project_root = scan_root.join("multi-milestone-project");
    let planning_dir = project_root.join(".planning");
    let pool = migrated_pool(&temp_dir.path().join("cache.db")).await;

    write_valid_planning_project(&project_root, "Multi Milestone Project");
    fs::write(
        planning_dir.join("ROADMAP.md"),
        r#"# Roadmap

**Milestone:** v0.9 Setup
**Milestone:** v1.0 MVP

- [ ] **Phase 1: Foundation**
"#,
    )
    .expect("roadmap should be written");
    fs::write(
        planning_dir.join("STATE.md"),
        r#"## Current Position

**Milestone:** v1.0 MVP
**Phase:** 1 (Foundation)
"#,
    )
    .expect("state should be written");

    scan_service::scan_roots(
        pool.clone(),
        vec![scan_root],
        home_dir.to_path_buf(),
        |_| Ok(()),
    )
    .await
    .expect("scan should parse project");

    let connection = pool.get().await.expect("connection should be available");
    connection
        .interact(move |connection| {
            let project = project_repo::load_project_by_root(
                connection,
                project_root.to_string_lossy().as_ref(),
            )?
            .expect("project should be persisted");

            assert_eq!(project.current_milestone_name.as_deref(), Some("v1.0 MVP"));
            assert_eq!(project.current_milestone_index, Some(2));

            Ok::<_, AppError>(())
        })
        .await
        .expect("interaction should complete")
        .expect("project should load");
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
            global_sessions_default_range: "7d".to_string(),
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
