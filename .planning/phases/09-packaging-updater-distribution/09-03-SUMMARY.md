---
phase: 09-packaging-updater-distribution
plan: 03
subsystem: ui
tags: [tauri, updater, react, settings, vitest]

requires:
  - phase: 09-packaging-updater-distribution
    provides: Signed Tauri updater/process plugin configuration and release capabilities
provides:
  - Guarded frontend updater wrapper for browser/test-safe update checks
  - Compact Settings update panel with quiet status and explicit install action
  - Settings integration that renders update UX without startup prompts
affects: [updater, settings, distribution, release-ux]

tech-stack:
  added: []
  patterns:
    - Dynamic Tauri updater/process imports are gated behind __TAURI_INTERNALS__
    - Update UX is route-scoped to SettingsPage.css and defaults to a quiet up-to-date panel

key-files:
  created:
    - src/lib/update.ts
    - src/lib/update.test.ts
    - src/components/UpdatePrompt.tsx
    - src/components/UpdatePrompt.test.tsx
  modified:
    - src/routes/SettingsPage.tsx
    - src/routes/SettingsPage.css
    - src/routes/SettingsPage.test.tsx

key-decisions:
  - "Use a nonthrowing updater wrapper that returns UI states rather than exposing Tauri updater exceptions to React components."
  - "Settings renders the update panel in a quiet default state; checks, installs, and relaunches require explicit user actions."

patterns-established:
  - "Frontend Tauri updater APIs use dynamic imports only after a __TAURI_INTERNALS__ guard so tests and browser rendering never invoke native APIs."
  - "Updater failures are normalized into amber network/check errors or red signature verification errors."

requirements-completed: [UPD-01]

duration: 7 min
completed: 2026-05-03
---

# Phase 09 Plan 03: In-App Update UX Summary

**Quiet Settings update UX with guarded Tauri updater checks, explicit install/relaunch flow, and nonblocking failure states**

## Performance

- **Duration:** 7 min
- **Started:** 2026-05-03T14:27:06Z
- **Completed:** 2026-05-03T14:33:56Z
- **Tasks:** 3
- **Files modified:** 7

## Accomplishments

- Added `checkForUpdate()` and `installAndRestart()` wrappers that avoid native updater imports outside Tauri and return nonthrowing states for unsupported, up-to-date, available, error, and signature-error results.
- Built a compact Settings update panel with exact required copy for up-to-date, available update, install, restart, network/check failure, and signature verification failure states.
- Inserted the update panel below cache controls and above indexing controls, with tests proving Settings renders it without requiring Tauri internals.

## Task Commits

Each task was committed atomically:

1. **Task 1: Create guarded updater wrapper** - `36235b2` (test), `42ae3fa` (feat)
2. **Task 2: Build compact Settings update panel** - `226b14a` (test), `44213ed` (feat)
3. **Task 3: Insert update panel into Settings** - `f9532bd` (feat)

Additional verification fix:

- `b95210f` (fix) - Satisfied TypeScript build typing for the focused update mock.

**Plan metadata:** this docs commit

## Files Created/Modified

- `src/lib/update.ts` - Guarded Tauri updater/process wrapper with normalized UI states.
- `src/lib/update.test.ts` - Tests unsupported browser mode, nonblocking updater errors, and install-before-relaunch ordering.
- `src/components/UpdatePrompt.tsx` - Compact Settings update panel with quiet, available, installing, restart, and failure states.
- `src/components/UpdatePrompt.test.tsx` - Tests up-to-date, available update, and nonblocking error rendering.
- `src/routes/SettingsPage.tsx` - Renders `UpdatePrompt` between cache controls and indexing controls.
- `src/routes/SettingsPage.css` - Adds route-scoped update panel styles, including amber and red failure variants.
- `src/routes/SettingsPage.test.tsx` - Asserts Settings renders update copy and check action without native updater internals.

## Verification

- `npm test -- src/lib/update.test.ts` passed.
- `npm test -- src/components/UpdatePrompt.test.tsx` passed.
- `npm test -- src/routes/SettingsPage.test.tsx src/components/UpdatePrompt.test.tsx src/lib/update.test.ts` passed.
- `npm test -- src/lib/update.test.ts src/components/UpdatePrompt.test.tsx src/routes/SettingsPage.test.tsx` passed.
- `npm test` passed: 20 test files and 81 tests.
- `npm run build` passed.
- All task acceptance grep gates passed.

## Decisions Made

- Used dynamic imports from `@tauri-apps/plugin-updater` and `@tauri-apps/plugin-process` only after the Tauri internals guard, matching the existing browser-safe listener pattern.
- Defaulted Settings to the quiet up-to-date state so opening Settings does not automatically call the updater or show startup prompts.
- Kept network/check failures amber and signature verification failures red, matching the UI spec's severity split.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed TypeScript-only update mock mismatch**
- **Found during:** Plan-level verification after Task 3
- **Issue:** `npm run build` failed because the focused mocked update object in `UpdatePrompt.test.tsx` implemented only `downloadAndInstall()`, while the mocked `checkForUpdate()` return type expected the full Tauri `Update` type.
- **Fix:** Cast the focused mock through the typed updater return path without changing component behavior or test assertions.
- **Files modified:** `src/components/UpdatePrompt.test.tsx`
- **Verification:** `npm test -- src/components/UpdatePrompt.test.tsx`, `npm run build`, and `npm test` passed.
- **Committed in:** `b95210f`

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** No scope expansion. The fix was required for TypeScript build correctness.

## Issues Encountered

- `npm run build` emitted a nonblocking Rolldown plugin timing warning for `@tailwindcss/vite:generate:build`; the build still completed successfully.

## User Setup Required

None - no external service configuration required.

## Known Stubs

None.

## Threat Flags

None - the new updater trust boundaries and failure surfaces were covered by the plan threat model.

## Next Phase Readiness

Ready for later Phase 09 release workflow, manifest, Pages, and distribution plans. The app now has the user-mediated Settings update UX needed for signed updater availability without automatic install behavior.

## Self-Check: PASSED

- Summary file created.
- Task commits `36235b2`, `42ae3fa`, `226b14a`, `44213ed`, `f9532bd`, and fix commit `b95210f` exist.
- Key update wrapper, component, Settings integration, CSS, and tests exist.

---
*Phase: 09-packaging-updater-distribution*
*Completed: 2026-05-03*
