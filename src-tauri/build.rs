fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "get_portfolio",
            "get_project",
            "get_boot_status",
            "get_settings",
            "index_sessions",
            "rebuild_cache",
            "scan_projects",
            "save_settings",
        ]),
    ))
    .expect("failed to build Tauri application metadata");
}
