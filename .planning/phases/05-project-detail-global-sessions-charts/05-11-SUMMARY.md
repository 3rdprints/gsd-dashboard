---
phase: 05-project-detail-global-sessions-charts
plan: 11
subsystem: ui
tags: [react, typescript, recharts, tanstack-query, sessions, charts]

requires:
  - phase: 05-project-detail-global-sessions-charts
    provides: Global Sessions backend chart DTOs from Plan 05-07 and URL-backed filters from Plan 05-10
provides:
  - Filter-aligned Global Sessions chart IPC/query contract
  - Four Recharts Global Sessions chart components
  - Global Sessions page chart block above the table
affects: [global-sessions, frontend-ipc, charts]

tech-stack:
  added: []
  patterns: [Recharts stacked bar components with React-rendered legend chips, URL-derived filters reused by table and chart queries]

key-files:
  created:
    - src/components/charts/StackedSourcesChart.tsx
    - src/components/charts/StackedProjectsChart.tsx
    - src/components/charts/TimeOfDayHistogram.tsx
    - src/components/charts/DayOfWeekChart.tsx
  modified:
    - src/lib/types.ts
    - src/lib/ipc.ts
    - src/lib/queryClient.ts
    - src/routes/GlobalSessionsPage.tsx
    - src/routes/GlobalSessionsPage.css
    - src/components/charts/GlobalCharts.test.tsx

key-decisions:
  - "Global chart queries reuse the same filtersToGlobalSessionFilters output as the table query."
  - "Project chart legend text is rendered as React text children, not raw HTML."
  - "Global chart card and legend styles live in GlobalSessionsPage.css because the shared chart CSS was route-scoped to ProjectDetailPage.css."

patterns-established:
  - "Global chart components accept backend DTO arrays directly and normalize only display buckets needed by the chart."
  - "Chart tests mock Recharts primitives to assert data keys, stack IDs, colors, and bucket counts without relying on SVG layout in jsdom."

requirements-completed: [GLOB-03]

duration: 5min
completed: 2026-04-27
---

# Phase 05 Plan 11: Global Sessions Charts Summary

**Global Sessions now renders four filter-aligned Recharts chart cards above the table: stacked source volume, stacked project tokens, 24-hour histogram, and Mon-first weekday distribution.**

## Performance

- **Duration:** 5 min
- **Started:** 2026-04-27T15:27:26Z
- **Completed:** 2026-04-27T15:32:55Z
- **Tasks:** 2
- **Files modified:** 10

## Accomplishments

- Added `getGlobalChartData(filters)` and `globalChartsQueryKey(filters)` using the same active filter DTO as `listGlobalSessions`.
- Replaced the Global Sessions chart placeholder with four stable `ChartCard` panels above the sessions table.
- Implemented Recharts charts for source stacks, top-project token stacks plus Other, 24 hour buckets, and Mon-first weekday buckets.
- Replaced the scaffold chart test with executable coverage for query wiring, stack IDs/colors, bucket normalization, empty state, and React-escaped legend text.

## Task Commits

1. **Task 1 RED: Global chart IPC/query/page wiring tests** - `da5b21d` (test)
2. **Task 1 GREEN: Global chart query contract and page cards** - `516de9f` (feat)
3. **Task 2 RED: Global chart component behavior tests** - `6e7b510` (test)
4. **Task 2 GREEN: Global chart components** - `edf857a` (feat)

## Files Created/Modified

- `src/components/charts/StackedSourcesChart.tsx` - Stacked Claude/Codex sessions-per-day chart.
- `src/components/charts/StackedProjectsChart.tsx` - Stacked top-project token chart with Other bucket.
- `src/components/charts/TimeOfDayHistogram.tsx` - 24-bucket hourly histogram with 4-hour ticks.
- `src/components/charts/DayOfWeekChart.tsx` - Seven-bucket weekday chart displayed Monday first.
- `src/components/charts/GlobalCharts.test.tsx` - Global chart contract and component behavior tests.
- `src/lib/types.ts`, `src/lib/ipc.ts`, `src/lib/queryClient.ts` - Global chart DTOs, IPC wrapper, and TanStack query key.
- `src/routes/GlobalSessionsPage.tsx` - Filter-aligned chart query and chart block above the table.
- `src/routes/GlobalSessionsPage.css` - Global chart card, grid, and legend styles.

## Decisions Made

Global charts reuse the table's `ipcFilters` value instead of deriving a separate chart filter. This keeps D-17 intact: changing URL filters updates both table and charts through their own TanStack queries.

Project names in chart legends are rendered as React text children. This satisfies the chart trust boundary without introducing sanitization code or raw HTML handling.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Added route-local chart card styles for Global Sessions**
- **Found during:** Task 1 (page wiring)
- **Issue:** Existing chart card styles were imported by Project Detail only, so the Global Sessions route could render unstyled chart cards if visited directly.
- **Fix:** Replaced the placeholder CSS with Global Sessions chart card, grid, skeleton, empty, and legend styles.
- **Files modified:** `src/routes/GlobalSessionsPage.css`
- **Verification:** `npm test -- --run src/components/charts/GlobalCharts.test.tsx && npx tsc --noEmit` passed.
- **Committed in:** `516de9f`, `edf857a`

**2. [Rule 1 - Bug] Guarded optional chart DTO arrays during route rendering**
- **Found during:** Task 1 GREEN verification
- **Issue:** A mocked chart response without final DTO field names caused the page to read `.length` from `undefined`.
- **Fix:** Used optional property access and empty array fallbacks before rendering chart components.
- **Files modified:** `src/routes/GlobalSessionsPage.tsx`
- **Verification:** `GlobalCharts.test.tsx` passed after the fix.
- **Committed in:** `516de9f`

---

**Total deviations:** 2 auto-fixed (Rule 1: 1, Rule 2: 1)  
**Impact on plan:** Both fixes were required for stable direct route rendering and robust query state handling. No feature scope was added.

## Issues Encountered

None beyond the auto-fixed issues above.

## Known Stubs

None introduced by this plan.

## Threat Flags

| Flag | File | Description |
|------|------|-------------|
| threat_flag: chart-project-name-rendering | `src/components/charts/StackedProjectsChart.tsx` | Backend project names are displayed in chart legend chips; they are rendered only as React text children. |
| threat_flag: global-chart-ipc | `src/lib/ipc.ts` | Global chart query exposes existing backend aggregate command through the frontend IPC wrapper. |

## TDD Gate Compliance

- RED commit present for Task 1: `da5b21d`
- GREEN commit present after Task 1 RED: `516de9f`
- RED commit present for Task 2: `6e7b510`
- GREEN commit present after Task 2 RED: `edf857a`
- No separate refactor commit was needed.

## User Setup Required

None - no external service configuration required.

## Verification

- `npm test -- --run src/components/charts/GlobalCharts.test.tsx && npx tsc --noEmit` - passed.
- Task 1 acceptance grep for `getGlobalChartData|globalChartsQueryKey` in `src/lib/ipc.ts`, `src/lib/queryClient.ts`, and `src/routes/GlobalSessionsPage.tsx` - passed.
- Task 2 acceptance grep for `stackId="src"` in `StackedSourcesChart.tsx` and `stackId="projects"` in `StackedProjectsChart.tsx` - passed.
- `GlobalCharts.test.tsx` asserts source/project stacked series, 24 hour buckets, 7 weekday buckets, empty state, and React-escaped legend text.

## Next Phase Readiness

GLOB-03 is complete. Plan 05-12 can proceed with the portfolio heatmap without needing more Global Sessions chart wiring.

## Self-Check: PASSED

- Confirmed summary and all four chart component files exist.
- Confirmed task commits exist: `da5b21d`, `516de9f`, `6e7b510`, `edf857a`.
- Confirmed plan verification and acceptance gates passed.

---
*Phase: 05-project-detail-global-sessions-charts*
*Completed: 2026-04-27*
