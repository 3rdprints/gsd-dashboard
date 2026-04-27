---
phase: 04
slug: session-indexer
status: verified
threats_open: 0
asvs_level: 1
created: 2026-04-27
---

# Phase 04 - Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| Local session JSONL -> parser | Private local session records are treated as untrusted local input. | Session metadata, paths, malformed JSONL |
| Parser/indexer -> SQLite | Parsed records are reduced to metadata-only rows and byte-offset state. | Session metadata, index state |
| SQLite session aggregates -> frontend DTO | Private session metadata is summarized for UI display. | Counts, token totals, source mix, source paths |
| Frontend IPC -> backend command | Renderer can trigger indexing but cannot choose arbitrary session roots. | Command invocation and progress events |

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
|-----------|----------|-----------|-------------|------------|--------|
| T-04-01 | I | `sessions` table | mitigate | Metadata-only schema; no prompt/transcript/content storage. Evidence: `src-tauri/src/store/migrations.rs`, `src-tauri/tests/session_repo.rs`. | closed |
| T-04-02 | T | `sessions.project_id` FK | mitigate | Nullable FK uses `ON DELETE SET NULL`. Evidence: `src-tauri/src/store/migrations.rs`. | closed |
| T-04-03 | D | SQLite aggregate indexes | mitigate | Session aggregate indexes exist for project, source, unmatched, and started-at queries. Evidence: `src-tauri/src/store/migrations.rs`. | closed |
| T-04-04 | R | repository writes | mitigate | Session writes are centralized in app-owned SQLite repository helpers. Evidence: `src-tauri/src/sessions/repo.rs`. | closed |
| T-04-20 | T | `persist_indexed_file_result` | mitigate | Sessions and byte-offset state persist in one transaction; regression tests cover rollback on failure. Evidence: `src-tauri/src/sessions/repo.rs`, `src-tauri/tests/session_repo.rs`. | closed |
| T-04-05 | I | `claude.rs`, `codex.rs` | mitigate | Parsers extract metadata only; prompt/content/transcript are not persisted. Evidence: `src-tauri/src/sessions/claude.rs`, `src-tauri/src/sessions/codex.rs`. | closed |
| T-04-06 | T | `indexer.rs` byte offsets | mitigate | Offsets advance only for newline-terminated parsed or skipped records. Evidence: `src-tauri/src/sessions/indexer.rs`. | closed |
| T-04-07 | D | malformed JSONL | mitigate | Malformed middle lines are nonfatal and final incomplete records are tracked as live partials. Evidence: `src-tauri/src/sessions/indexer.rs`, `src-tauri/tests/session_indexer.rs`. | closed |
| T-04-08 | S/T | `matcher.rs` cwd/path attribution | mitigate | Telemetry paths are matched only against known project roots and never authorize reads or writes. Evidence: `src-tauri/src/sessions/matcher.rs`, `src-tauri/tests/session_indexer.rs`. | closed |
| T-04-09 | I | fixture data | mitigate | Test fixtures are sanitized fake session files under `src-tauri/fixtures/sessions`. | closed |
| T-04-10 | I | `SessionIndexEvent` | mitigate | Events carry source/path/count/status fields only, not message bodies. Evidence: `src-tauri/src/events.rs`. | closed |
| T-04-11 | E/T | `index_sessions` command input | mitigate | IPC command accepts no user-supplied path. Evidence: `src-tauri/src/commands/sessions.rs`, `src-tauri/src/sessions/indexer.rs`. | closed |
| T-04-12 | D | filesystem traversal | mitigate | Discovery is limited to fixed Claude/Codex roots and `.jsonl` files. Evidence: `src-tauri/src/sessions/indexer.rs`. | closed |
| T-04-13 | D | async runtime | mitigate | Sync filesystem work runs in `spawn_blocking`; SQLite work runs through deadpool `interact`. Evidence: `src-tauri/src/sessions/indexer.rs`. | closed |
| T-04-14 | T | release capability | mitigate | Release capability grants `allow-index-sessions` without frontend `fs:*` read scopes. Evidence: `src-tauri/capabilities/default.json`, `src-tauri/tests/session_indexer.rs`. | closed |
| T-04-21 | T | per-file persistence | mitigate | Indexer only advances state through transactional `persist_indexed_file_result`; failure test preserves prior offset. Evidence: `src-tauri/src/sessions/indexer.rs`, `src-tauri/src/sessions/repo.rs`, `src-tauri/tests/session_indexer.rs`. | closed |
| T-04-15 | I | `get_portfolio_for_app` | mitigate | Portfolio DTOs expose metadata aggregates and source paths only. Evidence: `src-tauri/src/commands/projects.rs`, `src/lib/types.ts`. | closed |
| T-04-16 | I | `RightRail` unmatched paths | accept | Accepted local-only path disclosure for unmatched visibility; no message content is displayed. | closed |
| T-04-17 | D | duplicate indexing | mitigate | UI disables duplicate `Index Sessions` trigger while indexing is active. Evidence: `src/routes/PortfolioPage.tsx`. | closed |
| T-04-18 | I | sparkline UI | mitigate | Sparkline exposes aggregate counts and accessible summary text only. Evidence: `src/components/ProjectCard.tsx`. | closed |
| T-04-19 | T | query refresh | mitigate | Successful indexing invalidates portfolio query data instead of relying on speculative local cache. Evidence: `src/routes/PortfolioPage.tsx`. | closed |

## Accepted Risks Log

| Risk ID | Threat Ref | Rationale | Accepted By | Date |
|---------|------------|-----------|-------------|------|
| AR-04-01 | T-04-16 | Unmatched source paths are required to explain unmatched sessions in the local-only dashboard; prompt, transcript, and message content remain excluded. | gsd-security-auditor | 2026-04-27 |

## Threat Flags

| Flag | Mapping | Status |
|------|---------|--------|
| threat_flag: local-session-metadata-storage | T-04-01 | registered |
| threat_flag: local-session-jsonl-parser | T-04-05, T-04-06, T-04-07 | registered |
| threat_flag: telemetry-path-attribution | T-04-08 | registered |

Unregistered flags: none.

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-04-27 | 21 | 21 | 0 | gsd-security-auditor |

## Verification Commands

| Command | Result |
|---------|--------|
| `cargo test --manifest-path src-tauri/Cargo.toml --test session_repo -- --nocapture` | 6 passed |
| `cargo test --manifest-path src-tauri/Cargo.toml --test session_indexer -- --nocapture` | 17 passed |
| `cargo test --manifest-path src-tauri/Cargo.toml --test portfolio_commands -- --nocapture` | 8 passed |
| `npm test -- --run` | 22 passed |

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-04-27
