use std::{
    path::{Component, Path, PathBuf},
    str::FromStr,
};

use crate::error::AppError;

const BROAD_ROOT_REASON: &str = "This scan root is too broad. Choose a specific folder inside your home directory, such as ~/Documents or a project workspace.";

/// Validates that a scan root is a proper subdirectory of the user's home directory.
pub fn validate_scan_root(candidate: &Path, home: &Path) -> Result<(), AppError> {
    let normalized_candidate = normalize_scan_root(candidate, home);
    let normalized_home = normalize_path(home);

    if normalized_candidate == Path::new("/")
        || normalized_candidate == normalized_home
        || !normalized_candidate.starts_with(&normalized_home)
    {
        return Err(AppError::InvalidScanRoot {
            path: normalized_candidate.display().to_string(),
            reason: BROAD_ROOT_REASON.to_string(),
        });
    }

    Ok(())
}

/// Expands tilde and normalizes path components for a scan root.
pub(crate) fn normalize_scan_root(candidate: &Path, home: &Path) -> PathBuf {
    let raw = candidate.to_string_lossy();

    if raw == "~" {
        return normalize_path(home);
    }

    if let Some(stripped) = raw.strip_prefix("~/") {
        return normalize_path(&home.join(stripped));
    }

    normalize_path(candidate)
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(
                PathBuf::from_str(other.as_os_str().to_string_lossy().as_ref()).unwrap_or_default(),
            ),
        }
    }

    normalized
}
