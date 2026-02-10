use serde::Serialize;
use storage::StorageError;

/// IPC error response following the architecture error contract.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    pub retryable: bool,
}

impl From<StorageError> for AppError {
    fn from(err: StorageError) -> Self {
        match &err {
            StorageError::NotFound(id) => AppError {
                code: "NOT_FOUND".to_string(),
                message: format!("Session not found: {}", id),
                details: None,
                retryable: false,
            },
            StorageError::InvalidInput(msg) => AppError {
                code: "INVALID_INPUT".to_string(),
                message: msg.clone(),
                details: None,
                retryable: false,
            },
            StorageError::Database(e) => AppError {
                code: "DATABASE_ERROR".to_string(),
                message: "A database error occurred".to_string(),
                details: Some(e.to_string()),
                retryable: true,
            },
            StorageError::Migration(e) => AppError {
                code: "MIGRATION_ERROR".to_string(),
                message: "Database migration failed".to_string(),
                details: Some(e.to_string()),
                retryable: false,
            },
            StorageError::Io(e) => AppError {
                code: "IO_ERROR".to_string(),
                message: "A file system error occurred".to_string(),
                details: Some(e.to_string()),
                retryable: true,
            },
            StorageError::NotDeleted(id) => AppError {
                code: "NOT_DELETED".to_string(),
                message: format!("Session is not deleted: {}", id),
                details: None,
                retryable: false,
            },
            StorageError::RestoreExpired(id) => AppError {
                code: "RESTORE_EXPIRED".to_string(),
                message: format!("Restore window expired for session: {}", id),
                details: None,
                retryable: false,
            },
            StorageError::ParquetWrite(msg) => AppError {
                code: "PARQUET_WRITE_ERROR".to_string(),
                message: msg.clone(),
                details: None,
                retryable: false,
            },
            StorageError::ParquetRead(msg) => AppError {
                code: "PARQUET_READ_ERROR".to_string(),
                message: msg.clone(),
                details: None,
                retryable: false,
            },
            StorageError::ChecksumMismatch { path } => AppError {
                code: "CHECKSUM_MISMATCH".to_string(),
                message: format!("Checksum mismatch for file: {}", path),
                details: None,
                retryable: false,
            },
            StorageError::UnsupportedFormat(msg) => AppError {
                code: "UNSUPPORTED_FORMAT".to_string(),
                message: msg.clone(),
                details: None,
                retryable: false,
            },
            StorageError::ParseError(msg) => AppError {
                code: "PARSE_ERROR".to_string(),
                message: msg.clone(),
                details: None,
                retryable: false,
            },
            StorageError::SchemaMismatch(msg) => AppError {
                code: "SCHEMA_MISMATCH".to_string(),
                message: msg.clone(),
                details: None,
                retryable: false,
            },
            StorageError::DuplicateSession(id) => AppError {
                code: "DUPLICATE_SESSION".to_string(),
                message: format!("Duplicate session found: {}", id),
                details: None,
                retryable: false,
            },
            StorageError::AlreadyDeleted(id) => AppError {
                code: "ALREADY_DELETED".to_string(),
                message: format!("Session is already deleted: {}", id),
                details: None,
                retryable: false,
            },
            StorageError::JsonSerialization(e) => AppError {
                code: "JSON_SERIALIZATION_ERROR".to_string(),
                message: "JSON serialization failed".to_string(),
                details: Some(e.to_string()),
                retryable: false,
            },
        }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}
