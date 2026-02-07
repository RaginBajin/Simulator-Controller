use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("Session not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Session is not deleted: {0}")]
    NotDeleted(String),

    #[error("Restore window expired for session: {0}")]
    RestoreExpired(String),

    #[error("Parquet write error: {0}")]
    ParquetWrite(String),

    #[error("Parquet read error: {0}")]
    ParquetRead(String),

    #[error("Checksum mismatch for file: {path}")]
    ChecksumMismatch { path: String },

    #[error("Unsupported import format: {0}")]
    UnsupportedFormat(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Schema mismatch: {0}")]
    SchemaMismatch(String),

    #[error("Duplicate session found: {0}")]
    DuplicateSession(String),

    #[error("Session is already deleted: {0}")]
    AlreadyDeleted(String),
}
