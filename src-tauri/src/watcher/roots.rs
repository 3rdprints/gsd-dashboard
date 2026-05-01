use std::path::{Path, PathBuf};

use deadpool_sqlite::Pool;

use crate::{
    error::AppError,
    scan_roots::{normalize_scan_root, validate_scan_root},
    settings::AppSettings,
    store::project_repo,
};

pub async fn derive_watcher_roots(
    pool: &Pool,
    home_dir: &Path,
    settings: &AppSettings,
) -> Result<Vec<PathBuf>, AppError> {
    derive_polling_scan_roots(home_dir, settings)?;

    let connection = pool.get().await.map_err(AppError::store)?;
    let mut roots = connection
        .interact(project_repo::list_project_snapshots)
        .await
        .map_err(AppError::store)??
        .into_iter()
        .map(|project| PathBuf::from(project.planning_path))
        .collect::<Vec<_>>();

    for root in [
        home_dir.join(".claude/projects"),
        home_dir.join(".codex/sessions"),
    ] {
        if root.exists() && !roots.contains(&root) {
            roots.push(root);
        }
    }

    Ok(roots)
}

pub fn derive_polling_scan_roots(
    home_dir: &Path,
    settings: &AppSettings,
) -> Result<Vec<PathBuf>, AppError> {
    settings
        .scan_roots
        .iter()
        .map(|root| {
            let normalized = normalize_scan_root(Path::new(root), home_dir);
            validate_scan_root(&normalized, home_dir)?;
            Ok(normalized)
        })
        .collect()
}
