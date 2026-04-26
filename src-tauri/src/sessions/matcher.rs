use std::path::Path;

use crate::sessions::{IndexedSession, ProjectRoot, SessionSource};

pub fn match_project(session: &mut IndexedSession, known_projects: &[ProjectRoot]) {
    if let Some(cwd) = session.cwd.as_deref() {
        if let Some(project) = match_known_root(cwd, known_projects) {
            session.project_id = Some(project.id.clone());
            session.attribution_method = "cwd".to_string();
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
    let candidate_path = Path::new(candidate);
    known_projects
        .iter()
        .filter(|project| {
            let root_path = Path::new(&project.root_path);
            candidate_path == root_path || candidate_path.starts_with(root_path)
        })
        .max_by_key(|project| project.root_path.len())
}

fn match_encoded_claude_path<'a>(
    source_path: &str,
    known_projects: &'a [ProjectRoot],
) -> Option<&'a ProjectRoot> {
    let encoded_project_dir = encoded_claude_project_dir(source_path)?;
    let decoded_candidate = decode_claude_project_dir(encoded_project_dir);

    known_projects.iter().find(|project| {
        encode_known_root_for_claude(&project.root_path) == encoded_project_dir
            || decoded_candidate
                .as_deref()
                .is_some_and(|candidate| roots_overlap(candidate, &project.root_path))
    })
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

fn decode_claude_project_dir(encoded: &str) -> Option<String> {
    encoded
        .strip_prefix('-')
        .map(|absolute| format!("/{}", absolute.replace('-', "/")))
}

fn encode_known_root_for_claude(root_path: &str) -> String {
    root_path.replace('/', "-")
}

fn roots_overlap(candidate: &str, known_root: &str) -> bool {
    let candidate = Path::new(candidate);
    let known_root = Path::new(known_root);
    candidate == known_root
        || candidate.starts_with(known_root)
        || known_root.starts_with(candidate)
}
