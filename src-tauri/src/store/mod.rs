use std::{path::Path, time::Duration};

use deadpool_sqlite::{Config, Hook, HookError, Pool, Runtime};
use rusqlite::Connection;

use crate::error::AppError;

pub mod migrations;
pub mod settings_repo;

pub async fn open_pool(db_path: &Path) -> Result<Pool, AppError> {
    let config = Config::new(db_path);
    let pool = config
        .builder(Runtime::Tokio1)
        .map_err(AppError::store)?
        .post_create(Hook::async_fn(|connection, _| {
            Box::pin(async move {
                connection
                    .interact(configure_connection)
                    .await
                    .map_err(|error| HookError::message(error.to_string()))?
                    .map_err(HookError::Backend)
            })
        }))
        .build()
        .map_err(AppError::store)?;

    Ok(pool)
}

pub async fn run_migrations(pool: &Pool) -> Result<(), AppError> {
    let connection = pool.get().await.map_err(AppError::store)?;
    connection
        .interact(migrations::run)
        .await
        .map_err(AppError::store)?
        .map_err(AppError::from)
}

pub async fn migration_version(pool: &Pool) -> Result<u32, AppError> {
    let connection = pool.get().await.map_err(AppError::store)?;
    let version = connection
        .interact(|connection| {
            connection.pragma_query_value(None, "user_version", |row| row.get::<_, u32>(0))
        })
        .await
        .map_err(AppError::store)??;

    Ok(version)
}

pub async fn wal_enabled(pool: &Pool) -> Result<bool, AppError> {
    let connection = pool.get().await.map_err(AppError::store)?;
    let journal_mode = connection
        .interact(|connection| {
            connection.pragma_query_value(None, "journal_mode", |row| row.get::<_, String>(0))
        })
        .await
        .map_err(AppError::store)??;

    Ok(journal_mode.eq_ignore_ascii_case("wal"))
}

fn configure_connection(connection: &mut Connection) -> rusqlite::Result<()> {
    connection.pragma_update(None, "journal_mode", "WAL")?;
    connection.pragma_update(None, "synchronous", "NORMAL")?;
    connection.pragma_update(None, "foreign_keys", "ON")?;
    connection.busy_timeout(Duration::from_secs(5))?;
    Ok(())
}
