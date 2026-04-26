use std::{collections::BTreeMap, path::Path};

use gsd_dashboard::{
    sessions::{
        repo::{
            load_index_state, load_portfolio_session_summary, persist_indexed_file_result,
            rematch_unmatched_sessions_against_projects, save_index_state, upsert_indexed_session,
        },
        IndexedSession, ProjectRoot, SessionIndexState, SessionSource,
    },
    store::migrations,
};
use rusqlite::{Connection, OptionalExtension};

fn migrated_connection(db_path: &Path) -> Connection {
    let mut connection = Connection::open(db_path).expect("connection should open");
    connection
        .pragma_update(None, "foreign_keys", "ON")
        .expect("foreign keys should be enabled");
    migrations::run(&mut connection).expect("migrations should run");
    connection
}

fn sanitized_path(temp_dir: &tempfile::TempDir, path: &str) -> String {
    temp_dir.path().join(path).to_string_lossy().to_string()
}

fn indexed_session(
    temp_dir: &tempfile::TempDir,
    id: &str,
    project_id: Option<&str>,
    started_at: i64,
) -> IndexedSession {
    IndexedSession {
        id: id.to_string(),
        source: SessionSource::Claude,
        source_path: sanitized_path(temp_dir, "sessions/claude/project/session.jsonl"),
        source_session_id: Some(format!("{id}-source")),
        project_id: project_id.map(str::to_string),
        cwd: Some(sanitized_path(temp_dir, "workspace/project")),
        started_at: Some(started_at),
        ended_at: Some(started_at + 1_000),
        duration_ms: Some(1_000),
        message_count: 2,
        tokens_in: Some(10),
        tokens_out: Some(15),
        model: Some("test-model".to_string()),
        attribution_method: project_id.map_or("unmatched", |_| "cwd").to_string(),
        index_error: None,
    }
}

fn index_state(temp_dir: &tempfile::TempDir, offset: i64) -> SessionIndexState {
    SessionIndexState {
        source_path: sanitized_path(temp_dir, "sessions/claude/project/session.jsonl"),
        source: SessionSource::Claude,
        file_size: 256,
        file_mtime: Some(1_777_000_000),
        last_parsed_byte_offset: offset,
        live_partial: true,
        last_error: Some("live partial".to_string()),
    }
}

fn insert_project(connection: &mut Connection, id: &str, root_path: &str) {
    let snapshot = gsd_dashboard::store::project_repo::StoredProjectSnapshot {
        id: id.to_string(),
        name: id.to_string(),
        root_path: root_path.to_string(),
        planning_path: format!("{root_path}/.planning"),
        current_milestone_name: Some("v1.0".to_string()),
        current_milestone_index: Some(1),
        current_phase_number: Some("04".to_string()),
        current_phase_name: Some("Session Indexer".to_string()),
        milestone_progress_pct: 25.0,
        next_command: "/gsd-next".to_string(),
        parsed_blob: r#"{"source":"test"}"#.to_string(),
        parse_error: None,
        last_activity_at: Some(1_777_000_000),
        last_scanned_at: 1_777_000_000,
        created_at: 0,
        updated_at: 0,
    };

    gsd_dashboard::store::project_repo::upsert_project_snapshot(
        connection,
        snapshot,
        Vec::new(),
        1_777_000_000,
    )
    .expect("project should insert");
}

#[test]
fn session_tables_persist_metadata_without_raw_content() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let connection = migrated_connection(&temp_dir.path().join("cache.db"));

    let tables = ["sessions", "session_index_state"]
        .into_iter()
        .map(|table_name| {
            connection
                .query_row(
                    "SELECT name FROM sqlite_master WHERE type = 'table' AND name = ?1",
                    [table_name],
                    |row| row.get::<_, String>(0),
                )
                .optional()
                .expect("table lookup should run")
        })
        .collect::<Vec<_>>();

    assert_eq!(
        tables,
        vec![
            Some("sessions".to_string()),
            Some("session_index_state".to_string())
        ]
    );

    let forbidden_columns = [
        "content",
        "prompt",
        "transcript",
        "message_json",
        "tool_calls_json",
        "fts_rowid",
    ];
    for table_name in ["sessions", "session_index_state"] {
        let mut statement = connection
            .prepare(&format!("PRAGMA table_info({table_name})"))
            .expect("table info should prepare");
        let columns = statement
            .query_map([], |row| row.get::<_, String>(1))
            .expect("table info should query")
            .collect::<Result<Vec<_>, _>>()
            .expect("columns should collect");

        for forbidden_column in forbidden_columns {
            assert!(
                !columns.iter().any(|column| column == forbidden_column),
                "{table_name} should not contain {forbidden_column}"
            );
        }
    }
}

#[test]
fn session_upsert_replaces_metadata_and_preserves_unmatched() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let mut connection = migrated_connection(&temp_dir.path().join("cache.db"));
    let root_path = sanitized_path(&temp_dir, "workspace/project");
    insert_project(&mut connection, "project-1", &root_path);

    let mut session = indexed_session(&temp_dir, "session-1", None, 1_777_000_000);
    {
        let transaction = connection
            .transaction()
            .expect("transaction should start");
        upsert_indexed_session(&transaction, &session, 1_777_000_100)
            .expect("unmatched session should insert");
        transaction.commit().expect("transaction should commit");
    }

    let first_project_id: Option<String> = connection
        .query_row(
            "SELECT project_id FROM sessions WHERE id = 'session-1'",
            [],
            |row| row.get(0),
        )
        .expect("session should exist");
    assert_eq!(first_project_id, None);

    session.project_id = Some("project-1".to_string());
    session.message_count = 5;
    session.tokens_in = Some(25);
    session.tokens_out = Some(35);
    session.attribution_method = "cwd".to_string();
    {
        let transaction = connection
            .transaction()
            .expect("transaction should start");
        upsert_indexed_session(&transaction, &session, 1_777_000_200)
            .expect("matched session should update");
        transaction.commit().expect("transaction should commit");
    }

    let row = connection
        .query_row(
            "SELECT project_id, message_count, tokens_in, tokens_out, attribution_method
             FROM sessions WHERE id = 'session-1'",
            [],
            |row| {
                Ok((
                    row.get::<_, Option<String>>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Option<i64>>(2)?,
                    row.get::<_, Option<i64>>(3)?,
                    row.get::<_, String>(4)?,
                ))
            },
        )
        .expect("session should load");

    assert_eq!(
        row,
        (
            Some("project-1".to_string()),
            5,
            Some(25),
            Some(35),
            "cwd".to_string()
        )
    );
}

#[test]
fn session_index_state_round_trips_offsets() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let mut connection = migrated_connection(&temp_dir.path().join("cache.db"));
    let state = index_state(&temp_dir, 128);

    {
        let transaction = connection
            .transaction()
            .expect("transaction should start");
        save_index_state(&transaction, &state, 1_777_000_000).expect("state should save");
        transaction.commit().expect("transaction should commit");
    }

    let loaded = load_index_state(&mut connection, &state.source_path)
        .expect("state should load")
        .expect("state should exist");

    assert_eq!(loaded.last_parsed_byte_offset, 128);
    assert!(loaded.live_partial);
    assert_eq!(loaded.last_error.as_deref(), Some("live partial"));
}

#[test]
fn persist_indexed_file_result_rolls_back_offset_when_session_write_fails() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let mut connection = migrated_connection(&temp_dir.path().join("cache.db"));
    let good_session = indexed_session(&temp_dir, "good-session", None, 1_777_000_000);
    let bad_session = indexed_session(&temp_dir, "bad-session", Some("missing-project"), 1_777_000_100);
    let state = index_state(&temp_dir, 512);

    let result = persist_indexed_file_result(
        &mut connection,
        &[good_session, bad_session],
        &state,
        1_777_000_200,
    );

    assert!(result.is_err());
    let session_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get(0))
        .expect("session count should load");
    let saved_offset: Option<i64> = connection
        .query_row(
            "SELECT last_parsed_byte_offset FROM session_index_state WHERE source_path = ?1",
            [&state.source_path],
            |row| row.get(0),
        )
        .optional()
        .expect("offset lookup should run");

    assert_eq!(session_count, 0);
    assert_eq!(saved_offset, None);
}

#[test]
fn rematch_unmatched_sessions_against_projects_restores_project_id_after_rebuild() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let mut connection = migrated_connection(&temp_dir.path().join("cache.db"));
    let root_path = sanitized_path(&temp_dir, "workspace/project");
    let session = indexed_session(&temp_dir, "session-to-rematch", None, 1_777_000_000);
    let state = index_state(&temp_dir, 128);

    persist_indexed_file_result(&mut connection, &[session], &state, 1_777_000_000)
        .expect("unmatched session and offset should persist");
    insert_project(&mut connection, "project-1", &root_path);

    let rematched = rematch_unmatched_sessions_against_projects(
        &mut connection,
        &[ProjectRoot {
            id: "project-1".to_string(),
            root_path: root_path.clone(),
        }],
        1_777_000_100,
    )
    .expect("rematch should run");

    assert_eq!(rematched, 1);
    let row = connection
        .query_row(
            "SELECT project_id, attribution_method FROM sessions WHERE id = 'session-to-rematch'",
            [],
            |row| Ok((row.get::<_, Option<String>>(0)?, row.get::<_, String>(1)?)),
        )
        .expect("rematched session should load");
    let loaded_state = load_index_state(&mut connection, &state.source_path)
        .expect("state should load")
        .expect("state should exist");

    assert_eq!(row, (Some("project-1".to_string()), "cwd".to_string()));
    assert_eq!(loaded_state.last_parsed_byte_offset, 128);
}

#[test]
fn portfolio_session_summary_counts_today_and_sparkline() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let mut connection = migrated_connection(&temp_dir.path().join("cache.db"));
    let project_root = sanitized_path(&temp_dir, "workspace/project");
    insert_project(&mut connection, "project-1", &project_root);

    let day_ms = 86_400_000;
    let seven_days_start_ms = 1_776_441_600_000;
    let today_start_ms = seven_days_start_ms + (6 * day_ms);
    let mut sessions = Vec::new();
    for bucket in 0..7 {
        let mut session = indexed_session(
            &temp_dir,
            &format!("project-session-{bucket}"),
            Some("project-1"),
            seven_days_start_ms + (bucket * day_ms) + 1_000,
        );
        session.tokens_in = Some(bucket + 1);
        session.tokens_out = Some((bucket + 1) * 2);
        sessions.push(session);
    }
    sessions.push(IndexedSession {
        id: "unmatched-codex".to_string(),
        source: SessionSource::Codex,
        source_path: sanitized_path(&temp_dir, "sessions/codex/2026/04/26/session.jsonl"),
        source_session_id: Some("codex-source".to_string()),
        project_id: None,
        cwd: None,
        started_at: Some(today_start_ms + 2_000),
        ended_at: Some(today_start_ms + 3_000),
        duration_ms: Some(1_000),
        message_count: 1,
        tokens_in: Some(100),
        tokens_out: Some(50),
        model: Some("codex-test".to_string()),
        attribution_method: "unmatched".to_string(),
        index_error: Some("no cwd".to_string()),
    });

    let state = index_state(&temp_dir, 1024);
    persist_indexed_file_result(&mut connection, &sessions, &state, 1_777_000_000)
        .expect("sessions should persist");

    let summary = load_portfolio_session_summary(
        &mut connection,
        &["project-1".to_string()],
        today_start_ms,
        seven_days_start_ms,
    )
    .expect("summary should load");

    let mut expected_sparkline = BTreeMap::new();
    expected_sparkline.insert("project-1".to_string(), [1, 1, 1, 1, 1, 1, 1]);

    assert_eq!(summary.sessions_today, 2);
    assert_eq!(summary.tokens_today, 168);
    assert_eq!(summary.sparkline_by_project, expected_sparkline);
    assert_eq!(summary.unmatched_count, 1);
    assert_eq!(summary.unmatched_claude_count, 0);
    assert_eq!(summary.unmatched_codex_count, 1);
    assert_eq!(summary.recent_unmatched.len(), 1);
    assert_eq!(summary.recent_unmatched[0].id, "unmatched-codex");
}
