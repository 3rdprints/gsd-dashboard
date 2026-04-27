---
phase: 05-project-detail-global-sessions-charts
plan: 06
subsystem: backend
tags: [rust, sqlite, sessions, heatmap, tauri]

requires:
  - phase: 05-project-detail-global-sessions-charts
    provides: Phase 5 daily_activity schema, DailyActivityUpdated event, and backend scaffold tests from Plans 05-01 and 05-02
provides:
  - daily_activity rebuild and zero-filled load lifecycle
  - post-index daily activity rebuild with empty invalidation event
  - Portfolio heatmap command body ready for Plan 05-07 registration
affects: [portfolio-heatmap, session-indexer, frontend-invalidation]

tech-stack:
  added: []
  patterns: [derived SQLite aggregate rebuild, command DTO from store row, non-fatal post-index maintenance]

key-files:
  created:
    - src-tauri/src/store/daily_activity.rs
  modified:
    - src-tauri/src/store/mod.rs
    - src-tauri/src/sessions/indexer.rs
    - src-tauri/src/events.rs
    - src-tauri/src/commands/projects.rs
    - src-tauri/tests/daily_activity_rebuild.rs
    - src-tauri/tests/portfolio_heatmap.rs

key-decisions:
  - "daily_activity load_window clamps all callers to 1..365 and zero-fills rows in Rust."
  - "Session indexing emits DailyActivityUpdated through the session-index event stream after a successful non-fatal rebuild."

patterns-established:
  - "Derived heatmap storage is rebuilt after successful indexing; rebuild errors are logged and do not fail indexing."
  - "Portfolio heatmap command bodies can be implemented before central command registration plans."

requirements-completed: [PORT-05]

duration: 4min
completed: 2026-04-27
---

# Phase 05 Plan 06: Daily Activity and Heatmap Backend Summary

**Rolling daily session activity is now rebuilt after indexing and exposed as a zero-filled Portfolio heatmap backend query.**

## Performance

- **Duration:** 4 min
- **Started:** 2026-04-27T14:24:49Z
- **Completed:** 2026-04-27T14:28:45Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments

- Added `daily_activity::rebuild_window` using the D-16 token formula: input + output + cache read + cache creation.
- Added `daily_activity::load_window` with 1..365 day clamping, Rust-side zero-fill, and top project name resolution.
- Hooked successful session indexing to rebuild the rolling 90-day window and emit `DailyActivityUpdated`.
- Added `get_portfolio_heatmap(days)` command body without registering it, preserving Plan 05-07 ownership.

## Task Commits

1. **Task 1: Add daily_activity rebuild/load module and indexer event hook** - `37db879` (feat)
2. **Task 2: Add get_portfolio_heatmap command body** - `560be7b` (feat)

## Files Created/Modified

- `src-tauri/src/store/daily_activity.rs` - Rebuilds rolling aggregate rows and loads clamped zero-filled windows.
- `src-tauri/src/store/mod.rs` - Exports the daily activity store module.
- `src-tauri/src/sessions/indexer.rs` - Rebuilds daily activity after successful indexing and emits the invalidation event.
- `src-tauri/src/events.rs` - Allows the session index event stream to carry the app-level daily activity invalidation event.
- `src-tauri/src/commands/projects.rs` - Adds heatmap DTOs and `get_portfolio_heatmap`.
- `src-tauri/tests/daily_activity_rebuild.rs` - Covers idempotent rebuild, token totals, top project selection, and event emission.
- `src-tauri/tests/portfolio_heatmap.rs` - Covers 90-day default output, zero-fill, top project names, and day clamping.

## Decisions Made

`load_window` ends at the latest aggregated `daily_activity` row when data exists, falling back to the current date only when the table is empty. This makes command tests deterministic while still returning a complete clamped window for UI callers.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

The RED scaffold blocker from Plan 05-02 was resolved by replacing both scaffold panics with integration tests and implementation.

## Known Stubs

None introduced by this plan.

## Threat Flags

| Flag | File | Description |
|------|------|-------------|
| threat_flag: event-bridge | `src-tauri/src/events.rs` | Session index progress events can now carry the empty `DailyActivityUpdated` invalidation event; payload remains content-free per T-05-06-01. |

## TDD Gate Compliance

- RED scaffold commit inherited from Plan 05-02: `ec02a8b`
- GREEN commits produced by this plan: `37db879`, `560be7b`
- No separate refactor commit was needed.

## User Setup Required

None - no external service configuration required.

## Verification

- `cd src-tauri && cargo test --test daily_activity_rebuild` - passed.
- `cd src-tauri && cargo test --test portfolio_heatmap` - passed.
- `cd src-tauri && cargo test --test daily_activity_rebuild --test portfolio_heatmap` - passed.
- `grep -nE 'pub fn rebuild_window|pub fn load_window' src-tauri/src/store/daily_activity.rs` - returned both functions.
- `grep -nE 'DailyActivityUpdated' src-tauri/src/sessions/indexer.rs` - returned the indexer emission.
- `grep -nE 'pub async fn get_portfolio_heatmap' src-tauri/src/commands/projects.rs` - returned one command body match.

## Next Phase Readiness

Plan 05-07 can register `get_portfolio_heatmap` centrally and wire command permissions without overlapping this plan's implementation files.

## Self-Check: PASSED

- Confirmed summary and key created file exist.
- Confirmed task commits exist: `37db879`, `560be7b`.
- Confirmed plan verification and acceptance gates passed.

---
*Phase: 05-project-detail-global-sessions-charts*
*Completed: 2026-04-27*
