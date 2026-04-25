---
phase: 3
slug: portfolio-vertical-slice
status: draft
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-25
---

# Phase 3 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Vitest 4.1.5, React Testing Library 16.3.2, Cargo test |
| **Config file** | `vite.config.ts`, `src-tauri/Cargo.toml` |
| **Quick run command** | `npm test -- src/App.test.tsx && cargo test --manifest-path src-tauri/Cargo.toml settings -- --nocapture && cargo test --manifest-path src-tauri/Cargo.toml project_repo -- --nocapture` |
| **Full suite command** | `npm test && cargo test --manifest-path src-tauri/Cargo.toml` |
| **Estimated runtime** | ~45 seconds |

---

## Sampling Rate

- **After every task commit:** Run `npm test -- src/App.test.tsx && cargo test --manifest-path src-tauri/Cargo.toml settings -- --nocapture && cargo test --manifest-path src-tauri/Cargo.toml project_repo -- --nocapture`
- **After every plan wave:** Run `npm test && cargo test --manifest-path src-tauri/Cargo.toml`
- **Before `$gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 3-01-01 | 01 | 1 | PORT-01, PORT-03, PORT-06, PORT-07, DET-01 | T-3-01, T-3-02, T-3-03, T-3-04 | Portfolio/detail DTOs filter hidden projects without deleting cache rows and sort by `COALESCE(last_activity_at,last_scanned_at)` | Rust integration | `cargo test --manifest-path src-tauri/Cargo.toml portfolio_commands -- --nocapture` | W0 | green |
| 3-02-01 | 02 | 2 | SCAN-02, SCAN-03, SCAN-04, SET-01, SET-02, SET-04, SET-05 | T-3-05, T-3-06, T-3-07, T-3-08 | Rebuild cache clears only derived rows, preserves settings/hidden IDs, and reuses scan guardrails/events | Rust integration | `cargo test --manifest-path src-tauri/Cargo.toml rebuild_cache -- --nocapture` | W0 | green |
| 3-03-01 | 03 | 3 | CLIP-01, CLIP-02, DET-01 | T-3-09, T-3-10, T-3-11, T-3-12 | Clipboard/open actions use official plugins, preserve VS Code path separators, and do not mutate project source files or `.planning/` | Vitest + capability check | `npm test -- src/App.test.tsx && cargo check --manifest-path src-tauri/Cargo.toml` | W0 | green |
| 3-04-01 | 04 | 4 | SCAN-02, SCAN-03, SCAN-04, PORT-01, PORT-03, PORT-04, PORT-06, PORT-07, CLIP-01, CLIP-02, DET-01, SET-01, SET-02, SET-04, SET-05 | T-3-13, T-3-14, T-3-15, T-3-16, T-3-17 | Routed UI shows non-hidden cards, detail actions, Settings scan roots, hidden/unhide, rebuild, and disabled toggles | Vitest + build | `npm test -- src/App.test.tsx && npm run build` | W0 | green |
| 3-05-01 | 05 | 5 | SCAN-02, SCAN-03, SCAN-04, PORT-01, PORT-03, PORT-04, PORT-06, PORT-07, CLIP-01, CLIP-02, DET-01, SET-01, SET-02, SET-04, SET-05 | T-3-18, T-3-19, T-3-20, T-3-21 | Final hardening verifies full suites, exact Tauri capability IDs, no command execution, no source writes, and UI contract constraints | Full suite + ACL smoke | `node -e 'const fs=require("fs"); const cap=JSON.parse(fs.readFileSync("src-tauri/capabilities/default.json","utf8")); const ids=cap.permissions.map((permission)=>typeof permission==="string"?permission:permission.identifier); const required=["allow-get-portfolio","allow-get-project","allow-rebuild-cache","clipboard-manager:allow-write-text","opener:allow-open-path","opener:allow-open-url"]; const forbidden=["fs:allow-read-dir","fs:allow-write-file","shell:allow-execute","shell:allow-spawn"]; const missing=required.filter((id)=>!ids.includes(id)); const presentForbidden=forbidden.filter((id)=>ids.includes(id)); if(missing.length||presentForbidden.length){ console.error(JSON.stringify({missing,presentForbidden},null,2)); process.exit(1); }' && npm test && cargo test --manifest-path src-tauri/Cargo.toml` | W0 | green |

*Status: pending, green, red, flaky*

---

## Wave 0 Requirements

- [x] `src-tauri/tests/portfolio_commands.rs` — portfolio/detail command coverage for PORT and DET requirements.
- [x] `src-tauri/tests/rebuild_cache.rs` — rebuild-cache coverage for SCAN and SET requirements.
- [x] `src/App.test.tsx` — frontend coverage for cards, detail route, settings roots, hide/unhide, disabled toggles, and copy feedback.
- [x] Tauri plugin capability smoke check parses `src-tauri/capabilities/default.json` and requires exact IDs: `clipboard-manager:allow-write-text`, `opener:allow-open-path`, and `opener:allow-open-url`.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Open in Finder | DET-01 | Requires OS file manager integration | Launch dev app, open Project Detail, click Open in Finder, verify the project root is revealed. |
| Open in VS Code | DET-01 | Depends on local VS Code installation and platform app registration | Launch dev app, open Project Detail, click Open in VS Code, verify the project root opens in VS Code or a surfaced error appears. |
| Clipboard action | CLIP-01, CLIP-02 | Automated tests should mock plugin calls, but OS clipboard needs smoke coverage | Copy next command from card hover and detail view, paste into a text field, verify the expected command text. |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 60s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** automated green, 2026-04-25
