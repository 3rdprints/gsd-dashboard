use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use deadpool_sqlite::Pool;
use serde::Serialize;

use crate::{
    error::AppError,
    events::{AppEvent, SessionIndexEvent},
    sessions::{
        file_indexer::load_known_project_roots,
        parallel::index_session_files_bounded,
        repo::{
            prune_indexed_paths_under, prune_orphan_index_states,
            prune_tokenless_codex_index_states, prune_unmatched_sessions,
        },
        SessionSource,
    },
    store::daily_activity,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SessionIndexSummary {
    pub root_count: usize,
    pub files_processed: usize,
    pub sessions_persisted: usize,
    pub unmatched_count: usize,
    pub error_count: usize,
}

#[derive(Debug, Clone)]
struct SessionRoot {
    source: SessionSource,
    path: PathBuf,
}

/// Discovers and indexes all session JSONL files from known roots.
pub async fn index_session_roots(
    pool: Pool,
    home_dir: PathBuf,
    on_event: impl Fn(SessionIndexEvent) -> Result<(), AppError> + Send + Sync + 'static,
) -> Result<SessionIndexSummary, AppError> {
    let roots = discover_existing_roots(home_dir).await?;
    on_event(SessionIndexEvent::Started {
        root_count: roots.len(),
    })?;

    let known_projects = load_known_project_roots(&pool).await?;
    prune_existing_unmatched_sessions(&pool).await?;
    prune_existing_orphan_index_states(&pool).await?;
    prune_existing_tokenless_codex_index_states(&pool).await?;
    let mut summary = SessionIndexSummary {
        root_count: roots.len(),
        files_processed: 0,
        sessions_persisted: 0,
        unmatched_count: 0,
        error_count: 0,
    };

    for root in roots {
        if root.source == SessionSource::Codex {
            prune_ignored_codex_index_paths(&pool, root.path.join("index")).await?;
        }

        on_event(SessionIndexEvent::SourceStarted {
            source: root.source.as_str().to_string(),
            root_path: root.path.display().to_string(),
        })?;

        let files = discover_jsonl_files(root.path.clone()).await?;
        let outcomes =
            index_session_files_bounded(pool.clone(), root.source, files, known_projects.clone())
                .await;
        for outcome in outcomes {
            summary.files_processed += 1;
            match outcome.result {
                Ok(result) => {
                    summary.sessions_persisted += result.sessions_persisted;
                    on_event(SessionIndexEvent::FileIndexed {
                        source: root.source.as_str().to_string(),
                        source_path: outcome.source_path.display().to_string(),
                        sessions_persisted: result.sessions_persisted,
                        live_partial: result.live_partial,
                    })?;
                }
                Err(error) => {
                    summary.error_count += 1;
                    on_event(SessionIndexEvent::FileIndexError {
                        source: root.source.as_str().to_string(),
                        source_path: outcome.source_path.display().to_string(),
                        message: error.to_string(),
                    })?;
                }
            }
        }
    }

    summary.unmatched_count = load_unmatched_count(&pool).await?;
    if let Err(error) = rebuild_daily_activity(&pool).await {
        eprintln!("daily_activity rebuild failed after session index: {error}");
    } else {
        on_event(SessionIndexEvent::App(AppEvent::DailyActivityUpdated))?;
    }
    on_event(SessionIndexEvent::Finished {
        files_processed: summary.files_processed,
        sessions_persisted: summary.sessions_persisted,
        unmatched_count: summary.unmatched_count,
        error_count: summary.error_count,
    })?;

    Ok(summary)
}

async fn prune_ignored_codex_index_paths(
    pool: &Pool,
    path_prefix: PathBuf,
) -> Result<(), AppError> {
    let path_prefix = path_prefix.display().to_string();
    let connection = pool.get().await.map_err(AppError::store)?;
    connection
        .interact(move |connection| prune_indexed_paths_under(connection, &path_prefix))
        .await
        .map_err(AppError::store)??;

    Ok(())
}

async fn rebuild_daily_activity(pool: &Pool) -> Result<(), AppError> {
    let now_ms = current_epoch_ms();
    let connection = pool.get().await.map_err(AppError::store)?;
    connection
        .interact(move |connection| daily_activity::rebuild_window(connection, 90, now_ms))
        .await
        .map_err(AppError::store)?
}

fn current_epoch_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().try_into().unwrap_or(0))
        .unwrap_or(0)
}

async fn discover_existing_roots(home_dir: PathBuf) -> Result<Vec<SessionRoot>, AppError> {
    tokio::task::spawn_blocking(move || {
        let candidates = [
            SessionRoot {
                source: SessionSource::Claude,
                path: home_dir.join(".claude/projects"),
            },
            SessionRoot {
                source: SessionSource::Codex,
                path: home_dir.join(".codex/sessions"),
            },
        ];

        candidates
            .into_iter()
            .filter(|root| root.path.is_dir())
            .collect::<Vec<_>>()
    })
    .await
    .map_err(AppError::io)
}

async fn discover_jsonl_files(root: PathBuf) -> Result<Vec<PathBuf>, AppError> {
    tokio::task::spawn_blocking(move || {
        let mut files = Vec::new();
        collect_jsonl_files(&root, &mut files)?;
        files.sort();

        Ok::<_, AppError>(files)
    })
    .await
    .map_err(AppError::io)?
}

fn collect_jsonl_files(path: &Path, files: &mut Vec<PathBuf>) -> Result<(), AppError> {
    for entry in std::fs::read_dir(path).map_err(AppError::io)? {
        let entry = entry.map_err(AppError::io)?;
        let entry_path = entry.path();
        let file_type = entry.file_type().map_err(AppError::io)?;

        if file_type.is_dir() {
            if entry_path.file_name().and_then(|name| name.to_str()) == Some("index") {
                continue;
            }
            collect_jsonl_files(&entry_path, files)?;
        } else if file_type.is_file()
            && entry_path
                .extension()
                .and_then(|extension| extension.to_str())
                == Some("jsonl")
        {
            files.push(entry_path);
        }
    }

    Ok(())
}

async fn load_unmatched_count(pool: &Pool) -> Result<usize, AppError> {
    let connection = pool.get().await.map_err(AppError::store)?;
    connection
        .interact(|connection| {
            connection
                .query_row(
                    "SELECT COUNT(*) FROM sessions WHERE project_id IS NULL",
                    [],
                    |row| row.get::<_, i64>(0),
                )
                .map(|count| count as usize)
                .map_err(AppError::from)
        })
        .await
        .map_err(AppError::store)?
}

async fn prune_existing_unmatched_sessions(pool: &Pool) -> Result<(), AppError> {
    let connection = pool.get().await.map_err(AppError::store)?;
    connection
        .interact(prune_unmatched_sessions)
        .await
        .map_err(AppError::store)??;

    Ok(())
}

async fn prune_existing_orphan_index_states(pool: &Pool) -> Result<(), AppError> {
    let connection = pool.get().await.map_err(AppError::store)?;
    connection
        .interact(prune_orphan_index_states)
        .await
        .map_err(AppError::store)??;

    Ok(())
}

async fn prune_existing_tokenless_codex_index_states(pool: &Pool) -> Result<(), AppError> {
    let connection = pool.get().await.map_err(AppError::store)?;
    connection
        .interact(prune_tokenless_codex_index_states)
        .await
        .map_err(AppError::store)??;

    Ok(())
}
