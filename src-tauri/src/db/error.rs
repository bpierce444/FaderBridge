//! Database error types

use thiserror::Error;

/// Database operation errors
#[derive(Error, Debug)]
pub enum DbError {
    #[error("Database connection error: {0}")]
    Connection(String),

    #[error("Database query error: {0}")]
    Query(String),

    #[error("Project not found: {0}")]
    ProjectNotFound(String),

    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Mapping not found: {0}")]
    MappingNotFound(String),

    #[error("Duplicate project name: {0}")]
    DuplicateProjectName(String),

    #[error("Duplicate mapping: MIDI CC {cc} on channel {channel}")]
    DuplicateMapping { channel: u8, cc: u8 },

    #[error("Invalid data: {0}")]
    InvalidData(String),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Export/Import error: {0}")]
    ExportImport(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

impl From<rusqlite::Error> for DbError {
    fn from(err: rusqlite::Error) -> Self {
        match err {
            rusqlite::Error::QueryReturnedNoRows => {
                DbError::Query("No rows returned".to_string())
            }
            rusqlite::Error::SqliteFailure(err, ref msg) => {
                if err.code == rusqlite::ErrorCode::ConstraintViolation {
                    if let Some(msg) = msg {
                        if msg.contains("UNIQUE constraint failed: projects.name") {
                            return DbError::DuplicateProjectName(
                                "Project name already exists".to_string(),
                            );
                        }
                        if msg.contains("UNIQUE constraint failed: mappings") {
                            return DbError::DuplicateMapping {
                                channel: 0,
                                cc: 0,
                            };
                        }
                    }
                }
                DbError::Query(format!("{:?}: {:?}", err, msg))
            }
            _ => DbError::Query(err.to_string()),
        }
    }
}

/// Result type for database operations
pub type DbResult<T> = Result<T, DbError>;
