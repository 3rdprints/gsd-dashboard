---
phase: 07-live-updates
plan: 04
subsystem: backend-live-updates
tags: [rust, tokio, sqlite, sessions, watcher, tdd]
requires:
  - phase: 07-live-updates
    provides: watcher contracts, targeted project refresh, and session indexer offset behavior
provides:
  - focused single-file session indexing module
  - bounded parallel session root indexing with worker limit 2
  - targeted session-file watcher refresh with tiny invalidation events
affects: [live-updates, session-indexer, watcher, daily-activity]
tech-stack:
  added: []
  patterns: [file-level JoinSet concurrency, single-file refresh entry point, tiny event invalidation]
key-files:
  created:
    - src-tauri/src/sessions/file_indexer.rs
    - src-tauri/src/sessions/parallel.rs
  modified:
    - src-tauri/src/sessions/indexer.rs
    - src-tauri/src/sessions/mod.rs
    - src-tauri/src/watcher/refresh.rs
    - src-tauri/src/watcher/service.rs
    - src-tauri/tests/session_indexer.rs
    - src-tauri/tests/live_updates_targeted_refresh.rs
key-decisions:
  - "Single-file JSONL streaming and persistence now lives in sessions::file_indexer so sessions/indexer.rs stays below the 500-line project limit."
  - "Bounded session root indexing uses Tokio JoinSet with SESSION_INDEX_WORKER_LIMIT = 2 and emits file progress in completion order."
  - "Watcher session refresh emits only session:new { id, projectId? } plus daily_activity_updated after derived SQLite persistence."
patterns-established:
  - "Session refresh reuses last_parsed_byte_offset and the existing per-file transaction path before emitting app invalidation events."
  - "Session file events coalesce by concrete .jsonl path through SessionFileDebouncer before calling refresh_session_file_for_app."
requirements-completed: [LIVE-01, LIVE-02, LIVE-05]
duration: 7min
completed: 2026-05-01
---

# Phase 07 Plan 04: Bounded Live Session Indexing Summary

**Focused session file indexing with bounded two-worker catch-up and targeted live session refresh using byte offsets and tiny events**

## Performance

- **Duration:** 7 min
- **Started:** 2026-05-01T19:20:33Z
- **Completed:** 2026-05-01T19:28:01Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments

- Split single-file JSONL streaming, offset handling, matching, and persistence into `sessions::file_indexer`.
- Reduced `src-tauri/src/sessions/indexer.rs` from 627 lines to 252 lines.
- Added `sessions::parallel` with `SESSION_INDEX_WORKER_LIMIT = 2` and completion-order per-file outcomes.
- Added targeted `refresh_session_file` for live appends, emitting `session:new` and `daily_activity_updated` only after SQLite persistence.
- Added `SessionFileDebouncer` and `refresh_session_file_for_app` in watcher service for source-root/session-file coalescing.

## Task Commits

1. **Task 1 RED: Add failing file indexer split gate** - `a98f6c8` (test)
2. **Task 1 GREEN: Split single-file indexing out of oversized indexer** - `1115fdc` (feat)
3. **Task 2 RED: Add failing session refresh gates** - `af8df0e` (test)
4. **Task 2 GREEN: Add bounded parallel root indexing and targeted session refresh** - `56e2553` (feat)

**Plan metadata:** pending final docs commit

## Files Created/Modified

- `src-tauri/src/sessions/file_indexer.rs` - Single-file stream, incremental offset, matching, and transactional persistence helper.
- `src-tauri/src/sessions/parallel.rs` - Worker-limited file indexing using Tokio `JoinSet`.
- `src-tauri/src/sessions/indexer.rs` - Root-level orchestration now delegates file work to focused modules.
- `src-tauri/src/watcher/refresh.rs` - Targeted session-file refresh, daily activity rebuild, and tiny app invalidations.
- `src-tauri/src/watcher/service.rs` - Session-file debounce helper and refresh handoff.
- `src-tauri/tests/session_indexer.rs` - Worker-limit gate and existing active session indexer coverage.
- `src-tauri/tests/live_updates_targeted_refresh.rs` - Active session refresh offset and event payload coverage.

## Decisions Made

- Used Context7 for Tokio `JoinSet` API docs before adding bounded parallel indexing.
- Kept worker concurrency at the planned limit of 2, below the SQLite pool size and aligned with T-07-01.
- Treated persisted session changes as the material update signal for `session:new`; unchanged offsets produce no event.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- The plan verification command uses Cargo test-name filters, so it does not execute every integration test in the target files. Ran the exact integration test targets as an additional verification step.

## Known Stubs

None.

## Threat Flags

None. The new watcher/session surface stays local-file input to derived SQLite metadata and tiny invalidation events.

## Verification

- `cargo test --manifest-path src-tauri/Cargo.toml session_indexer -- --nocapture` passed.
- `cargo test --manifest-path src-tauri/Cargo.toml live_updates_targeted_refresh -- --nocapture` passed.
- `cargo test --manifest-path src-tauri/Cargo.toml --test session_indexer -- --nocapture` passed: 24 tests.
- `cargo test --manifest-path src-tauri/Cargo.toml --test live_updates_targeted_refresh -- --nocapture` passed: 4 tests.
- Acceptance grep gates passed for `last_parsed_byte_offset`, `SESSION_INDEX_WORKER_LIMIT`, `refresh_session_file`, `session:new`, `daily_activity_updated`, no `CODEX_HOME`/`archived_sessions`, and no raw transcript/prompt/tool-output/raw-content strings in the new session surfaces.

## TDD Gate Compliance

- RED commit present before GREEN for Task 1: `a98f6c8` -> `1115fdc`.
- RED commit present before GREEN for Task 2: `af8df0e` -> `56e2553`.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 07-05 can build on focused session modules, bounded catch-up indexing, targeted live session refresh, and tiny event invalidation semantics.

## Self-Check: PASSED

- Found summary file and key created/modified files.
- Found task commits `a98f6c8`, `1115fdc`, `af8df0e`, and `56e2553` in git history.

---
*Phase: 07-live-updates*
*Completed: 2026-05-01*
