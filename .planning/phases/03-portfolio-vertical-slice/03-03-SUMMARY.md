---
phase: 03-portfolio-vertical-slice
plan: 03
subsystem: desktop-integration
tags: [tauri, clipboard, opener, react, vitest]

requires:
  - phase: 03-portfolio-vertical-slice
    provides: Backend portfolio/detail/rebuild commands and release capability entries
provides:
  - Official Tauri clipboard-manager and opener plugin dependencies on npm and Cargo sides
  - Tauri builder plugin registration for clipboard and opener integration
  - Release capability allowlist entries for clipboard text writes, path opening, and VS Code URL opening
  - Typed frontend action wrappers for copy/open behaviors with Vitest coverage
affects: [phase-03-ui, project-detail, project-card-actions, release-capabilities]

tech-stack:
  added: [@tauri-apps/plugin-clipboard-manager 2.3.2, @tauri-apps/plugin-opener 2.5.3, tauri-plugin-clipboard-manager 2.3.2, tauri-plugin-opener 2.5.3]
  patterns: [client-side official Tauri plugin actions, narrow release capability permissions, TDD for action wrappers]

key-files:
  created:
    - src/lib/actions.ts
  modified:
    - package.json
    - package-lock.json
    - src-tauri/Cargo.toml
    - src-tauri/Cargo.lock
    - src-tauri/src/main.rs
    - src-tauri/capabilities/default.json
    - src/App.test.tsx

key-decisions:
  - "Copy/open actions use official Tauri plugins directly from the frontend; no backend invoke or shell/process command path was added."
  - "VS Code project opening uses vscode://file/${encodeURI(rootPath)} so path separators remain path separators."
  - "Release capabilities allow only clipboard text write plus opener path and URL commands for this plan."

patterns-established:
  - "Frontend desktop actions live in src/lib/actions.ts and wrap Tauri plugin APIs without command execution."
  - "Plugin capability checks should assert exact plugin IDs alongside generated command permissions."

requirements-completed: [CLIP-01, CLIP-02, DET-01]

duration: 8min
completed: 2026-04-25
---

# Phase 03 Plan 03: Clipboard and Opener Actions Summary

**Official Tauri clipboard/opener plugins with narrow release permissions and tested frontend wrappers for copy, Finder, and VS Code actions**

## Performance

- **Duration:** 8 min
- **Started:** 2026-04-25T15:38:17Z
- **Completed:** 2026-04-25T15:46:18Z
- **Tasks:** 3
- **Files modified:** 7

## Accomplishments

- Installed exact npm and Cargo plugin dependencies for clipboard-manager and opener.
- Registered both Tauri plugins before app setup and added release-safe plugin permissions.
- Added `copyNextCommand`, `openProjectInFinder`, and `openProjectInVsCode` wrappers with mocked Vitest coverage.
- Preserved the no-command-execution invariant; wrappers do not call backend `invoke`.

## Task Commits

Each task was committed atomically:

1. **Task 1: Install official clipboard and opener plugins** - `8a0a846` (feat)
2. **Task 2: Register plugins and release capabilities** - `5253587` (feat)
3. **Task 3 RED: Add safe frontend action wrapper tests** - `d9a49a2` (test)
4. **Task 3 GREEN: Add safe frontend action wrappers** - `ee0ff25` (feat)

_Note: Task 3 was marked TDD, so it produced separate RED and GREEN commits._

## Files Created/Modified

- `src/lib/actions.ts` - Frontend wrappers for clipboard text write, project path open, and VS Code URL open.
- `src/App.test.tsx` - Vitest mocks and behavior coverage for safe action wrappers.
- `src-tauri/src/main.rs` - Registered clipboard-manager and opener plugins.
- `src-tauri/capabilities/default.json` - Added narrow plugin permissions alongside existing generated command allows.
- `package.json` / `package-lock.json` - Added exact npm plugin dependencies.
- `src-tauri/Cargo.toml` / `src-tauri/Cargo.lock` - Added Rust plugin dependencies and lockfile updates.

## Decisions Made

- Used `@tauri-apps/plugin-clipboard-manager` `writeText` for command copy, with tests proving no backend `invoke`.
- Used `@tauri-apps/plugin-opener` `openPath` for Finder and `openUrl` for `vscode://file/` URLs.
- Kept capabilities limited to `clipboard-manager:allow-write-text`, `opener:allow-open-path`, and `opener:allow-open-url`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Corrected npm dependency range format**
- **Found during:** Task 1 (Install official clipboard and opener plugins)
- **Issue:** `npm install` wrote caret ranges (`^2.3.2`, `^2.5.3`), but the plan required exact package pins.
- **Fix:** Re-ran npm with `--save-exact` so `package.json` and `package-lock.json` matched the acceptance criteria.
- **Files modified:** `package.json`, `package-lock.json`
- **Verification:** Exact package grep checks passed.
- **Committed in:** `8a0a846`

---

**Total deviations:** 1 auto-fixed (Rule 1).
**Impact on plan:** Required to satisfy exact dependency pinning; no scope change.

## Issues Encountered

- The literal Task 3 forbidden-pattern check `! grep -R "Command::new\\|child_process\\|spawn\\|exec(" src src-tauri/src` fails on pre-existing `tokio::task::spawn_blocking` calls in backend scan/bootstrap code. Those are not process execution and were not introduced by this plan. A narrower process/shell check passed for `Command::new`, `child_process`, `exec(`, and word-boundary `spawn(`.

## Verification

- `npm test -- src/App.test.tsx` - passed (19 tests)
- `npm run build` - passed
- `cargo check --manifest-path src-tauri/Cargo.toml` - passed
- Capability smoke check for `allow-get-portfolio`, `allow-get-project`, `allow-rebuild-cache`, `clipboard-manager:allow-write-text`, `opener:allow-open-path`, and `opener:allow-open-url` - passed
- Narrow process-execution grep checks - passed

## Known Stubs

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 03-04 can wire `src/lib/actions.ts` into ProjectCard and ProjectDetail UI surfaces. Release capabilities already include the custom backend commands from Plans 03-01/03-02 and the plugin permissions needed by the frontend actions.

## TDD Gate Compliance

- RED commit present: `d9a49a2`
- GREEN commit present: `ee0ff25`
- REFACTOR commit: not needed

## Self-Check: PASSED

- Verified created files exist: `src/lib/actions.ts`, `.planning/phases/03-portfolio-vertical-slice/03-03-SUMMARY.md`.
- Verified task commits exist: `8a0a846`, `5253587`, `d9a49a2`, `ee0ff25`.

---
*Phase: 03-portfolio-vertical-slice*
*Completed: 2026-04-25*
