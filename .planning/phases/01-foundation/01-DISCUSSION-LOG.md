# Phase 1: Foundation - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-23T20:33:21-04:00
**Phase:** 1-Foundation
**Areas discussed:** Persistence proof, scan-root guardrails, first-run defaults, Tauri capabilities, error and event contracts, foundation UI surface

---

## Persistence Proof

| Option | Description | Selected |
|--------|-------------|----------|
| Real round-trip | Create SQLite, run migrations, persist settings, and verify relaunch/readback behavior in tests. | yes |
| Schema only | Set up database and migrations but leave most settings behavior to later phases. | |
| Minimal boot | Only prove the app launches; defer persistence details despite the Phase 1 success criteria. | |

**User's choice:** Real round-trip.
**Notes:** This locks a real persistence proof into Phase 1 rather than a cosmetic scaffold.

---

## Scan-Root Guardrails

| Option | Description | Selected |
|--------|-------------|----------|
| Shared validation | One backend validator powers settings persistence and UI errors for `/` and bare `$HOME`. | yes |
| UI-only message | Show the refusal in the frontend first and harden backend validation later. | |
| Backend only | Reject invalid roots in commands/tests, with polished UI messaging deferred. | |

**User's choice:** Shared validation.
**Notes:** The refusal should be enforced centrally and surfaced clearly in the UI.

---

## First-Run Defaults

| Option | Description | Selected |
|--------|-------------|----------|
| Quiet defaults | Initialize `~/Documents`, empty hidden projects, autostart off, and tray defaults without a setup wizard. | yes |
| Setup prompt | Show an initial choice screen before creating scan roots. | |
| Empty config | Start with no scan roots and require the user to configure Settings later. | |

**User's choice:** Quiet defaults.
**Notes:** First run should reach an empty-or-populated dashboard without configuration.

---

## Tauri Capabilities

| Option | Description | Selected |
|--------|-------------|----------|
| Release-strict | Define least-privilege capabilities now and include a release-build smoke check so dev/release do not diverge. | yes |
| Dev-first | Use broad dev permissions to move fast, then tighten before packaging. | |
| Backend-only | Avoid frontend filesystem/plugin capability concerns until scan and settings UI land. | |

**User's choice:** Release-strict.
**Notes:** Phase 1 planning should include a release capability verification path.

---

## Error and Event Contracts

| Option | Description | Selected |
|--------|-------------|----------|
| Typed contracts | Add a shared `AppError`, typed `AppEvent`, and thin command pattern for future DB-as-truth invalidation. | yes |
| Error only | Standardize command errors now, but defer event shapes until live updates. | |
| Loose now | Use simple strings/results while the app shell is still forming. | |

**User's choice:** Typed contracts.
**Notes:** This locks shared backend/frontend command behavior early.

---

## Foundation UI Surface

| Option | Description | Selected |
|--------|-------------|----------|
| Usable shell | Show a sparse dashboard/settings surface that proves launch, defaults, invalid-root errors, and cache status. | yes |
| Placeholder page | Render a simple app shell only; no settings controls yet. | |
| No UI focus | Prioritize backend foundation and leave visible UI to Phase 3. | |

**User's choice:** Usable shell.
**Notes:** The UI should be practical but not expand into Phase 3 portfolio work.

---

## the agent's Discretion

- Exact module filenames, test layout, and scaffold details may be decided during research/planning.

## Deferred Ideas

- None.
