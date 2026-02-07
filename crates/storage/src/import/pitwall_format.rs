use std::path::Path;

use serde::Deserialize;
use tracing::info;

use crate::error::StorageError;
use crate::parquet::{read_telemetry, SCHEMA_VERSION};
use crate::types::NewLap;

use super::types::{ImportedSession, ImportedSessionMetadata};

/// Pitwall export JSON metadata structure.
#[derive(Debug, Deserialize)]
struct PitwallExportMetadata {
    #[serde(default)]
    #[allow(dead_code)]
    pitwall_version: Option<String>,
    schema_version: String,
    session: PitwallSessionMeta,
    #[serde(default)]
    laps: Vec<PitwallLapMeta>,
}

#[derive(Debug, Deserialize)]
struct PitwallSessionMeta {
    track_name: String,
    car_name: String,
    session_type: String,
    started_at: String,
    #[serde(default)]
    ended_at: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    lap_count: Option<i32>,
    #[serde(default)]
    #[allow(dead_code)]
    best_lap_time_ms: Option<i64>,
    #[serde(default)]
    #[allow(dead_code)]
    status: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PitwallLapMeta {
    lap_number: i32,
    lap_time_ms: i64,
    #[serde(default = "default_is_valid")]
    is_valid: bool,
    #[serde(default = "default_completion_status")]
    completion_status: String,
}

fn default_is_valid() -> bool {
    true
}

fn default_completion_status() -> String {
    "complete".to_string()
}

/// Parse a Pitwall export directory containing `{session_id}.parquet` and `{session_id}.json`.
///
/// `parquet_path` is the path to the `.parquet` file. The corresponding `.json` is found
/// by replacing the extension.
pub fn parse_pitwall_export(parquet_path: &Path) -> Result<ImportedSession, StorageError> {
    // Derive JSON path from parquet path
    let json_path = parquet_path.with_extension("json");

    if !json_path.exists() {
        return Err(StorageError::ParseError(format!(
            "Pitwall export metadata file not found: {}",
            json_path.display()
        )));
    }

    if !parquet_path.exists() {
        return Err(StorageError::ParseError(format!(
            "Pitwall export parquet file not found: {}",
            parquet_path.display()
        )));
    }

    info!(
        json_path = %json_path.display(),
        parquet_path = %parquet_path.display(),
        "Parsing Pitwall export"
    );

    // Parse JSON metadata
    let json_content = std::fs::read_to_string(&json_path).map_err(|e| {
        StorageError::ParseError(format!(
            "Failed to read metadata file {}: {}",
            json_path.display(),
            e
        ))
    })?;

    let export_meta: PitwallExportMetadata =
        serde_json::from_str(&json_content).map_err(|e| {
            StorageError::ParseError(format!(
                "Failed to parse metadata JSON {}: {}",
                json_path.display(),
                e
            ))
        })?;

    // Validate schema version
    if export_meta.schema_version != SCHEMA_VERSION {
        return Err(StorageError::SchemaMismatch(
            "This export was created with a newer version of Pitwall".to_string(),
        ));
    }

    // Read Parquet telemetry (no checksum validation for imports)
    let telemetry = read_telemetry(parquet_path, None)?;

    // Build laps from JSON metadata (session_id will be filled in by orchestrator)
    let laps: Vec<NewLap> = export_meta
        .laps
        .iter()
        .map(|lap| NewLap {
            session_id: String::new(), // Placeholder; set by orchestrator
            lap_number: lap.lap_number,
            lap_time_ms: lap.lap_time_ms,
            is_valid: lap.is_valid,
            completion_status: lap.completion_status.clone(),
        })
        .collect();

    let metadata = ImportedSessionMetadata {
        track_name: export_meta.session.track_name,
        car_name: export_meta.session.car_name,
        session_type: export_meta.session.session_type,
        started_at: export_meta.session.started_at,
        ended_at: export_meta.session.ended_at,
        import_source: parquet_path.display().to_string(),
        import_format: "pitwall_export".to_string(),
    };

    Ok(ImportedSession {
        metadata,
        laps,
        telemetry,
    })
}
