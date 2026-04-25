# Roadmap: GSD Dashboard

**Milestone:** v1.0 MVP — glanceable portfolio monitor across all GSD projects + unified Claude/Codex session analytics, shipped as signed native installers on all three OSes.

**Granularity:** standard
**Phases:** 9
**v1 requirements covered:** 58 / 58

## Phases

- [x] **Phase 1: Foundation** - Tauri 2 skeleton, WAL-SQLite pool, settings persistence, capabilities, error/events infra, first-run defaults
- [x] **Phase 2: Planning Parser & Scanner** - Pure-function parsers over ROADMAP/STATE/PLAN/config + gitignore-respecting scanner with guardrails
- [ ] **Phase 3: Portfolio Vertical Slice** - First visible demo: scan on launch, project cards with progress bars, Settings scan-roots UI, Rebuild Cache
- [ ] **Phase 4: Session Indexer** - Claude + Codex JSONL streaming with byte-offset incremental parsing and project attribution
- [ ] **Phase 5: Project Detail, Global Sessions & Charts** - Drill-in view, milestone timeline, charts, filterable global sessions table, 90-day heatmap
- [ ] **Phase 6: Tray Icon with Milestone Bars** - Dynamic openusage-style bar graph, tooltip, click-toggle, right-click menu, copy-next-command
- [ ] **Phase 7: Live Updates** - notify watchers on .planning/ and session dirs with project-level debouncing and 60s polling fallback
- [ ] **Phase 8: Autostart & OS Polish** - Launch-on-login with hidden-startup on all three OSes
- [ ] **Phase 9: Packaging, Updater & Distribution** - CI matrix, signed releases, GitHub Pages site + install.sh + updater manifest, cargo install channel

## Phase Details

### Phase 1: Foundation
**Goal**: App launches on all three OSes with working persistence layer and guardrails in place.
**Depends on**: Nothing (first phase)
**Requirements**: FND-01, FND-02, FND-03, FND-04, FND-05
**Success Criteria** (what must be TRUE):
  1. User can launch a Tauri 2 app on macOS, Windows, and Linux built from a single codebase.
  2. On first launch the app creates a WAL-mode SQLite cache in the OS-appropriate app-data directory and runs migrations cleanly; relaunching preserves state.
  3. User settings (scan roots, hidden projects, autostart, tray config) round-trip to disk and survive a restart.
  4. On a clean first run with no config, the app defaults scan roots to `~/Documents` and reaches a populated-or-empty dashboard without any user configuration.
  5. Attempting to configure `/` or bare `$HOME` as a scan root is refused with a clear error surfaced in the UI.
**Plans**: 4 plans
Plans:
- [x] 01-01-PLAN.md — Scaffold pinned Tauri 2 + React/Vite/Tailwind app with strict capabilities
- [x] 01-02-PLAN.md — Implement WAL SQLite cache, migrations, settings defaults, and scan-root guardrails
- [x] 01-03-PLAN.md — Wire AppState, AppError, AppEvent, and thin boot/settings commands
- [x] 01-04-PLAN.md — Build sparse Phase 1 UI shell for boot/cache/settings/error/empty states
**UI hint**: yes

### Phase 2: Planning Parser & Scanner
**Goal**: Given a directory containing `.planning/`, the system reliably extracts current milestone, phase, progress, and next-command without touching the filesystem from parsers.
**Depends on**: Phase 1
**Requirements**: SCAN-01, SCAN-05, SCAN-06, PARSE-01, PARSE-02, PARSE-03, PARSE-04, PARSE-05, PARSE-06, PARSE-07, PARSE-08
**Success Criteria** (what must be TRUE):
  1. Running a scan over a directory tree discovers every `.planning/`-containing project beneath configured roots and streams progress to the UI.
  2. For each discovered project the parser returns current milestone (name + index), current phase (number + name, including decimals like "72.1"), milestone progress percentage, per-phase plan checklists, and a "next command" recommendation (defaulting to `/gsd-next`).
  3. Parsers validate cleanly against at least 5 real in-the-wild fixtures (deckpilot-web, listingguru, locdirectory, plus two others) with zero panics.
  4. A malformed ROADMAP or STATE file in one project never aborts the scan; the affected project surfaces a parse-error state and the failure is recorded in `scan_log`.
**Plans**: 4 plans
Plans:
- [x] 02-01-PLAN.md — Build pure planning parsers for ROADMAP, STATE, PLAN, and config
- [x] 02-02-PLAN.md — Add SQLite project cache tables and repositories for snapshots and scan logs
- [x] 02-03-PLAN.md — Implement safe scanner discovery, persistence orchestration, and scan progress command
- [x] 02-04-PLAN.md — Validate real fixtures and expose scan progress in the Phase 2 shell

### Phase 3: Portfolio Vertical Slice
**Goal**: User opens the app and sees a card per project with milestone and phase info, and can control what gets scanned.
**Depends on**: Phase 2
**Requirements**: SCAN-02, SCAN-03, SCAN-04, PORT-01, PORT-03, PORT-04, PORT-06, PORT-07, CLIP-01, CLIP-02, DET-01, SET-01, SET-02, SET-04, SET-05
**Success Criteria** (what must be TRUE):
  1. User launches the app and sees a card per non-hidden project sorted by last-activity descending, each showing name, current milestone + progress bar, current phase label, and relative last-activity.
  2. Clicking a card opens a Project Detail view showing project name, root path, and working "Open in Finder", "Open in VS Code", and "Copy next command" actions; hovering a card on the portfolio reveals a "Copy next command" action that writes to the OS clipboard without ever mutating `.planning/`.
  3. From Settings the user can add or remove scan-root directories and see the portfolio update without restarting; the user can hide or unhide individual projects and hidden ones disappear from the portfolio while remaining discoverable via a right-rail / unmatched-sessions panel.
  4. "Rebuild cache" in Settings drops the derived cache and runs a full rescan with visible progress, without requiring an app restart.
  5. Header stats show total projects tracked, active milestones, sessions today (zero until Phase 4), and tokens today; Phase 2 and Phase 3 settings toggles are visible but disabled.
**Plans**: 5 plans
Plans:
- [x] 03-01-PLAN.md — Add backend portfolio/detail DTO commands and project cache read helpers
- [x] 03-02-PLAN.md — Add derived-only rebuild cache command and settings-preserving backend coverage
- [x] 03-03-PLAN.md — Wire clipboard/opener plugins, release capabilities, and safe frontend action wrappers
- [ ] 03-04-PLAN.md — Build routed portfolio, detail, and settings UI vertical slice
- [ ] 03-05-PLAN.md — Harden validation, security invariant checks, and Phase 3 sign-off
**UI hint**: yes

### Phase 4: Session Indexer
**Goal**: Every Claude Code and Codex session on the machine is indexed into SQLite and attributed to a project where possible.
**Depends on**: Phase 3
**Requirements**: SESS-01, SESS-02, SESS-03, SESS-04, SESS-05, SESS-06, PORT-02
**Success Criteria** (what must be TRUE):
  1. The indexer streams every Claude Code `.jsonl` file under `~/.claude/projects/` and every Codex session file under `~/.codex/sessions/` and persists start/end, duration, message count, tokens in/out, and model (best-effort for Codex) into the SQLite `sessions` table with indexes supporting portfolio/detail/global queries.
  2. A live Claude session with a half-written last JSONL line is indexed without being marked corrupt; byte offsets are tracked so subsequent parses are incremental.
  3. Claude sessions are attributed to projects by reversing the directory-name encoding; Codex sessions are attributed via parsed `cwd` when present; sessions that cannot be attributed show up in a "Global / Unmatched" bucket rather than being dropped.
  4. Portfolio cards now show an accurate 7-day session-count sparkline per project driven by indexed session data.
**Plans**: TBD

### Phase 5: Project Detail, Global Sessions & Charts
**Goal**: User can drill into any project for milestone and session analytics and can explore all sessions across the portfolio.
**Depends on**: Phase 4
**Requirements**: DET-02, DET-03, DET-04, DET-05, GLOB-01, GLOB-02, GLOB-03, PORT-05
**Success Criteria** (what must be TRUE):
  1. Project Detail shows a milestone timeline (one horizontal bar per milestone, shaded by completion, expandable to reveal phases), a current-phase panel with plan checklist, path links, and a STATE.md excerpt, and a Sessions tab listing all sessions attributed to that project in a sortable table.
  2. Project Detail Charts tab shows sessions/day over 30 days, tokens/day, average session duration, and milestone velocity, all driven by real indexed session data.
  3. A Global Sessions view presents a filterable table across Claude and Codex sessions with filters for source, project, date range, duration, tokens, and unmatched-only, plus top charts (sessions/day stacked by source, tokens/day stacked by top-5 projects + "other", time-of-day histogram, day-of-week distribution).
  4. The Portfolio landing view now includes a 90-day GitHub-style activity heatmap rendered from pre-aggregated daily counts.
**Plans**: TBD
**UI hint**: yes

### Phase 6: Tray Icon with Milestone Bars
**Goal**: With the app running, a glance at the menu bar / system tray tells the user the current progress of every active project.
**Depends on**: Phase 5
**Requirements**: TRAY-01, TRAY-02, TRAY-03, TRAY-04, TRAY-05, TRAY-06, TRAY-07, SET-03
**Success Criteria** (what must be TRUE):
  1. The tray renders a dynamic PNG bar graph with one bar per active non-hidden project, ordered by the user's configured sort (name / progress / recent activity), where each bar's height is proportional to the project's current milestone completion percentage.
  2. On macOS the tray icon is a template image (pure black + alpha) that adapts to light and dark menu bars; on Windows and Linux the icon renders correctly at native tray sizes with Retina/HiDPI handling.
  3. Hovering the tray icon shows a concise tooltip summary (e.g. "3 active projects · Foo 62% · Bar 31%"); left-click toggles show/hide of the main window on every platform including Linux AppIndicator (where "Show dashboard" is always present in the right-click menu as a fallback).
  4. Right-clicking the tray reveals per-project entries that open Project Detail, a "Copy next command" submenu per active project that writes to the OS clipboard, plus Show dashboard / Preferences / Quit.
  5. The tray re-renders promptly on scan completion, on settings changes (max bars, sort order, per-project tray show/hide independent of overall hidden state), and on progress updates — without blocking click handling — and tray display preferences (max bars, sort, per-project show/hide) are configurable in Settings.
**Plans**: TBD
**UI hint**: yes

### Phase 7: Live Updates
**Goal**: The dashboard stays in sync with source files automatically, without the user ever clicking Rebuild Cache.
**Depends on**: Phase 6
**Requirements**: LIVE-01, LIVE-02, LIVE-03, LIVE-04, LIVE-05
**Success Criteria** (what must be TRUE):
  1. Editing a `STATE.md` or `ROADMAP.md` in any discovered `.planning/` directory causes the corresponding portfolio card, Project Detail view, and tray bar to update within ~1 second without user action; a new Claude or Codex session file appearing triggers the same liveness for sessions views.
  2. Updates are project-level debounced at ~500ms so that a `git checkout` that rewrites many files triggers at most a small number of coalesced re-parses, not one per file.
  3. When a filesystem watcher fails (inotify exhaustion on Linux, permission denied, FSEvents limits on macOS), the system transparently falls back to 60-second polling for the affected root and keeps the dashboard current.
  4. When any watcher is in fallback polling mode, Settings surfaces a visible banner identifying the degraded root and reason.
  5. The frontend refetches fresh data via commands on tiny `project:updated` / `session:new` events that carry only IDs (DB-as-truth, events-as-invalidation).
**Plans**: TBD
**UI hint**: yes

### Phase 8: Autostart & OS Polish
**Goal**: User can opt into having the dashboard always running in their tray from login onward, without an unwanted window popping up.
**Depends on**: Phase 7
**Requirements**: AUTO-01, AUTO-02
**Success Criteria** (what must be TRUE):
  1. A Settings toggle enables launch-on-login via the OS-appropriate mechanism (macOS LaunchAgent, Windows Run key, Linux `.desktop` autostart), defaults to off, and can be toggled on/off without restarting the app.
  2. When the user enables autostart and then reboots, the app starts with only the tray icon visible — no main window — and the tray right-click menu still offers a path to show the dashboard.
**Plans**: TBD

### Phase 9: Packaging, Updater & Distribution
**Goal**: Anyone on macOS, Windows, or Linux can install the dashboard from a signed artifact and receive automatic updates thereafter.
**Depends on**: Phase 8
**Requirements**: PKG-01, PKG-02, PKG-03, PKG-04, PKG-05, PKG-06, UPD-01, UPD-02, UPD-03, UPD-04, UPD-05
**Success Criteria** (what must be TRUE):
  1. Pushing a `v*.*.*` tag triggers a GitHub Actions matrix build that produces and attaches a macOS universal `.dmg` (signed+notarized when an Apple Developer cert is provided, unsigned otherwise), Windows `.msi` + `.exe`, and Linux `.deb` + `.AppImage` + `.rpm` to a GitHub Release, plus a source bundle `.tar.gz` with `cargo vendor` and pinned node deps.
  2. Running `cargo install gsd-dashboard` produces a working binary with the frontend bundled in and documented caveats (no app bundle, no auto-update, macOS Gatekeeper warning).
  3. The GitHub Pages site hosts a landing `index.html` with screenshots and per-platform downloads, an `install.sh` one-liner that detects OS/arch and installs the matching latest artifact, and a signed `updates/latest.json` manifest regenerated on every tagged release.
  4. An installed dashboard detects a newer released version, verifies its signature against the public key baked into `tauri.conf.json`, and auto-updates — proven end-to-end on at least one staging release before v1.0.
  5. The updater private key is never stored in plaintext in the repo; it lives in GH Actions secrets and an offline backup, and release CI fails fast if the signing secret is absent.
**Plans**: TBD

## Progress

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Foundation | 4/4 | Complete | 2026-04-24 |
| 2. Planning Parser & Scanner | 4/4 | Complete | 2026-04-25 |
| 3. Portfolio Vertical Slice | 3/5 | In Progress | - |
| 4. Session Indexer | 0/? | Not started | - |
| 5. Project Detail, Global Sessions & Charts | 0/? | Not started | - |
| 6. Tray Icon with Milestone Bars | 0/? | Not started | - |
| 7. Live Updates | 0/? | Not started | - |
| 8. Autostart & OS Polish | 0/? | Not started | - |
| 9. Packaging, Updater & Distribution | 0/? | Not started | - |

---
*Roadmap created: 2026-04-23*
