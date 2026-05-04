use std::{env, path::PathBuf};

const FRONTEND_ASSET_ERROR: &str =
    "frontend assets missing: run npm run build before cargo package/install";

fn main() {
    assert_frontend_assets_for_package_builds();

    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "get_portfolio",
            "get_portfolio_heatmap",
            "get_global_chart_data",
            "get_project",
            "get_project_chart_data",
            "get_project_milestones",
            "get_project_phase_panel",
            "get_boot_status",
            "get_settings",
            "clear_session_index",
            "index_sessions",
            "list_global_sessions",
            "list_project_sessions",
            "rebuild_cache",
            "scan_projects",
            "save_settings",
        ]),
    ))
    .expect("failed to build Tauri application metadata");
}

fn assert_frontend_assets_for_package_builds() {
    let profile = env::var("PROFILE").unwrap_or_default();
    let require_assets = env::var("GSD_DASHBOARD_REQUIRE_FRONTEND_ASSETS").is_ok();
    if profile != "release" && !require_assets {
        return;
    }

    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set"));
    let index_paths = [
        manifest_dir.join("dist/index.html"),
        manifest_dir.join("../dist/index.html"),
    ];
    if !index_paths.iter().any(|index_path| index_path.is_file()) {
        panic!("{FRONTEND_ASSET_ERROR}");
    }
}
