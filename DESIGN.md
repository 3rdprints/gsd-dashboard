---
name: GSD Dashboard
description: Local command-center UI for GSD project state and Codex session telemetry.
colors:
  app-bg: "#F7F8FA"
  app-surface: "#FFFFFF"
  app-surface-muted: "#F9FAFB"
  app-text: "#111827"
  app-muted: "#4B5563"
  app-subtle: "#6B7280"
  app-border: "#D1D5DB"
  app-border-soft: "#E5E7EB"
  app-control: "#2563EB"
  app-control-text: "#FFFFFF"
  app-danger: "#DC2626"
  dark-bg: "#0E1116"
  dark-surface: "#171B22"
  dark-surface-muted: "#11151C"
  dark-text: "#F3F4F6"
  dark-muted: "#CBD5E1"
  dark-subtle: "#94A3B8"
  dark-border: "#334155"
  dark-border-soft: "#263241"
  dark-control: "#60A5FA"
  dark-control-text: "#07111F"
  dark-danger: "#F87171"
typography:
  title:
    fontFamily: "-apple-system, BlinkMacSystemFont, Segoe UI, Inter, sans-serif"
    fontSize: "28px"
    fontWeight: 600
    lineHeight: 1.2
    letterSpacing: "normal"
  section-title:
    fontFamily: "-apple-system, BlinkMacSystemFont, Segoe UI, Inter, sans-serif"
    fontSize: "20px"
    fontWeight: 600
    lineHeight: 1.2
    letterSpacing: "normal"
  body:
    fontFamily: "-apple-system, BlinkMacSystemFont, Segoe UI, Inter, sans-serif"
    fontSize: "14px"
    fontWeight: 400
    lineHeight: 1.5
    letterSpacing: "normal"
  label:
    fontFamily: "-apple-system, BlinkMacSystemFont, Segoe UI, Inter, sans-serif"
    fontSize: "12px"
    fontWeight: 600
    lineHeight: 1.4
    letterSpacing: "normal"
rounded:
  sm: "6px"
  md: "8px"
  shadcn-base: "0.625rem"
spacing:
  xs: "4px"
  sm: "8px"
  md: "16px"
  lg: "24px"
components:
  panel:
    backgroundColor: "{colors.app-surface}"
    textColor: "{colors.app-text}"
    borderColor: "{colors.app-border}"
    rounded: "{rounded.md}"
    padding: "{spacing.lg}"
  button-primary:
    backgroundColor: "{colors.app-control}"
    textColor: "{colors.app-control-text}"
    rounded: "{rounded.md}"
    padding: "0 16px"
  badge-neutral:
    backgroundColor: "{colors.app-surface-muted}"
    textColor: "{colors.app-muted}"
    rounded: "{rounded.sm}"
    padding: "0 8px"
---

# Design System: GSD Dashboard

## 1. Overview

**Creative North Star: "Local Ops Console"**

GSD Dashboard should feel like a native desktop control surface for work already in motion. It is a product interface with command-center energy: status, recency, errors, phase position, and next action should be visible without turning the app into a marketing page.

The current system is compact, panel-based, and neutral. It uses small labels, tabular numeric emphasis, progress bars, charts, tables, and routed drill-in views. Visual personality should come from crisp hierarchy, exact status language, and operational density, not decorative gradients, glass, or brand theater.

**Key Characteristics:**

- Product register, not brand register.
- Dense but readable surfaces.
- Light and dark mode support.
- Native desktop restraint.
- Read-only trust around `.planning/` data.
- Specific error and watcher states treated as primary UI data.

## 2. Colors

The palette is a restrained operational neutral system with a single blue control accent and explicit warning or danger colors for state.

### Primary

- **Control Blue** (`#2563EB`, dark `#60A5FA`): Primary actions, active progress, focused links, and selected states. Use sparingly so it still means "interactive or active."

### Secondary

- **Danger Red** (`#DC2626`, dark `#F87171`): Destructive or failed states only.
- **Polling Amber** (`#fffbeb`, `#b45309`, `#92400e`): Watcher degradation, update warnings, and non-fatal operational attention.

### Neutral

- **App Background** (`#F7F8FA`, dark `#0E1116`): Whole-window backdrop.
- **Surface** (`#FFFFFF`, dark `#171B22`): Panels, cards, tables, and controls.
- **Muted Surface** (`#F9FAFB`, dark `#11151C`): Nested modules, skeletons, chips, tracks, and low-emphasis regions.
- **Primary Text** (`#111827`, dark `#F3F4F6`): Headings and core values.
- **Muted Text** (`#4B5563`, dark `#CBD5E1`): Labels, metadata, and helper copy.
- **Subtle Text** (`#6B7280`, dark `#94A3B8`): Secondary descriptions and empty-state detail.
- **Border** (`#D1D5DB`, dark `#334155`): Panel and control boundaries.
- **Soft Border** (`#E5E7EB`, dark `#263241`): Internal dividers and low-emphasis separation.

### Named Rules

**The One Accent Rule.** Blue is the control and active-state color. Do not introduce unrelated accent hues unless they encode status or chart series.

**The State Color Rule.** Red, amber, purple, and green must mean something specific. Do not use them as decoration.

**The Neutral Discipline Rule.** Future palette work should move toward tinted OKLCH neutrals rather than pure white or pure black surfaces, but existing tokens remain the current source of truth until code changes.

## 3. Typography

Typography is system-native and utilitarian. The base stack is `-apple-system, BlinkMacSystemFont, "Segoe UI", Inter, sans-serif`, with `@fontsource-variable/geist` available for future token consolidation.

- **Page titles**: `28px`, weight `600`, line-height `1.2`. Use for route-level headings only.
- **Panel and chart titles**: `20px`, weight `600`, line-height `1.2`.
- **Body text**: `14px`, weight `400`, line-height `1.5`.
- **Labels and nav controls**: `12px`, weight `600`, line-height `1.4`.
- **Numeric values**: Use tabular numerals where comparison matters.

Keep type compact. Do not scale text with viewport width. Use hierarchy through size, weight, placement, and value alignment.

## 4. Elevation

The current system is mostly flat. Hierarchy comes from surface layering, borders, spacing, and grid placement rather than shadows.

- Panels use `background: #FFFFFF`, `border: 1px solid #D1D5DB`, `border-radius: 8px`, and `padding: 24px`.
- Nested modules use muted backgrounds such as `#F9FAFB` or tighter padding.
- Interactive controls use borders and focus outlines instead of drop shadows.
- Skeletons and tracks use soft neutral fills such as `#E5E7EB`.

Do not add decorative elevation. If shadow is needed for popovers or menus, keep it shallow, functional, and rare.

## 5. Components

### App Shell

Use a full-window shell with `24px` outer padding and a centered max width near `1280px`. Navigation is compact, horizontal, and control-like.

### Panels

Panels are the default framed unit for settings, status summaries, detail groups, right rail sections, and empty states. Use `8px` radius, `1px` borders, and `24px` padding. Avoid nested panels.

### Project Cards

Project cards summarize one project with milestone, phase, parse state, session recency, and compact metric modules. They should scan quickly in a portfolio grid and never look like marketing feature cards.

### Tables And Lists

Session tables, milestone rows, and watcher details should prioritize stable alignment. Right-align numeric columns. Use hover states only to improve row tracking.

### Tabs

Tabs route within detail surfaces: Overview, Sessions, Charts. Active tabs should be obvious without large decoration. Preserve keyboard semantics.

### Charts

Charts live in framed chart cards. Keep legends compact, use consistent label sizing, and show empty states when data is absent. Chart colors must be distinguishable without relying on hue alone.

### Status Badges

Badges encode source, parse state, update status, watcher mode, or plan counts. Use short labels and restrained fills. Do not turn badges into decorative chips.

### Buttons And Actions

Buttons copy commands, open Finder, open VS Code, rescan, refresh, or save settings. Labels should be verbs. Actions that only open or copy should not look destructive.

## 6. Do's and Don'ts

### Do:

- **Do** prioritize active project state, milestone progress, current phase, last activity, and next useful command.
- **Do** use `#2563EB` for primary control state and focus, with dark mode `#60A5FA`.
- **Do** keep panels compact with `8px` radius, `1px` borders, and predictable `16px` or `24px` spacing.
- **Do** surface parse failures, watcher degradation, update status, and indexing errors with concrete language.
- **Do** preserve visible focus states and keyboard access for navigation, tabs, filters, tables, and settings.
- **Do** communicate read-only trust: the dashboard reads `.planning/`, it does not edit it.

### Don't:

- **Don't** use generic hero metrics, oversized marketing sections, or vague productivity copy.
- **Don't** use gradient text.
- **Don't** add decorative glassmorphism.
- **Don't** create identical icon-card grids.
- **Don't** use neon-on-black, floating orbs, or decorative workflow diagrams.
- **Don't** make read-only project data look editable.
- **Don't** hide critical scan, parse, watcher, or session-indexing errors behind vague status badges.
- **Don't** use side-stripe borders as card, list, callout, or alert accents.
- **Don't** add a modal when inline disclosure, routed detail, or a persistent panel would work.

