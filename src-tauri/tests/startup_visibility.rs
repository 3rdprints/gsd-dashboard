use gsd_dashboard::bootstrap::{startup_visibility_action, StartupVisibilityAction};

#[test]
fn normal_launch_shows_dashboard_after_tray_setup_succeeds() {
    assert_eq!(
        startup_visibility_action(false, true),
        StartupVisibilityAction::ShowDashboard
    );
}

#[test]
fn autostart_launch_keeps_window_hidden_when_tray_setup_succeeds() {
    assert_eq!(
        startup_visibility_action(true, true),
        StartupVisibilityAction::KeepHidden
    );
}

#[test]
fn autostart_launch_shows_dashboard_when_tray_setup_fails() {
    assert_eq!(
        startup_visibility_action(true, false),
        StartupVisibilityAction::ShowDashboard
    );
}
