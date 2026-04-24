# Phase 1: Foundation - Context

**Gathered:** 2026-04-23
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 1 delivers the launchable desktop foundation: Tauri 2 app skeleton, Rust backend shell, React/TypeScript/Tailwind frontend shell, WAL-mode SQLite cache, settings persistence, first-run defaults, scan-root guardrails, Tauri capabilities, and shared error/event infrastructure.

This phase should prove the app can launch and preserve settings across restarts. It should not implement project scanning, planning parsers, session indexing, tray bars, live watchers, autostart behavior, or packaging.

</domain>

<decisions>
## Implementation Decisions

### Persistence Proof
- **D-01:** Phase 1 should prove a real persistence round-trip: create the SQLite cache in the OS-appropriate app-data directory, enable WAL mode, run migrations, persist settings, and verify settings can be read back after relaunch/reopen.
- **D-02:** The storage foundation should use the locked stack from project context: `rusqlite` with `deadpool-sqlite`, `rusqlite_migration`, and WAL pragmas applied at connection open.

### Scan-Root Guardrails
- **D-03:** Scan-root validation should be centralized in backend/shared domain logic and reused by settings persistence and command handlers. The app must refuse `/` and bare `$HOME` from Phase 1.
- **D-04:** The refusal should surface as a clear UI error in the Phase 1 shell, not just as a logged backend failure.

### First-Run Defaults
- **D-05:** A clean first run should quietly initialize defaults without a setup wizard: scan roots default to `~/Documents`, hidden projects is empty, autostart is off, and tray settings use the project defaults.
- **D-06:** With no discovered data yet, the app should still reach a populated-or-empty dashboard state without requiring configuration.

### Tauri Capabilities
- **D-07:** Capabilities should be release-strict from Phase 1. Avoid broad dev-only permissions that would later diverge from release behavior.
- **D-08:** Include a release-build or release-capability smoke check in the Phase 1 verification plan so Tauri capabilities do not work only in development.

### Error and Event Contracts
- **D-09:** Lock the shared command contract early: every Tauri command returns `Result<T, AppError>` through a single serializable `AppError` type.
- **D-10:** Add a typed `AppEvent` enum using the planned DB-as-truth, events-as-invalidation pattern. Phase 1 only needs the infrastructure and any minimal boot/settings events required by the shell.
- **D-11:** Commands should stay thin; business logic belongs in backend modules such as `store`, `settings`, and guardrail validation.

### Foundation UI Surface
- **D-12:** The visible UI should be a usable shell, not only a placeholder page: it should prove app launch, current/default settings, invalid scan-root error surfacing, and cache/migration status.
- **D-13:** Keep the UI sparse and utilitarian. It should support Phase 1 verification without trying to become the Phase 3 portfolio experience.

### the agent's Discretion
- The planner/researcher may choose exact test structure, module filenames, and scaffold details as long as they follow the project architecture and decisions above.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project Scope
- `.planning/ROADMAP.md` — Phase 1 goal, requirements, and success criteria.
- `.planning/REQUIREMENTS.md` — Foundation requirements `FND-01` through `FND-05`.
- `.planning/PROJECT.md` — project vision, constraints, key decisions, and non-goals.
- `.planning/STATE.md` — current accumulated technical decisions and risk flags.

### Design and Architecture
- `docs/superpowers/specs/2026-04-23-gsd-dashboard-design.md` — canonical design spec for architecture, data model, views, error handling, testing, and packaging constraints.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- No source scaffold exists yet. jCodemunch could not index source files because there are no app source files in the repository.

### Established Patterns
- Planning docs define the intended backend module boundaries: `app_state`, `error`, `events`, `store`, `commands`, and settings/guardrail helpers.
- Project context locks DB-as-truth with typed invalidation events, Tauri 2, Rust, React 19, TypeScript, Tailwind v4 via `@tailwindcss/vite`, Zustand, and TanStack Query.

### Integration Points
- New source should create the initial Tauri/Rust/React scaffold and wire minimal settings/cache commands into the UI shell.
- The read-only `.planning/` invariant must be enforced from the beginning through centralized code and tests; the dashboard must never write into discovered project `.planning/` directories.

</code_context>

<specifics>
## Specific Ideas

- Prefer a practical first-run flow: no onboarding wizard, just sensible defaults and a dashboard/settings shell that can report empty state cleanly.
- The user chose real verification over a cosmetic scaffold. Tests should prove persistence, guardrails, and capabilities where feasible.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within Phase 1 scope.

</deferred>

---

*Phase: 01-foundation*
*Context gathered: 2026-04-23*
