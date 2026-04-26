---
phase: 04-session-indexer
plan: 01
subsystem: database
tags: [rust, sqlite, rusqlite, sessions, metadata]

requires:
  - phase: 03-portfolio-vertical-slice
    provides: project cache, portfolio DTO paths, rebuild-cache behavior
provides:
  - Metadata-only sessions and session_index_state SQLite schema
  - Synchronous rusqlite session repository helpers
  - Transactional per-file session persistence with byte-offset rollback coverage
  - Portfolio session aggregate helper with seven-day project buckets
affects: [phase-04-session-indexer, portfolio, project-detail, global-sessions]

tech-stack:
  added: []
  patterns: [synchronous rusqlite repository helpers, metadata-only session storage, transaction-scoped offset advancement]

key-files:
  created:
    - src-tauri/src/sessions/mod.rs
    - src-tauri/src/sessions/repo.rs
    - src-tauri/tests/session_repo.rs
  modified:
    - src-tauri/src/store/migrations.rs
    - src-tauri/src/lib.rs
    - src-tauri/tests/store_migrations.rs

key-decisions:
  - "Session rows store only metadata columns; prompt, transcript, content, tool-call JSON, and FTS columns remain absent."
  - "Per-file indexing state advances in the same transaction as session upserts."
  - "Unmatched sessions remain first-class rows with nullable project_id and can be rematched after project cache rebuilds."

patterns-established:
  - "SessionSource converts to SQLite text values through as_str and TryFrom<&str>."
  - "persist_indexed_file_result is the only per-file write path for sessions plus byte offsets."
  - "Portfolio aggregates return BTreeMap<String, [i64; 7]> for deterministic per-project sparklines."

requirements-completed: [SESS-06]

duration: 7min
completed: 2026-04-26
---

# Phase 04 Plan 01: Session Repository Foundation Summary

**Metadata-only Claude/Codex session persistence with transactional offset state and portfolio aggregate helpers**

## Performance

- **Duration:** 7 min
- **Started:** 2026-04-26T11:54:16Z
- **Completed:** 2026-04-26T12:01:34Z
- **Tasks:** 3
- **Files modified:** 6

## Accomplishments

- Added migration 3 with `sessions` and `session_index_state` tables plus portfolio/detail/global supporting indexes.
- Added session metadata contracts and synchronous repository helpers for upsert, index-state load/save, atomic per-file persistence, rematching, and portfolio summaries.
- Added integration coverage for metadata-only schema, unmatched preservation, offset round trips, rollback on failed writes, rematching, and seven-day aggregates.

## Task Commits

Each task was committed atomically:

1. **Task 0: Add session repository tests and sanitized fixtures** - `9e56725` (test)
2. **Task 1: Add session tables and indexes** - `621c6b9` (feat)
3. **Task 2: Implement session repository helpers** - `6d62348` (feat)

**Plan metadata:** pending final docs commit

## Files Created/Modified

- `src-tauri/src/store/migrations.rs` - Added metadata-only session schema and query indexes.
- `src-tauri/src/sessions/mod.rs` - Added shared session metadata contracts.
- `src-tauri/src/sessions/repo.rs` - Added rusqlite repository helpers and aggregate DTO structs.
- `src-tauri/src/lib.rs` - Exported the sessions module.
- `src-tauri/tests/session_repo.rs` - Added integration tests for SESS-06 behavior.
- `src-tauri/tests/store_migrations.rs` - Updated expected migration version to 3.

## Decisions Made

- Used nullable `sessions.project_id` with `ON DELETE SET NULL` so derived project cache rebuilds cannot delete session history.
- Kept offset advancement inside `persist_indexed_file_result` transaction scope to prevent skipped bytes when any session write fails.
- Returned deterministic seven-day sparkline buckets by visible project ID for later portfolio UI wiring.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Updated migration version assertion**
- **Found during:** Task 1 (Add session tables and indexes)
- **Issue:** Existing `store_migrations` test expected migration version 2, which became stale after adding migration 3.
- **Fix:** Updated the expected version to 3.
- **Files modified:** `src-tauri/tests/store_migrations.rs`
- **Verification:** `cargo test --manifest-path src-tauri/Cargo.toml --test store_migrations -- --nocapture`
- **Committed in:** `621c6b9`

**2. [Rule 1 - Bug] Corrected portfolio token fixture expectation**
- **Found during:** Task 2 (Implement session repository helpers)
- **Issue:** The test fixture expected `tokens_today = 168`, but its own rows sum to 171.
- **Fix:** Updated the assertion to 171.
- **Files modified:** `src-tauri/tests/session_repo.rs`
- **Verification:** `cargo test --manifest-path src-tauri/Cargo.toml --test session_repo -- --nocapture`
- **Committed in:** `6d62348`

**Total deviations:** 2 auto-fixed (2 bugs)
**Impact on plan:** Both fixes were required for accurate tests after the planned schema and repository work.

## Issues Encountered

- The plan-level command `cargo test --manifest-path src-tauri/Cargo.toml session_repo store_migrations -- --nocapture` is invalid because Cargo accepts only one test-name filter before `--`. Equivalent target-specific commands were run instead.
- The literal `session_repo` filter compiled successfully but ran zero tests because it matches test names, not the integration test target. The actual verification used `--test session_repo`.

## Verification

- `cargo test --manifest-path src-tauri/Cargo.toml --test session_repo -- --nocapture` - 6 passed.
- `cargo test --manifest-path src-tauri/Cargo.toml --test store_migrations -- --nocapture` - 4 passed.

## Known Stubs

None.

## Threat Flags

| Flag | File | Description |
|------|------|-------------|
| threat_flag: local-session-metadata-storage | `src-tauri/src/store/migrations.rs` | Adds local SQLite tables for private session metadata; mitigated by metadata-only columns and tests asserting no prompt/transcript/content fields. |

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 04-02 can build parsers and attribution on top of `IndexedSession`, `ProjectRoot`, and `persist_indexed_file_result`. The session repository preserves unmatched rows and offset state needed for incremental indexing.

## Self-Check: PASSED

- Verified all created/modified files exist.
- Verified task commits `9e56725`, `621c6b9`, and `6d62348` exist in git history.

---
*Phase: 04-session-indexer*
*Completed: 2026-04-26*
