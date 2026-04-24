# GSD Dashboard — Design Spec

**Date:** 2026-04-23
**Status:** Ready for implementation planning
**Target:** Cross-platform desktop dashboard for Get Shit Done (GSD) project data and AI session analytics

---

## 1. Purpose

A native-feeling desktop application (macOS primary, Windows/Linux supported) that aggregates and visualizes all Get Shit Done project data across a user's machine, augmented with activity telemetry from Claude Code and Codex session logs. Runs persistently with a menu bar / system tray presence showing live milestone progress bars across active projects (inspired by openusage).

Primary day-to-day use case: glanceable portfolio monitor of all projects. Secondary: per-project drill-in and AI session analytics.

---

## 2. Non-Goals

- Not a GSD editor. Never writes to any `.planning/` directory. The CLI skills (Claude Code / Codex) remain the sole mutators.
- Not a transcript search engine (Phase 1). Message-content indexing is a later, toggleable capability.
- Not a remote/cloud dashboard. Local-first, reads files from the user's own machine.
- Not a replacement for `/gsd-*` slash commands. Dashboard copies recommended next commands to the clipboard so the user pastes them into their CLI.

---

## 3. Tech Stack

| Layer | Choice | Reason |
|---|---|---|
| Shell | **Tauri 2** | Small footprint (~10–20MB), native tray APIs across macOS/Win/Linux, first-class file watchers, SQLite support. Critical for the openusage-style tray bar rendering. |
| Backend | **Rust** | Fast file scanning, JSONL streaming, markdown parsing; well suited to the scope. |
| Frontend | **React + TypeScript + Tailwind** | Familiar stack, large ecosystem, fast dev cycles. |
| Charts | **Recharts** | Lightweight, composable, covers all planned views. |
| State | **Zustand** | Minimal, ergonomic; avoids Redux overhead. |
| Storage | **SQLite (rusqlite)** | Derived cache; raw `.planning/` and session files remain source of truth. |
| Markdown | **pulldown-cmark** + **gray_matter** | CommonMark + frontmatter parsing. |
| File watching | **notify** crate | Debounced re-parse on change; polling fallback on watcher failure. |
| Autostart | **tauri-plugin-autostart** | Wraps macOS LaunchAgents, Windows Run key, Linux `.desktop` autostart. |
| Updater | **tauri-plugin-updater** | Reads signed manifest from GitHub Pages. |

---

## 4. Architecture

### 4.1 Backend modules (Rust, under `src-tauri/src/`)

- `scanner` — walks configured scan roots (default `~/Documents`), finds directories containing `.planning/`, emits project candidates. Respects `.gitignore` / skips hidden dirs by default.
- `planning_parser` — parses `ROADMAP.md`, `MILESTONES.md`, `STATE.md`, `config.json`, `phases/*/PLAN.md`. Returns structured project state.
- `session_indexer` — streams `~/.claude/projects/**/*.jsonl` and `~/.codex/sessions/**/*` session files into metadata rows (Phase 1). Tool-usage and message-content indexing are Phase 2/3, gated behind settings toggles.
- `store` — SQLite cache at the OS-appropriate data dir (`~/Library/Application Support/gsd-dashboard/cache.db` on macOS; `%APPDATA%\gsd-dashboard\cache.db` on Windows; `~/.local/share/gsd-dashboard/cache.db` on Linux). Schema migrations via `rusqlite_migration`.
- `watcher` — uses `notify` to watch each discovered `.planning/` directory plus `~/.claude/projects` and `~/.codex/sessions`. Debounces events (500ms) and re-parses only the affected project or session file. On watcher failure (FSEvents limits, permission denied), falls back to a 60-second polling scan for that root and surfaces a banner in Settings.
- `tray` — native Tauri tray icon. Renders an openusage-style bar graph as a dynamically-generated PNG: one bar per active non-hidden project, height proportional to current milestone % complete. Re-renders on scan completion, on settings change (`tray_bar_max_projects`, `tray_bar_sort`), and on any watcher-triggered re-parse that changes milestone progress.
- `commands` — Tauri IPC handlers exposing queries to the frontend. Includes a `copy_next_command` helper that writes to OS clipboard via `arboard` or Tauri's clipboard plugin.
- `autostart` — thin wrapper around `tauri-plugin-autostart`; exposed to frontend settings UI.

### 4.2 Frontend (React, under `src/`)

- **Routes:** `/` (Portfolio), `/project/:id` (Project Detail), `/sessions` (Sessions), `/settings`.
- **Sidebar** nav with route links + collapsed "Hidden projects" and "Unmatched sessions" sections.
- **Data layer:** `invoke()` calls wrap every backend command; results cached in Zustand stores keyed by query. Live updates subscribe to Tauri events emitted by the watcher (`scan:complete`, `project:updated`, `session:new`).

### 4.3 App lifecycle

1. Launch (manual or via autostart).
2. Install tray icon immediately (with placeholder bars).
3. If autostart launch → start hidden (no main window). Else → show main window.
4. Kick off initial full scan in a background thread; emit progress events.
5. Once scan completes, render real tray bars + any open window.
6. Register watchers for every discovered `.planning/` and both session dirs.
7. On watcher event (debounced) → targeted re-parse → update SQLite → emit event to frontend → re-render tray.

---

## 5. Data Model

All tables live in the SQLite cache. Schema is rebuildable from source files (`Settings → Rebuild cache`).

### 5.1 `projects`
| Column | Type | Notes |
|---|---|---|
| `id` | TEXT PK | stable hash of `root_path` |
| `name` | TEXT | from `PROJECT.md` title or dir name |
| `root_path` | TEXT | absolute path to project root |
| `planning_path` | TEXT | `root_path + /.planning` |
| `current_milestone_name` | TEXT NULL | |
| `current_milestone_index` | INT NULL | |
| `current_phase_number` | TEXT NULL | e.g. "22" or "72.1" |
| `current_phase_name` | TEXT NULL | |
| `milestone_progress_pct` | REAL | 0–100; see §5.5 |
| `last_activity_at` | INTEGER | unix seconds, max of `.planning/` file mtimes and matched session end times |
| `hidden` | INT | 0/1 |
| `discovered_at` | INTEGER | |
| `last_scanned_at` | INTEGER | |
| `parsed_blob` | TEXT | JSON dump of parsed roadmap + state for detail view |
| `parse_error` | TEXT NULL | most recent parse error message, if any |

### 5.2 `milestones`
Per-project rows.
| Column | Type |
|---|---|
| `project_id` | TEXT FK |
| `name` | TEXT |
| `status` | TEXT CHECK IN ('active','shipped','planned') |
| `phase_range` | TEXT (e.g. "22-32") |
| `shipped_at` | INTEGER NULL |
| `phase_count` | INT |
| `completed_phase_count` | INT |

PK: `(project_id, name)`.

### 5.3 `phases`
Per-project+milestone rows.
| Column | Type |
|---|---|
| `project_id` | TEXT FK |
| `milestone_name` | TEXT |
| `number` | TEXT (supports decimal like "72.1") |
| `name` | TEXT |
| `status` | TEXT CHECK IN ('completed','active','planned') |
| `plan_count` | INT |
| `completed_plan_count` | INT |
| `completed_at` | INTEGER NULL |
| `path` | TEXT (phase dir) |

PK: `(project_id, milestone_name, number)`.

### 5.4 `sessions`
Unified Claude + Codex.
| Column | Type | Notes |
|---|---|---|
| `id` | TEXT PK | session file basename |
| `source` | TEXT | 'claude' or 'codex' |
| `project_id` | TEXT FK NULL | null if unmatched |
| `started_at` | INTEGER | |
| `ended_at` | INTEGER | |
| `duration_sec` | INT | |
| `message_count` | INT | |
| `tokens_in` | INT NULL | |
| `tokens_out` | INT NULL | |
| `model` | TEXT NULL | best-effort |
| `file_path` | TEXT | raw source file |
| `tool_calls_json` | TEXT NULL | Phase 2 |
| `mcp_calls_json` | TEXT NULL | Phase 2 |
| `fts_rowid` | INT NULL | Phase 3 link to FTS index |

Indexes: `(project_id, started_at)`, `(started_at)`, `(source)`.

### 5.5 `settings`
Single-row key/value.
- `scan_roots` — JSON array, default `["~/Documents"]`
- `hidden_project_ids` — JSON array
- `autostart_enabled` — bool, default false
- `refresh_interval_sec` — int, default `0` (event-driven only; fallback only)
- `tray_bar_max_projects` — int, default `8`
- `tray_bar_sort` — enum: `'name' | 'progress' | 'recent_activity'`, default `'recent_activity'`
- `phase2_index_tools` — bool, default false
- `phase3_index_messages` — bool, default false

### 5.6 `scan_log`
Rolling debug history, capped at last 100 rows.
- `ran_at`, `duration_ms`, `roots` (JSON), `projects_found`, `sessions_indexed`, `errors_json`.

### 5.7 Key derivations

- **Milestone % complete** = `completed_phase_count / phase_count` for the current milestone, derived from `ROADMAP.md` top-level checkbox state (`- [x]` vs `- [ ]`). Fallback: plan-level checkboxes summed across all phases in the milestone if top-level isn't reliable.
- **Current phase** = first phase in the active milestone with an unchecked ROADMAP entry, unless `STATE.md` names one explicitly (in which case STATE wins).
- **Next command** for clipboard = value from `STATE.md` if it specifies one (look for a `## Next Command` section or equivalent), else `/gsd-next`.
- **Project ↔ Claude session match** = reverse the Claude dir-name encoding (`-Users-smacdonald-homegit-deckpilot-web` → `/Users/smacdonald/homegit/deckpilot-web`) and lookup `projects.root_path`.
- **Project ↔ Codex session match** = parse `cwd` field if present in the session metadata; else leave `project_id` null. Unmatched sessions are surfaced in a "Global" bucket in the UI.

---

## 6. Views

### 6.1 Portfolio (landing)
- Header stats: total projects tracked, active milestones, sessions today, tokens today.
- **Project card grid** — one card per non-hidden project, sorted by `last_activity_at` desc. Card contents: project name, current milestone name + progress bar, current phase label ("Phase 22: Foundation"), relative last-activity ("2h ago"), 7-day session-count mini-sparkline. Click → Project Detail. Hover → "Copy next command" action button.
- **Activity heatmap** — 90-day GitHub-style heatmap of total sessions across all projects.
- **Right rail** — collapsed "Hidden projects" list and "Unmatched sessions" count.

### 6.2 Project Detail
- Header: project name, root path, buttons: "Open in Finder" / "Open in VS Code" / "Copy next command".
- **Milestone timeline** — horizontal bars, one per milestone, shaded by completion; click expands phase list.
- **Current phase panel** — phase number + name, plan checklist from `PLAN.md`, path links, `STATE.md` excerpt.
- **Sessions tab** — table of all Claude + Codex sessions attributed to this project. Sortable by date/duration/tokens.
- **Charts tab** — sessions/day (30d), tokens/day, avg session duration, milestone velocity (phases completed per week).

### 6.3 Sessions (global)
- Filterable table: source, project, date range, duration, tokens. Unmatched filter included.
- Top charts: sessions/day stacked by source, tokens/day stacked by project (top 5 + "other"), time-of-day histogram, day-of-week distribution.

### 6.4 Settings
- **Scan roots** — add/remove directories. Default `~/Documents`. Must prevent scanning `/` or `$HOME` directly as a guardrail.
- **Hidden projects** — list with unhide.
- **Launch on login** toggle.
- **Refresh interval** — only relevant as polling fallback; surface only if watcher is in fallback mode.
- **Tray display** — max bars, sort order, show/hide individual projects from the tray independently of hidden state.
- **Rebuild cache** button — drops SQLite and re-scans.
- **Phase 2 / Phase 3 toggles** — placeholders: "Index tool usage", "Index message content".

### 6.5 Tray (menu bar / system tray)
- **Icon:** dynamically-rendered PNG of bar graph. One bar per active non-hidden project, up to `tray_bar_max_projects`, ordered by `tray_bar_sort`. Height = `milestone_progress_pct`. Monochrome (template image) on macOS so it adapts to light/dark menu bars.
- **Left click:** toggle show/hide main window.
- **Right click (menu):**
  - Per-project items (active non-hidden): click → open Project Detail for that project
  - "Copy next command" submenu per active project
  - "Show dashboard" / "Preferences" / "Quit"

---

## 7. Error Handling

- Per-file parse failures are non-fatal: logged to `scan_log.errors_json`, surfaced as a "parse error" badge on the project card with a link to the offending file.
- Missing optional files (`STATE.md`, `config.json`) → parser returns `None`; UI shows em-dash.
- Truncated / corrupt JSONL → skip bad lines, count them, record in `scan_log`.
- Watcher failures → fall back to 60s polling for affected root; banner in Settings.
- All Tauri `invoke` handlers return `Result<T, AppError>`; frontend renders a toast on error with a "Copy details" button.
- SQLite corruption → on open failure, move `cache.db` aside with timestamp, recreate, trigger full rescan.

---

## 8. Testing

- **Rust unit tests** for parsers against fixture `.planning/` dirs copied from real repos (deckpilot-web, listingguru). Covers roadmap checkbox parsing, phase extraction, `STATE.md` parsing, decimal phase numbers.
- **Rust unit tests** for session indexer with fixture `.jsonl` files for both Claude and Codex formats, including truncation edge cases.
- **Rust integration test** for the full scan → SQLite → query pipeline using a tempdir.
- **Property test** (`proptest`) on Claude dir-name ↔ project path round-trip.
- **Frontend component tests** (Vitest + React Testing Library) for portfolio card, milestone timeline, sessions table.
- **Playwright smoke test** of the built app: launch, point it at a fixture dir, verify portfolio renders. Single test; runs in CI.
- **Tray PNG generation** unit-tested against expected byte output; no end-to-end pixel snapshots.
- **Codex ↔ project matching** explicitly noted as best-effort; test suite includes known-unmatchable fixtures to confirm they land in the "Global" bucket cleanly.

---

## 9. Packaging & Distribution

### 9.1 Artifacts per platform

| OS | Artifacts |
|---|---|
| macOS | `.dmg` + `.app` bundle, universal binary (arm64 + x86_64); signed & notarized when Apple dev cert is available |
| Windows | `.msi` (WiX) and `.exe` (NSIS) |
| Linux | `.deb`, `.AppImage`, `.rpm` (via `cargo-generate-rpm`) |
| Source | `gsd-dashboard-v<ver>-src.tar.gz` — `git archive` + `cargo vendor` + `npm ci` lockfile preserved, with `BUILD.md` |

### 9.2 CI

- GitHub Actions release workflow: matrix on `macos-latest`, `windows-latest`, `ubuntu-latest`.
- Triggered on tag push (`v*.*.*`).
- Steps per platform: build → sign (where applicable) → upload artifacts to GitHub Release.
- Extra jobs after the matrix: build source bundle, regenerate `updates/latest.json`, push to `gh-pages`.

### 9.3 GitHub Pages site

- Hosted on `gh-pages` branch (or `docs/` folder — workflow picks one consistent with Tauri convention).
- `index.html` — landing page: tagline, screenshots, install one-liner, platform download buttons.
- `install.sh` — detects OS + arch, downloads matching artifact from latest GitHub Release, installs to `/usr/local/bin` (or `~/Applications` on macOS without sudo). Idempotent.
- `updates/latest.json` — `tauri-plugin-updater` manifest. Regenerated on every release.
- Update signing: generate key pair during first release; private key stored as GH Actions secret; public key baked into `tauri.conf.json`.

### 9.4 Autostart

- Toggle in Settings wires `tauri-plugin-autostart`.
- On launch via autostart: detect via CLI arg or env var and start hidden (no window; tray only).
- Default: off.

### 9.5 crates.io (tertiary install channel)

- `cargo install gsd-dashboard` supported for Rust developers.
- Frontend `dist/` bundled into the published crate via `include_dir!`; publish job runs `npm ci && npm run build` before `cargo publish`.
- Documented caveats (README install section):
  - No `.app` / Start Menu / Launchpad entry.
  - First-run Gatekeeper warning on macOS (no code signature).
  - Auto-updater disabled for cargo-installed binaries; update via `cargo install --force gsd-dashboard`.
- Separate CI job `publish-cargo` runs on tag, gated on the binary matrix succeeding. Version in `Cargo.toml` kept in sync with the git tag by the release workflow.

---

## 10. Repo Layout

```
gsd-dashboard/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   ├── scanner.rs
│   │   ├── planning_parser.rs
│   │   ├── session_indexer.rs
│   │   ├── store.rs            # SQLite + migrations
│   │   ├── watcher.rs
│   │   ├── tray.rs
│   │   ├── commands.rs         # Tauri IPC handlers
│   │   └── autostart.rs
│   ├── tests/
│   │   ├── fixtures/           # sample .planning/ dirs + .jsonl
│   │   └── integration.rs
│   ├── tauri.conf.json
│   └── Cargo.toml
├── src/                        # React frontend
│   ├── routes/
│   │   ├── Portfolio.tsx
│   │   ├── ProjectDetail.tsx
│   │   ├── Sessions.tsx
│   │   └── Settings.tsx
│   ├── components/
│   ├── stores/                 # Zustand
│   ├── lib/ipc.ts              # invoke wrappers
│   └── main.tsx
├── tests/                      # Playwright
├── docs/
│   └── pages/                  # GitHub Pages source (index.html, install.sh)
├── .github/workflows/
│   ├── ci.yml                  # PR checks
│   └── release.yml             # tag → build matrix → pages update → crates.io
├── BUILD.md
├── README.md
└── LICENSE
```

---

## 11. Implementation Phasing (suggested)

The following is a rough grouping for the GSD planner. The planner is expected to refine into phases/plans.

1. **Foundation** — Tauri app skeleton, SQLite, settings UI shell, scan-roots config, default scan of `~/Documents`.
2. **Planning parser + Portfolio view** — parse `.planning/` dirs, projects table, project cards, milestone progress bars.
3. **Tray icon with milestone bars** — dynamic PNG generation, tray menu, copy-next-command action.
4. **Session indexer (metadata only)** — Claude + Codex ingestion, unified sessions table, matching heuristics.
5. **Project Detail view** — milestone timeline, phase panel, sessions tab, charts tab.
6. **Sessions global view + charts** — filtering table, activity heatmap, aggregate charts.
7. **Watcher + live updates** — notify integration, debouncing, fallback polling, event wiring to frontend.
8. **Autostart + OS polish** — launch-on-login, hidden-on-autostart, OS-specific tray refinements.
9. **Packaging & CI** — matrix build, signing, GitHub Pages site, install script, updater manifest.
10. **crates.io channel** — cargo publish workflow, frontend bundling, docs.
11. **(Phase 2, deferred)** Tool-usage indexing behind toggle.
12. **(Phase 3, deferred)** Message-content indexing + FTS search.

---

## 12. Open Questions / Decisions Deferred to Planning

- Exact `ROADMAP.md` checkbox parsing heuristic will need real-file validation against at least 3–5 different project layouts. Test fixtures pulled from `~/homegit/deckpilot-web`, `~/homegit/listingguru`, `~/homegit/locdirectory`, and at least two others.
- `STATE.md` format varies; planner should catalog variants before committing to a parser schema.
- Codex session `cwd` availability: confirm by sampling actual files before depending on it; may need fallback heuristics (e.g., match on file paths mentioned in the session).
- Tray bar rendering: exact visual style (spacing, color for dark vs light menu bar, highlight on active project) — deferred to an initial design pass during the tray-implementation phase.
- Apple code signing + notarization cert procurement: out of scope for the spec; release workflow must degrade gracefully when the secret is absent.
