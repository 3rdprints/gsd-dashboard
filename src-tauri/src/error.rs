use serde::{ser::SerializeStruct, Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{message}")]
    Store { message: String },
    #[error("{message}")]
    Settings { message: String },
    #[error("{message}")]
    Io { message: String },
    #[error("{reason}")]
    InvalidScanRoot { path: String, reason: String },
}

impl AppError {
    /// Creates a Store error variant from a displayable value.
    pub fn store(error: impl std::fmt::Display) -> Self {
        Self::Store {
            message: error.to_string(),
        }
    }

    /// Creates a Settings error variant from a displayable value.
    pub fn settings(error: impl std::fmt::Display) -> Self {
        Self::Settings {
            message: error.to_string(),
        }
    }

    /// Creates an Io error variant from a displayable value.
    pub fn io(error: impl std::fmt::Display) -> Self {
        Self::Io {
            message: error.to_string(),
        }
    }
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Store { message } => serialize_message_error(serializer, "store", message),
            Self::Settings { message } => serialize_message_error(serializer, "settings", message),
            Self::Io { message } => serialize_message_error(serializer, "io", message),
            Self::InvalidScanRoot { path, reason } => {
                let mut state = serializer.serialize_struct("AppError", 4)?;
                state.serialize_field("kind", "invalidScanRoot")?;
                state.serialize_field("message", reason)?;
                state.serialize_field("path", path)?;
                state.serialize_field("reason", reason)?;
                state.end()
            }
        }
    }
}

fn serialize_message_error<S>(
    serializer: S,
    kind: &'static str,
    message: &str,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut state = serializer.serialize_struct("AppError", 2)?;
    state.serialize_field("kind", kind)?;
    state.serialize_field("message", message)?;
    state.end()
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

impl From<serde_json::Error> for AppError {
    fn from(error: serde_json::Error) -> Self {
        Self::store(error)
    }
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        Self::io(error)
    }
}

impl From<tauri::Error> for AppError {
    fn from(error: tauri::Error) -> Self {
        Self::io(error)
    }
}
