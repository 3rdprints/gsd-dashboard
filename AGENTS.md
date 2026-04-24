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
- **Frontend:** React 19 + TypeScript 5.6+ + Vite 6
- **Styling:** Tailwind v4 via `@tailwindcss/vite` (not PostCSS)
- **State:** Zustand 5 (UI state) + TanStack Query 5 (IPC/server cache) — don't reinvent refetch/dedup in Zustand
- **Charts:** Recharts 2.15
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
