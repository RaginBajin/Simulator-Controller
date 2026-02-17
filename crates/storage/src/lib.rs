pub mod error;
pub mod import;
pub mod parquet;
pub mod sqlite;
pub mod types;
pub mod validation;

// Re-export public API
pub use error::StorageError;
pub use sqlite::Database;
pub use types::*;

// Re-export parquet API
pub use parquet::{
    cleanup_orphaned_temps, compute_checksum, read_telemetry, read_telemetry_window,
    telemetry_schema, validate_checksum, write_telemetry, TelemetryWriteResult, SCHEMA_VERSION,
    TELEMETRY_CHANNELS,
};

// Re-export import types
pub use import::{
    parse_pitwall_export, BatchImportResult, ImportProgress, ImportResult, ImportedSession,
    ImportedSessionMetadata,
};

// Re-export validation API
pub use validation::{IntegrityReport, STATUS_NOT_VALIDATED, STATUS_VALID};
