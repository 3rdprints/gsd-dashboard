use gsd_dashboard::{
    bootstrap,
    commands::projects::{get_portfolio_for_app, get_project_for_app},
    settings::{self, SettingsInput, TrayBarSort},
    store::project_repo::{self, StoredProjectSnapshot},
};

fn project_snapshot(
    id: &str,
    name: &str,
    root_path: &str,
    last_activity_at: Option<i64>,
    last_scanned_at: i64,
) -> StoredProjectSnapshot {
    StoredProjectSnapshot {
        id: id.to_string(),
        name: name.to_string(),
        root_path: root_path.to_string(),
        planning_path: format!("{root_path}/.planning"),
        current_milestone_name: Some("v1.0".to_string()),
        current_milestone_index: Some(1),
        current_phase_number: Some("03".to_string()),
        current_phase_name: Some("Portfolio".to_string()),
        milestone_progress_pct: 42.0,
        next_command: "/gsd-next".to_string(),
        parsed_blob: r#"{"source":"test"}"#.to_string(),
        parse_error: None,
        last_activity_at,
        last_scanned_at,
        created_at: 0,
        updated_at: 0,
    }
}

async fn test_state() -> (tempfile::TempDir, gsd_dashboard::app_state::AppState) {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let state = bootstrap::bootstrap_from_paths(
        temp_dir.path().join("app-data"),
        temp_dir.path().join("home"),
    )
    .await
    .expect("bootstrap should succeed");

    (temp_dir, state)
}

async fn save_hidden_projects(state: &gsd_dashboard::app_state::AppState, hidden_ids: Vec<&str>) {
    settings::save(
        &state.pool,
        &state.home_dir,
        SettingsInput {
            scan_roots: vec!["~/Documents".to_string()],
            hidden_project_ids: hidden_ids.into_iter().map(str::to_string).collect(),
            autostart_enabled: false,
            tray_bar_max_projects: 8,
            tray_bar_sort: TrayBarSort::RecentActivity,
        },
    )
    .await
    .expect("settings should save");
}

async fn insert_projects(
    state: &gsd_dashboard::app_state::AppState,
    projects: Vec<StoredProjectSnapshot>,
) {
    let connection = state
        .pool
        .get()
        .await
        .expect("connection should be available");
    connection
        .interact(move |connection| {
            for project in projects {
                project_repo::upsert_project_snapshot(
                    connection,
                    project,
                    Vec::new(),
                    1_777_100_000,
                )?;
            }

            Ok::<_, gsd_dashboard::error::AppError>(())
        })
        .await
        .expect("interaction should complete")
        .expect("projects should insert");
}

#[tokio::test]
async fn portfolio_filters_hidden_projects() {
    let (_temp_dir, state) = test_state().await;
    insert_projects(
        &state,
        vec![
            project_snapshot(
                "visible-project",
                "Visible Project",
                "/tmp/visible-project",
                Some(1_777_000_200),
                1_777_000_100,
            ),
            project_snapshot(
                "hidden-project",
                "Hidden Project",
                "/tmp/hidden-project",
                Some(1_777_000_300),
                1_777_000_100,
            ),
        ],
    )
    .await;
    save_hidden_projects(&state, vec!["hidden-project"]).await;

    let portfolio = get_portfolio_for_app(&state)
        .await
        .expect("portfolio should load");

    assert_eq!(portfolio.projects.len(), 1);
    assert_eq!(portfolio.projects[0].id, "visible-project");
    assert_eq!(portfolio.hidden_projects.len(), 1);
    assert_eq!(portfolio.hidden_projects[0].id, "hidden-project");
}

#[tokio::test]
async fn portfolio_sorts_by_activity_descending() {
    let (_temp_dir, state) = test_state().await;
    insert_projects(
        &state,
        vec![
            project_snapshot(
                "fallback-newest",
                "Fallback Newest",
                "/tmp/fallback",
                None,
                300,
            ),
            project_snapshot(
                "activity-newest",
                "Activity Newest",
                "/tmp/activity",
                Some(500),
                100,
            ),
            project_snapshot("oldest", "Oldest", "/tmp/oldest", Some(200), 900),
        ],
    )
    .await;

    let portfolio = get_portfolio_for_app(&state)
        .await
        .expect("portfolio should load");
    let ids = portfolio
        .projects
        .iter()
        .map(|project| project.id.as_str())
        .collect::<Vec<_>>();

    assert_eq!(ids, vec!["activity-newest", "fallback-newest", "oldest"]);
}

#[tokio::test]
async fn get_project_returns_detail_with_phase_and_next_command() {
    let (_temp_dir, state) = test_state().await;
    let mut project = project_snapshot(
        "detail-project",
        "Detail Project",
        "/tmp/detail-project",
        Some(1_777_000_300),
        1_777_000_400,
    );
    project.current_phase_number = Some("03".to_string());
    project.current_phase_name = Some("Portfolio Vertical Slice".to_string());
    project.milestone_progress_pct = 75.0;
    project.next_command = "/gsd-execute-phase 3".to_string();
    project.parse_error = Some("STATE.md missing phase".to_string());
    insert_projects(&state, vec![project]).await;

    let detail = get_project_for_app(&state, "detail-project")
        .await
        .expect("project detail should load");

    assert_eq!(detail.id, "detail-project");
    assert_eq!(detail.name, "Detail Project");
    assert_eq!(detail.root_path, "/tmp/detail-project");
    assert_eq!(detail.current_phase_number.as_deref(), Some("03"));
    assert_eq!(
        detail.current_phase_name.as_deref(),
        Some("Portfolio Vertical Slice")
    );
    assert_eq!(detail.milestone_progress_pct, 75.0);
    assert_eq!(
        detail.parse_error.as_deref(),
        Some("STATE.md missing phase")
    );
    assert_eq!(detail.next_command, "/gsd-execute-phase 3");
}

#[tokio::test]
async fn portfolio_stats_count_visible_projects_and_active_milestones() {
    let (_temp_dir, state) = test_state().await;
    let mut without_milestone = project_snapshot(
        "without-milestone",
        "Without Milestone",
        "/tmp/no-milestone",
        None,
        200,
    );
    without_milestone.current_milestone_name = None;
    insert_projects(
        &state,
        vec![
            project_snapshot("active-project", "Active Project", "/tmp/active", None, 300),
            without_milestone,
            project_snapshot(
                "hidden-active",
                "Hidden Active",
                "/tmp/hidden-active",
                None,
                400,
            ),
        ],
    )
    .await;
    save_hidden_projects(&state, vec!["hidden-active"]).await;

    let portfolio = get_portfolio_for_app(&state)
        .await
        .expect("portfolio should load");

    assert_eq!(portfolio.stats.projects_tracked, 2);
    assert_eq!(portfolio.stats.active_milestones, 1);
    assert_eq!(portfolio.stats.sessions_today, 0);
    assert_eq!(portfolio.stats.tokens_today, 0);
}
