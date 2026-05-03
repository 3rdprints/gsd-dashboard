---
phase: 08-autostart-os-polish
plan: 01
subsystem: native-os
tags: [tauri, rust, autostart, settings]

requires:
  - phase: 01-foundation
    provides: Tauri 2 shell, settings persistence, AppError, and command structure
  - phase: 06-tray-icon-with-milestone-bars
    provides: Tray refresh side effects after settings changes
provides:
  - Backend-owned launch-on-login plugin registration with exact autostart launch flag
  - AutostartBackend abstraction over Tauri autolaunch manager
  - Settings save coordination that prevents persisted autostart intent drift on plugin failure
affects: [phase-08-autostart-os-polish, settings, native-startup]

tech-stack:
  added: [tauri-plugin-autostart 2.5.1]
  patterns: [backend-owned OS integration, injectable native backend for tests, validated persist then plugin mutation with rollback]

key-files:
  created:
    - src-tauri/src/autostart.rs
    - src-tauri/tests/autostart_settings.rs
  modified:
    - src-tauri/Cargo.toml
    - src-tauri/Cargo.lock
    - src-tauri/src/lib.rs
    - src-tauri/src/main.rs
    - src-tauri/src/commands/settings.rs

key-decisions:
  - "Use tauri-plugin-autostart 2.5.1 with MacosLauncher::LaunchAgent and exact --autostart launch argument."
  - "Accepted override after code review: validate and persist Settings first, then apply OS autostart enable/disable, and roll back persisted intent or return the rollback error if plugin mutation fails."
  - "Keep autostart mutation backend-owned; no frontend plugin calls or capability expansion were added."

patterns-established:
  - "AutostartBackend trait: native plugin calls are injectable for tests and isolated from settings command orchestration."
  - "Settings save ordering: load current settings, persist validated settings, mutate OS autostart only when intent changes, roll back persisted settings on plugin failure, then emit events and refresh tray only after success."

requirements-completed: [AUTO-01]

duration: 9min
completed: 2026-05-02T19:16:41Z
---

# Phase 08 Plan 01: Backend Autostart Contract Summary

**Launch-on-login is wired through the official Tauri autostart plugin with backend-owned Settings coordination that prevents SQLite/plugin intent drift.**

## Accepted Contract Override

Code review rejected the original plugin-before-persist ordering because it mutated OS autostart state before SQLite validation/persistence could fail. The accepted AUTO-01 contract is now:

1. Load current settings.
2. Validate and persist the requested Settings input.
3. If `autostart_enabled` changed, call the OS autostart backend.
4. If the backend fails, restore the previous persisted settings; if rollback persistence fails, return that persistence error instead of silently claiming the setting was unchanged.
5. Emit `settings-changed`, watcher status, and tray refresh only after both persistence and OS mutation succeed.

This preserves the no-drift user contract while avoiding OS login-item mutation for invalid Settings input.

## Performance

- **Duration:** 9 min
- **Started:** 2026-05-02T19:07:00Z
- **Completed:** 2026-05-02T19:16:41Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments

- Added `src-tauri/src/autostart.rs` with the exact `--autostart` launch flag helper, plugin registration helper, `AutostartBackend`, and `TauriAutostartBackend`.
- Registered `tauri-plugin-autostart` in the real Tauri builder with `MacosLauncher::LaunchAgent` and `--autostart`.
- Updated `save_settings_for_app` to use an injectable autostart backend with validated persistence before OS mutation and rollback on plugin failure.
- Added integration coverage for exact launch argument matching, validation failure before backend mutation, enable success, disable success, and backend failure preserving persisted settings and suppressing `settings-changed`.

## Task Commits

1. **Task 1 RED: Autostart launch flag test** - `7d4089a` (test)
2. **Task 1 GREEN: Plugin registration and autostart helper** - `860208b` (feat)
3. **Task 2 RED: Autostart settings save tests** - `6f105a0` (test)
4. **Task 2 GREEN: Settings/autostart coordination** - `cb10575` (feat)

**Plan metadata:** committed separately with this summary.

## Files Created/Modified

- `src-tauri/Cargo.toml` - Added `tauri-plugin-autostart`.
- `src-tauri/Cargo.lock` - Locked `tauri-plugin-autostart` and transitive dependencies.
- `src-tauri/src/autostart.rs` - New autostart service contract, exact launch flag helper, plugin registration helper, and Tauri backend wrapper.
- `src-tauri/src/lib.rs` - Exposes the autostart module.
- `src-tauri/src/main.rs` - Registers the official autostart plugin in the Tauri builder through the shared helper.
- `src-tauri/src/commands/settings.rs` - Persists validated settings, delegates changed autostart intent through the backend, and rolls back persistence on backend failure.
- `src-tauri/tests/autostart_settings.rs` - Covers exact flag matching and fake-backend settings save behavior.

## Decisions Made

- Used the current official `tauri-plugin-autostart` 2.5.1 API verified via Context7 and local crate source.
- Kept plugin calls on the Rust settings save path and did not grant frontend autostart plugin capabilities.
- Preserved existing watcher restart, `settings-changed`, watcher-status event, and tray refresh behavior after successful settings persistence.

## Deviations from Plan

- Code review changed the autostart consistency contract from plugin-before-persist to persist-then-plugin-with-rollback. This is intentional: invalid Settings input must not mutate OS login-item state, plugin failure must not emit `settings-changed`, and rollback persistence failure must surface instead of being swallowed.

## Issues Encountered

- `cargo test autostart_settings -- --nocapture` filters test names, not only the integration-test file. The settings tests were named with an `autostart_settings_` prefix so the planned command actually exercises them.
- The orchestrator-owned `.planning/STATE.md` and `.planning/ROADMAP.md` were already modified in the worktree and were intentionally left untouched.

## Verification

- `cd src-tauri && cargo test autostart_launch_arg -- --nocapture` - passed.
- `cd src-tauri && cargo test autostart_settings -- --nocapture` - passed; 5 settings tests ran after review fixes.
- `cd src-tauri && cargo test settings_guardrails -- --nocapture` - passed command as planned, but filtered zero tests by name.
- `cd src-tauri && cargo test --test settings_guardrails -- --nocapture` - passed; 9 guardrail tests ran.
- `cd src-tauri && grep -R "tauri_plugin_autostart::init" src` - passed.
- Task acceptance greps for `AUTOSTART_ARG`, `pub mod autostart;`, `MacosLauncher::LaunchAgent`, `ManagerExt`, `autolaunch()`, fake backend, `settings-changed`, and watcher restart text all passed.

## Known Stubs

None.

## Threat Flags

| Flag | File | Description |
|------|------|-------------|
| threat_flag: os-autostart-plugin | src-tauri/src/autostart.rs | Adds backend-owned OS login-item mutation through the official Tauri autostart plugin; covered by the plan threat model. |

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

AUTO-01 backend behavior is ready for Phase 08 startup-hidden work. The frontend Plan 08-03 work can keep using the existing `autostartEnabled` setting shape without direct plugin access.

## Self-Check: PASSED

- Found created files: `src-tauri/src/autostart.rs`, `src-tauri/tests/autostart_settings.rs`.
- Found task commits: `7d4089a`, `860208b`, `6f105a0`, `cb10575`.
- Confirmed no summary-time edits were made to `.planning/STATE.md` or `.planning/ROADMAP.md`.

---
*Phase: 08-autostart-os-polish*
*Completed: 2026-05-02T19:16:41Z*
