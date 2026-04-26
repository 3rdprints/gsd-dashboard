use std::{fs, path::Path};

use gsd_dashboard::sessions::{
    indexer::{stream_session_file, StreamFileStatus},
    matcher::match_project,
    IndexedSession, ProjectRoot, SessionIndexState, SessionSource,
};

fn fixture_path(name: &str) -> &'static str {
    match name {
        "claude-basic" => concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/fixtures/sessions/claude-basic.jsonl"
        ),
        "claude-partial" => concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/fixtures/sessions/claude-partial.jsonl"
        ),
        "codex-basic" => concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/fixtures/sessions/codex-basic.jsonl"
        ),
        _ => panic!("unknown fixture"),
    }
}

fn empty_session(source: SessionSource, source_path: &str) -> IndexedSession {
    IndexedSession {
        id: "test-session".to_string(),
        source,
        source_path: source_path.to_string(),
        source_session_id: None,
        project_id: None,
        cwd: None,
        started_at: None,
        ended_at: None,
        duration_ms: None,
        message_count: 0,
        tokens_in: None,
        tokens_out: None,
        model: None,
        attribution_method: "unmatched".to_string(),
        index_error: None,
    }
}

#[test]
fn claude_session_fixture_extracts_metadata() {
    let source_path = fixture_path("claude-basic");
    let (accumulator, status) =
        stream_session_file(SessionSource::Claude, Path::new(source_path), None)
            .expect("claude fixture should stream");

    assert!(matches!(status, StreamFileStatus::Complete { .. }));
    assert_eq!(accumulator.session.source, SessionSource::Claude);
    assert_eq!(
        accumulator.session.source_session_id.as_deref(),
        Some("claude-session-1")
    );
    assert_eq!(accumulator.session.started_at, Some(1_716_811_200_000));
    assert_eq!(accumulator.session.ended_at, Some(1_716_811_215_000));
    assert_eq!(accumulator.session.duration_ms, Some(15_000));
    assert_eq!(accumulator.session.message_count, 2);
    assert_eq!(accumulator.session.tokens_in, Some(120));
    assert_eq!(accumulator.session.tokens_out, Some(45));
    assert_eq!(
        accumulator.session.model.as_deref(),
        Some("claude-3-5-sonnet")
    );
    assert_eq!(
        accumulator.session.cwd.as_deref(),
        Some("/tmp/gsd-dashboard-fixture")
    );
}

#[test]
fn codex_session_fixture_extracts_best_effort_metadata() {
    let source_path = fixture_path("codex-basic");
    let (accumulator, status) =
        stream_session_file(SessionSource::Codex, Path::new(source_path), None)
            .expect("codex fixture should stream");

    assert!(matches!(status, StreamFileStatus::Complete { .. }));
    assert_eq!(accumulator.session.source, SessionSource::Codex);
    assert_eq!(
        accumulator.session.source_session_id.as_deref(),
        Some("codex-session-1")
    );
    assert_eq!(accumulator.session.started_at, Some(1_716_814_800_000));
    assert_eq!(accumulator.session.ended_at, Some(1_716_814_812_500));
    assert_eq!(accumulator.session.duration_ms, Some(12_500));
    assert_eq!(
        accumulator.session.cwd.as_deref(),
        Some("/tmp/gsd-dashboard-fixture")
    );
}

#[test]
fn partial_trailing_line_keeps_offset_before_partial() {
    let source_path = fixture_path("claude-partial");
    let bytes = fs::read(source_path).expect("fixture should be readable");
    let partial_line_start = bytes
        .iter()
        .rposition(|byte| *byte == b'\n')
        .map(|position| position + 1)
        .expect("fixture should contain a trailing partial line");

    let (accumulator, status) =
        stream_session_file(SessionSource::Claude, Path::new(source_path), None)
            .expect("partial fixture should stream");

    assert_eq!(
        accumulator.live_partial_message.as_deref(),
        Some("Live session still writing")
    );
    assert_eq!(
        status,
        StreamFileStatus::LivePartial {
            committed_offset: partial_line_start as i64,
            message: "Live session still writing".to_string()
        }
    );
}

#[test]
fn matcher_prefers_cwd_and_retains_unmatched() {
    let known_projects = vec![ProjectRoot {
        id: "project-1".to_string(),
        root_path: "/tmp/gsd-dashboard-fixture".to_string(),
    }];
    let mut cwd_session = empty_session(SessionSource::Codex, "/tmp/codex.jsonl");
    cwd_session.cwd = Some("/tmp/gsd-dashboard-fixture/subdir".to_string());

    match_project(&mut cwd_session, &known_projects);

    assert_eq!(cwd_session.project_id.as_deref(), Some("project-1"));
    assert_eq!(cwd_session.attribution_method, "cwd");

    let mut unmatched = empty_session(SessionSource::Codex, "/tmp/codex-unmatched.jsonl");
    unmatched.cwd = Some("/tmp/other-project".to_string());

    match_project(&mut unmatched, &known_projects);

    assert_eq!(unmatched.project_id, None);
    assert_eq!(unmatched.attribution_method, "unmatched");
}

#[test]
fn claude_path_fallback_decodes_directory_encoding_against_known_roots() {
    let known_projects = vec![ProjectRoot {
        id: "gsd-dashboard".to_string(),
        root_path: "/Users/smacdonald/homegit/gsd-dashboard".to_string(),
    }];
    let mut session = empty_session(
        SessionSource::Claude,
        "~/.claude/projects/-Users-smacdonald-homegit-gsd-dashboard/claude-session-1.jsonl",
    );

    match_project(&mut session, &known_projects);

    assert_eq!(session.project_id.as_deref(), Some("gsd-dashboard"));
    assert_eq!(session.attribution_method, "claude_path");
}

#[test]
fn incremental_state_starts_at_previous_committed_offset() {
    let source_path = fixture_path("claude-basic");
    let bytes = fs::read(source_path).expect("fixture should be readable");
    let first_line_end = bytes
        .iter()
        .position(|byte| *byte == b'\n')
        .map(|position| position + 1)
        .expect("fixture should contain multiple lines");
    let previous_state = SessionIndexState {
        source_path: source_path.to_string(),
        source: SessionSource::Claude,
        file_size: bytes.len() as i64,
        file_mtime: None,
        last_parsed_byte_offset: first_line_end as i64,
        live_partial: false,
        last_error: None,
    };

    let (accumulator, status) = stream_session_file(
        SessionSource::Claude,
        Path::new(source_path),
        Some(&previous_state),
    )
    .expect("fixture should stream from previous offset");

    assert_eq!(accumulator.session.message_count, 1);
    assert_eq!(accumulator.session.started_at, Some(1_716_811_215_000));
    assert_eq!(
        status,
        StreamFileStatus::Complete {
            committed_offset: bytes.len() as i64
        }
    );
}
