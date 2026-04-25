use std::{fs, path::Path};

use gsd_dashboard::{
    error::AppError,
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

            assert_eq!(bad.id, "bad-project");
            assert!(bad.parse_error.is_some());
            assert_eq!(good.id, "good-project");
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
                "SELECT status FROM scan_log WHERE project_id = ?1",
                ["bad-project"],
                |row| row.get::<_, String>(0),
            )?;

            assert_eq!(status, "parseError");

            Ok::<_, AppError>(())
        })
        .await
        .expect("interaction should complete")
        .expect("scan log should be readable");
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
