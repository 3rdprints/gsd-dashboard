---
phase: 09-packaging-updater-distribution
plan: 05
subsystem: distribution
tags: [github-pages, install-script, updater, tauri, static-site]

requires:
  - phase: 09-packaging-updater-distribution
    provides: release workflow Pages assembly and release validators
provides:
  - GitHub Pages product/install landing page
  - Prompt-by-default platform-aware install script
  - Generated bitmap screenshot-style dashboard visual
affects: [packaging, updater, distribution, public-docs]

tech-stack:
  added: []
  patterns:
    - Static docs/public Pages source consumed by release workflow site-dist assembly
    - POSIX shell installer with allowlisted OS/arch selection and default confirmation

key-files:
  created:
    - docs/public/index.html
    - docs/public/install.sh
    - docs/public/assets/gsd-dashboard-screenshot.png
  modified: []

key-decisions:
  - "Native installers remain the primary install path; cargo install is documented as a developer fallback with caveats."
  - "The install script derives download and manual URLs from GSD_DASHBOARD_BASE_URL so release/staging Pages roots can be swapped without editing the script."

patterns-established:
  - "Pages install surfaces expose /updates/latest.json as the stable updater manifest debug link."
  - "Installer scripts read interactive confirmation from /dev/tty so curl-piped script input is not consumed as an answer."

requirements-completed: [UPD-04, UPD-05, PKG-02, PKG-03, PKG-04, PKG-06]

duration: 4 min
completed: 2026-05-03
---

# Phase 09 Plan 05: GitHub Pages Install Surface Summary

**Install-focused GitHub Pages source with native platform downloads, updater manifest visibility, and a cautious prompt-by-default installer**

## Performance

- **Duration:** 4 min
- **Started:** 2026-05-03T14:43:15Z
- **Completed:** 2026-05-03T14:47:32Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Added a polished static `docs/public/index.html` product page with first-viewport install clarity, native installer links, the required `curl -fsSL ... | sh` command, unsigned artifact caveat, cargo fallback caveat, source bundle link, and `/updates/latest.json` debug link.
- Added a generated 1280x800 PNG screenshot-style dashboard visual showing cards, progress bars, and activity charts.
- Added `docs/public/install.sh` with OS/arch detection, native artifact selection, `GSD_DASHBOARD_BASE_URL` staging override, default confirmation, `--yes`, quoted variables, and no `eval`.

## Task Commits

Each task was committed atomically:

1. **Task 1: Create install-focused product page** - `e3cfe39` (feat)
2. **Task 2: Create cautious platform-aware install script** - `adcaac6` (feat)

**Plan metadata:** this docs commit

## Files Created/Modified

- `docs/public/index.html` - Static GitHub Pages landing page with installer CTAs, screenshots, caveats, cargo fallback, source bundle, and updater manifest link.
- `docs/public/install.sh` - Prompt-by-default installer script with OS/arch allowlists and base URL override support.
- `docs/public/assets/gsd-dashboard-screenshot.png` - Generated bitmap screenshot-style visual asset for the first viewport and screenshots section.

## Verification

- `grep -q '<h1>GSD Dashboard</h1>' docs/public/index.html` passed.
- `grep -q 'Download for macOS' docs/public/index.html` passed.
- `grep -q 'curl -fsSL https://smacdonald.github.io/gsd-dashboard/install.sh | sh' docs/public/index.html` passed.
- `grep -q 'Updater manifest' docs/public/index.html` passed.
- `grep -q '/updates/latest.json' docs/public/index.html` passed.
- `grep -q 'cargo install gsd-dashboard' docs/public/index.html` passed.
- `test -s docs/public/assets/gsd-dashboard-screenshot.png` passed.
- `grep -q 'set -euo pipefail' docs/public/install.sh` passed.
- `grep -q -- '--yes' docs/public/install.sh` passed.
- `grep -q 'uname -s' docs/public/install.sh` passed.
- `grep -q 'uname -m' docs/public/install.sh` passed.
- `grep -Fq 'Install `${artifact}` for `${os}/${arch}`?' docs/public/install.sh` passed.
- `grep -q 'curl -fsSL' docs/public/install.sh` passed.
- `grep -q 'GSD_DASHBOARD_BASE_URL' docs/public/install.sh` passed.
- `! grep -q 'eval ' docs/public/install.sh` passed.
- `sh -n docs/public/install.sh` passed.
- `node scripts/release/verify-pages-site.mjs` passed.
- `npm test` was not run because no frontend application source was touched.

## Decisions Made

- Kept the public install page as plain static HTML/CSS in `docs/public` so the existing release workflow can copy it directly into `site-dist`.
- Used Pages `/downloads/<artifact>` URLs for script downloads because Plan 09-04 assembles release artifacts into `site-dist/downloads`.
- Preserved the planned `curl ... | sh` command while making the installer read confirmation from `/dev/tty`, which keeps prompts interactive instead of reading from the piped script body.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Made curl-piped installer confirmation read from the terminal**
- **Found during:** Task 2 (Create cautious platform-aware install script)
- **Issue:** A prompt in a `curl | sh` script can accidentally read from the piped script content instead of the user's terminal.
- **Fix:** Read confirmation from `/dev/tty` by default and require `--yes` for noninteractive installs without a terminal.
- **Files modified:** `docs/public/install.sh`
- **Verification:** `sh -n docs/public/install.sh`, task acceptance greps, and `node scripts/release/verify-pages-site.mjs` passed.
- **Committed in:** `adcaac6`

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** The fix preserves the planned prompt-by-default behavior and avoids unsafe noninteractive confirmation behavior. No scope expansion.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Known Stubs

None. The empty shell variable initializers in `docs/public/install.sh` are runtime state placeholders for detected OS/arch/artifact, not UI stubs.

## Threat Flags

None - the public download page and `curl | sh` installer surface were covered by the plan threat model, and mitigations were implemented with HTTPS URLs, allowlisted OS/arch selection, quoted variables, no `eval`, selected-artifact output, and default confirmation.

## Next Phase Readiness

Ready for Plan 09-06. The release workflow can now assemble `docs/public` into Pages output with both required source files present, and the Pages validator passes against the real source files.

## Self-Check: PASSED

- Summary file created.
- Task commits `e3cfe39` and `adcaac6` exist.
- Key files `docs/public/index.html`, `docs/public/install.sh`, and `docs/public/assets/gsd-dashboard-screenshot.png` exist.
- Plan-level verification commands passed.

---
*Phase: 09-packaging-updater-distribution*
*Completed: 2026-05-03*
