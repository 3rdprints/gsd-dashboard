use std::{collections::VecDeque, path::PathBuf};

use deadpool_sqlite::Pool;
use tokio::task::JoinSet;

use crate::{
    error::AppError,
    sessions::{
        file_indexer::{index_session_file, IndexedFileResult},
        ProjectRoot, SessionSource,
    },
};

pub const SESSION_INDEX_WORKER_LIMIT: usize = 2;

#[derive(Debug)]
pub(crate) struct SessionFileIndexOutcome {
    pub(crate) source_path: PathBuf,
    pub(crate) result: Result<IndexedFileResult, AppError>,
}

/// Indexes multiple session files with bounded concurrency.
pub(crate) async fn index_session_files_bounded(
    pool: Pool,
    source: SessionSource,
    source_paths: Vec<PathBuf>,
    known_projects: Vec<ProjectRoot>,
) -> Vec<SessionFileIndexOutcome> {
    let mut pending = VecDeque::from(source_paths);
    let mut active = JoinSet::new();
    let mut outcomes = Vec::new();

    spawn_until_limit(&mut active, &mut pending, &pool, source, &known_projects);

    while let Some(joined) = active.join_next().await {
        match joined {
            Ok(outcome) => outcomes.push(outcome),
            Err(error) => outcomes.push(SessionFileIndexOutcome {
                source_path: PathBuf::new(),
                result: Err(AppError::io(error)),
            }),
        }
        spawn_until_limit(&mut active, &mut pending, &pool, source, &known_projects);
    }

    outcomes
}

fn spawn_until_limit(
    active: &mut JoinSet<SessionFileIndexOutcome>,
    pending: &mut VecDeque<PathBuf>,
    pool: &Pool,
    source: SessionSource,
    known_projects: &[ProjectRoot],
) {
    while active.len() < SESSION_INDEX_WORKER_LIMIT {
        let Some(source_path) = pending.pop_front() else {
            break;
        };
        let task_pool = pool.clone();
        let task_known_projects = known_projects.to_vec();
        active.spawn(async move {
            let result = index_session_file(
                &task_pool,
                source,
                source_path.clone(),
                &task_known_projects,
            )
            .await;
            SessionFileIndexOutcome {
                source_path,
                result,
            }
        });
    }
}
