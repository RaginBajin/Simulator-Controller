//! Import orchestrator: coordinates parsing, duplicate detection, and storage for session imports.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use tracing::info;
use uuid::Uuid;

use storage::error::StorageError;
use storage::import::types::{BatchImportResult, ImportProgress, ImportResult};
use storage::Database;

use crate::ibt_importer;

/// Default duplicate detection tolerance in seconds.
const DUPLICATE_TOLERANCE_SECS: i64 = 60;

/// Import a single session from a file path.
///
/// Detects format from extension, parses the file, checks for duplicates,
/// and writes to storage. Returns the import result.
pub async fn import_session(
    db: &Database,
    data_dir: &Path,
    file_path: &Path,
    force_duplicate: bool,
) -> Result<ImportResult, StorageError> {
    // Detect format from extension
    let extension = file_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let imported = match extension.as_str() {
        "ibt" => ibt_importer::parse_ibt_file(file_path)?,
        "parquet" => storage::parse_pitwall_export(file_path)?,
        other => {
            return Err(StorageError::UnsupportedFormat(format!(
                "Unsupported file extension: .{}",
                other
            )));
        }
    };

    // Check for duplicate session
    if !force_duplicate {
        if let Some(existing_id) = db
            .find_duplicate_session(
                &imported.metadata.track_name,
                &imported.metadata.car_name,
                &imported.metadata.started_at,
                DUPLICATE_TOLERANCE_SECS,
            )
            .await?
        {
            return Ok(ImportResult::Duplicate {
                existing_session_id: existing_id,
            });
        }
    }

    // Generate new session ID
    let session_id = Uuid::new_v4().to_string();

    // Compute lap stats
    let lap_count = imported.laps.len() as i32;
    let best_lap_time_ms = imported
        .laps
        .iter()
        .filter(|l| l.is_valid)
        .map(|l| l.lap_time_ms)
        .min();

    // Insert session record first
    let _session = db
        .insert_imported_session(
            &session_id,
            &imported.metadata.track_name,
            &imported.metadata.car_name,
            &imported.metadata.session_type,
            &imported.metadata.started_at,
            imported.metadata.ended_at.as_deref(),
            lap_count,
            best_lap_time_ms,
            &imported.metadata.import_source,
            &imported.metadata.import_format,
        )
        .await?;

    // Write telemetry Parquet file (now session exists)
    let mut file_metadata = HashMap::new();
    file_metadata.insert("session_id".to_string(), session_id.clone());
    file_metadata.insert(
        "schema_version".to_string(),
        storage::SCHEMA_VERSION.to_string(),
    );

    if let Err(e) = db
        .write_telemetry(&session_id, data_dir, &imported.telemetry, file_metadata)
        .await
    {
        // Rollback: delete the session record
        let _ = db.permanently_delete_session(&session_id).await;
        return Err(e);
    }

    // Insert lap summaries
    for mut lap in imported.laps {
        lap.session_id = session_id.clone();
        if let Err(e) = db.insert_lap(&lap).await {
            info!(
                session_id = %session_id,
                lap_number = lap.lap_number,
                error = %e,
                "Failed to insert lap, continuing"
            );
        }
    }

    // Update lap count and best lap on session
    let update = storage::SessionUpdate {
        lap_count: Some(lap_count),
        best_lap_time_ms,
        status: Some("completed".to_string()),
        ..Default::default()
    };
    let _ = db.update_session(&session_id, update).await;

    info!(
        session_id = %session_id,
        lap_count = lap_count,
        source = %imported.metadata.import_source,
        "Session imported successfully"
    );

    Ok(ImportResult::Success {
        session_id,
        lap_count,
    })
}

/// Import multiple session files in batch, emitting progress via a callback.
///
/// Each file is processed sequentially. Progress is reported via the `on_progress` callback.
pub async fn import_session_batch(
    db: &Database,
    data_dir: &Path,
    file_paths: &[PathBuf],
    force_duplicate: bool,
    mut on_progress: impl FnMut(ImportProgress),
) -> Result<BatchImportResult, StorageError> {
    let total = file_paths.len();
    let mut succeeded = 0usize;
    let mut duplicates_skipped = 0usize;
    let mut failed = 0usize;
    let mut results = Vec::with_capacity(total);

    for (i, file_path) in file_paths.iter().enumerate() {
        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        on_progress(ImportProgress {
            current: i + 1,
            total,
            file_name: file_name.clone(),
            status: "importing".to_string(),
        });

        match import_session(db, data_dir, file_path, force_duplicate).await {
            Ok(result) => {
                match &result {
                    ImportResult::Success { .. } => {
                        succeeded += 1;
                        on_progress(ImportProgress {
                            current: i + 1,
                            total,
                            file_name,
                            status: "success".to_string(),
                        });
                    }
                    ImportResult::Duplicate { .. } => {
                        duplicates_skipped += 1;
                        on_progress(ImportProgress {
                            current: i + 1,
                            total,
                            file_name,
                            status: "duplicate_skipped".to_string(),
                        });
                    }
                    ImportResult::Failed { .. } => {
                        failed += 1;
                        on_progress(ImportProgress {
                            current: i + 1,
                            total,
                            file_name,
                            status: "failed".to_string(),
                        });
                    }
                }
                results.push(result);
            }
            Err(e) => {
                failed += 1;
                on_progress(ImportProgress {
                    current: i + 1,
                    total,
                    file_name: file_name.clone(),
                    status: "failed".to_string(),
                });
                results.push(ImportResult::Failed {
                    error: e.to_string(),
                    file_path: file_path.display().to_string(),
                });
            }
        }
    }

    Ok(BatchImportResult {
        total,
        succeeded,
        duplicates_skipped,
        failed,
        results,
    })
}

/// Return the list of supported import file extensions.
pub fn supported_import_formats() -> Vec<String> {
    vec![".ibt".to_string(), ".parquet".to_string()]
}
