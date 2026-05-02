# GSD Dashboard

## What This Is

A native-feeling desktop application (macOS primary, Windows/Linux supported) that aggregates Get Shit Done project data across the user's machine and augments it with activity telemetry from Claude Code and Codex session logs. It surfaces current milestone and phase status for every tracked project, shows when and where AI coding sessions happened, and exposes milestone progress as live bars in the OS menu bar / system tray.

## Core Value

At a glance, the user knows what every GSD project is doing right now — which milestone, which phase, how far along — without opening a terminal or reading markdown files.

## Requirements

### Validated

- [x] Phase 04: Session indexer (metadata only for v1): Claude Code `.jsonl` and Codex session files — start/end, duration, message count, tokens, model, byte-offset incremental parsing, live partial tolerance, and project attribution.
- [x] Phase 04: Project ↔ session attribution: reverse Claude directory-name encoding; parse Codex `cwd` when present; unmatched sessions remain visible in the portfolio right rail.
- [x] Phase 04: Portfolio landing session sparkline data is backed by indexed session rows.
- [x] Phase 06: Menu bar / tray icon with dynamically rendered milestone-progress bars, configured sort order, independent tray visibility, macOS template behavior, and native tray refresh on scan/settings changes.
- [x] Phase 06: Tray right-click menu with Show Dashboard, Preferences, visible project rows, per-project Copy Next Command submenu, and Quit.
- [x] Phase 06: Settings Tray Display controls for max bars, sort order, and per-project tray visibility independent from Portfolio hidden state.

### Active

- [ ] Cross-platform desktop app: macOS, Windows, Linux (single codebase)
- [ ] Auto-discover `.planning/` projects under configurable scan roots (default `~/Documents`) with hide/unhide toggle
- [ ] Configurable scan roots (add/remove directories); refuse to scan `/` or `$HOME` directly as a guardrail
- [ ] Portfolio landing view: card per project with current milestone + progress bar, current phase, last activity, 7-day session sparkline
- [ ] Project Detail view: milestone timeline, current-phase panel with plan checklist, sessions tab, per-project charts
- [ ] Sessions global view: filterable table across all Claude + Codex sessions plus aggregate charts
- [ ] 90-day activity heatmap on portfolio landing (GitHub-style)
- [ ] Strictly read-only against `.planning/` directories (never mutate)
- [ ] Click action on a project copies the recommended next GSD command to the OS clipboard for paste into Claude Code / Codex
- [ ] Session indexer (metadata only for v1): Claude Code `.jsonl` and Codex session files — start/end, duration, message count, tokens, model, project attribution
- [ ] Project ↔ session attribution: reverse Claude directory-name encoding; parse Codex `cwd` when present; unmatched sessions visible in a "Global" bucket
- [ ] Live updates via filesystem watchers on `.planning/` dirs and session-log dirs; 60-second polling fallback when watchers fail
- [ ] SQLite-backed derived cache in the OS-appropriate app-data dir with "Rebuild cache" action in Settings
- [ ] Launch on login support on all three OSes, toggle in Settings, default off; autostart launch starts hidden (tray only)
- [ ] Distributable artifacts: macOS `.dmg` + `.app` (universal), Windows `.msi` + `.exe`, Linux `.deb` + `.AppImage` + `.rpm`, plus source bundle `.tar.gz`
- [ ] GitHub Pages hosting: landing page, `install.sh` one-liner, and `updates/latest.json` manifest for `tauri-plugin-updater`
- [ ] Auto-updater wired to the GitHub Pages manifest with signed releases
- [ ] Settings UI: scan roots, hidden projects, autostart toggle, rebuild cache, phase 2/3 toggles
- [ ] Per-file parse resilience: malformed ROADMAP/STATE/JSONL never crashes the scan; errors surfaced as per-project badges and logged to `scan_log`
- [ ] `cargo install gsd-dashboard` as a tertiary install channel for Rust developers (frontend bundled into published crate; documented caveats)
- [ ] Codex parser hardening: discover `CODEX_HOME`, `~/.codex/sessions/`, and `~/.codex/archived_sessions/`; parse surface, subagent, model/provider, and reasoning metadata where present
- [ ] GSD run attribution: detect `$gsd-*` and `/gsd-*` calls from session activity and attach them to project, phase, command, and source session where possible
- [ ] jCodemunch/jDocMunch usage metrics: capture derived MCP telemetry such as tool name, timing, result counts, confidence/errors, and token-efficiency metadata per GSD call
- [ ] Tool Efficiency dashboard: show per-GSD-call jSuite usage, high-value queries, expensive/failed queries, and token-saved trends without storing raw MCP result bodies by default

### Out of Scope

- Writing to `.planning/` directories — single-source-of-truth stays with the CLI skills; two writers invite conflicts
- In-app execution of GSD commands — we copy to clipboard so the user pastes into their existing Claude Code / Codex CLI
- Full raw transcript indexing and message-content search by default — deferred to a future opt-in milestone; Phase 10 only stores derived metadata and redacted attribution by default
- Raw tool-call and MCP-call payload storage by default — deferred to a future opt-in milestone; Phase 10 stores jSuite usage metrics and attribution, not raw result bodies
- Remote / cloud dashboard — this is a local-first desktop app
- End-to-end pixel snapshot tests of the tray icon — impractical; tray PNG generation is unit-tested directly instead
- Apple code signing + notarization cert procurement workflow — release pipeline degrades gracefully when the secret is absent
- Scanning `/` or `$HOME` directly — guardrail against runaway scans; only explicit subdirectories allowed

## Context

- User maintains 8+ GSD projects simultaneously across `~/homegit/*`; current "which project needs attention" signal is ad hoc.
- Both Claude Code (`~/.claude/projects/<encoded-path>/*.jsonl`) and Codex (`~/.codex/sessions/YYYY/MM/DD/*`) session stores contain rich activity data that is currently invisible outside the CLIs.
- Claude session dirs encode the project path (`-Users-smacdonald-homegit-deckpilot-web` → `/Users/smacdonald/homegit/deckpilot-web`), giving us a reliable project-attribution path. Codex sessions are looser and will rely on best-effort `cwd` parsing with a "Global" fallback bucket.
- Real `.planning/` directories are available as parsing fixtures (deckpilot-web, listingguru, locdirectory, and others) to validate the ROADMAP/STATE/PHASE parsers against actual in-the-wild variance.
- Inspiration for the tray bar-graph aesthetic: robinebers/openusage.
- Design spec with full architecture, data model, views, testing, and packaging plan lives at `docs/superpowers/specs/2026-04-23-gsd-dashboard-design.md` (committed on `main`).

## Constraints

- **Tech stack**: Tauri 2 + Rust (backend) + React + TypeScript + Tailwind (frontend) — chosen for small binary size, native tray rendering across OSes, and file-system-heavy backend work that suits Rust.
- **Compatibility**: First-class support for macOS (Apple Silicon + Intel via universal binary), Windows 10+, and Linux (deb/rpm/AppImage families).
- **Read-only coupling**: Must never write into any discovered `.planning/` directory; the CLI skills are the sole mutators of that data.
- **Local-first**: No cloud services, no remote aggregation. All data stays on the user's machine. SQLite cache is derived; raw files remain source of truth.
- **Footprint**: Dashboard is expected to stay running; idle CPU ~0% and RAM targeted under ~150MB (Tauri's footprint profile, not Electron's).
- **Performance**: Initial scan of a realistic workspace (10+ projects, thousands of session files) should complete in under ~15 seconds; incremental updates via watcher events should be sub-second.
- **Charting library**: Recharts (lightweight, composable) — deliberately not a heavier chart framework.
- **State library**: Zustand — deliberately not Redux.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Tauri 2 over Electron | ~10-20MB binary vs ~100-150MB, ~50-100MB RAM vs 150-300MB, and native tray-rendering APIs on all three OSes (critical for the openusage-style bar graph) | — Pending |
| React + TypeScript + Tailwind frontend | Familiar stack, large ecosystem, fast dev cycles; no reason to pick an exotic UI framework for this surface | — Pending |
| SQLite via `rusqlite` for derived cache | Zero-ops local DB fits single-user desktop profile; FTS available later for transcript search | — Pending |
| Strictly read-only against `.planning/` | Prevents split-brain between dashboard and CLI skills; single source of truth stays with the authoring workflow | — Pending |
| "Copy next command" instead of in-app execution | Keeps dashboard simple, avoids PTY/terminal-emulator complexity, lets users paste into whichever CLI they're already in | — Pending |
| Session metadata first (Phase 1); tool-usage and message-content indexing deferred behind toggles | ~80% of useful analytics come from metadata alone; FTS is a separate project's worth of work | — Pending |
| Default scan root = `~/Documents` with explicit roots list | Avoids scanning entire home dir; user owns the list; guardrail refuses `/` and `$HOME` | — Pending |
| Milestone progress from `ROADMAP.md` top-level checkboxes with plan-level fallback | Matches how projects actually report progress; degrades gracefully when top-level isn't reliable | — Pending |
| GitHub Pages for install script + updater manifest | Static, free, no server to run; `tauri-plugin-updater` reads a signed JSON manifest | — Pending |
| `cargo install` as tertiary channel with documented caveats | Serves Rust-native install habits; first-class installers remain `.dmg`/`.msi`/`.rpm`/`.deb`/`.AppImage` | — Pending |
| Launch on login default off | Respect user autonomy; obvious opt-in via Settings | — Pending |
| Tauri `notify` watchers with 60s polling fallback | Handles FSEvents quirks and permission-denied roots without losing liveness | — Pending |
| Phase 10 parser/tool telemetry before broader replay | The high-value slice is GSD run attribution plus jCodemunch/jDocMunch efficiency metrics; raw replay/search stays opt-in later | — Pending |

## Future Milestone: v1.1 Parser & Agent Telemetry Expansion

Goal: broaden the dashboard from v1.0's focused GSD and jSuite observability into a fuller local agent-session analytics layer, after installers and the high-value Phase 10 slice are stable.

Candidate upgrades:

- Full session replay and transcript hydration with explicit opt-in, redaction, and searchable local indexes.
- Wider agent-session parser coverage inspired by cc-lens and agent-sessions, including additional Codex surfaces and other agent CLIs where local logs are available.
- gsd-sdk parity checks or an optional SDK-backed adapter for `.planning/` parsing drift detection.
- Deeper jSuite optimization analytics: repeated query detection, budget-warning trends, low-confidence query analysis, and suggested query rewrites.
- Cross-project and cross-milestone efficiency reports for token use, tool mix, and GSD command outcomes.
- Exportable anonymized performance summaries for retrospectives without exposing raw prompts, transcripts, or tool outputs.

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-05-01 after adding Phase 10 parser/tool telemetry and the v1.1 telemetry expansion milestone*
