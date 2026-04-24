use serde::Serialize;

#[derive(Debug, Serialize, thiserror::Error)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum AppError {
    #[error("{message}")]
    Store { message: String },
    #[error("{reason}")]
    InvalidScanRoot { path: String, reason: String },
}

impl AppError {
    pub fn store(error: impl std::fmt::Display) -> Self {
        Self::Store {
            message: error.to_string(),
        }
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(error: rusqlite::Error) -> Self {
        Self::store(error)
    }
}

impl From<rusqlite_migration::Error> for AppError {
    fn from(error: rusqlite_migration::Error) -> Self {
        Self::store(error)
    }
}
