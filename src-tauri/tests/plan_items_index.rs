use std::{fs, path::Path, sync::Mutex};

use gsd_dashboard::{
    events::ScanEvent,
    parser::plan::parse_plan_items_with_lines,
    scan_service,
    store::{self, project_repo},
};

async fn migrated_pool(db_path: &Path) -> deadpool_sqlite::Pool {
    let pool = store::open_pool(db_path).await.expect("pool should open");
    store::run_migrations(&pool)
        .await
        .expect("migrations should run");
    pool
}

fn write_planning_project(project_root: &Path) {
    let planning_dir = project_root.join(".planning");
    let phase_dir = planning_dir.join("phases/05-project-detail");
    fs::create_dir_all(&phase_dir).expect("phase dir should be created");
    fs::write(
        planning_dir.join("ROADMAP.md"),
        r#"# Roadmap

**Milestone:** v1.0 MVP

- [ ] **Phase 5: Project Detail**
"#,
    )
    .expect("roadmap should be written");
    fs::write(
        planning_dir.join("STATE.md"),
        r#"---
milestone: v1.0
milestone_name: v1.0 MVP
---

## Current Position

**Phase:** 5 (Project Detail)
"#,
    )
    .expect("state should be written");
    fs::write(
        phase_dir.join("05-04-PLAN.md"),
        r#"---
phase: 05-project-detail
plan: 04
type: execute
---

- [ ] Parse checklist rows
- [x] Persist completed rows
- [X] Preserve uppercase markers
"#,
    )
    .expect("plan should be written");
}

#[tokio::test]
async fn plan_items_insert_replace_and_scan_integration() {
    let parsed = parse_plan_items_with_lines(
        b"heading\n- [ ] First item\ntext\n- [x] Second item\n- [X] Third item\n",
    )
    .expect("plan items should parse");

    assert_eq!(parsed.len(), 3);
    assert_eq!(parsed[0].ord, 0);
    assert_eq!(parsed[0].text, "First item");
    assert!(!parsed[0].checked);
    assert_eq!(parsed[0].line_no, 2);
    assert!(parsed[1].checked);
    assert_eq!(parsed[2].line_no, 5);

    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let home_dir = temp_dir.path().join("home");
    let scan_root = home_dir.join("workspace");
    let project_root = scan_root.join("demo-project");
    fs::create_dir_all(&project_root).expect("project root should be created");
    write_planning_project(&project_root);

    let pool = migrated_pool(&temp_dir.path().join("cache.db")).await;
    let events = Mutex::new(Vec::<ScanEvent>::new());
    scan_service::scan_roots(
        pool.clone(),
        vec![scan_root.clone()],
        home_dir,
        move |event| {
            events
                .lock()
                .expect("events lock should be available")
                .push(event);
            Ok(())
        },
    )
    .await
    .expect("scan should succeed");

    let connection = pool.get().await.expect("connection should be available");
    connection
        .interact(|connection| {
            let project = project_repo::list_project_snapshots(connection)?
                .into_iter()
                .next()
                .expect("project should be persisted");
            let plans = project_repo::load_phase_plans(connection, &project.id)?;
            let plan_path = plans
                .first()
                .expect("phase plan should be persisted")
                .plan_path
                .clone();
            let items = project_repo::load_plan_items(connection, &project.id, &plan_path)?;

            assert_eq!(items.len(), 3);
            assert_eq!(items[0].text, "Parse checklist rows");
            assert_eq!(items[0].line_no, 7);
            assert!(!items[0].checked);
            assert!(items[1].checked);
            assert!(items[2].checked);

            project_repo::set_plan_completed_at_if_all_checked(
                connection,
                &project.id,
                &plan_path,
                1_777_000_000,
            )?;
            let completed_at: Option<i64> = connection.query_row(
                "SELECT completed_at FROM phase_plans WHERE project_id = ?1 AND plan_path = ?2",
                [&project.id, &plan_path],
                |row| row.get(0),
            )?;
            assert_eq!(completed_at, None);

            let checked_items = items
                .into_iter()
                .map(|mut item| {
                    item.checked = true;
                    item
                })
                .collect::<Vec<_>>();
            project_repo::replace_plan_items(connection, &project.id, &plan_path, checked_items)?;
            project_repo::set_plan_completed_at_if_all_checked(
                connection,
                &project.id,
                &plan_path,
                1_777_000_001,
            )?;
            let completed_at: Option<i64> = connection.query_row(
                "SELECT completed_at FROM phase_plans WHERE project_id = ?1 AND plan_path = ?2",
                [&project.id, &plan_path],
                |row| row.get(0),
            )?;
            assert_eq!(completed_at, Some(1_777_000_001));

            Ok::<_, gsd_dashboard::error::AppError>(())
        })
        .await
        .expect("interaction should complete")
        .expect("plan item assertions should pass");
}
