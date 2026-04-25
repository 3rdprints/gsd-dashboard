use std::{fs, path::Path};

use gsd_dashboard::{error::AppError, scanner::discover_planning_dirs};

fn create_planning_dir(project_root: &Path) {
    fs::create_dir_all(project_root.join(".planning")).expect("planning dir should be created");
}

#[test]
fn scanner_discovers_planning_dirs() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let home_dir = temp_dir.path();
    let scan_root = home_dir.join("workspace");
    let project_root = scan_root.join("project-a");

    create_planning_dir(&project_root);

    let candidates =
        discover_planning_dirs(&scan_root, home_dir).expect("scan root should be discoverable");

    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].project_root, project_root);
    assert_eq!(candidates[0].planning_path, project_root.join(".planning"));
}

#[test]
fn scanner_rejects_bare_home_root() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let home_dir = temp_dir.path();

    let error = discover_planning_dirs(home_dir, home_dir)
        .expect_err("bare home root should be rejected");

    assert!(matches!(error, AppError::InvalidScanRoot { .. }));
}

#[test]
fn scanner_deduplicates_symlinked_planning_dirs() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let home_dir = temp_dir.path();
    let scan_root = home_dir.join("workspace");
    let project_root = scan_root.join("project-a");

    create_planning_dir(&project_root);

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&project_root, scan_root.join("project-link"))
            .expect("project symlink should be created");
    }

    let candidates =
        discover_planning_dirs(&scan_root, home_dir).expect("scan root should be discoverable");

    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].planning_path, project_root.join(".planning"));
}
