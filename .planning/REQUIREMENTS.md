# Requirements: GSD Dashboard

**Defined:** 2026-04-23
**Core Value:** At a glance, the user knows what every GSD project is doing right now — which milestone, which phase, how far along — without opening a terminal or reading markdown files.

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### Foundation

- [x] **FND-01**: Tauri 2 application launches on macOS, Windows, and Linux from a single codebase
- [x] **FND-02**: App creates and migrates a SQLite cache in the OS-appropriate app-data directory using WAL mode
- [x] **FND-03**: App reads and persists user settings (scan roots, hidden projects, autostart, tray config) to disk
- [x] **FND-04**: First-run experience defaults scan roots to `~/Documents` with a zero-configuration path to a populated dashboard
- [x] **FND-05**: App refuses to scan `/` or the bare `$HOME` directory as a guardrail against runaway scans

### Discovery & Scanning

- [ ] **SCAN-01**: Scanner discovers every directory containing `.planning/` under each configured scan root
- [x] **SCAN-02**: User can add or remove scan-root directories in Settings; changes take effect without restart
- [x] **SCAN-03**: User can hide and unhide individual projects; hidden projects remain discoverable but disappear from the portfolio and tray
- [x] **SCAN-04**: "Rebuild cache" action in Settings drops the derived cache and re-runs a full scan
- [ ] **SCAN-05**: Scanner reports progress via a streaming channel so the UI can display scan status
- [ ] **SCAN-06**: Per-file parse failures are non-fatal — scanning continues, errors are logged, and affected projects surface a parse-error badge

### Planning Parser

- [ ] **PARSE-01**: Parser extracts current milestone (name + index) for each project from `ROADMAP.md` and `MILESTONES.md`
- [ ] **PARSE-02**: Parser extracts current phase (number + name) for each project, preferring `STATE.md` explicit declaration, falling back to the first unchecked ROADMAP entry
- [ ] **PARSE-03**: Parser derives milestone % complete from ROADMAP top-level phase checkboxes with plan-level fallback
- [ ] **PARSE-04**: Parser extracts per-phase plan checklists from `phases/*/PLAN.md`
- [ ] **PARSE-05**: Parser reads `config.json` project settings when present
- [ ] **PARSE-06**: Parser derives a "next command" recommendation from `STATE.md`, defaulting to `/gsd-next` when unspecified
- [ ] **PARSE-07**: Parser handles decimal phase numbers (e.g. "72.1") correctly
- [ ] **PARSE-08**: Parser validates cleanly against real fixtures from at least 5 existing `.planning/` directories

### Session Indexer (metadata only)

- [ ] **SESS-01**: Indexer streams every Claude Code `.jsonl` file under `~/.claude/projects/` and extracts session metadata (start, end, duration, message count, tokens in/out, model)
- [ ] **SESS-02**: Indexer streams every Codex session file under `~/.codex/sessions/` and extracts the same metadata (best-effort for tokens/model)
- [ ] **SESS-03**: Indexer tolerates partially-written JSONL files (live Claude sessions) without marking them corrupt; tracks byte offsets for incremental parsing
- [ ] **SESS-04**: Indexer attributes Claude sessions to projects by reversing the `.claude/projects` directory-name encoding
- [ ] **SESS-05**: Indexer attributes Codex sessions to projects via parsed `cwd` when available; unmatched sessions surface in a "Global" bucket
- [x] **SESS-06**: Sessions are persisted to SQLite with indexes supporting portfolio, detail, and global-sessions queries

### Portfolio View

- [x] **PORT-01**: Landing view displays one card per non-hidden project, sorted by last activity descending
- [ ] **PORT-02**: Each project card shows name, current milestone + progress bar, current phase label, relative last activity, and a 7-day session sparkline
- [x] **PORT-03**: Clicking a card opens the Project Detail view for that project
- [x] **PORT-04
**: Hovering a card reveals a "Copy next command" action
- [ ] **PORT-05**: Landing view shows a 90-day GitHub-style activity heatmap across all projects
- [x] **PORT-06**: Right rail shows collapsed "Hidden projects" and "Unmatched sessions" sections
- [x] **PORT-07**: Header stats show total projects tracked, active milestones, sessions today, and tokens today

### Project Detail View

- [x] **DET-01**: Detail view shows project name, root path, and action buttons: "Open in Finder", "Open in VS Code", "Copy next command"
- [ ] **DET-02**: Milestone timeline renders one horizontal bar per milestone, shaded by completion; expanding a milestone reveals its phase list
- [ ] **DET-03**: Current phase panel shows phase number/name, plan checklist from PLAN.md, path links, and a STATE.md excerpt
- [ ] **DET-04**: Sessions tab shows all Claude + Codex sessions attributed to this project in a sortable table
- [ ] **DET-05**: Charts tab shows sessions/day (30d), tokens/day, average session duration, and milestone velocity for this project

### Global Sessions View

- [ ] **GLOB-01**: Filterable table across all sessions with filters for source (Claude/Codex), project, date range, duration, and tokens
- [ ] **GLOB-02**: "Unmatched" filter surfaces sessions we couldn't attribute
- [ ] **GLOB-03**: Top charts: sessions/day stacked by source, tokens/day stacked by top-5 projects + "other", time-of-day histogram, day-of-week distribution

### Tray Icon

- [ ] **TRAY-01**: Tray icon renders a dynamic PNG bar graph — one bar per active non-hidden project, ordered by user-configured sort (name/progress/recent activity)
- [ ] **TRAY-02**: Each bar's height is proportional to the project's current milestone % complete
- [ ] **TRAY-03**: On macOS the icon is rendered as a template image so it adapts to light and dark menu bars; proper Retina sizing on all platforms
- [ ] **TRAY-04**: Tray tooltip shows a concise summary (e.g. "3 active projects · Foo 62% · Bar 31%") on hover (added from research — table stakes)
- [ ] **TRAY-05**: Left-click toggles show/hide of the main window (all platforms, including Linux AppIndicator where left-click may not fire)
- [ ] **TRAY-06**: Right-click menu includes: per-project entries (click → open Project Detail), "Copy next command" submenu per active project, Show dashboard, Preferences, Quit
- [ ] **TRAY-07**: Tray re-renders on scan completion, settings changes (max bars, sort order), and watcher-triggered progress updates — without blocking the main thread

### Live Updates

- [ ] **LIVE-01**: Filesystem watcher observes each discovered `.planning/` directory and both session-log roots; debounced at ~500ms at the project level
- [ ] **LIVE-02**: On watcher events, only the affected project or session file is re-parsed and re-persisted
- [ ] **LIVE-03**: On watcher failure (FSEvents limits, Linux inotify exhaustion, permission denied), system falls back to 60-second polling for the affected root
- [ ] **LIVE-04**: Settings surfaces a banner when any watcher is in fallback-polling mode
- [ ] **LIVE-05**: Frontend subscribes to `project:updated` / `session:new` events carrying only IDs; re-queries via commands to refetch fresh data

### Clipboard Integration

- [x] **CLIP-01**: "Copy next command" writes the project-recommended command (or `/gsd-next` default) to the OS clipboard
- [x] **CLIP-02**: The dashboard never executes commands directly and never writes into `.planning/` directories

### Autostart

- [ ] **AUTO-01**: Settings toggle enables/disables launch-on-login via OS-appropriate mechanism (macOS LaunchAgent, Windows Run key, Linux .desktop autostart); default off
- [ ] **AUTO-02**: When launched via autostart, app starts hidden (tray only, no main window)

### Settings UI

- [x] **SET-01**: Scan-roots editor lets users add/remove directories
- [x] **SET-02**: Hidden-projects list shows every hidden project with an Unhide action
- [ ] **SET-03**: Tray display preferences: max bars, sort order, per-project show/hide independent of overall hidden state
- [x] **SET-04**: "Rebuild cache" action available; confirms destructive operation
- [x] **SET-05**: Phase 2 and Phase 3 toggles exist as placeholders (index tool usage / index message content) — visible but disabled

### Packaging & Distribution

- [ ] **PKG-01**: CI matrix (macOS, Windows, Ubuntu) builds on tag push and attaches artifacts to a GitHub Release
- [ ] **PKG-02**: macOS artifact is a universal (arm64 + x86_64) `.dmg` containing a signed `.app` bundle when an Apple Developer cert is provided; unsigned build still succeeds
- [ ] **PKG-03**: Windows artifacts include both `.msi` (WiX) and `.exe` (NSIS) installers
- [ ] **PKG-04**: Linux artifacts include `.deb`, `.AppImage`, and `.rpm`, all produced by Tauri 2's native bundler
- [ ] **PKG-05**: Release workflow also publishes a source bundle tarball (`cargo vendor` + pinned node deps + `BUILD.md`)
- [ ] **PKG-06**: `cargo install gsd-dashboard` installs a working binary with the frontend bundled in; documented caveats (no app bundle, no auto-update, Gatekeeper warning on macOS)

### Auto-Update & Install Site

- [ ] **UPD-01**: `tauri-plugin-updater` fetches `updates/latest.json` from the project's GitHub Pages site and verifies artifact signatures with a baked-in public key
- [ ] **UPD-02**: Release workflow regenerates and publishes the updater manifest on every tagged release
- [ ] **UPD-03**: Updater private key is never stored in plaintext in the repo; stored as a GH Actions secret and backed up offline
- [ ] **UPD-04**: GitHub Pages site hosts `install.sh` (`curl -fsSL … | sh`) that detects OS/arch and installs the matching latest artifact
- [ ] **UPD-05**: GitHub Pages site hosts an `index.html` landing page with screenshots, one-liner install, and per-platform download buttons

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Deep Session Indexing

- **DEEP-01**: Indexer extracts per-session tool-call counts (Edit, Bash, Task, MCP calls) behind a Settings toggle
- **DEEP-02**: Indexer extracts per-session MCP-server usage behind a Settings toggle

### Transcript Search

- **FTS-01**: Message content indexed in SQLite FTS with full-text search UI in the Sessions view

### Tray Popover (v1.x candidate)

- **POP-01**: Left-click opens a compact popover panel anchored to the tray icon, listing top-N projects and current activity — lighter weight than opening the full main window

### Usage Analytics (v1.x candidate)

- **BURN-01**: Token burn-rate and 5-hour billing-window block view (ccusage-style) for Claude Pro/Max users
- **COST-01**: Cost estimation per project and per session based on a bundled pricing table
- **IDLE-01**: Stale-project badge on portfolio cards ("Idle 14d") for long-untouched projects
- **EXPORT-01**: CSV/JSON export of sessions and portfolio snapshots for standups/retros
- **SHORT-01**: Global keyboard shortcut (Cmd+Shift+G) toggles the dashboard

### Milestone Notifications

- **NOTIF-01**: Optional OS notification when a tracked project ships a milestone (opt-in only)

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Writing to `.planning/` directories | Dashboard is strictly read-only; single source of truth stays with the CLI skills |
| In-app execution of GSD commands | Keeps app simple and PTY-free; user pastes into their own CLI session |
| Transcript full-text search (v1) | Deferred to Phase 3 behind a toggle — separate project's worth of scope |
| Tool-call / MCP-call indexing (v1) | Deferred to Phase 2 behind a toggle; v1 metadata alone unlocks ~80% of value |
| Remote / cloud dashboard | Local-first app by design; all data stays on user's machine |
| Cloud sync of settings or hidden-project list | Local-only for v1; add later if multi-device demand emerges |
| Team dashboards / multi-user views | Single-user tool; team features belong in a different product |
| Focus timers or pomodoro features | Adjacent productivity features would dilute core value |
| Git commit tracking per project | Already visible via each project's git log; not this tool's job |
| End-to-end pixel snapshot tests of the tray | Impractical across OSes; tray PNG generation is unit-tested directly |
| Scanning `/` or `$HOME` bare | Runaway-scan guardrail; explicit subdirectories only |
| Apple Developer cert procurement tooling | Out of app scope; release workflow degrades gracefully when absent |
| In-app editor for `.planning/` files | Two writers to the same files invites conflicts with the CLI workflow |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| FND-01 | Phase 1 | Complete |
| FND-02 | Phase 1 | Complete |
| FND-03 | Phase 1 | Complete |
| FND-04 | Phase 1 | Complete |
| FND-05 | Phase 1 | Complete |
| SCAN-01 | Phase 2 | Pending |
| SCAN-02 | Phase 3 | Complete |
| SCAN-03 | Phase 3 | Complete |
| SCAN-04 | Phase 3 | Complete |
| SCAN-05 | Phase 2 | Pending |
| SCAN-06 | Phase 2 | Pending |
| PARSE-01 | Phase 2 | Pending |
| PARSE-02 | Phase 2 | Pending |
| PARSE-03 | Phase 2 | Pending |
| PARSE-04 | Phase 2 | Pending |
| PARSE-05 | Phase 2 | Pending |
| PARSE-06 | Phase 2 | Pending |
| PARSE-07 | Phase 2 | Pending |
| PARSE-08 | Phase 2 | Pending |
| SESS-01 | Phase 4 | Pending |
| SESS-02 | Phase 4 | Pending |
| SESS-03 | Phase 4 | Pending |
| SESS-04 | Phase 4 | Pending |
| SESS-05 | Phase 4 | Pending |
| SESS-06 | Phase 4 | Pending |
| PORT-01 | Phase 3 | Complete |
| PORT-02 | Phase 4 | Pending |
| PORT-03 | Phase 3 | Complete |
| PORT-04 | Phase 3 | Complete |
| PORT-05 | Phase 5 | Pending |
| PORT-06 | Phase 3 | Complete |
| PORT-07 | Phase 3 | Complete |
| DET-01 | Phase 3 | Complete |
| DET-02 | Phase 5 | Pending |
| DET-03 | Phase 5 | Pending |
| DET-04 | Phase 5 | Pending |
| DET-05 | Phase 5 | Pending |
| GLOB-01 | Phase 5 | Pending |
| GLOB-02 | Phase 5 | Pending |
| GLOB-03 | Phase 5 | Pending |
| TRAY-01 | Phase 6 | Pending |
| TRAY-02 | Phase 6 | Pending |
| TRAY-03 | Phase 6 | Pending |
| TRAY-04 | Phase 6 | Pending |
| TRAY-05 | Phase 6 | Pending |
| TRAY-06 | Phase 6 | Pending |
| TRAY-07 | Phase 6 | Pending |
| LIVE-01 | Phase 7 | Pending |
| LIVE-02 | Phase 7 | Pending |
| LIVE-03 | Phase 7 | Pending |
| LIVE-04 | Phase 7 | Pending |
| LIVE-05 | Phase 7 | Pending |
| CLIP-01 | Phase 3 | Complete |
| CLIP-02 | Phase 3 | Complete |
| AUTO-01 | Phase 8 | Pending |
| AUTO-02 | Phase 8 | Pending |
| SET-01 | Phase 3 | Complete |
| SET-02 | Phase 3 | Complete |
| SET-03 | Phase 6 | Pending |
| SET-04 | Phase 3 | Complete |
| SET-05 | Phase 3 | Complete |
| PKG-01 | Phase 9 | Pending |
| PKG-02 | Phase 9 | Pending |
| PKG-03 | Phase 9 | Pending |
| PKG-04 | Phase 9 | Pending |
| PKG-05 | Phase 9 | Pending |
| PKG-06 | Phase 9 | Pending |
| UPD-01 | Phase 9 | Pending |
| UPD-02 | Phase 9 | Pending |
| UPD-03 | Phase 9 | Pending |
| UPD-04 | Phase 9 | Pending |
| UPD-05 | Phase 9 | Pending |

**Coverage:**
- v1 requirements: 72 total (mapped across all v1 categories)
- Mapped to phases: 72
- Unmapped: 0

---
*Requirements defined: 2026-04-23*
*Last updated: 2026-04-23 — traceability filled during roadmap creation*
