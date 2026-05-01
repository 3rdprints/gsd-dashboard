---
phase: 07-live-updates
plan: 03
subsystem: backend-live-updates
tags: [rust, tauri, watcher, sqlite, tdd]
requires:
  - phase: 07-live-updates
    provides: watcher runtime status contracts and tiny live-update event enum variants
provides:
  - deterministic watcher project debounce coverage
  - targeted single-project parse and SQLite persistence helper
  - project refresh invalidation with tray refresh request tracking
affects: [live-updates, scanner, tray, settings]
tech-stack:
  added: []
  patterns: [single-project refresh helper, deterministic debounce seam, runtime tray refresh request counter]
key-files:
  created:
    - src-tauri/src/scan_refresh.rs
    - src-tauri/src/watcher/refresh.rs
    - src-tauri/src/tray/mod.rs
    - src-tauri/src/tray/service.rs
  modified:
    - src-tauri/src/app_state.rs
    - src-tauri/src/lib.rs
    - src-tauri/src/scan_service.rs
    - src-tauri/src/watcher/mod.rs
    - src-tauri/src/watcher/service.rs
    - src-tauri/tests/live_updates_watcher.rs
    - src-tauri/tests/live_updates_targeted_refresh.rs
key-decisions:
  - "Plan 07-03 uses a deterministic ProjectDebouncer seam for watcher coalescing tests instead of relying on OS notification timing."
  - "Targeted refresh lives in scan_refresh/watcher::refresh and reuses the existing parser and persist_project_scan path."
  - "Tray refresh requests are tracked on AppState because this tree had no existing native tray service module to call."
patterns-established:
  - "Project refresh emits AppEvent::ProjectUpdated with only the project id after SQLite persistence."
  - "Read-only .planning invariant is covered by targeted refresh tests and grep gates."
requirements-completed: [LIVE-01, LIVE-02, LIVE-03, LIVE-05]
duration: 9min
completed: 2026-05-01
---

# Phase 07 Plan 03: Backend Live Project Refresh Summary

**Deterministic watcher debounce tests plus targeted single-project refresh that persists derived SQLite state, emits `project:updated`, and records tray refresh requests**

## Performance

- **Duration:** 9 min
- **Started:** 2026-05-01T19:09:06Z
- **Completed:** 2026-05-01T19:18:15Z
- **Tasks:** 2
- **Files modified:** 11

## Accomplishments

- Activated watcher debounce coverage with a deterministic `ProjectDebouncer` seam and retained root/fallback tests for 500ms debounce and 60s polling fallback status.
- Added `scan_refresh::scan_single_project_candidate` so watcher refresh can reparse and persist one affected project without invoking full cache rebuild.
- Added `watcher::refresh::refresh_project_planning_dir_for_app`, which persists first, emits only `project:updated { id }`, and records a tray refresh request.
- Activated targeted refresh tests covering affected-project-only updates, tiny event payloads, tray refresh request recording, and read-only `.planning` sources.

## Task Commits

1. **Task 1 RED: watcher debounce test** - `c757467` (test)
2. **Task 1 GREEN: watcher project debouncer** - `9af5056` (feat)
3. **Task 2 RED: targeted refresh tests** - `fba66d9` (test)
4. **Task 2 GREEN: targeted project refresh** - `1a97bc9` (feat)

_Note: TDD tasks intentionally produced RED and GREEN commits._

## Files Created/Modified

- `src-tauri/src/scan_refresh.rs` - Single-project scan/persist helper and refresh outcome DTO.
- `src-tauri/src/watcher/refresh.rs` - Targeted planning directory refresh orchestration.
- `src-tauri/src/tray/service.rs` - Runtime tray refresh request hook.
- `src-tauri/src/app_state.rs` - Tracks tray refresh request count.
- `src-tauri/src/watcher/service.rs` - Deterministic project debounce helper.
- `src-tauri/tests/live_updates_watcher.rs` - Active watcher root, debounce, and fallback tests.
- `src-tauri/tests/live_updates_targeted_refresh.rs` - Active targeted refresh tests.

## Decisions Made

- Used injected logical millisecond timestamps for debounce tests to avoid depending on FSEvents/inotify timing.
- Kept single-project refresh in a focused `scan_refresh` module so `scan_service.rs` stays under the AGENTS.md 500-line limit.
- Added a concrete tray refresh request counter instead of a no-op because the current tree did not contain the Phase 06 native tray service module referenced by the plan.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Added concrete tray refresh request tracking**
- **Found during:** Task 2 (Implement targeted project refresh and watcher service)
- **Issue:** The plan referenced `src-tauri/src/tray/service.rs`, but this working tree had no tray module. A no-op hook would not satisfy the refresh-request contract.
- **Fix:** Added `tray::service::request_tray_refresh` backed by an `AppState` runtime counter and covered it in targeted refresh tests.
- **Files modified:** `src-tauri/src/app_state.rs`, `src-tauri/src/tray/service.rs`, `src-tauri/tests/live_updates_targeted_refresh.rs`
- **Verification:** `cargo test --manifest-path src-tauri/Cargo.toml live_updates_targeted_refresh -- --nocapture`
- **Committed in:** `1a97bc9`

**2. [Rule 2 - AGENTS.md] Preserved 500-line file limit**
- **Found during:** Task 2 (Implement targeted project refresh and watcher service)
- **Issue:** Adding the helper directly to `scan_service.rs` pushed the file above the AGENTS.md 500-line limit.
- **Fix:** Moved the helper and DTO into `scan_refresh.rs`; `scan_service.rs` is 499 lines after formatting.
- **Files modified:** `src-tauri/src/scan_refresh.rs`, `src-tauri/src/scan_service.rs`, `src-tauri/src/lib.rs`
- **Verification:** `wc -l src-tauri/src/scan_service.rs src-tauri/src/scan_refresh.rs`
- **Committed in:** `1a97bc9`

---

**Total deviations:** 2 auto-fixed (Rule 2)
**Impact on plan:** The implementation stays within the plan's live-update scope and project rules. Native OS watcher registration remains represented by the existing runtime root/status service and deterministic debounce seam; the new targeted refresh path is ready for a later event-source integration if needed.

## Issues Encountered

- The plan assumed a native tray service module already existed, but this tree only had tray settings fields. The plan was adjusted to record tray refresh requests on `AppState`.
- `src-tauri/src/scan_service.rs` was already at 499 lines before Task 2; the new helper had to move to a focused module to comply with AGENTS.md.

## Known Stubs

None.

## Verification

- `cargo test --manifest-path src-tauri/Cargo.toml live_updates_watcher -- --nocapture`
- `cargo test --manifest-path src-tauri/Cargo.toml live_updates_targeted_refresh -- --nocapture`
- `cargo test --manifest-path src-tauri/Cargo.toml scanner -- --nocapture`
- `cargo test --manifest-path src-tauri/Cargo.toml scan_service -- --nocapture`
- Acceptance grep gates for `refresh_project_planning_dir`, `project:updated`, `request_tray_refresh`, no watcher `rebuild_cache`, and no `.planning` writes all passed.

## TDD Gate Compliance

- RED commit present before GREEN for Task 1: `c757467` -> `9af5056`.
- RED commit present before GREEN for Task 2: `fba66d9` -> `1a97bc9`.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 07-04 can build on active backend invalidation: project refresh now has an event payload, tray refresh request tracking, and deterministic tests. If a later plan needs actual native tray redraws, it should replace the runtime counter with the native tray renderer once that module exists in this tree.

## Self-Check: PASSED

- Found summary file and key created/modified files.
- Found task commits `c757467`, `9af5056`, `fba66d9`, and `1a97bc9`.

---
*Phase: 07-live-updates*
*Completed: 2026-05-01*
