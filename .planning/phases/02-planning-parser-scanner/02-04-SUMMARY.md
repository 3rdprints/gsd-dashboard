---
phase: 02-planning-parser-scanner
plan: 04
subsystem: parser-ui
tags: [rust, parser, react, tauri-channel, vitest]

requires:
  - phase: 02-planning-parser-scanner
    provides: Scanner service and scan_projects command with typed backend events
provides:
  - Five-fixture ignored parser integration coverage against real local GSD projects
  - Frontend ScanSummary and ScanEvent contracts plus scanProjects Channel wrapper
  - Phase 2 scan status UI with progress, completion, and compact parse-error state
affects: [portfolio-ui, project-detail, scan-settings, parser-hardening]

tech-stack:
  added: []
  patterns:
    - Tauri Channel wrapper creates a typed Channel, assigns onmessage, and invokes scan_projects with onEvent
    - Phase 2 UI keeps scan progress local to App until project cache queries exist
    - Real fixture parser coverage is ignored and run explicitly because it depends on local homegit projects

key-files:
  created:
    - src-tauri/tests/parser_fixtures.rs
  modified:
    - src-tauri/src/parser/plan.rs
    - src-tauri/src/parser/roadmap.rs
    - src/lib/types.ts
    - src/lib/ipc.ts
    - src/App.tsx
    - src/App.test.tsx
    - src/styles.css

key-decisions:
  - "PLAN parsing falls back to raw frontmatter scalar extraction when real-world YAML is malformed."
  - "ROADMAP phase extraction includes headings so inserted decimal phases such as 25.1 and 06.1 remain visible."
  - "The Phase 2 UI displays scan status only; portfolio cards, project detail, rebuild cache, and clipboard actions remain out of scope."

patterns-established:
  - "Frontend scan event fixtures must remain metadata-only and exclude raw markdown/config document bodies."
  - "Scan progress uses aria-live polite text and a compact non-blocking parse-error alert."

requirements-completed: [SCAN-05, PARSE-01, PARSE-02, PARSE-03, PARSE-04, PARSE-05, PARSE-06, PARSE-07, PARSE-08]

duration: 9min
completed: 2026-04-25
---

# Phase 02 Plan 04: Parser Fixtures and Scan Status UI Summary

**Five real GSD fixtures now guard parser tolerance while the Phase 2 shell can invoke scan_projects and render live scan progress plus parse-error summaries**

## Performance

- **Duration:** 9 min
- **Started:** 2026-04-25T01:10:01Z
- **Completed:** 2026-04-25T01:18:58Z
- **Tasks:** 3
- **Files modified:** 8

## Accomplishments

- Added ignored integration coverage for `deckpilot-web`, `listingguru`, `locdirectory`, `getrovix`, and `youtubeauto` real planning fixtures.
- Added typed frontend `ScanSummary`, `ScanEvent`, and `scanProjects` Channel wrapper for the backend `scan_projects` command.
- Added Phase 2 scan status UI with `Scan Projects`, live `aria-live="polite"` progress text, completion state, and compact parse-error alert.

## Task Commits

Each task was committed atomically:

1. **Task 1 RED: Real fixture parser coverage** - `f21fb4a` (test)
2. **Task 1 GREEN: Fixture parser hardening** - `804b274` (feat)
3. **Task 2 RED: Scan IPC wrapper coverage** - `a16e993` (test)
4. **Task 2 GREEN: Typed scan IPC wrapper** - `e1e8480` (feat)
5. **Task 3 RED: Scan status UI coverage** - `d029a57` (test)
6. **Task 3 GREEN: Phase 2 scan status UI** - `fb5497b` (feat)
7. **Task 3 REFACTOR: Test file size cleanup** - `e6efd9b` (refactor)

**Plan metadata:** pending final docs commit.

## Files Created/Modified

- `src-tauri/tests/parser_fixtures.rs` - Ignored integration test over five real `.planning` fixtures.
- `src-tauri/src/parser/plan.rs` - Tolerates malformed PLAN frontmatter while retaining scalar metadata.
- `src-tauri/src/parser/roadmap.rs` - Extracts phase headings so decimal inserted phases remain strings in parsed output.
- `src/lib/types.ts` - Adds frontend scan summary and discriminated scan event types.
- `src/lib/ipc.ts` - Adds `scanProjects` wrapper using Tauri `Channel<ScanEvent>`.
- `src/App.tsx` - Adds scan CTA, local scan state reducer, progress panel, and parse-error alert.
- `src/App.test.tsx` - Adds scan wrapper and UI regression coverage; refactored to stay under 500 lines.
- `src/styles.css` - Adds scan status panel, progress bar, CTA width, and parse-error alert styling.

## Decisions Made

- Used raw PLAN frontmatter fallback instead of rejecting malformed real-world YAML because parser tolerance is required for in-the-wild planning files.
- Kept scan status state local to `App.tsx`; TanStack Query project-cache wiring belongs to later portfolio/project-detail phases.
- Kept parse-error details compact: first project/file only, no modal and no raw document content.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Tolerated malformed PLAN frontmatter in real fixtures**
- **Found during:** Task 1 (Validate real planning fixtures)
- **Issue:** `locdirectory` contains malformed YAML in a PLAN frontmatter block, causing the new real-fixture test to fail.
- **Fix:** Added a fallback that splits frontmatter text and extracts scalar `phase`, `plan`, and `type` values when typed YAML deserialization fails.
- **Files modified:** `src-tauri/src/parser/plan.rs`
- **Verification:** `cd src-tauri && cargo test real_fixture_planning_docs_parse --test parser_fixtures -- --ignored`
- **Committed in:** `804b274`

**2. [Rule 1 - Bug] Parsed ROADMAP phase headings for decimal inserted phases**
- **Found during:** Task 1 (Validate real planning fixtures)
- **Issue:** Decimal phases in `deckpilot-web` and `youtubeauto` appear as headings such as `### Phase 25.1`, while the parser only read checkbox rows.
- **Fix:** Added phase-heading extraction through the existing string-based phase identity parser.
- **Files modified:** `src-tauri/src/parser/roadmap.rs`
- **Verification:** `cd src-tauri && cargo test real_fixture_planning_docs_parse --test parser_fixtures -- --ignored`
- **Committed in:** `804b274`

**3. [Rule 2 - Missing Critical] Kept edited test file under project file-size limit**
- **Found during:** Task 3 (Render Phase 2 scan status surface)
- **Issue:** Added UI coverage pushed `src/App.test.tsx` to 512 lines, violating the project rule that no file exceed 500 lines.
- **Fix:** Extracted repeated frontend IPC mocks into helpers and reduced the file to 430 lines.
- **Files modified:** `src/App.test.tsx`
- **Verification:** `wc -l src/App.test.tsx` and `npm run test -- --run src/App.test.tsx`
- **Committed in:** `e6efd9b`

---

**Total deviations:** 3 auto-fixed (2 Rule 1 bugs, 1 Rule 2 missing critical).  
**Impact on plan:** All fixes were required for real fixture correctness or project constraints; no Phase 3 UI scope was added.

## Issues Encountered

- React Testing Library reported duplicate status text for header and panel states; tests were adjusted to assert the repeated text intentionally.
- Fast mocked `scan_projects` completion could overwrite richer event counts; the UI now preserves the maximum discovered/error counts seen from events and summaries.

## Known Stubs

None.

## User Setup Required

None - no external service configuration required.

## Threat Flags

None - the only trust-boundary surface added is the planned frontend command invocation and metadata-only event consumption.

## Verification

- `cd src-tauri && cargo test real_fixture_planning_docs_parse --test parser_fixtures -- --ignored` - PASSED
- `npm run test -- --run src/App.test.tsx` - PASSED
- `npm run build` - PASSED
- Task 1 acceptance greps for all five fixture names, `catch_unwind`, and `#[ignore]` - PASSED
- Task 2 acceptance greps for `ScanEvent`, `ScanSummary`, `scan_projects`, `projectParseError`, and `finished` - PASSED
- Task 3 acceptance greps for `Scan Projects`, `aria-live="polite"`, parse-error copy, `scan-status`, and absence of Phase 3 strings in `src/App.tsx` - PASSED
- Stub scan for `TODO`, `FIXME`, placeholder text, raw empty UI values, and hardcoded empty scan surfaces - PASSED

## Next Phase Readiness

Phase 2 can now be verified end-to-end: real planning parser fixtures run without panics, frontend IPC can start scans through the backend stream, and the shell exposes progress/error status without waiting for Phase 3 portfolio cards.

## Self-Check: PASSED

- Summary file exists at `.planning/phases/02-planning-parser-scanner/02-04-SUMMARY.md`.
- Key created file exists: `src-tauri/tests/parser_fixtures.rs`.
- Key modified files exist: `src-tauri/src/parser/plan.rs`, `src-tauri/src/parser/roadmap.rs`, `src/lib/types.ts`, `src/lib/ipc.ts`, `src/App.tsx`, `src/App.test.tsx`, and `src/styles.css`.
- Task commits found: `f21fb4a`, `804b274`, `a16e993`, `e1e8480`, `d029a57`, `fb5497b`, and `e6efd9b`.
- Shared state files were intentionally not updated because wave/worktree orchestration owns `STATE.md` and `ROADMAP.md` writes.

---
*Phase: 02-planning-parser-scanner*
*Completed: 2026-04-25*
