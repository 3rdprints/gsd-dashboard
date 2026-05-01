#[test]
#[ignore = "Phase 07 implementation gate"]
fn live_updates_targeted_refresh_reparses_only_affected_project() {
    // LIVE-02, T-07-02: project refresh must reparse only the affected `.planning`
    // source and update derived cache state without touching unrelated projects.
}

#[test]
#[ignore = "Phase 07 implementation gate"]
fn live_updates_targeted_refresh_reuses_session_byte_offsets() {
    // LIVE-02, T-07-04: session refresh must reuse per-file byte offsets and emit
    // tiny ID/progress invalidation payloads without raw transcript or tool output.
}

#[test]
#[ignore = "Phase 07 implementation gate"]
fn live_updates_targeted_refresh_does_not_write_to_planning_sources() {
    // LIVE-02, T-07-02: `.planning` files are read-only inputs; targeted refresh
    // may change only derived SQLite/cache state and app invalidation events.
}
