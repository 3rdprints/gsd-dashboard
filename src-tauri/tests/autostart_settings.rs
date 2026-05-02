use gsd_dashboard::autostart::is_autostart_launch;

#[test]
fn autostart_launch_arg_matches_exact_flag_only() {
    assert!(is_autostart_launch([
        "/app/GSD Dashboard",
        "--autostart",
    ]));
    assert!(!is_autostart_launch([
        "/app/GSD Dashboard",
        "--autostart=1",
    ]));
    assert!(!is_autostart_launch([
        "/app/GSD Dashboard",
        "--startup",
    ]));
}
