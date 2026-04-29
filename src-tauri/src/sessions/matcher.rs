use std::path::{Component, Path, PathBuf};

use crate::sessions::{IndexedSession, ProjectRoot, SessionSource};

/// Performs blocking filesystem checks. Async callers must run this inside
/// `tokio::task::spawn_blocking`; the session indexer does so before calling it.
pub fn match_project(session: &mut IndexedSession, known_projects: &[ProjectRoot]) {
    if let Some(cwd) = session.cwd.as_deref() {
        if let Some(project) = match_known_root(cwd, known_projects) {
            session.project_id = Some(project.id.clone());
            session.attribution_method = "cwd".to_string();
            return;
        }

        if let Some(project) = match_git_worktree_root(cwd, known_projects) {
            session.project_id = Some(project.id.clone());
            session.attribution_method = "worktree_cwd".to_string();
            return;
        }
    }

    if session.source == SessionSource::Claude {
        if let Some(project) = match_encoded_claude_path(&session.source_path, known_projects) {
            session.project_id = Some(project.id.clone());
            session.attribution_method = "claude_path".to_string();
            return;
        }
    }

    session.project_id = None;
    session.attribution_method = "unmatched".to_string();
}

fn match_known_root<'a>(
    candidate: &str,
    known_projects: &'a [ProjectRoot],
) -> Option<&'a ProjectRoot> {
    match_known_root_path(Path::new(candidate), known_projects)
}

fn match_known_root_path<'a>(
    candidate_path: &Path,
    known_projects: &'a [ProjectRoot],
) -> Option<&'a ProjectRoot> {
    // canonicalize performs filesystem I/O; keep this function inside match_project's blocking contract.
    let canonical_candidate = candidate_path.canonicalize().ok();
    let canonical_projects = known_projects
        .iter()
        .map(|project| {
            let root_path = Path::new(&project.root_path);
            (project, root_path, root_path.canonicalize().ok())
        })
        .collect::<Vec<_>>();

    canonical_projects
        .iter()
        .filter(|(_project, root_path, canonical_root)| {
            path_is_inside(candidate_path, root_path)
                || canonical_candidate
                    .as_deref()
                    .zip(canonical_root.as_deref())
                    .is_some_and(|(candidate, root)| path_is_inside(candidate, root))
        })
        .max_by_key(|(project, _root_path, _canonical_root)| project.root_path.len())
        .map(|(project, _root_path, _canonical_root)| *project)
}

fn path_is_inside(candidate_path: &Path, root_path: &Path) -> bool {
    candidate_path == root_path || candidate_path.starts_with(root_path)
}

fn match_git_worktree_root<'a>(
    candidate: &str,
    known_projects: &'a [ProjectRoot],
) -> Option<&'a ProjectRoot> {
    let base_root = git_worktree_base_root(Path::new(candidate))?;
    match_known_root_path(&base_root, known_projects)
}

fn git_worktree_base_root(candidate_path: &Path) -> Option<PathBuf> {
    let mut current_path = candidate_path;

    loop {
        let git_file = current_path.join(".git");
        // is_file performs filesystem I/O; callers reach this through match_project's blocking contract.
        if git_file.is_file() {
            return gitdir_base_root(&git_file, current_path);
        }

        current_path = current_path.parent()?;
    }
}

fn gitdir_base_root(git_file: &Path, worktree_root: &Path) -> Option<PathBuf> {
    // read_to_string performs filesystem I/O; callers reach this through match_project's blocking contract.
    let contents = std::fs::read_to_string(git_file).ok()?;
    let gitdir = contents.trim().strip_prefix("gitdir:")?.trim();
    let gitdir_path = Path::new(gitdir);
    let resolved_gitdir = if gitdir_path.is_absolute() {
        gitdir_path.to_path_buf()
    } else {
        worktree_root.join(gitdir_path)
    };

    base_root_from_worktree_gitdir(&resolved_gitdir)
}

fn base_root_from_worktree_gitdir(gitdir: &Path) -> Option<PathBuf> {
    let components = gitdir.components().collect::<Vec<_>>();
    let git_index = components.windows(2).position(|window| {
        matches!(window[0], Component::Normal(name) if name == ".git")
            && matches!(window[1], Component::Normal(name) if name == "worktrees")
    })?;
    let mut base_root = PathBuf::new();

    for component in &components[..git_index] {
        base_root.push(component.as_os_str());
    }

    if base_root.as_os_str().is_empty() {
        None
    } else {
        Some(base_root)
    }
}

fn match_encoded_claude_path<'a>(
    source_path: &str,
    known_projects: &'a [ProjectRoot],
) -> Option<&'a ProjectRoot> {
    let encoded_project_dir = encoded_claude_project_dir(source_path)?;

    known_projects
        .iter()
        .filter(|project| encoded_roots_overlap(encoded_project_dir, &project.root_path))
        .max_by_key(|project| project.root_path.len())
}

fn encoded_claude_project_dir(source_path: &str) -> Option<&str> {
    let parts = source_path.split('/').collect::<Vec<_>>();
    let projects_index = parts
        .windows(2)
        .position(|window| window == [".claude", "projects"])?;
    parts
        .get(projects_index + 2)
        .copied()
        .filter(|part| !part.is_empty())
}

fn encode_known_root_for_claude(root_path: &str) -> String {
    root_path.replace('/', "-")
}

fn encoded_roots_overlap(encoded_project_dir: &str, known_root: &str) -> bool {
    let encoded_known_root = encode_known_root_for_claude(known_root);
    encoded_project_dir == encoded_known_root
        || encoded_project_dir
            .strip_prefix(&encoded_known_root)
            .is_some_and(|rest| rest.starts_with('-'))
        || encoded_known_root
            .strip_prefix(encoded_project_dir)
            .is_some_and(|rest| rest.starts_with('-'))
}

#[cfg(test)]
mod tests {
    use super::base_root_from_worktree_gitdir;
    use std::path::{Path, PathBuf};

    #[test]
    fn base_root_from_worktree_gitdir_extracts_absolute_root() {
        assert_eq!(
            base_root_from_worktree_gitdir(Path::new("/repo/.git/worktrees/name")),
            Some(PathBuf::from("/repo"))
        );
    }

    #[test]
    fn base_root_from_worktree_gitdir_extracts_relative_root() {
        assert_eq!(
            base_root_from_worktree_gitdir(Path::new("repo/.git/worktrees/name")),
            Some(PathBuf::from("repo"))
        );
    }

    #[test]
    fn base_root_from_worktree_gitdir_rejects_non_worktree_paths() {
        assert_eq!(
            base_root_from_worktree_gitdir(Path::new("/repo/.git")),
            None
        );
        assert_eq!(
            base_root_from_worktree_gitdir(Path::new("/repo/some/other/.git/worktree")),
            None
        );
        assert_eq!(
            base_root_from_worktree_gitdir(Path::new("/repo/.git/other")),
            None
        );
        assert_eq!(
            base_root_from_worktree_gitdir(Path::new(".git/worktrees/name")),
            None
        );
    }
}
