---
phase: 08-autostart-os-polish
plan: 02
subsystem: native-os
tags: [tauri, rust, autostart, tray, startup]

requires:
  - phase: 08-autostart-os-polish
    provides: "Plan 08-01 exact --autostart launch argument helper and backend autostart registration"
provides:
  - "Default-hidden Tauri main window with explicit startup visibility decisions"
  - "Autostart tray-only startup when native tray setup succeeds"
  - "Visible dashboard fallback when autostart tray setup fails"
  - "Tested Show Dashboard tray recovery action for hidden startup"
affects: [phase-08-autostart-os-polish, startup, tray, packaging-smoke]

tech-stack:
  added: []
  patterns:
    - "Pure StartupVisibilityAction helper isolates native startup window decisions from Tauri runtime behavior."
    - "Native tray recovery labels stay centralized in tray service and resolve through existing menu action IDs."

key-files:
  created:
    - src-tauri/tests/startup_visibility.rs
    - .planning/phases/08-autostart-os-polish/08-02-SUMMARY.md
  modified:
    - src-tauri/tauri.conf.json
    - src-tauri/src/bootstrap.rs
    - src-tauri/src/tray/service.rs
    - src-tauri/tests/tray_service.rs
    - src/config.test.ts

key-decisions:
  - "Set the configured main window visible=false and show it explicitly after startup for normal launches."
  - "Use Plan 08-01's exact is_autostart_launch helper instead of parsing arbitrary startup arguments."
  - "Swallow tray setup errors only for autostart launches after requesting the visible dashboard fallback."
  - "Keep native tray recovery copy exactly as Show Dashboard."

patterns-established:
  - "Startup visibility is decided by startup_visibility_action(is_autostart_launch, tray_setup_succeeded)."
  - "Cargo filter verification tests are named with startup_visibility_ and tray_service_ prefixes so planned commands run real assertions."

requirements-completed: [AUTO-02]

duration: 7min
completed: 2026-05-02T19:26:42Z
---

# Phase 08 Plan 02: Hidden Startup and Tray Recovery Summary

**Autostart launches now stay tray-only after successful tray setup, while normal launches and tray failures explicitly surface the dashboard.**

## Performance

- **Duration:** 7 min
- **Started:** 2026-05-02T19:19:45Z
- **Completed:** 2026-05-02T19:26:42Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- Configured the Tauri `main` window with `"visible": false` to prevent autostart window flash.
- Added `StartupVisibilityAction` and `startup_visibility_action` coverage for normal launch, autostart success, and autostart tray-failure fallback.
- Updated `manage_app_state_and_tray` to manage `AppState`, run tray setup, use exact `--autostart` detection, and request dashboard visibility when required.
- Made `show_dashboard_window` public to bootstrap with the planned `Option<&str>` route signature.
- Preserved the native `Show Dashboard` recovery menu action and added a focused hidden-startup recovery assertion.

## Task Commits

Each task was committed atomically:

1. **Task 1 RED: Startup visibility tests** - `b983e69` (test)
2. **Task 1 GREEN: Startup visibility implementation** - `19c85b2` (feat)
3. **Task 2 RED: Tray recovery test** - `a2f0cd2` (test)
4. **Task 2 GREEN: Tray recovery implementation** - `ae4ba09` (feat)

**Plan metadata:** committed separately with this summary.

## Files Created/Modified

- `src-tauri/tauri.conf.json` - Starts the configured main window hidden until Rust startup logic chooses visibility.
- `src-tauri/src/bootstrap.rs` - Adds the pure startup visibility decision helper and applies normal/autostart/tray-failure behavior.
- `src-tauri/src/tray/service.rs` - Exposes `show_dashboard_window` to bootstrap and centralizes the native `Show Dashboard` label.
- `src-tauri/tests/startup_visibility.rs` - Covers the pure startup visibility decision matrix.
- `src-tauri/tests/tray_service.rs` - Covers the hidden-startup `Show Dashboard` recovery action.
- `src/config.test.ts` - Asserts the Tauri main window is configured hidden by default.

## Decisions Made

- Followed the plan's tray-first behavior: successful `--autostart` launches remain hidden, but normal launches still show the dashboard.
- Returned `Ok(())` for autostart tray setup failure only after requesting the dashboard fallback, so the app does not remain invisibly broken.
- Did not add notifications, dock/taskbar controls, new routes, or frontend recovery UI.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Named tests to match Cargo filter commands**
- **Found during:** Task 1 and Task 2 verification.
- **Issue:** `cargo test startup_visibility` and `cargo test tray_service` filter by test name, not integration-test filename, so the planned commands initially compiled but ran zero new assertions.
- **Fix:** Prefixed new Rust test names with `startup_visibility_` and `tray_service_`.
- **Files modified:** `src-tauri/tests/startup_visibility.rs`, `src-tauri/tests/tray_service.rs`
- **Verification:** `cargo test startup_visibility -- --nocapture` ran 3 tests; `cargo test tray_service -- --nocapture` ran 1 test.
- **Committed in:** `19c85b2`, `ae4ba09`

---

**Total deviations:** 1 auto-fixed (Rule 3).
**Impact on plan:** Verification became stricter; product scope did not expand.

## Issues Encountered

- `cargo fmt --check` flagged formatting in the new tray-service test after the RED commit. `cargo fmt` was applied before the GREEN commit and `cargo fmt --check` passed afterward.
- `.planning/STATE.md` and `.planning/ROADMAP.md` had pre-existing orchestrator edits and were preserved without staging during task commits.

## User Setup Required

None - no external service configuration required.

## Verification

- `cd src-tauri && cargo test startup_visibility -- --nocapture` - passed; 3 startup visibility tests ran.
- `cd src-tauri && cargo test tray_service -- --nocapture` - passed; 1 Phase 8 tray recovery test ran.
- `npm test -- config.test.ts` - passed; 3 config tests ran.
- Task 1 acceptance greps for `"visible": false`, `StartupVisibilityAction`, `show_dashboard_window`, and `is_autostart_launch` passed.
- Task 2 acceptance greps for `Show Dashboard`, `SHOW_DASHBOARD_ID`, and `TrayMenuAction::ShowDashboard` passed.
- `cd src-tauri && cargo fmt --check` - passed.

## Known Stubs

None.

## Threat Flags

None.

## Next Phase Readiness

AUTO-02 native behavior is ready for Phase 8 integration verification and Phase 9 packaging smoke checks. Normal launches explicitly show the dashboard, autostart launches stay hidden only when tray setup succeeds, and the native tray menu retains a tested recovery action.

## Self-Check: PASSED

- Summary file exists at `.planning/phases/08-autostart-os-polish/08-02-SUMMARY.md`.
- Created file exists: `src-tauri/tests/startup_visibility.rs`.
- Task commits exist: `b983e69`, `19c85b2`, `a2f0cd2`, `ae4ba09`.
- Confirmed planned source files remain under the 500-line AGENTS.md limit.
- Existing `.planning/STATE.md` and `.planning/ROADMAP.md` edits were preserved and not included in task commits.

---
*Phase: 08-autostart-os-polish*
*Completed: 2026-05-02T19:26:42Z*
