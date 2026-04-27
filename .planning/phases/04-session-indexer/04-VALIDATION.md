---
phase: 04
slug: session-indexer
status: verified
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-26
---

# Phase 04 - Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust cargo test + Vitest |
| **Config file** | `src-tauri/Cargo.toml`, `package.json`, `vite.config.ts` |
| **Focused run commands** | `cargo test --manifest-path src-tauri/Cargo.toml session_repo -- --nocapture`; `cargo test --manifest-path src-tauri/Cargo.toml session_indexer -- --nocapture`; `cargo test --manifest-path src-tauri/Cargo.toml rebuild_cache_rematches_existing_sessions_after_project_roots_refresh -- --nocapture`; `npm test -- --run` |
| **Full suite command** | `cargo test --manifest-path src-tauri/Cargo.toml && npm test -- --run` |
| **Estimated runtime** | focused commands <30 seconds each; full wave/phase suite ~90 seconds |

---

## Sampling Rate

- **After every backend task commit:** Run the plan-specific Cargo command from the map below.
- **After every frontend task commit:** Run `npm test -- --run`
- **After every plan wave:** Run `cargo test --manifest-path src-tauri/Cargo.toml && npm test -- --run`
- **Before `$gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** <30 seconds for focused per-task commands; the ~90-second full suite is reserved for wave and phase gates.

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 04-01-00 | 04-01 | 1 | SESS-06 | T-04-01, T-04-20 | Repository tests assert metadata-only schema and transactional offset behavior before production code lands | Rust integration | `cargo test --manifest-path src-tauri/Cargo.toml session_repo -- --nocapture` | Yes | green |
| 04-01-01 | 04-01 | 1 | SESS-06 | T-04-01, T-04-02, T-04-03 | Migration stores only metadata, preserves session rows across project deletion, and creates query indexes | Rust integration | `cargo test --manifest-path src-tauri/Cargo.toml session_repo store_migrations -- --nocapture` | Yes | green |
| 04-01-02 | 04-01 | 1 | SESS-06 | T-04-04, T-04-20 | `persist_indexed_file_result(...)` commits sessions and offset state atomically | Rust integration | `cargo test --manifest-path src-tauri/Cargo.toml session_repo -- --nocapture` | Yes | green |
| 04-01-03 | 04-01 | 1 | SESS-04, SESS-06 | T-04-02, T-04-20 | `rematch_unmatched_sessions_against_projects(...)` restores `project_id` after rebuild without touching offsets | Rust integration | `cargo test --manifest-path src-tauri/Cargo.toml rematch_unmatched_sessions_against_projects_restores_project_id_after_rebuild -- --nocapture` | Yes | green |
| 04-02-00 | 04-02 | 2 | SESS-01, SESS-02, SESS-03, SESS-04, SESS-05 | T-04-05, T-04-09 | Sanitized fixtures assert Claude/Codex `started_at`, `ended_at`, `duration_ms`, partial-line, and attribution behavior | Rust integration | `cargo test --manifest-path src-tauri/Cargo.toml session_indexer -- --nocapture` | Yes | green |
| 04-02-01 | 04-02 | 2 | SESS-01, SESS-02 | T-04-05 | Parsers extract metadata only and parse timestamps to epoch milliseconds for normal records | Rust integration | `cargo test --manifest-path src-tauri/Cargo.toml session_indexer -- --nocapture` | Yes | green |
| 04-02-02 | 04-02 | 2 | SESS-03, SESS-04, SESS-05 | T-04-06, T-04-07, T-04-08 | Complete-line streaming preserves final partial bytes and attribution decodes Claude encoded project directories against known roots without dropping unmatched sessions | Rust integration | `cargo test --manifest-path src-tauri/Cargo.toml claude_path_fallback_decodes_directory_encoding_against_known_roots -- --nocapture` | Yes | green |
| 04-03-01 | 04-03 | 3 | SESS-01, SESS-02, SESS-03, SESS-04, SESS-05, SESS-06 | T-04-10, T-04-21 | Command orchestration reuses offsets and proves failed persistence does not advance offsets | Rust integration | `cargo test --manifest-path src-tauri/Cargo.toml session_indexer -- --nocapture` | Yes | green |
| 04-03-02 | 04-03 | 3 | SESS-01, SESS-02, SESS-03, SESS-04, SESS-05, SESS-06 | T-04-10, T-04-11, T-04-12, T-04-13 | `index_sessions` scans only fixed roots, emits metadata-only events, and persists through repository transaction helper | Rust integration | `cargo test --manifest-path src-tauri/Cargo.toml session_indexer -- --nocapture` | Yes | green |
| 04-03-03 | 04-03 | 3 | SESS-01, SESS-02 | T-04-14 | Command registration and release capability allow indexing without frontend filesystem read scope | Rust integration | `cargo test --manifest-path src-tauri/Cargo.toml session_indexer -- --nocapture` | Yes | green |
| 04-03-04 | 04-03 | 3 | SESS-04, SESS-06 | T-04-02, T-04-21 | Rebuild-cache orchestration rematches preserved session rows after project roots refresh and does not clear offsets | Rust integration | `cargo test --manifest-path src-tauri/Cargo.toml rebuild_cache_rematches_existing_sessions_after_project_roots_refresh -- --nocapture` | Yes | green |
| 04-04-01 | 04-04 | 4 | SESS-05, SESS-06, PORT-02 | T-04-15 | Portfolio DTO aggregates expose derived counts/tokens and unmatched metadata only | Rust integration | `cargo test --manifest-path src-tauri/Cargo.toml portfolio_commands session_repo -- --nocapture` | Yes | green |
| 04-04-02 | 04-04 | 4 | SESS-05, SESS-06, PORT-02 | T-04-17, T-04-19 | Frontend manual `Index Sessions` trigger invalidates portfolio query and shows nonblocking progress | Vitest | `npm test -- --run` | Yes | green |
| 04-04-03 | 04-04 | 4 | SESS-05, PORT-02 | T-04-16, T-04-18 | Cards render fixed seven-day sparklines and right rail renders neutral unmatched summary | Vitest | `npm test -- --run` | Yes | green |

*Status: pending, green, red, flaky.*

---

## Wave 0 Requirements

- [x] `src-tauri/tests/session_indexer.rs` - fixtures for Claude, Codex, concrete start/end/duration timestamps, partial trailing JSONL, Claude no-cwd encoded-directory fallback, unmatched sessions, incremental offsets, command orchestration, and persistence-failure offset rollback
- [x] `src-tauri/tests/session_repo.rs` - SQLite persistence, indexes, replacement/upsert behavior, transactional `persist_indexed_file_result(...)`, offset rollback on failed session writes, rematch helper coverage, and aggregate queries
- [x] `src-tauri/tests/rebuild_cache.rs` - rebuild-cache rematch coverage proving sessions and offsets survive project-row refresh and `project_id` is restored
- [x] `src/App.test.tsx` - portfolio session stat and 7-day sparkline assertions
- [x] `src-tauri/fixtures/sessions/` - small sanitized JSONL fixtures with no private prompt content

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| None | n/a | No Nyquist gaps require manual verification | Automated fixtures and integration tests cover Phase 04 requirement behavior |

---

## Validation Sign-Off

- [x] All tasks have automated verify commands or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all missing references
- [x] No watch-mode flags
- [x] Focused feedback latency < 30s; full suite ~90s only at wave/phase gates
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** verified 2026-04-27

## Validation Audit 2026-04-27

| Metric | Count |
|--------|-------|
| Gaps found | 0 |
| Resolved | 0 |
| Escalated | 0 |

| Requirement | Coverage | Evidence |
|-------------|----------|----------|
| SESS-01 | COVERED | Claude fixture parsing and fixed-root app indexing tests in `src-tauri/tests/session_indexer.rs` |
| SESS-02 | COVERED | Codex fixture parsing and sparse optional-field fallback tests in `src-tauri/tests/session_indexer.rs` |
| SESS-03 | COVERED | Partial-line, incremental offset, append, truncation, and failed-persistence rollback tests in `src-tauri/tests/session_indexer.rs` and `src-tauri/tests/session_repo.rs` |
| SESS-04 | COVERED | Encoded Claude project-path attribution and rebuild rematch tests in `src-tauri/tests/session_indexer.rs`, `src-tauri/tests/session_repo.rs`, and `src-tauri/tests/rebuild_cache.rs` |
| SESS-05 | COVERED | `cwd` matching, unmatched session aggregate, right rail, and sparkline tests in `src-tauri/tests/session_indexer.rs`, `src-tauri/tests/portfolio_commands.rs`, and `src/App.test.tsx` |
| SESS-06 | COVERED | Metadata-only schema, transactional persistence, indexes, offsets, and portfolio aggregate tests in `src-tauri/tests/session_repo.rs`, `src-tauri/tests/store_migrations.rs`, and `src-tauri/tests/portfolio_commands.rs` |
| PORT-02 | COVERED | Seven-day project-card sparkline tests in `src-tauri/tests/portfolio_commands.rs` and `src/App.test.tsx` |

Verification run:

- `cargo test --manifest-path src-tauri/Cargo.toml --test session_repo -- --nocapture` - 6 passed
- `cargo test --manifest-path src-tauri/Cargo.toml --test session_indexer -- --nocapture` - 17 passed
- `cargo test --manifest-path src-tauri/Cargo.toml --test portfolio_commands -- --nocapture` - 8 passed
- `npm test -- --run` - 22 passed
- `cargo test --manifest-path src-tauri/Cargo.toml` - full Rust suite passed
