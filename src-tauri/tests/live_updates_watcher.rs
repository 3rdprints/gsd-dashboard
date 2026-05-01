#[test]
#[ignore = "Phase 07 implementation gate"]
fn live_updates_watcher_registers_only_discovered_planning_and_existing_session_roots() {
    // LIVE-01, T-07-01: watcher roots must be limited to discovered `.planning`
    // directories and existing supported session roots, excluding `/`, bare `$HOME`,
    // broad scan-root recursive watching, Phase 10 Codex roots, and archived roots.
}

#[test]
#[ignore = "Phase 07 implementation gate"]
fn live_updates_watcher_debounces_project_changes_at_500ms() {
    // LIVE-01, T-07-01: project `.planning` changes must use injected watcher/time
    // seams so debounce assertions do not depend on real OS watcher timing.
}

#[test]
#[ignore = "Phase 07 implementation gate"]
fn live_updates_watcher_enters_60s_polling_fallback_for_failed_root() {
    // LIVE-03, T-07-03: failed roots must expose explicit polling fallback status
    // with root, reason category, fix hint, 60s cadence, and retry state.
}
