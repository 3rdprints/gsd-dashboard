---
phase: 05-project-detail-global-sessions-charts
plan: 04
subsystem: backend
tags: [rust, sqlite, parser, scanner, tdd]

requires:
  - phase: 05-project-detail-global-sessions-charts
    provides: Phase 5 plan_items schema and backend scaffold tests from Plans 05-01 and 05-02
provides:
  - PLAN.md checklist parsing with ordinals and source line numbers
  - SQLite plan_items replace/load lifecycle keyed by project_id and plan_path
  - phase_plans.completed_at updates from indexed checklist completion
  - STATE Current Position excerpt extraction persisted in the project parsed blob
affects: [project-detail, overview, plan-checklist, state-excerpt]

tech-stack:
  added: []
  patterns: [derived SQLite cache rows, scan persistence helper module, parser pure functions]

key-files:
  created:
    - src-tauri/src/scan_persistence.rs
  modified:
    - src-tauri/src/parser/plan.rs
    - src-tauri/src/parser/state.rs
    - src-tauri/src/parser/mod.rs
    - src-tauri/src/parser/roadmap.rs
    - src-tauri/src/store/project_repo.rs
    - src-tauri/src/scan_service.rs
    - src-tauri/src/lib.rs
    - src-tauri/tests/plan_items_index.rs
    - src-tauri/tests/state_excerpt.rs

key-decisions:
  - "PhasePlan now carries the discovered PLAN.md path so phase_plans and plan_items use the same stable key."
  - "STATE excerpts are stored inside the derived ProjectSnapshot parsed blob rather than writing back to source markdown."

patterns-established:
  - "Plan item indexing is replace-on-scan and writes only derived SQLite rows."
  - "Scan persistence conversions live outside scan_service.rs to keep scanner orchestration focused."

requirements-completed: [DET-03]

duration: 8min
completed: 2026-04-27
---

# Phase 05 Plan 04: Project Detail Checklist and STATE Excerpt Summary

**PLAN.md checklist rows and Current Position STATE excerpts now flow from parser to SQLite-derived cache without mutating `.planning/` source files.**

## Performance

- **Duration:** 8 min
- **Started:** 2026-04-27T14:14:40Z
- **Completed:** 2026-04-27T14:22:27Z
- **Tasks:** 1
- **Files modified:** 10

## Accomplishments

- Replaced Plan 05-02 DET-03 scaffolds with behavior tests for checklist parsing, scan persistence, repository replacement/loading, completed_at updates, and STATE excerpt capping.
- Added `PlanItem` parsing with 0-based ordinals and 1-based source line numbers for `- [ ]`, `- [x]`, and `- [X]` markdown task rows.
- Persisted derived plan checklist rows to SQLite after `phase_plans` upsert and logged per-plan persistence failures without writing to discovered `.planning/` files.
- Extracted safe Current Position markdown excerpts with 20-line / 2048-byte caps and fallback behavior.

## Task Commits

1. **Task 1 RED: Failing DET-03 parser/repository/scan tests** - `8140eb9` (test)
2. **Task 1 GREEN: Plan item and STATE excerpt backend lifecycle** - `3b479e5` (feat)

## Files Created/Modified

- `src-tauri/src/scan_persistence.rs` - Derived-cache persistence helpers for project snapshots, phase plans, plan items, and scan logs.
- `src-tauri/src/parser/plan.rs` - Added `PlanItem` and `parse_plan_items_with_lines`.
- `src-tauri/src/parser/state.rs` - Added `extract_state_excerpt`.
- `src-tauri/src/parser/mod.rs` - Extended project/phase plan parsed contracts with plan paths, plan items, and state excerpts.
- `src-tauri/src/store/project_repo.rs` - Added plan item replace/load helpers and completed_at update helper.
- `src-tauri/src/scan_service.rs` - Carries discovered plan paths and parsed state excerpts through scan output.
- `src-tauri/tests/plan_items_index.rs` - Covers parser, repository, and scan integration lifecycle.
- `src-tauri/tests/state_excerpt.rs` - Covers Current Position extraction and fallback capping.

## Decisions Made

Used discovered PLAN.md paths as the `plan_path` key instead of synthetic phase/plan names so `phase_plans` and `plan_items` share a stable row identity. STATE excerpt text is stored as derived JSON in the existing project snapshot blob, preserving the read-only `.planning/` invariant.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Kept touched scanner module within AGENTS.md file-size rule**
- **Found during:** Task 1 (scan persistence wiring)
- **Issue:** Adding persistence lifecycle code directly to `scan_service.rs` would expand an already oversized touched file.
- **Fix:** Moved project snapshot, phase plan, and plan item persistence conversions into `src-tauri/src/scan_persistence.rs`.
- **Files modified:** `src-tauri/src/scan_service.rs`, `src-tauri/src/scan_persistence.rs`, `src-tauri/src/lib.rs`
- **Verification:** `wc -l src-tauri/src/scan_service.rs src-tauri/src/scan_persistence.rs src-tauri/src/store/project_repo.rs` keeps touched modules under 500 lines; plan tests pass.
- **Committed in:** `3b479e5`

---

**Total deviations:** 1 auto-fixed (Rule 2)
**Impact on plan:** The split preserves the planned behavior and project constraints without adding user-facing scope.

## Issues Encountered

- `cd src-tauri && cargo test` still stops at the pre-existing Plan 05-06 scaffold `tests/daily_activity_rebuild.rs::daily_activity_rebuild_is_idempotent_and_emits_event`. This is out of scope for Plan 05-04 and was logged in `deferred-items.md`.

## Known Stubs

None introduced by this plan.

## Threat Flags

| Flag | File | Description |
|------|------|-------------|
| threat_flag: derived-cache-write | `src-tauri/src/scan_persistence.rs` | New SQLite write lifecycle for plan_items and phase_plans.completed_at at the parser/cache boundary; covered by T-05-04-02 read-only invariant verification. |

## User Setup Required

None - no external service configuration required.

## Verification

- `cd src-tauri && cargo test --test plan_items_index --test state_excerpt` - passed.
- `cd src-tauri && cargo test --lib` - passed.
- `grep -nE 'pub struct PlanItem|parse_plan_items_with_lines' src-tauri/src/parser/plan.rs` - passed.
- `grep -nE 'extract_state_excerpt' src-tauri/src/parser/state.rs` - passed.
- `grep -nE 'replace_plan_items|set_plan_completed_at_if_all_checked' src-tauri/src/store/project_repo.rs src-tauri/src/scan_service.rs src-tauri/src/scan_persistence.rs` - passed.
- `grep -rnE 'fs::write|File::create|create_dir' src-tauri/src/scan_service.rs src-tauri/src/parser src-tauri/src/store/project_repo.rs src-tauri/src/scan_persistence.rs | grep -v '^#' | grep -i '.planning' | wc -l` - returned `0`.

## TDD Gate Compliance

- RED commit present: `8140eb9`
- GREEN commit present after RED: `3b479e5`
- Refactor was included in GREEN because the module split was required before the first passing implementation commit.

## Next Phase Readiness

Plan 05-05 can query checklist rows from SQLite and use the parsed project snapshot for safe STATE excerpt text. The dashboard still does not mutate discovered `.planning/` source files.

## Self-Check: PASSED

- Confirmed summary and key created file exist.
- Confirmed task commits exist: `8140eb9`, `3b479e5`.
- Confirmed plan verification and acceptance gates passed.

---
*Phase: 05-project-detail-global-sessions-charts*
*Completed: 2026-04-27*
