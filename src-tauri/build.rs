fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "get_portfolio",
            "get_project",
            "get_project_chart_data",
            "get_project_milestones",
            "get_project_phase_panel",
            "get_boot_status",
            "get_settings",
            "index_sessions",
            "list_project_sessions",
            "rebuild_cache",
            "scan_projects",
            "save_settings",
        ]),
    ))
    .expect("failed to build Tauri application metadata");
}
