use std::{fs, path::Path};

use gsd_dashboard::{
    bootstrap, events::AppEvent, scan_refresh, scanner::PlanningProjectCandidate,
    sessions::{repo::load_index_state, SessionSource},
    store::project_repo, watcher::refresh::refresh_project_planning_dir_for_app,
};

fn write_planning_project(project_root: &Path, project_name: &str, phase: u8, phase_name: &str) {
    let planning_dir = project_root.join(".planning");
    fs::create_dir_all(&planning_dir).expect("planning dir should be created");
    fs::write(
        planning_dir.join("ROADMAP.md"),
        format!("# Roadmap\n\n**Milestone:** v1.0 MVP\n\n- [ ] **Phase {phase}: {phase_name}**\n"),
    )
    .expect("roadmap should be written");
    fs::write(
        planning_dir.join("STATE.md"),
        format!(
            "---\nmilestone: v1.0\nmilestone_name: v1.0 MVP\n---\n\n# State: {project_name}\n\n## Current Position\n\n**Phase:** {phase} ({phase_name})\n"
        ),
    )
    .expect("state should be written");
    fs::write(planning_dir.join("config.json"), "{\"workflow\":{}}")
        .expect("config should be written");
}

async fn project_phase_name(
    state: &gsd_dashboard::app_state::AppState,
    project_id: &str,
) -> String {
    let project_id = project_id.to_string();
    let connection = state.pool.get().await.expect("connection should open");
    connection
        .interact(move |connection| {
            let projects =
                project_repo::list_project_snapshots(connection).expect("projects should load");
            projects
                .into_iter()
                .find(|project| project.id == project_id)
                .and_then(|project| project.current_phase_name)
                .ok_or_else(|| rusqlite::Error::QueryReturnedNoRows)
        })
        .await
        .expect("interaction should complete")
        .expect("project should exist")
}

#[tokio::test]
async fn live_updates_targeted_refresh_reparses_only_affected_project() {
    // LIVE-02, T-07-02: project refresh must reparse only the affected `.planning`
    // source and update derived cache state without touching unrelated projects.
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let home_dir = temp_dir.path().join("home");
    let app_data_dir = temp_dir.path().join("app-data");
    let first_root = home_dir.join("workspace/project-one");
    let second_root = home_dir.join("workspace/project-two");
    write_planning_project(&first_root, "Project One", 3, "Portfolio");
    write_planning_project(&second_root, "Project Two", 4, "Sessions");

    let state = bootstrap::bootstrap_from_paths(app_data_dir, home_dir)
        .await
        .expect("bootstrap should succeed");
    let first_outcome = scan_refresh::scan_single_project_candidate(
        &state.pool,
        PlanningProjectCandidate {
            project_root: first_root.clone(),
            planning_path: first_root.join(".planning"),
        },
    )
    .await
    .expect("first project should scan");
    let second_outcome = scan_refresh::scan_single_project_candidate(
        &state.pool,
        PlanningProjectCandidate {
            project_root: second_root.clone(),
            planning_path: second_root.join(".planning"),
        },
    )
    .await
    .expect("second project should scan");

    write_planning_project(&first_root, "Project One", 7, "Live Updates");
    let events = std::sync::Mutex::new(Vec::new());
    let refreshed =
        refresh_project_planning_dir_for_app(&state, &first_root.join(".planning"), |event| {
            events.lock().expect("events lock should work").push(event);
            Ok(())
        })
        .await
        .expect("project should refresh");

    assert_eq!(refreshed.project_id, first_outcome.project_id);
    assert_eq!(
        events.lock().expect("events lock should work").as_slice(),
        &[AppEvent::ProjectUpdated {
            id: first_outcome.project_id.clone()
        }]
    );
    assert_eq!(state.tray_refresh_request_count(), 1);
    assert_eq!(
        project_phase_name(&state, &first_outcome.project_id).await,
        "Live Updates"
    );
    assert_eq!(
        project_phase_name(&state, &second_outcome.project_id).await,
        "Sessions"
    );
}

#[tokio::test]
async fn live_updates_targeted_refresh_reuses_session_byte_offsets() {
    // LIVE-02, T-07-04: project refresh emits only tiny project invalidation
    // payloads; session byte-offset refresh remains a separate source-root path.
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let home_dir = temp_dir.path().join("home");
    let app_data_dir = temp_dir.path().join("app-data");
    let project_root = home_dir.join("workspace/project-one");
    write_planning_project(&project_root, "Project One", 7, "Live Updates");
    let state = bootstrap::bootstrap_from_paths(app_data_dir, home_dir)
        .await
        .expect("bootstrap should succeed");

    let events = std::sync::Mutex::new(Vec::new());
    refresh_project_planning_dir_for_app(&state, &project_root.join(".planning"), |event| {
        events.lock().expect("events lock should work").push(event);
        Ok(())
    })
    .await
    .expect("project should refresh");

    let serialized = serde_json::to_string(&events.lock().expect("events lock should work")[0])
        .expect("event should serialize");
    assert!(serialized.contains("project:updated"));
    assert!(!serialized.contains("source_path"));
    assert!(!serialized.contains("transcript"));
    assert!(!serialized.contains("tool"));
}

#[tokio::test]
async fn live_updates_targeted_session_refresh_reuses_offsets_and_emits_tiny_events() {
    // LIVE-02, T-07-03, T-07-04: session-root refresh parses only appended
    // bytes, persists derived metadata first, and emits tiny invalidations.
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let home_dir = temp_dir.path().join("home");
    let app_data_dir = temp_dir.path().join("app-data");
    let project_root = home_dir.join("workspace/project-one");
    write_planning_project(&project_root, "Project One", 7, "Live Updates");
    let state = bootstrap::bootstrap_from_paths(app_data_dir, home_dir.clone())
        .await
        .expect("bootstrap should succeed");
    scan_refresh::scan_single_project_candidate(
        &state.pool,
        PlanningProjectCandidate {
            project_root: project_root.clone(),
            planning_path: project_root.join(".planning"),
        },
    )
    .await
    .expect("known project should scan");

    let session_dir = home_dir.join(".claude/projects/-workspace-project-one");
    fs::create_dir_all(&session_dir).expect("session dir should be created");
    let session_path = session_dir.join("claude-live.jsonl");
    fs::write(
        &session_path,
        "{\"type\":\"user\",\"timestamp\":\"2024-05-27T12:00:00Z\",\"cwd\":\"",
    )
    .expect("partial session should be written");

    let events = std::sync::Mutex::new(Vec::new());
    gsd_dashboard::watcher::refresh::refresh_session_file(
        &state,
        SessionSource::Claude,
        &session_path,
        |event| {
            events.lock().expect("events lock should work").push(event);
            Ok(())
        },
    )
    .await
    .expect("partial refresh should succeed");
    assert!(events.lock().expect("events lock should work").is_empty());

    let completed_line = format!(
        "{{\"type\":\"user\",\"timestamp\":\"2024-05-27T12:00:00Z\",\"cwd\":\"{}\",\"sessionId\":\"claude-live\"}}\n",
        project_root.display()
    );
    fs::write(&session_path, completed_line).expect("completed session should be written");
    gsd_dashboard::watcher::refresh::refresh_session_file(
        &state,
        SessionSource::Claude,
        &session_path,
        |event| {
            events.lock().expect("events lock should work").push(event);
            Ok(())
        },
    )
    .await
    .expect("completed refresh should succeed");

    let source_path = session_path.display().to_string();
    let offset = state
        .pool
        .get()
        .await
        .expect("connection should be available")
        .interact(move |connection| {
            load_index_state(connection, &source_path)
                .map(|state| state.expect("session index state should exist"))
        })
        .await
        .expect("interaction should complete")
        .expect("state should load")
        .last_parsed_byte_offset;
    let events = events.lock().expect("events lock should work").clone();
    let session_events = events
        .iter()
        .filter(|event| matches!(event, AppEvent::SessionNew { .. }))
        .count();

    assert_eq!(
        offset,
        fs::metadata(&session_path)
            .expect("session metadata should load")
            .len() as i64
    );
    assert_eq!(session_events, 1);
    assert!(events.contains(&AppEvent::DailyActivityUpdated));
    for event in events {
        let serialized = serde_json::to_string(&event).expect("event should serialize");
        assert!(!serialized.contains("source_path"));
        assert!(!serialized.contains("transcript"));
        assert!(!serialized.contains("prompt"));
        assert!(!serialized.contains("tool_output"));
    }
}

#[tokio::test]
async fn live_updates_targeted_refresh_does_not_write_to_planning_sources() {
    // LIVE-02, T-07-02: `.planning` files are read-only inputs; targeted refresh
    // may change only derived SQLite/cache state and app invalidation events.
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let home_dir = temp_dir.path().join("home");
    let app_data_dir = temp_dir.path().join("app-data");
    let project_root = home_dir.join("workspace/project-one");
    write_planning_project(&project_root, "Project One", 7, "Live Updates");
    let planning_dir = project_root.join(".planning");
    let state_before =
        fs::read_to_string(planning_dir.join("STATE.md")).expect("state source should be readable");
    let roadmap_before = fs::read_to_string(planning_dir.join("ROADMAP.md"))
        .expect("roadmap source should be readable");

    let state = bootstrap::bootstrap_from_paths(app_data_dir, home_dir)
        .await
        .expect("bootstrap should succeed");
    refresh_project_planning_dir_for_app(&state, &planning_dir, |_| Ok(()))
        .await
        .expect("project should refresh");

    assert_eq!(
        fs::read_to_string(planning_dir.join("STATE.md")).expect("state source should be readable"),
        state_before
    );
    assert_eq!(
        fs::read_to_string(planning_dir.join("ROADMAP.md"))
            .expect("roadmap source should be readable"),
        roadmap_before
    );
}
