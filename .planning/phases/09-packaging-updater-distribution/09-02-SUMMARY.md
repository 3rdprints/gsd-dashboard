---
phase: 09-packaging-updater-distribution
plan: 02
subsystem: packaging
tags: [tauri, updater, process, signing, distribution]

requires:
  - phase: 09-packaging-updater-distribution
    provides: Wave 0 release and Tauri updater config validators
provides:
  - Official Tauri updater and process plugin dependencies
  - Bundle/updater artifact configuration with stable GitHub Pages endpoint
  - Runtime updater/process plugin registration with least release capabilities
affects: [packaging, updater, release-ci, distribution]

tech-stack:
  added:
    - "@tauri-apps/plugin-updater 2.10.1"
    - "@tauri-apps/plugin-process 2.3.1"
    - "tauri-plugin-updater 2.10.1"
    - "tauri-plugin-process 2.3.1"
  patterns:
    - Thin Tauri plugin registration helpers wrap Builder chains before app setup
    - Static updater trust is configured through tauri.conf.json pubkey and a single HTTPS manifest endpoint

key-files:
  created:
    - src-tauri/src/updater.rs
  modified:
    - package.json
    - package-lock.json
    - src-tauri/Cargo.toml
    - src-tauri/Cargo.lock
    - src-tauri/tauri.conf.json
    - src-tauri/src/lib.rs
    - src-tauri/src/main.rs
    - src-tauri/capabilities/default.json

key-decisions:
  - "Use Tauri's official updater/process plugins and built-in signature verification; no custom updater client or crypto path was added."
  - "Grant updater:default plus process:allow-restart rather than process:default so frontend update flows can relaunch without exposing process exit."
  - "Generate the updater keypair locally, commit only the public key, and keep private material outside the repository."

patterns-established:
  - "src-tauri/src/updater.rs owns updater/process plugin registration and returns the builder for main.rs composition."
  - "Updater config points only to https://smacdonald.github.io/gsd-dashboard/updates/latest.json for the stable channel."

requirements-completed: [UPD-01, UPD-02, UPD-03, PKG-02, PKG-03, PKG-04]

duration: 7 min
completed: 2026-05-03
---

# Phase 09 Plan 02: Tauri Updater Configuration Summary

**Signed Tauri updater configuration with stable GitHub Pages manifest trust, updater/process plugins, and narrow release capabilities**

## Performance

- **Duration:** 7 min
- **Started:** 2026-05-03T14:16:29Z
- **Completed:** 2026-05-03T14:23:39Z
- **Tasks:** 3
- **Files modified:** 8

## Accomplishments

- Added official Tauri updater/process JavaScript and Rust plugin dependencies with generated npm and Cargo lockfile updates.
- Enabled Tauri bundle generation and updater artifacts, with a baked public key and the single stable updater endpoint at `/updates/latest.json`.
- Registered updater/process plugins before the existing clipboard/opener/autostart runtime setup and added only updater plus process restart permissions.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add official updater and process dependencies** - `ea86968` (feat)
2. **Task 2: Activate bundle and stable updater endpoint config** - `df26e29` (feat)
3. **Task 3: Register updater and process plugins with least capabilities** - `b5254d6` (feat)

**Plan metadata:** this docs commit

## Files Created/Modified

- `package.json` - Adds `@tauri-apps/plugin-updater` and `@tauri-apps/plugin-process`.
- `package-lock.json` - Locks the new Tauri JavaScript plugin dependencies.
- `src-tauri/Cargo.toml` - Adds `tauri-plugin-updater` and `tauri-plugin-process`.
- `src-tauri/Cargo.lock` - Locks the new Rust plugin dependency graph.
- `src-tauri/tauri.conf.json` - Enables bundle/updater artifacts and configures public key, stable endpoint, and passive Windows install mode.
- `src-tauri/src/updater.rs` - Registers updater and process plugins through a thin builder helper.
- `src-tauri/src/lib.rs` - Exports the updater module.
- `src-tauri/src/main.rs` - Composes updater/process registration into the app builder before existing plugins.
- `src-tauri/capabilities/default.json` - Grants `updater:default` and `process:allow-restart` without shell permissions.

## Verification

- `npm pkg get dependencies.@tauri-apps/plugin-updater dependencies.@tauri-apps/plugin-process` passed.
- `npm install --package-lock-only` passed.
- `cargo metadata --manifest-path src-tauri/Cargo.toml --no-deps` passed.
- `cd src-tauri && cargo check` passed.
- `node scripts/release/verify-tauri-config.mjs --updater` passed.
- `cd src-tauri && cargo test` passed.
- `npm test` passed: 18 test files and 75 tests passed.
- All task acceptance grep gates passed, including stable endpoint, public key, no placeholder, no shell permission, and no private-key marker in the planned source/docs/script surfaces.

## Decisions Made

- Followed Tauri's official updater plugin path rather than adding custom signature verification.
- Used `process:allow-restart` instead of `process:default` to avoid granting frontend process exit permission.
- Stored the generated updater private key outside the repository under the user's config directory and committed only the public key.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- The plan's private-key grep initially traversed `src-tauri/target` and ran longer than intended. It was stopped and rerun successfully against the intended source/docs/script surfaces.

## User Setup Required

The generated updater private key is not committed. Before the first signed updater release, move the local private key into the planned GitHub Actions secret and an offline backup per D-05. Do not commit the private key or print its contents in logs.

## Known Stubs

None.

## Threat Flags

None - updater endpoint, signature public key, private-key handling, and release capabilities were all covered by the plan threat model.

## Next Phase Readiness

Ready for the release workflow and updater manifest plans. The app now has the runtime plugins, static trust configuration, and generated updater artifact settings those plans need.

## Self-Check: PASSED

- Summary file created.
- Task commits `ea86968`, `df26e29`, and `b5254d6` exist.
- Key source, config, manifest, and lock files exist.

---
*Phase: 09-packaging-updater-distribution*
*Completed: 2026-05-03*
