---
phase: 05-project-detail-global-sessions-charts
plan: 12
subsystem: ui
tags: [react, typescript, tanstack-query, heatmap, portfolio]

requires:
  - phase: 05-project-detail-global-sessions-charts
    provides: Daily activity backend command and DailyActivityUpdated event from Plans 05-06 and 05-07
provides:
  - Portfolio 90-day activity heatmap backed by get_portfolio_heatmap
  - Daily activity event invalidation for the heatmap query
  - Heatmap bucket, tooltip, and stylesheet guard tests
affects: [portfolio-heatmap, frontend-ipc, app-events]

tech-stack:
  added: [react-calendar-heatmap, "@types/react-calendar-heatmap"]
  patterns: [typed Tauri invoke wrapper, TanStack Query key invalidated by app event, local CSS heatmap bucket classes]

key-files:
  created:
    - src/components/charts/ActivityHeatmap.tsx
    - src/lib/portfolioHeatmapContracts.test.ts
  modified:
    - package.json
    - package-lock.json
    - src/lib/types.ts
    - src/lib/ipc.ts
    - src/lib/queryClient.ts
    - src/lib/appListeners.ts
    - src/routes/PortfolioPage.tsx
    - src/components/charts/ActivityHeatmap.test.tsx
    - src/styles.css

key-decisions:
  - "Portfolio heatmap uses react-calendar-heatmap with only local CSS classes; the package stylesheet remains unimported."
  - "DailyActivityUpdated invalidates only portfolioHeatmapQueryKey; no event payload is trusted."

patterns-established:
  - "Heatmap display logic is tested through exported bucket and title helpers plus rendered zero-state cells."

requirements-completed: [PORT-05]

duration: 4min
completed: 2026-04-27
---

# Phase 05 Plan 12: Portfolio Activity Heatmap Summary

**Portfolio now shows a 90-day session activity heatmap with local bucket styling, typed IPC/query wiring, and daily activity invalidation.**

## Performance

- **Duration:** 4 min
- **Started:** 2026-04-27T15:35:38Z
- **Completed:** 2026-04-27T15:40:04Z
- **Tasks:** 2
- **Files modified:** 10

## Accomplishments

- Installed `react-calendar-heatmap@1.10.0` and added the typed `getPortfolioHeatmap(90)` frontend contract.
- Added `ActivityHeatmap` with 0/1/2-3/4-7/8-14/15+ bucket mapping and token/project tooltip text.
- Rendered the heatmap below Portfolio header stats and invalidated it on `daily_activity_updated`.

## Task Commits

1. **Task 1 RED: Heatmap frontend contract test** - `a7f22f6` (test)
2. **Task 1 GREEN: Heatmap dependency and IPC/query contracts** - `3ee2296` (feat)
3. **Task 2 RED: ActivityHeatmap behavior tests** - `e829f88` (test)
4. **Task 2 GREEN: Portfolio heatmap UI and invalidation** - `13f2da6` (feat)

## Files Created/Modified

- `src/components/charts/ActivityHeatmap.tsx` - Calendar heatmap component, bucket mapper, and tooltip formatter.
- `src/components/charts/ActivityHeatmap.test.tsx` - Replaced scaffold with bucket, tooltip, zero-grid, and stylesheet guard tests.
- `src/lib/portfolioHeatmapContracts.test.ts` - IPC wrapper and query key contract test.
- `src/lib/types.ts`, `src/lib/ipc.ts`, `src/lib/queryClient.ts` - Heatmap DTO, invoke wrapper, and query key.
- `src/lib/appListeners.ts` - `daily_activity_updated` listener invalidating `portfolioHeatmapQueryKey`.
- `src/routes/PortfolioPage.tsx` - Heatmap query and placement below stats.
- `src/styles.css` - Local heatmap wrapper, container, skeleton, and six bucket classes.
- `package.json`, `package-lock.json` - Heatmap runtime package and TypeScript definitions.

## Decisions Made

Used `@types/react-calendar-heatmap` as a dev dependency because the runtime package does not ship TypeScript declarations and `npx tsc --noEmit` must remain strict.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added missing TypeScript declarations**
- **Found during:** Task 2 (component implementation)
- **Issue:** `npx tsc --noEmit` failed because `react-calendar-heatmap` has no bundled declaration file.
- **Fix:** Added `@types/react-calendar-heatmap@1.9.0` as a dev dependency.
- **Files modified:** `package.json`, `package-lock.json`
- **Verification:** `npm test -- --run src/components/charts/ActivityHeatmap.test.tsx && npx tsc --noEmit && npm run build` passed.
- **Committed in:** `13f2da6`

---

**Total deviations:** 1 auto-fixed (Rule 3)
**Impact on plan:** Required for strict TypeScript compilation; runtime chart dependency remains exactly `react-calendar-heatmap@1.10.0`.

## Issues Encountered

- The package stylesheet grep initially matched the literal guard string in the test itself; the test now constructs that forbidden path in two parts while still checking the component source.

## Known Stubs

None introduced by this plan.

## User Setup Required

None - no external service configuration required.

## Threat Flags

| Flag | File | Description |
|------|------|-------------|
| threat_flag: third-party-ui-dependency | `src/components/charts/ActivityHeatmap.tsx` | New heatmap package renders SVG cells; package stylesheet is not imported and local classes own visual styling. |
| threat_flag: app-event-invalidation | `src/lib/appListeners.ts` | Empty `daily_activity_updated` event triggers cache invalidation only; no payload data is trusted. |

## TDD Gate Compliance

- RED commits present: `a7f22f6`, `e829f88`
- GREEN commits present after RED: `3ee2296`, `13f2da6`
- No separate refactor commit was needed.

## Verification

- `npm test -- --run src/components/charts/ActivityHeatmap.test.tsx src/lib/portfolioHeatmapContracts.test.ts` - passed.
- `npx tsc --noEmit` - passed.
- `npm run build` - passed with the existing Vite chunk-size warning.
- Task 1 acceptance greps for `react-calendar-heatmap@1.10.x` and no package stylesheet import passed.
- Task 2 acceptance greps for six bucket classes and `daily_activity_updated` / `portfolioHeatmapQueryKey` passed.

## Next Phase Readiness

PORT-05 is complete. Phase 6 can rely on Portfolio showing session activity once session indexing populates `daily_activity`.

## Self-Check: PASSED

- Confirmed summary and key created files exist.
- Confirmed task commits exist: `a7f22f6`, `3ee2296`, `e829f88`, `13f2da6`.

---
*Phase: 05-project-detail-global-sessions-charts*
*Completed: 2026-04-27*
