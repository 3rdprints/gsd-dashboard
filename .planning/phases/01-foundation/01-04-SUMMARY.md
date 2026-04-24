---
phase: 01-foundation
plan: 04
subsystem: ui
tags: [react, typescript, tanstack-query, tauri-ipc, vitest, tailwind]

requires:
  - phase: 01-foundation
    provides: Managed AppState plus get_boot_status, get_settings, and save_settings commands
provides:
  - Typed frontend IPC wrappers for boot and settings commands
  - TanStack Query app-root provider and settings mutation invalidation
  - Query-backed Phase 1 shell for boot/cache/settings/default/empty states
  - Visible invalid scan-root error handling for slash and bare-home backend rejections
affects: [foundation, frontend, settings, ipc]

tech-stack:
  added: []
  patterns:
    - Tauri invoke wrappers live under src/lib/ipc.ts with backend command names kept literal
    - TanStack Query owns IPC/server state; component state is limited to scan-root form draft
    - Save failures render backend AppError fields without mutating persisted settings

key-files:
  created:
    - src/lib/types.ts
    - src/lib/ipc.ts
    - src/lib/queryClient.ts
    - src/components/BootStatus.tsx
    - src/components/ScanRootsEditor.tsx
    - src/App.test.tsx
  modified:
    - .gitignore
    - src/main.tsx
    - src/App.tsx
    - src/styles.css

key-decisions:
  - "Keep Phase 1 frontend state sparse: TanStack Query for IPC data and local React state only for the scan-root draft."
  - "Render Settings saved only after settings are loaded or a save succeeds; failed saves hide success and preserve the rejected draft."
  - "Scope the Phase 1 shell to boot/settings/empty/error readiness and omit scanner, project, chart, session, and tray controls."

patterns-established:
  - "Frontend command contracts mirror backend camelCase payloads in src/lib/types.ts."
  - "Settings mutations invalidate only the settings query after successful save_settings completion."
  - "InvalidScanRoot UI uses the backend error path to show Rejected path details beside the form control."

requirements-completed: [FND-01, FND-02, FND-03, FND-04, FND-05]

duration: 10min
completed: 2026-04-24
---

# Phase 01 Plan 04: UI Shell Summary

**Query-backed Phase 1 dashboard shell with typed Tauri IPC, boot/cache status, persisted settings display, empty state, and invalid-root error surfacing**

## Performance

- **Duration:** 10 min
- **Started:** 2026-04-24T09:45:26Z
- **Completed:** 2026-04-24T09:55:37Z
- **Tasks:** 3
- **Files modified:** 10

## Accomplishments

- Added typed frontend models and Tauri `invoke` wrappers for `get_boot_status`, `get_settings`, and `save_settings`.
- Wrapped the React root in a shared TanStack `QueryClientProvider` and added settings save invalidation plumbing.
- Replaced the static scaffold with a sparse operational shell that reads boot/settings state from backend commands.
- Added scan-root editing with backend invalid-root errors shown inline for `/` and `/Users/smacdonald`, while preserving rejected input values.

## Task Commits

1. **Task 1 RED: IPC plumbing tests** - `3aef318` (test)
2. **Task 1 GREEN: Typed IPC and query plumbing** - `f9fbe6e` (feat)
3. **Task 2 RED: Shell state tests** - `c6bc8e4` (test)
4. **Task 2 GREEN: Foundation shell states** - `55b4064` (feat)
5. **Task 3 RED: Invalid-root UI tests** - `ce8bd52` (test)
6. **Task 3 GREEN: Invalid-root error UI** - `0bbc0a3` (feat)

## Files Created/Modified

- `src/lib/types.ts` - Frontend `BootStatus`, `AppSettings`, `SettingsInput`, `TrayBarSort`, and serializable `AppError` contracts.
- `src/lib/ipc.ts` - Typed Tauri invoke wrappers with exact backend command names.
- `src/lib/queryClient.ts` - Shared QueryClient, query keys, and settings mutation invalidation options.
- `src/main.tsx` - App root wrapped with `QueryClientProvider`.
- `src/App.tsx` - Sparse Phase 1 shell composition and empty dashboard state.
- `src/components/BootStatus.tsx` - Query-backed cache and migration status UI.
- `src/components/ScanRootsEditor.tsx` - Query-backed settings editor with save flow and invalid-root error rendering.
- `src/App.test.tsx` - Regression tests for IPC plumbing, shell copy, first-run defaults, scope boundaries, and invalid-root errors.
- `src/styles.css` - Phase 1 utility shell styling using the UI spec tokens.
- `.gitignore` - Narrowed root `lib/` ignore rule so `src/lib/*` source files can be tracked.

## Decisions Made

- Used a small `createSaveSettingsMutationOptions` helper so mutation invalidation can be tested without adding an IPC cache abstraction.
- Kept `Settings saved` visible after initial settings load because backend settings are initialized/persisted during boot; changes clear the success state until a save succeeds.
- Did not add setup wizard, scan actions, project cards, sessions, charts, tray controls, hidden-project controls, or parser states.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Narrowed root lib ignore rule**
- **Found during:** Task 1 GREEN (Typed IPC and query plumbing)
- **Issue:** The existing Python-style `.gitignore` rule `lib/` caused Git to ignore intentional frontend source files under `src/lib/`.
- **Fix:** Changed the rule to `/lib/`, preserving root-level build ignore behavior while allowing `src/lib/types.ts`, `src/lib/ipc.ts`, and `src/lib/queryClient.ts` to be tracked.
- **Files modified:** `.gitignore`
- **Verification:** `git add src/lib/...` succeeded and Task 1 acceptance greps passed.
- **Committed in:** `f9fbe6e`

---

**Total deviations:** 1 auto-fixed (Rule 3: 1)
**Impact on plan:** Required for planned source files to be committed. No product scope was added.

## Issues Encountered

None beyond the auto-fixed `.gitignore` blocker documented above.

## Known Stubs

None. The only empty-string pattern is local form draft initialization before backend settings load; it does not render placeholder/mock data.

## Threat Flags

None - the frontend-to-Tauri settings save surface and backend error rendering match the plan threat model.

## TDD Gate Compliance

- RED commits present: `3aef318`, `c6bc8e4`, `ce8bd52`
- GREEN commits present after RED gates: `f9fbe6e`, `55b4064`, `0bbc0a3`
- No separate refactor commits were needed.

## Verification

- `npm run test -- --run src/App.test.tsx` - passed; 8 tests.
- `npm test` - passed; 8 tests.
- `npm run build` - passed.
- `(cd src-tauri && cargo test)` - passed; bootstrap, settings command, settings guardrail, store migration, and doc test suites.
- `npm run tauri build -- --debug` - passed; debug app built at `src-tauri/target/debug/gsd-dashboard`.
- Acceptance greps passed for exact IPC command names, `QueryClientProvider`, no Zustand `create(` usage, and no Phase 3 UI scope copy outside tests.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Phase 1 is complete. Phase 2 can build scanner and parser commands on top of the established backend contracts and the frontend can continue using TanStack Query for IPC-backed state.

## Self-Check: PASSED

- Verified all key created files exist on disk.
- Verified task commits `3aef318`, `f9fbe6e`, `c6bc8e4`, `55b4064`, `ce8bd52`, and `0bbc0a3` exist in git history.
- Verified no blocking stubs remain in files created or modified by this plan.
- Verified full frontend, Rust, and Tauri debug-build gates passed.

---
*Phase: 01-foundation*
*Completed: 2026-04-24*
