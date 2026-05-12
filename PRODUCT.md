# GSD Dashboard Product Context

## Register

Product.

Design serves a desktop operations dashboard, not a marketing surface. Interfaces should help one technical user scan project state, detect stalled work, and jump into the right GSD or Codex action quickly.

## Product Purpose

GSD Dashboard is a local-first Tauri desktop app for monitoring Get Shit Done projects and AI coding session activity across a user's machine. It reads `.planning/` project data, Claude Code logs, and Codex session telemetry into a derived SQLite cache, then presents portfolio status, milestone progress, active phases, session history, and tray-level progress indicators.

The primary job is glanceable situational awareness: at any moment, the user should know which projects are active, which milestone and phase each project is in, what changed recently, and what command or workspace to open next.

The dashboard is not a GSD editor. It must never write to discovered `.planning/` directories. The CLI skills remain the source of truth for planning and execution changes.

## Users

Primary user: a technical operator who runs many local GSD projects and uses Codex or Claude Code for implementation work.

Use context:

- Mostly macOS desktop, with Windows and Linux supported.
- Repeated short check-ins during a workday, often between coding sessions.
- Fast scanning matters more than narrative explanation.
- The app may run persistently from a menu bar or system tray presence.
- The user already understands GSD concepts such as projects, milestones, phases, plans, and next commands.

## Product Personality

Command center.

The interface should feel operational, precise, and state-aware. It can be assertive about status, recency, progress, and problems, but it should avoid dramatics. It should feel like a native desktop control surface for work already in motion.

Desired qualities:

- Dense but readable.
- Calm under normal conditions.
- Direct when something needs attention.
- Native-feeling instead of web-marketing feeling.
- Built for repeated use, not first-impression spectacle.

## Anti-References

Avoid SaaS cliches:

- No generic hero metrics.
- No gradient text.
- No decorative glassmorphism.
- No identical icon-card grids.
- No oversized marketing sections inside the app.
- No vague productivity copy.
- No "AI dashboard" visual tropes such as neon-on-black, floating orbs, or decorative workflow diagrams.

Avoid false affordances:

- Do not make read-only project data look editable.
- Do not hide critical scan, parse, or watcher errors behind vague status badges.
- Do not use modals as the first answer when inline or routed flows work better.

## Strategic Design Principles

1. Status before decoration.
   Every major screen should answer: what is active, what changed, what is blocked, what should happen next.

2. Native desktop restraint.
   Use compact controls, clear focus states, restrained elevation, and predictable navigation. Prefer platform familiarity over web novelty.

3. Read-only trust.
   The app must consistently communicate that `.planning/` files are source material, not editable dashboard state. Actions can open, copy, rescan, or refresh, but not mutate project plans.

4. Scan first, drill second.
   Portfolio and tray views should support fast comparison across projects. Project detail views can expose timeline, phase, session, and chart depth after selection.

5. Error states are operational data.
   Parse failures, watcher degradation, release capability issues, and session indexing problems should be visible, specific, and actionable.

6. Density with hierarchy.
   Tables, charts, cards, and panels can be information-rich, but headings, spacing, numeric alignment, and status color must make the next useful detail easy to find.

## Accessibility And Inclusion

- Preserve keyboard access for navigation, filters, tabs, settings, and action controls.
- Use visible focus states and semantic controls.
- Do not rely on color alone for project, phase, warning, or completion state.
- Respect light and dark modes because the app may be checked in bright daytime work and low-light evening sessions.
- Keep copy concrete and short. The user knows the domain.

