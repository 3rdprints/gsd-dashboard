---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: Parser & Agent Telemetry Expansion
status: verifying
last_updated: "2026-05-02T11:59:36.855Z"
progress:
  total_phases: 9
  completed_phases: 7
  total_plans: 40
  completed_plans: 40
  percent: 100
---

# State: GSD Dashboard

## Project Reference

**Core Value:** At a glance, the user knows what every GSD project is doing right now — which milestone, which phase, how far along — without opening a terminal or reading markdown files.

**Current Focus:** Phase 07 — live-updates

## Current Position

Phase: 07 (live-updates) — EXECUTING
Plan: 5 of 5
**Milestone:** v1.0 MVP
**Phase:** 7
**Plan:** 5 of 5
**Status:** Phase complete — ready for verification

**Progress:** [█████████░] 90%

```
Milestone: [████......] 4/10 phases
Phase 1:   [██████████] 4/4 plans
Phase 2:   [██████████] 4/4 plans
Phase 3:   [██████████] 6/6 plans
Phase 4:   [██████████] 4/4 plans
Overall:   [████......] 40%
```

## Next Command

```
/gsd-next
```

## Performance Metrics

- Phases completed: 4 / 10
- Plans completed: 18
- Avg plan duration: 12.0 min
- Nodes retried: 0
- Plan 01-01 duration: 21 min; tasks: 3; files modified: 18
- Plan 01-02 duration: 8 min; tasks: 3; files modified: 11
- Plan 01-03 duration: 9 min; tasks: 2; files modified: 17
- Plan 01-04 duration: 10 min; tasks: 3; files modified: 10
- Plan 03-01 duration: 9 min; tasks: 3; files modified: 9
- Plan 03-02 duration: 6 min; tasks: 3; files modified: 7
- Plan 03-03 duration: 8 min; tasks: 3; files modified: 7
- Plan 03-04 duration: 21 min; tasks: 3; files modified: 14
- Plan 03-05 duration: 9 min; tasks: 3; files modified: 4
- Plan 03-06 duration: 9 min; tasks: 3; files modified: 4
- Plan 04-01 duration: 7 min; tasks: 3; files modified: 6
- Plan 04-02 duration: 6 min; tasks: 3; files modified: 11
- Plan 04-03 duration: 7 min; tasks: 3; files modified: 11
- Plan 04-04 duration: 18 min; tasks: 3; files modified: 11
- Plan 05-01 duration: 5 min; tasks: 2; files modified: 13
- Plan 05-02 duration: 1 min; tasks: 1; files modified: 10
- Plan 05-03 duration: 2 min; tasks: 1; files modified: 7
- Plan 05-05 duration: 6 min; tasks: 1; files modified: 15
- Plan 05-07 duration: 14 min; tasks: 2; files modified: 13
- Plan 05-08 duration: 6 min; tasks: 2; files modified: 11
- Plan 05-09 duration: 7 min; tasks: 3; files modified: 12
- Plan 05-10 duration: 7 min; tasks: 2; files modified: 11
- Plan 05-11 duration: 5 min; tasks: 2; files modified: 10
- Plan 05-12 duration: 4 min; tasks: 2; files modified: 10
- Plan 07-01 duration: 2 min; tasks: 2; files modified: 5
- Plan 07-02 duration: 7 min; tasks: 2; files modified: 16
- Plan 07-03 duration: 9 min; tasks: 2; files modified: 11
- Plan 07-04 duration: 7 min; tasks: 2; files modified: 8
- Plan 07-05 duration: 4 min; tasks: 2; files modified: 10

## Accumulated Context

### Decisions

- Tauri 2 + Rust + React + TypeScript + Tailwind v4 (locked by design spec; confidence HIGH from research)
- SQLite via `rusqlite` + `deadpool-sqlite` pool in WAL mode (not `Mutex<Connection>`)
- tiny-skia for tray PNG rasterization
- `tauri-plugin-clipboard-manager` over direct `arboard`
- Zustand + TanStack Query (split UI state from IPC cache)
- DB-as-truth, events-as-invalidation IPC pattern (events carry IDs, frontend refetches)
- Portfolio ships before Tray (research overrides spec §11 ordering)
- Sessions ship before Live Updates (watcher is fragile, de-risk last)
- Drop `cargo-generate-rpm` — Tauri 2 bundler produces `.rpm` natively
- Plan 01-01 scaffold uses Tauri 2 with release-strict core:default capability and Vite/Tailwind v4 wiring.
- Adjusted SQLite crate pins to the only compatible published Cargo graph: deadpool-sqlite 0.13.0 with rusqlite 0.38 and rusqlite_migration 2.4.
- Plan 01-02 added WAL SQLite cache migrations, settings persistence, first-run defaults, and scan-root guardrails before persistence.
- Plan 01-03 added managed AppState bootstrapping, stable AppError/AppEvent contracts, and thin boot/settings commands with narrow Tauri capabilities.
- Plan 01-03 uses Tauri app-data/home path resolvers in bootstrap_app, with bootstrap_from_paths only for tests.
- Plan 01-03 generates Tauri app-command permissions from build.rs and allows only get_boot_status, get_settings, and save_settings in default.json.
- Plan 01-04 uses TanStack Query for frontend IPC/server state and local React state only for scan-root drafts.
- Phase 1 shell intentionally omits scanner/project/session/chart/tray controls until later phases.
- Plan 03-01 keeps hidden project state in settings.hidden_project_ids; portfolio DTOs filter without mutating cached project rows.
- Plan 03-01 portfolio stats count visible projects only; sessionsToday and tokensToday remain zero until Phase 4.
- Plan 03-01 registers get_portfolio and get_project in Tauri build metadata and default capabilities for release IPC.
- Plan 03-02 rebuild_cache deletes only derived SQLite rows and preserves settings.
- Plan 03-02 reuses scan_projects_for_app for rebuild progress events and scan-root guardrails.
- Plan 03-02 registers rebuild_cache in Tauri metadata and default capabilities for release IPC.
- Plan 03-03 copy/open actions use official Tauri plugins directly from the frontend; no backend invoke or shell/process command path was added.
- Plan 03-03 release capabilities allow only clipboard text write plus opener path and URL commands for copy/open actions.
- Plan 03-04 uses React Router BrowserRouter/Routes for the portfolio vertical slice while TanStack Query remains the IPC cache owner.
- Settings UI saves only settings changes for scan roots and hidden project IDs; hidden/unhide never deletes project cache rows.
- Plan 03-05 used an empty validation commit for Task 2 because security and capability gates passed without source edits.
- Phase 3 validation treats broad spawn grep matches on tokio::task::spawn_blocking as false positives and verifies shell/process execution with narrower gates.
- Clipboard copied feedback is shown only after copyNextCommand resolves successfully
- Visible hide action uses settings.hiddenProjectIds with TanStack Query invalidation instead of local portfolio filtering
- Session rows store only metadata columns; prompt, transcript, content, tool-call JSON, and FTS columns remain absent.
- Per-file indexing state advances in the same transaction as session upserts.
- Unmatched sessions remain first-class rows with nullable project_id and can be rematched after project cache rebuilds.
- Session parsers operate on serde_json::Value and extract metadata only; raw text fields are neither stored nor fixture-backed.
- Final non-newline JSONL bytes are treated as Live session still writing and left unconsumed for the next index pass.
- Claude path fallback compares encoded directory names against known project roots rather than trusting decoded paths as filesystem targets.
- index_sessions accepts no frontend path input; it derives only /Users/smacdonald/.claude/projects and /Users/smacdonald/.codex/sessions from AppState.
- SessionIndexEvent emits source, path, counts, status, and error text only; prompt/message bodies never cross IPC.
- Rebuild cache rematches existing unmatched sessions after refreshed project rows are visible, preserving session rows and offsets.
- Portfolio stats and cards read session aggregates from SQLite via load_portfolio_session_summary.
- Index Sessions uses a Tauri Channel and invalidates portfolioQueryKey only after command completion.
- Project card sparklines use seven fixed CSS bars instead of adding a charting dependency.
- [Phase 05]: Plan 05-01 keys plan_items by (project_id, plan_path, ord) with a composite FK to phase_plans(project_id, plan_path).
- [Phase 05]: Plan 05-01 persists globalSessionsDefaultRange in settings and coerces invalid values to 7d.
- [Phase 05]: Plan 05-02 intentionally ships RED scaffold tests only; implementation plans 05-04 through 05-07 replace the scaffold panics.
- [Phase 05]: Plan 05-03 uses Vitest it.todo consistently for frontend scaffold tests so TypeScript stays green while later implementation plans replace the todos.
- [Phase 05]: Plan 05-04 uses discovered PLAN.md paths as the shared key for phase_plans and plan_items.
- [Phase 05]: Plan 05-04 stores STATE Current Position excerpts as derived project snapshot data without mutating source markdown.
- [Phase 05]: Plan 05-05 keeps Project Detail command SQL in focused sessions modules so command/repo files remain below the 500-line AGENTS.md limit.
- [Phase 05]: Plan 05-05 whitelists exact project session sort keys and asc/desc directions before interpolating ORDER BY SQL identifiers.
- [Phase 05]: Plan 05-07 keeps Global Sessions SQL in a focused sessions/global.rs module to preserve the AGENTS.md 500-line file limit.
- [Phase 05]: Plan 05-07 builds GlobalSessionFilters SQL from fixed active predicates with bound values and validates source as claude or codex.
- [Phase 05]: Plan 05-07 registers list_global_sessions, get_global_chart_data, and get_portfolio_heatmap for dev handlers and release capabilities.
- [Phase 05]: Plan 05-08 uses local React state for Project Detail tabs on /project/:id rather than nested routes.
- [Phase 05]: Plan 05-08 keeps Project Detail CSS route-scoped in ProjectDetailPage.css to avoid expanding the oversized global stylesheet.
- [Phase 05]: Plan 05-08 renders STATE excerpts as React text nodes with no raw HTML injection.
- [Phase 05]: Plan 05-09 keeps Project Detail Sessions and Charts CSS route-scoped in ProjectDetailPage.css rather than expanding the oversized global stylesheet.
- [Phase 05]: Plan 05-09 uses a single tab-level project chart range selector defaulting to 30d and includes the range in the TanStack Query key.
- [Phase 05]: Plan 05-10 uses browser URLSearchParams as live Global Sessions filter/page state and derives backend filters via strict coercion.
- [Phase 05]: Plan 05-10 keeps Global Sessions CSS route-scoped in GlobalSessionsPage.css because src/styles.css exceeds the AGENTS.md 500-line limit.
- [Phase 05]: Plan 05-11 renders chart project names only as React text children in legend chips.
- [Phase 05]: Plan 05-11 reuses filtersToGlobalSessionFilters for both Global Sessions table and chart queries.
- [Phase 05]: Plan 05-12 uses react-calendar-heatmap with local CSS classes and no package stylesheet import for the Portfolio activity heatmap.
- [Phase 05]: Plan 05-12 invalidates portfolioHeatmapQueryKey from daily_activity_updated without trusting event payloads.
- [Phase 06]: Plan 06-02 uses Portfolio getPortfolio / portfolioQueryKey rows for tray visibility controls.
- [Phase 06]: Plan 06-02 keeps Tray Display controls on the existing Settings Save Settings path.
- [Phase 06]: Plan 06-03 keeps tray ordering constrained to persisted sort choices: recent_activity, progress, and name.
- [Phase 06]: Plan 06-03 treats tray_bar_max_projects as an upper bound and reduces rendered bars to preserve at least 2px width.
- [Phase 06]: Plan 06-03 uses tiny-skia PNG rendering with black-only non-transparent pixels for macOS template safety.
- [Phase 06]: Plan 06-04 keeps project tray rows as /project/:id navigation while copy_next actions remain non-navigation clipboard actions. — Matches D-06 and preserves the no-shell-execution boundary for Plan 06-05 clipboard wiring.
- [Phase 06]: Plan 06-04 accepts only /, /settings, and /project/:id local routes from trayNavigate payloads. — Backend tray events control local routing, so frontend route handling is constrained to planned destinations.
- [Phase 06]: Plan 06-04 keeps focused tray listener coverage in src/lib/appListeners.test.ts rather than expanding App tests. — Maintains the project preference to avoid adding Phase 06 coverage to oversized App-level tests.
- [Phase 06]: Plan 06-05 wires setup_tray through a shared bootstrap helper so native tray startup happens after AppState is managed.
- [Phase 06]: Tray refreshes are requested from settings, scan/rebuild, and project cache update paths, then rebuilt from SQLite/settings rather than trusting event payloads.
- [Phase 06]: macOS tray template mode is applied during startup and after icon refresh to preserve light/dark menu bar adaptation.
- [Phase 10]: Parser & Tool Telemetry Foundation was added as the v1.0 high-value slice for Codex parser hardening, GSD run attribution, and jCodemunch/jDocMunch usage metrics.
- [Future Milestone v1.1]: Parser & Agent Telemetry Expansion is reserved for full replay/search, broader agent parser coverage, gsd-sdk parity checks, and deeper jSuite optimization analytics.
- [Phase 07]: Plan 07-01 uses ignored Rust tests and Vitest it.todo cases as compile-green implementation gates for live updates.
- [Phase 07]: Plan 07-02 keeps watcher status runtime-only on AppState and does not persist it in Settings.
- [Phase 07]: Plan 07-02 starts watcher ownership during bootstrap while deferring native OS watch registration to later Phase 07 plans.
- [Phase 07]: Plan 07-02 live update events carry only IDs or a no-payload invalidator.
- [Phase 07]: Plan 07-03 uses a deterministic ProjectDebouncer seam for watcher coalescing tests instead of relying on OS notification timing.
- [Phase 07]: Plan 07-03 keeps targeted refresh in scan_refresh/watcher::refresh and reuses existing parser plus persist_project_scan.
- [Phase 07]: Plan 07-03 tracks tray refresh requests on AppState because this tree had no existing native tray service module.
- [Phase 07]: Plan 07-04 splits single-file session indexing into sessions::file_indexer so sessions/indexer.rs stays below the 500-line project limit.
- [Phase 07]: Plan 07-04 uses Tokio JoinSet bounded by SESSION_INDEX_WORKER_LIMIT = 2 for completion-order session file indexing.
- [Phase 07]: Plan 07-04 emits session:new only after derived SQLite persistence and keeps the payload to session id plus optional project id.
- [Phase 07]: Plan 07-05 keeps watcher fallback UX Settings-only and avoids portfolio/project badges, toasts, countdowns, retry buttons, or a new Live Updates page.
- [Phase 07]: Plan 07-05 treats project/session/watcher events as tiny invalidation hints and refetches display data through TanStack Query.
- [Phase 07]: Plan 07-05 keeps watcher status styles route-scoped in SettingsPage.css rather than expanding src/styles.css.

### Roadmap Evolution

- 2026-05-01: Added Phase 10, Parser & Tool Telemetry Foundation, to capture high-value parser and tool-efficiency work before broader session replay scope.
- 2026-05-01: Added future v1.1 Parser & Agent Telemetry Expansion milestone for the remaining parser/session/MCP upgrades.

### Todos

- Plan Phase 03: Portfolio Vertical Slice.

### Blockers

- (none)

### Research Flags (from research/SUMMARY.md)

- **Phase 4 (Sessions):** Codex `cwd` presence rate audit required on real machine before implementation; multi-version JSONL fixtures needed
- **Phase 6 (Tray):** macOS 26 Liquid Glass interactions (Tauri #14979); Linux Wayland+GNOME fallback UX spike
- **Phase 7 (Watcher):** FSEvents coalescing + inotify fallback UX under git-checkout stress
- **Phase 9 (Packaging):** macOS hardened runtime entitlements, notarization poll+staple, GH Actions signing passphrase handling
- **Phase 10 (Parser & Tool Telemetry):** Codex/Claude session formats and MCP tool payload shapes are community-observed and unstable; keep parsers fixture-backed, metadata-first, and tolerant of unknown fields.

### Critical Risks (from research/PITFALLS.md)

- **Updater signing key is permanently irrecoverable if lost** — design key custody BEFORE first signed release (Phase 9)
- **Read-only invariant** against `.planning/` must be enforced from Phase 1 via a `read_only_fs` module + CI lint
- **WAL + pragmas** must be set at connection open from Phase 1; retrofitting is painful

## Session Continuity

**Last session:** 2026-05-02T11:59:36.850Z

**Next session should:** Run Phase 07 verification.

---
*State initialized: 2026-04-23*

**Completed Phase:** 2 (Planning Parser & Scanner) — 4 plans — 2026-04-25

**Planned Phase:** 4 (Session Indexer) — 4 plans — 2026-04-26T11:25:19.763Z

**Completed Phase:** 4 (Session Indexer) — 4 plans — 2026-04-26
