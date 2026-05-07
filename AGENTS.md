# GSD Dashboard

## Project

A Tauri 2 cross-platform desktop dashboard that aggregates Get Shit Done project data and Codex / Codex session telemetry. macOS primary; Windows and Linux supported. Menu-bar / system-tray presence with live milestone progress bars (openusage-inspired).

**Core value:** At a glance, the user knows what every GSD project is doing right now — which milestone, which phase, how far along.

## Planning Artifacts

All planning lives in `.planning/` (gitignored — local-only):

- `PROJECT.md` — project context, requirements, constraints, key decisions
- `REQUIREMENTS.md` — 72 v1 requirements across 14 categories, mapped to phases
- `ROADMAP.md` — 9-phase roadmap for v1.0 MVP
- `STATE.md` — current project state (pointing at active phase)
- `config.json` — workflow config (YOLO, standard granularity, parallelization on, commit_docs off)
- `research/` — STACK / FEATURES / ARCHITECTURE / PITFALLS / SUMMARY

**Canonical design spec** (committed to git): `docs/superpowers/specs/2026-04-23-gsd-dashboard-design.md`.

## Technology Stack

- **Shell:** Tauri 2.10.x (cross-platform native shell + tray + IPC + bundler)
- **Backend:** Rust 1.82+, Tokio, `spawn_blocking` for every sync I/O / CPU path
- **Storage:** SQLite via `rusqlite` 0.39 (`bundled`) + `rusqlite_migration` 2.4 + `deadpool-sqlite` pool, WAL mode
- **Filesystem watching:** `notify` 8.2.x + `notify-debouncer-full` 0.5 (never 9.0-rc)
- **Markdown:** `pulldown-cmark` 0.13 + `gray_matter` (transitive `serde_yaml` is deprecated — watch)
- **Tray rendering:** `tiny-skia` 0.12 (pure Rust, ~200KiB, PNG encoder built in)
- **Tauri plugins:** `updater`, `autostart`, `clipboard-manager` (prefer over direct `arboard`), `fs`, `dialog`, `opener`, `os`, `single-instance`, `log`
- **Frontend:** React 19 + TypeScript 6.0+ + Vite 8
- **Styling:** Tailwind v4 via `@tailwindcss/vite` (not PostCSS)
- **State:** Zustand 5 (UI state) + TanStack Query 5 (IPC/server cache) — don't reinvent refetch/dedup in Zustand
- **Charts:** Recharts 3.8
- **Router:** React Router 7

## Architecture

Rust backend modules (under `src-tauri/src/`):
- `app_state` — Tauri-managed aggregate (pool, scan flag, watcher registry, settings snapshot)
- `error` — single `AppError` enum returned from every `#[tauri::command]`
- `events` — typed `AppEvent` enum with `#[serde(tag="event", content="data")]`
- `scanner` — walks roots, respects `.gitignore`, emits candidates (no DB, no parsing)
- `parser/{roadmap,state,plan,config,mod}.rs` — pure `&[u8] → Result<T, ParseError>`, no I/O
- `sessions/{Codex,codex,matcher,mod}.rs` — JSONL streaming with byte-offset incremental parsing + attribution
- `store` — deadpool pool (4–8 conns) + WAL + migrations
- `watcher` — `notify` + debouncer + polling fallback on ENOSPC
- `tray` — `tiny-skia` PNG on a dedicated Tokio task w/ 250ms debounce; `icon_as_template(true)` macOS-only
- `commands/{projects,sessions,settings,tray_actions}.rs` — thin IPC handlers
- `autostart` — thin wrapper around `tauri-plugin-autostart`

**IPC pattern — DB-as-truth, events-as-invalidation:** queries = Tauri commands returning `Result<T, AppError>`; pushes = small events carrying only IDs; frontend refetches via `get_project(id)`. Streaming (initial scan, rebuild cache) uses Tauri 2 `Channel<T>`.

**Frontend:** domain-sliced Zustand stores (not per-view), TanStack Query wraps every `invoke()`, single `registerAppListeners()` in `App.tsx`.

## Hard Invariants

- **Read-only against `.planning/`** — the dashboard never writes into any discovered `.planning/` directory. Single source of truth stays with the CLI skills. Enforce via a centralized module + CI lint + integration test.
- **No scan of `/` or bare `$HOME`** — guardrail against runaway scans. Explicit subdirectories only. Default scan root is `~/Documents`.
- **Local-first** — no cloud services, no remote aggregation. SQLite cache is derived; raw files are source of truth.
- **Per-file parse failures are non-fatal** — logged to `scan_log`, surfaced as a per-project badge. Never crash the scan.

## Critical Pitfalls to Remember

1. **Updater signing key is permanently irrecoverable if lost** — existing installs will never auto-update again. Design key custody (1Password + encrypted offline backup + GH Actions secrets) BEFORE the first signed release.
2. **Linux inotify watch limit** — default `fs.inotify.max_user_watches=8192`; easily exhausted. Watch ONLY `<project>/.planning/` subtrees (small). On ENOSPC: polling fallback + Settings banner with sysctl fix.
3. **macOS tray template image** — pure black+alpha (no RGB); template flag macOS-only (`#[cfg(target_os = "macos")]`); render 2x (44px tall for 22pt menu bar).
4. **JSONL partial writes on live files** — Codex appends continuously; last line often mid-record. Track `(path, last_parsed_byte_offset)`; tolerate final-line parse error without marking file corrupt.
5. **Codex/Codex session formats are unstable** — community-reverse-engineered schemas; Codex auto-updates have deleted `.jsonl` in the past. Multi-version fixtures; `Option<T>` on every field.
6. **Linux tray cross-DE** — AppIndicator discards left-click on GNOME/Ubuntu. Always include "Show dashboard" at the top of the right-click menu on all platforms.
7. **Tauri capabilities work in dev but deny in release** — scope `fs:allow-read-dir` correctly and add a release-build smoke test in CI.

## GSD Workflow

This project was initialized with `/gsd-new-project`. Phase workflow:
1. `/gsd-plan-phase N` — create a detailed plan for phase N (with research, plan-check, verifier enabled per config)
2. `/gsd-execute-phase N` — execute the plan with atomic commits
3. `/gsd-next` — advance to the next logical step

**Config:** YOLO mode, standard granularity (5-8 phases), parallel execution on, commit_docs off (`.planning/` is gitignored).

**Next step:** `/gsd-plan-phase 2`

---
*Last updated: 2026-04-24 after Phase 01 foundation completion*


<claude-mem-context>
# Memory Context

# [gsd-dashboard] recent context, 2026-05-07 8:09am EDT

Legend: 🎯session 🔴bugfix 🟣feature 🔄refactor ✅change 🔵discovery ⚖️decision 🚨security_alert 🔐security_note
Format: ID TIME TYPE TITLE
Fetch details: get_observations([IDs]) | Search: mem-search skill

Stats: 45 obs (16,287t read) | 428,995t work | 96% savings

### May 3, 2026
1 4:28p 🔵 PR #26 Status: 2 Failing Builds, 22 Unresolved CodeRabbit Comments
2 " 🔵 Build Failures Occur During Tauri Desktop Build Phase
3 4:29p 🔵 Build Logs Contain No Standard Error Patterns
4 " 🔵 Failed Step Identified: Build Tauri Debug App Process Timeout/Termination
5 4:30p 🔵 Build Failures Root Cause: Missing Application Icons for Desktop Bundling
6 " 🔵 22 CodeRabbit Review Comments Identified Across Release Tooling and Components
7 4:31p 🔵 Icon Assets Exist but Bundler Can't Locate Them: Configuration Missing in Cargo.toml
8 " 🔵 Icon Assets Valid and Properly Formatted; Missing Reference in Cargo.toml Build Configuration
9 " 🔴 Fixed failing PR checks by stabilizing release validation and workflow
10 5:17p 🔴 URL signature validation expanded to reject all URL schemes
11 5:19p 🔵 GitHub Actions smoke tests passed on all platforms for URL signature validation fix
12 5:20p 🔵 All PR #26 checks passed including smoke tests and code review
81 9:55p 🔵 Cross-phase integration audit: Frontend-backend IPC and event-driven cache invalidation architecture
82 9:56p 🔵 Requirements documentation out of sync with phase implementation evidence
83 " 🔵 Settings-autostart-tray integration with transactional rollback on failure
84 9:57p 🔵 Milestone scoping mismatch: STATE.md lists Phase 10 in v1.0; ROADMAP.md does not
85 " 🔵 Frontend test infrastructure misconfigured; backend integration tests fully operational
110 10:21p 🔵 Phase 09.1 missing from ROADMAP.md despite having executable plan
111 " 🔵 Task 1 preflight verification passes - release infrastructure and evidence gates confirmed
113 10:23p 🔵 Phase 09.1 IS defined in ROADMAP.md despite roadmap.get-phase query returning not-found
114 " 🔵 Phase 9.1 plan structure fully valid; all three tasks properly formed for execution
### May 4, 2026
171 6:09a 🔵 Phase 9.1 structure and preflight verification completed
181 6:18a 🔵 Batch execution context tool times out at 120 seconds
182 " 🔵 Release verification infrastructure exists without automated test coverage
183 6:19a 🔵 All frontend and release script tests passing (86 tests, 0 failures)
184 6:21a 🔵 Rust cargo test suite exceeds 120-second timeout
185 " 🔵 Rust backend cargo test suite passes 60+ tests across multiple modules
186 " 🔵 Rust cargo test suite: 163+ tests across 30+ modules, 2 ignored, all passing
187 " 🔵 Rust cargo test suite completes successfully: 170+ tests pass, exit code 0
### May 6, 2026
1450 10:22p 🔵 v0.1.1 Release Job Failed with Completion Status
1451 10:23p ✅ Rerun Failed Release Jobs via GitHub Actions
1452 " ✅ Release Job Rerun Queued and Monitoring Initiated
1453 " ✅ Release Job Execution In Progress
1454 " 🔵 Release Build Job Status: Multi-Platform Progress
1455 10:24p 🔵 macOS Release Build Running Without Failures
1456 10:27p 🔵 Release Build Status: macOS Still Running, Platform Builds Progressing
1498 11:04p 🔵 macOS notarization failure blocking v0.1.1 release
1499 " 🔵 macOS notarization fails with 401 authentication error
1556 11:44p 🔵 GSD Phase 9.1 Execution In Progress
1557 " 🔵 macOS Release Build Job Has No Log Output
### May 7, 2026
1717 7:52a 🔵 GitHub Pages not enabled on repository despite deployment workflow in place
1737 8:08a 🔵 GitHub Pages URL references identified across codebase
1738 8:09a ✅ Updated GitHub Pages URL from smacdonald to horknfbr across all configuration and release scripts
1739 " ✅ Configured GitHub Pages for horknfbr/gsd-dashboard repository
1740 " 🔵 URL migration verification complete - all old references removed, new URLs confirmed

Access 429k tokens of past work via get_observations([IDs]) or mem-search skill.
</claude-mem-context>