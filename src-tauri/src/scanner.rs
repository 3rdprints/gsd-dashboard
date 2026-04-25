use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use ignore::WalkBuilder;
use serde::Serialize;

use crate::{error::AppError, scan_roots::validate_scan_root};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanningProjectCandidate {
    pub project_root: PathBuf,
    pub planning_path: PathBuf,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanSummary {
    pub discovered_count: usize,
    pub parsed_count: usize,
    pub error_count: usize,
}

pub fn discover_planning_dirs(
    root: &Path,
    home_dir: &Path,
) -> Result<Vec<PlanningProjectCandidate>, AppError> {
    validate_scan_root(root, home_dir)?;

    let mut seen_paths = HashSet::new();
    let mut candidates = Vec::new();

    for entry in WalkBuilder::new(root)
        .hidden(false)
        .git_ignore(true)
        .git_exclude(true)
        .parents(true)
        .follow_links(false)
        .build()
    {
        let entry = entry.map_err(AppError::io)?;
        if !entry
            .file_type()
            .is_some_and(|file_type| file_type.is_dir())
        {
            continue;
        }

        if entry.file_name() != ".planning" {
            continue;
        }

        let planning_path = entry.path().to_path_buf();
        let Some(project_root) = planning_path.parent().map(Path::to_path_buf) else {
            continue;
        };
        let dedupe_key = dedupe_path(&planning_path);

        if seen_paths.insert(dedupe_key) {
            candidates.push(PlanningProjectCandidate {
                project_root,
                planning_path,
            });
        }
    }

    Ok(candidates)
}

fn dedupe_path(path: &Path) -> String {
    path.canonicalize()
        .unwrap_or_else(|_| normalize_display_path(path))
        .display()
        .to_string()
}

fn normalize_display_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        normalized.push(component);
    }

    normalized
}
