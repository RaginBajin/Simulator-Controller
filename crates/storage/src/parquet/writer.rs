use std::collections::HashMap;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;
use parquet::basic::Compression;
use parquet::file::metadata::KeyValue;
use parquet::file::properties::WriterProperties;
use tracing::{error, warn};

use crate::error::StorageError;
use crate::parquet::checksum::compute_checksum;

/// Result of a successful telemetry write operation.
#[derive(Debug, Clone)]
pub struct TelemetryWriteResult {
    pub path: PathBuf,
    pub size_bytes: u64,
    pub checksum: String,
    pub row_count: usize,
}

/// Write a telemetry RecordBatch to a Parquet file with atomic write pattern.
///
/// Uses temp file + rename for crash safety. Retries once on failure.
pub async fn write_telemetry(
    session_id: &str,
    data_dir: &Path,
    batch: &RecordBatch,
    file_metadata: HashMap<String, String>,
) -> Result<TelemetryWriteResult, StorageError> {
    let telemetry_dir = data_dir.join("telemetry");
    fs::create_dir_all(&telemetry_dir)?;

    let final_path = telemetry_dir.join(format!("{}.parquet", session_id));
    let temp_path = telemetry_dir.join(format!("{}.parquet.tmp", session_id));

    // First attempt
    match write_parquet_atomic(&temp_path, &final_path, batch, &file_metadata) {
        Ok(result) => Ok(result),
        Err(first_err) => {
            warn!(
                session_id = session_id,
                error = %first_err,
                "Parquet write failed, retrying in 5 seconds"
            );

            // Clean up temp file from failed attempt
            let _ = fs::remove_file(&temp_path);

            // Non-blocking sleep before retry
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;

            match write_parquet_atomic(&temp_path, &final_path, batch, &file_metadata) {
                Ok(result) => Ok(result),
                Err(retry_err) => {
                    // Clean up temp file from failed retry
                    let _ = fs::remove_file(&temp_path);
                    error!(
                        session_id = session_id,
                        path = %final_path.display(),
                        error = %retry_err,
                        "Parquet write failed after retry"
                    );
                    Err(retry_err)
                }
            }
        }
    }
}

/// Write to a temp file and atomically rename to the final path.
fn write_parquet_atomic(
    temp_path: &Path,
    final_path: &Path,
    batch: &RecordBatch,
    file_metadata: &HashMap<String, String>,
) -> Result<TelemetryWriteResult, StorageError> {
    let row_count = batch.num_rows();

    // Build writer properties with Snappy compression and file-level metadata
    let props = WriterProperties::builder()
        .set_compression(Compression::SNAPPY)
        .set_created_by("Pitwall".to_string())
        .set_key_value_metadata(Some(
            file_metadata
                .iter()
                .map(|(k, v)| KeyValue::new(k.clone(), v.clone()))
                .collect(),
        ))
        .build();

    // Write to temp file
    let file = File::create(temp_path).map_err(|e| {
        StorageError::ParquetWrite(format!("Failed to create temp file: {}", e))
    })?;

    let mut writer =
        ArrowWriter::try_new(file, Arc::clone(batch.schema_ref()), Some(props))
            .map_err(|e| StorageError::ParquetWrite(format!("Failed to create ArrowWriter: {}", e)))?;

    writer
        .write(batch)
        .map_err(|e| StorageError::ParquetWrite(format!("Failed to write batch: {}", e)))?;

    writer
        .close()
        .map_err(|e| StorageError::ParquetWrite(format!("Failed to close writer: {}", e)))?;

    // Sync file to disk before rename to ensure data durability on power loss
    let file = File::open(temp_path).map_err(|e| {
        StorageError::ParquetWrite(format!("Failed to open temp file for fsync: {}", e))
    })?;
    file.sync_all().map_err(|e| {
        StorageError::ParquetWrite(format!("Failed to fsync temp file: {}", e))
    })?;

    // Atomic rename from temp to final
    fs::rename(temp_path, final_path)?;

    // Compute checksum of final file
    let checksum = compute_checksum(final_path)?;

    let size_bytes = fs::metadata(final_path)?.len();

    Ok(TelemetryWriteResult {
        path: final_path.to_path_buf(),
        size_bytes,
        checksum,
        row_count,
    })
}
