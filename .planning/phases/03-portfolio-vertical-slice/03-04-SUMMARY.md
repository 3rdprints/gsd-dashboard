---
phase: 03-portfolio-vertical-slice
plan: 04
subsystem: ui
tags: [react, react-router, tanstack-query, tauri-ipc, vitest]

requires:
  - phase: 03-portfolio-vertical-slice
    provides: Backend portfolio/detail/rebuild commands and safe copy/open action wrappers
provides:
  - Routed portfolio, project detail, and settings UI vertical slice
  - Typed frontend portfolio/detail/rebuild IPC contracts and query invalidation keys
  - Project cards with progress, parse badges, detail navigation, and copy feedback
  - Settings scan-root editor, hidden project unhide, rebuild confirmation, and disabled indexing toggles
affects: [phase-03-validation, phase-04-sessions, phase-05-detail-charts]

tech-stack:
  added: []
  patterns: [React Router route shell, TanStack Query IPC cache, componentized scan progress panel]

key-files:
  created:
    - src/routes/PortfolioPage.tsx
    - src/routes/ProjectDetailPage.tsx
    - src/routes/SettingsPage.tsx
    - src/components/ProjectCard.tsx
    - src/components/PortfolioHeaderStats.tsx
    - src/components/RightRail.tsx
    - src/components/ScanProgressPanel.tsx
  modified:
    - src/App.tsx
    - src/App.test.tsx
    - src/lib/types.ts
    - src/lib/ipc.ts
    - src/lib/queryClient.ts
    - src/components/ScanRootsEditor.tsx
    - src/styles.css

key-decisions:
  - "React Router BrowserRouter/Routes is used in the app shell, with TanStack Query remaining the owner of IPC state."
  - "Settings saves the full existing settings object and changes only scan roots or hidden IDs; project cache rows are never deleted by hide/unhide UI."
  - "Rebuild cache requires explicit confirmation text and disables duplicate rebuild actions while progress is active."

patterns-established:
  - "Route pages query typed IPC wrappers and invalidate portfolio/project caches after scan, rebuild, or settings changes."
  - "Project cards expose copy as a real button separate from the detail link so copy does not navigate."
  - "Scan progress rendering is shared by portfolio scan and settings rebuild flows."

requirements-completed: [SCAN-02, SCAN-03, SCAN-04, PORT-01, PORT-03, PORT-04, PORT-06, PORT-07, CLIP-01, CLIP-02, DET-01, SET-01, SET-02, SET-04, SET-05]

duration: 21min
completed: 2026-04-25
---

# Phase 03 Plan 04: Routed Portfolio UI Summary

**React Router portfolio vertical slice with real IPC query wrappers, project cards, detail actions, scan/rebuild progress, and settings controls**

## Performance

- **Duration:** 21 min
- **Started:** 2026-04-25T15:50:45Z
- **Completed:** 2026-04-25T16:11:40Z
- **Tasks:** 3
- **Files modified:** 14

## Accomplishments

- Added typed frontend contracts and IPC wrappers for portfolio, project detail, and rebuild cache.
- Replaced the Phase 2 shell with routed `/`, `/project/:id`, and `/settings` pages.
- Added reusable portfolio stats, project cards, right rail, and scan progress components.
- Extended Settings with multiple scan roots, hidden-project unhide, rebuild confirmation, and disabled indexing toggles.
- Updated Vitest coverage for the portfolio/detail/settings behavior and kept the production build green.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add frontend contracts, IPC wrappers, and route tests** - `19c37ee` (test)
2. **Task 2: Implement routed portfolio and detail views** - `e17a2f4` (feat)
3. **Task 3: Implement Settings scan controls and UI contract styling** - `c2f3177` (style)

## Files Created/Modified

- `src/routes/PortfolioPage.tsx` - Portfolio route with stats, initial scan, project grid, progress, and right rail.
- `src/routes/ProjectDetailPage.tsx` - Detail route with project summary and Finder, VS Code, and copy actions.
- `src/routes/SettingsPage.tsx` - Settings route with scan roots, hidden projects, rebuild cache, and disabled indexing toggles.
- `src/components/ProjectCard.tsx` - Focus/hover copy action, progress, phase, parse badge, and detail link.
- `src/components/PortfolioHeaderStats.tsx` - Header stats cells for tracked projects, milestones, sessions, and tokens.
- `src/components/RightRail.tsx` - Hidden projects and unmatched sessions panels.
- `src/components/ScanProgressPanel.tsx` - Shared scan/rebuild progress reducer and panel.
- `src/components/ScanRootsEditor.tsx` - Multiple-root add/remove/save behavior.
- `src/lib/types.ts`, `src/lib/ipc.ts`, `src/lib/queryClient.ts` - Portfolio/detail/rebuild contracts and cache invalidation.
- `src/App.tsx`, `src/App.test.tsx`, `src/styles.css` - Route shell, coverage, and UI contract styling.

## Decisions Made

- Used `BrowserRouter` and declarative `Routes` to keep the app shell simple for this desktop UI.
- Kept copy/open OS integrations behind `src/lib/actions.ts`; the UI never shells out or mutates `.planning/`.
- Used visible disabled checkboxes for Phase 4+ indexing settings instead of hiding future controls.

## Deviations from Plan

None - plan executed as written.

## Issues Encountered

- The RED task intentionally failed before route components existed. The following implementation commits brought the suite green.
- BrowserRouter click navigation is represented in tests by asserting the project card link target and rendering the detail route directly; this avoids jsdom history quirks while still proving `get_project` is called for `/project/:id`.

## Verification

- `npm test -- src/App.test.tsx` - passed (13 tests)
- `npm run build` - passed
- Acceptance grep checks for DTOs, IPC wrappers, query keys, route/component copy, settings labels, max shell width, and 8px progress bars - passed

## Known Stubs

- `src/routes/SettingsPage.tsx` - `Index tool usage` and `Index message content` are intentionally disabled placeholders required by SET-05.
- `src/components/RightRail.tsx` - `Available after session indexing` is an intentional Phase 4 placeholder for unmatched sessions.
- Portfolio stats for `sessionsToday` and `tokensToday` remain backend-provided zeros until Phase 4 session indexing.

## Threat Flags

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 03-05 can validate UI security invariants and release capability coverage against the completed vertical slice. Phase 4 can replace the unmatched-session and session/token placeholders with indexed session data.

## TDD Gate Compliance

- RED commit present: `19c37ee`
- GREEN commit present: `e17a2f4`
- REFACTOR/style commit present: `c2f3177`

## Self-Check: PASSED

- Verified created files exist: `src/routes/PortfolioPage.tsx`, `src/routes/ProjectDetailPage.tsx`, `src/routes/SettingsPage.tsx`, `src/components/ProjectCard.tsx`, `src/components/PortfolioHeaderStats.tsx`, `src/components/RightRail.tsx`, `src/components/ScanProgressPanel.tsx`.
- Verified task commits exist: `19c37ee`, `e17a2f4`, `c2f3177`.
- Verified modified files remain at or below 500 lines; `src/styles.css` is exactly 500 lines.

---
*Phase: 03-portfolio-vertical-slice*
*Completed: 2026-04-25*
