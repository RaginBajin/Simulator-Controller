use arrow::record_batch::RecordBatch;
use serde::{Deserialize, Serialize};

use crate::types::NewLap;

/// Metadata extracted from an imported file, used to create a session record.
#[derive(Debug, Clone)]
pub struct ImportedSessionMetadata {
    pub track_name: String,
    pub car_name: String,
    pub session_type: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub import_source: String,
    pub import_format: String,
}

/// A fully parsed session ready for storage. Produced by format-specific parsers.
pub struct ImportedSession {
    pub metadata: ImportedSessionMetadata,
    pub laps: Vec<NewLap>,
    pub telemetry: RecordBatch,
}

/// Result of a single import operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum ImportResult {
    Success {
        session_id: String,
        lap_count: i32,
    },
    Duplicate {
        existing_session_id: String,
    },
    Failed {
        error: String,
        file_path: String,
    },
}

/// Aggregate result of a batch import operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchImportResult {
    pub total: usize,
    pub succeeded: usize,
    pub duplicates_skipped: usize,
    pub failed: usize,
    pub results: Vec<ImportResult>,
}

/// Progress event payload emitted during batch import.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportProgress {
    pub current: usize,
    pub total: usize,
    pub file_name: String,
    pub status: String,
}
