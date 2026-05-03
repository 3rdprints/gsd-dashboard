---
phase: 09-packaging-updater-distribution
plan: 04
subsystem: infra
tags: [release, github-actions, tauri, updater, pages]

requires:
  - phase: 09-packaging-updater-distribution
    provides: Tauri updater configuration, release validators, and update UX
provides:
  - Tag-triggered cross-platform release workflow
  - Updater signing secret gate
  - Updater manifest generation from release artifacts and inline signatures
  - macOS universal app assertion script
affects: [packaging, updater, distribution, release-ci]

tech-stack:
  added: []
  patterns:
    - Node built-in release manifest generation and tests
    - GitHub Actions release matrix reusing the desktop smoke build setup
    - Bash release guards for updater signing and macOS universal binaries

key-files:
  created:
    - .github/workflows/release.yml
    - scripts/release/assert-release-secrets.sh
    - scripts/release/assert-macos-universal.sh
    - scripts/release/generate-updater-manifest.mjs
    - scripts/release/generate-updater-manifest.test.mjs
  modified: []

key-decisions:
  - "Updater manifest publishing fails fast on missing TAURI_SIGNING_PRIVATE_KEY while installer artifacts can still be built without Apple or Windows signing credentials."
  - "The release workflow generates updater metadata from actual Tauri artifact files and their .sig contents rather than handwritten signature values."

patterns-established:
  - "Release scripts use Node and Bash built-ins only, with temporary fixture tests for generated updater metadata."
  - "The release workflow assembles Pages output into site-dist before manifest validation and deployment."

requirements-completed: [PKG-01, PKG-02, PKG-03, PKG-04, UPD-02, UPD-03]

duration: 5 min
completed: 2026-05-03
---

# Phase 09 Plan 04: Release Workflow and Updater Manifest Summary

**Tag-triggered GitHub Actions release packaging with signed updater manifest generation from real artifact signatures**

## Performance

- **Duration:** 5 min
- **Started:** 2026-05-03T14:35:40Z
- **Completed:** 2026-05-03T14:40:25Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- Added an updater publishing secret gate that fails with the required `TAURI_SIGNING_PRIVATE_KEY` message while allowing the optional key password.
- Added a Node-only updater manifest generator and tests proving `.sig` file contents are inlined for `darwin-universal`, `windows-x86_64`, and `linux-x86_64`.
- Added a tag-triggered `release` workflow for macOS universal DMG, Windows MSI/NSIS, Linux deb/AppImage/rpm artifacts, release upload, manifest generation, and Pages deployment.
- Added a macOS universal assertion script that resolves the app executable and verifies both `x86_64` and `arm64` with `lipo -archs`.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add signer secret gate and manifest generator** - `3d0e768` (feat)
2. **Task 2: Add tag-triggered release workflow** - `1fb774f` (feat)

**Plan metadata:** this docs commit

## Files Created/Modified

- `.github/workflows/release.yml` - Tag and manual release workflow with OS matrix builds, artifact collection, updater manifest generation, release upload, and Pages deployment.
- `scripts/release/assert-release-secrets.sh` - Fails updater publishing when `TAURI_SIGNING_PRIVATE_KEY` is empty.
- `scripts/release/assert-macos-universal.sh` - Verifies a macOS `.app` executable contains both Intel and Apple Silicon slices.
- `scripts/release/generate-updater-manifest.mjs` - Generates `latest.json` from artifact filenames and inline `.sig` contents.
- `scripts/release/generate-updater-manifest.test.mjs` - Node test fixtures for signature inlining, missing-signature failure, and validator compatibility.

## Verification

- `node --test scripts/release/generate-updater-manifest.test.mjs` passed.
- `sh -n scripts/release/assert-macos-universal.sh` passed.
- `node scripts/release/verify-release-workflow.mjs --matrix` passed.
- Generated a temporary fixture manifest with `scripts/release/generate-updater-manifest.mjs` and verified it with `node scripts/release/verify-updater-manifest.mjs`.
- All task acceptance grep gates passed.

## Decisions Made

- Kept release automation to the planned workflow and release script files only; update UX and public docs were not modified.
- Used `actions/upload-artifact@v4` / `actions/download-artifact@v4` to carry matrix outputs into the manifest/Pages job before release upload.
- Included the unsigned artifact caveat in generated release notes so unsigned installer fallback is explicit.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Prevented manifest generator CLI execution during tests**
- **Found during:** Task 1 (Add signer secret gate and manifest generator)
- **Issue:** The generator ran its CLI `main()` path when imported by the Node test file, causing `--version is required` before tests could execute.
- **Fix:** Added an ESM direct-invocation guard using `fileURLToPath(import.meta.url)` so tests can import `generateUpdaterManifest()` without running the CLI.
- **Files modified:** `scripts/release/generate-updater-manifest.mjs`
- **Verification:** `node --test scripts/release/generate-updater-manifest.test.mjs` passed.
- **Committed in:** `3d0e768`

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** No scope expansion. The fix was required for testability and correct CLI/module behavior.

## Issues Encountered

- Context7 returned Tauri CLI/updater documentation successfully. The GitHub Actions documentation query failed once and the research retry did not return before implementation completed; existing project validator coverage and plan-specified workflow syntax were used for verification.

## User Setup Required

Before a real updater release, configure `TAURI_SIGNING_PRIVATE_KEY` in GitHub Actions secrets. `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` remains optional and is passed through when set.

## Known Stubs

None.

## Threat Flags

None - workflow token permissions, updater secret handling, manifest generation, unsigned fallback caveat, and stable update channel behavior were covered by the plan threat model.

## Next Phase Readiness

Ready for the remaining Phase 09 Pages/source-bundle distribution plans. The release workflow now has the CI path that later public site files and distribution artifacts can plug into.

## Self-Check: PASSED

- Summary file created.
- Task commits `3d0e768` and `1fb774f` exist.
- Key release workflow and script files exist.

---
*Phase: 09-packaging-updater-distribution*
*Completed: 2026-05-03*
