use std::fs::File;
use std::path::Path;
use std::sync::Arc;

use arrow::array::Float64Array;
use arrow::compute::filter_record_batch;
use arrow::record_batch::RecordBatch;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;

use crate::error::StorageError;
use crate::parquet::checksum::validate_checksum;

/// Read an entire Parquet file as a single RecordBatch.
///
/// Validates the SHA-256 checksum before returning data if `expected_checksum` is provided.
pub fn read_telemetry(
    path: &Path,
    expected_checksum: Option<&str>,
) -> Result<RecordBatch, StorageError> {
    if let Some(checksum) = expected_checksum {
        if !validate_checksum(path, checksum)? {
            return Err(StorageError::ChecksumMismatch {
                path: path.display().to_string(),
            });
        }
    }

    let file = File::open(path)?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)
        .map_err(|e| StorageError::ParquetRead(format!("Failed to open parquet: {}", e)))?;

    let reader = builder
        .build()
        .map_err(|e| StorageError::ParquetRead(format!("Failed to build reader: {}", e)))?;

    let batches: Vec<RecordBatch> = reader
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| StorageError::ParquetRead(format!("Failed to read batches: {}", e)))?;

    if batches.is_empty() {
        return Err(StorageError::ParquetRead(
            "Parquet file contains no data".to_string(),
        ));
    }

    // Concatenate all batches into one
    arrow::compute::concat_batches(&Arc::clone(batches[0].schema_ref()), &batches)
        .map_err(|e| StorageError::ParquetRead(format!("Failed to concat batches: {}", e)))
}

/// Read a windowed slice of telemetry data filtered by lap_distance range.
///
/// Returns only rows where `start_distance <= lap_distance <= end_distance`.
pub fn read_telemetry_window(
    path: &Path,
    start_distance: f64,
    end_distance: f64,
    expected_checksum: Option<&str>,
) -> Result<RecordBatch, StorageError> {
    let batch = read_telemetry(path, expected_checksum)?;

    let lap_distance_idx = batch
        .schema()
        .index_of("lap_distance")
        .map_err(|_| StorageError::ParquetRead("lap_distance column not found".to_string()))?;

    let lap_distance = batch
        .column(lap_distance_idx)
        .as_any()
        .downcast_ref::<Float64Array>()
        .ok_or_else(|| {
            StorageError::ParquetRead("lap_distance column is not Float64".to_string())
        })?;

    // Build boolean filter mask
    let filter_mask: arrow::array::BooleanArray = lap_distance
        .iter()
        .map(|val| val.map(|v| v >= start_distance && v <= end_distance))
        .collect();

    filter_record_batch(&batch, &filter_mask)
        .map_err(|e| StorageError::ParquetRead(format!("Failed to filter batch: {}", e)))
}
