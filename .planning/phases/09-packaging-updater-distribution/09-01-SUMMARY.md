---
phase: 09-packaging-updater-distribution
plan: 01
subsystem: infra
tags: [release, tauri, updater, github-pages, validation]

requires:
  - phase: 08-autostart-os-polish
    provides: startup behavior and OS integration context for packaged builds
provides:
  - Wave 0 release workflow static validator
  - Wave 0 Tauri bundle/updater config validator
  - Wave 0 updater manifest signature validator
  - Wave 0 GitHub Pages install-surface validator
affects: [packaging, updater, distribution, release-ci]

tech-stack:
  added: []
  patterns: [Node built-in release validators with self-test fixtures]

key-files:
  created:
    - scripts/release/verify-release-workflow.mjs
    - scripts/release/verify-tauri-config.mjs
    - scripts/release/verify-updater-manifest.mjs
    - scripts/release/verify-pages-site.mjs
  modified:
    - package.json

key-decisions:
  - "Wave 0 validators use Node built-ins only and self-test with temporary fixtures so later Phase 09 plans can depend on executable gates before release artifacts exist."

patterns-established:
  - "Release validation scripts expose --self-test fixtures for valid and invalid cases."
  - "Package scripts under release:verify-* provide stable gates for later packaging/updater plans."

requirements-completed: [PKG-01, PKG-02, PKG-03, PKG-04, PKG-05, PKG-06, UPD-01, UPD-02, UPD-03, UPD-04, UPD-05]

duration: 3 min
completed: 2026-05-03
---

# Phase 09 Plan 01: Wave 0 Validation Harness Summary

**Node built-in release, updater, manifest, and Pages validators with fixture self-tests for Phase 09 packaging gates**

## Performance

- **Duration:** 3 min
- **Started:** 2026-05-03T14:10:25Z
- **Completed:** 2026-05-03T14:13:35Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- Added release workflow validation for tag triggers, least-privilege permissions, platform matrix, signing gates, unsigned caveats, and Pages deployment actions.
- Added Tauri config validation for bundle targets, updater artifact generation, public key, HTTPS manifest endpoint, and passive Windows install mode.
- Added updater manifest and Pages install-surface validators that enforce signed platform entries, `/updates/latest.json`, native installer copy, cargo fallback copy, and prompt-by-default install script text.
- Wired `release:verify-workflow`, `release:verify-tauri-config`, `release:verify-manifest`, and `release:verify-pages` package scripts.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add release workflow and Tauri config validators** - `4bacbea` (feat)
2. **Task 2: Add updater manifest and Pages validators** - `70cb10c` (feat)

**Plan metadata:** this docs commit

## Files Created/Modified

- `scripts/release/verify-release-workflow.mjs` - Static release workflow validator with `--self-test` valid/invalid YAML fixtures.
- `scripts/release/verify-tauri-config.mjs` - Tauri bundle/updater config validator with `--self-test` JSON fixtures.
- `scripts/release/verify-updater-manifest.mjs` - Updater manifest validator requiring supported platform URLs and inline signatures.
- `scripts/release/verify-pages-site.mjs` - GitHub Pages HTML/install script validator for install and updater-surface copy.
- `package.json` - Adds the four `release:verify-*` package scripts.

## Verification

- `node --check` passed for all four validators.
- `node scripts/release/verify-release-workflow.mjs --self-test` passed.
- `node scripts/release/verify-tauri-config.mjs --self-test` passed.
- `node scripts/release/verify-updater-manifest.mjs --self-test` passed.
- `node scripts/release/verify-pages-site.mjs --self-test` passed.
- All task acceptance grep and `npm pkg get` gates passed.

Initial validation against real release workflow, Tauri updater config, site, and manifest artifacts is intentionally deferred until later Phase 09 plans create those artifacts.

## Decisions Made

- Node built-ins only were used for all Wave 0 validators, avoiding new dependencies for release gates.
- Validators fail default real-artifact checks when artifacts are missing or incomplete, while `--self-test` proves the validator logic independently.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Known Stubs

None.

## Next Phase Readiness

Ready for Plan 09-02. Later release, updater, source bundle, cargo package, and Pages plans can now call executable Wave 0 gates instead of relying on manual inspection.

## Self-Check: PASSED

- Summary file created.
- Task commits `4bacbea` and `70cb10c` exist.
- Key validator files exist.

---
*Phase: 09-packaging-updater-distribution*
*Completed: 2026-05-03*
