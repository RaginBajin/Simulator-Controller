use std::collections::HashMap;
use std::sync::Arc;

use arrow::array::{BooleanArray, Float64Array, Int32Array, Int64Array};
use arrow::record_batch::RecordBatch;
use tempfile::TempDir;

use storage::parquet::{
    cleanup_orphaned_temps, compute_checksum, read_telemetry, read_telemetry_window,
    telemetry_schema, validate_checksum, write_telemetry, TELEMETRY_CHANNELS,
};

/// Create a synthetic RecordBatch with all 35 channels and the given number of rows.
fn create_test_batch(num_rows: usize) -> RecordBatch {
    let schema = Arc::new(telemetry_schema());

    // Build columns in schema order
    let mut columns: Vec<Arc<dyn arrow::array::Array>> = Vec::with_capacity(35);

    for (i, name) in TELEMETRY_CHANNELS.iter().enumerate() {
        let col: Arc<dyn arrow::array::Array> = match *name {
            "timestamp_ms" => {
                let values: Vec<i64> = (0..num_rows).map(|r| r as i64 * 16).collect(); // ~60Hz
                Arc::new(Int64Array::from(values))
            }
            "gear" => {
                let values: Vec<i32> = (0..num_rows).map(|r| (r % 7) as i32 + 1).collect();
                Arc::new(Int32Array::from(values))
            }
            "abs_active" | "tc_active" => {
                let values: Vec<bool> = (0..num_rows).map(|r| r % 10 == 0).collect();
                Arc::new(BooleanArray::from(values))
            }
            "lap_distance" => {
                // 0 to num_rows meters
                let values: Vec<f64> = (0..num_rows).map(|r| r as f64).collect();
                Arc::new(Float64Array::from(values))
            }
            _ => {
                // All other Float64 channels
                let values: Vec<f64> = (0..num_rows)
                    .map(|r| (r as f64) * 0.1 + (i as f64))
                    .collect();
                Arc::new(Float64Array::from(values))
            }
        };
        columns.push(col);
    }

    RecordBatch::try_new(schema, columns).expect("Failed to create test batch")
}

fn test_metadata() -> HashMap<String, String> {
    HashMap::from([
        ("session_id".to_string(), "test-session-1".to_string()),
        ("track_name".to_string(), "Spa-Francorchamps".to_string()),
        ("car_name".to_string(), "McLaren 720S GT3".to_string()),
        (
            "started_at".to_string(),
            "2026-02-07T12:00:00Z".to_string(),
        ),
        ("sample_rate_hz".to_string(), "60".to_string()),
        ("channel_count".to_string(), "35".to_string()),
    ])
}

#[tokio::test]
async fn write_and_read_roundtrip() {
    let dir = TempDir::new().unwrap();
    let batch = create_test_batch(100);
    let metadata = test_metadata();

    let result =
        write_telemetry("test-session-1", dir.path(), &batch, metadata).await.unwrap();

    assert_eq!(result.row_count, 100);
    assert!(result.path.exists());
    assert!(result.size_bytes > 0);
    assert!(!result.checksum.is_empty());

    // Read back
    let read_batch = read_telemetry(&result.path, Some(&result.checksum)).unwrap();
    assert_eq!(read_batch.num_rows(), 100);
    assert_eq!(read_batch.num_columns(), 35);
}

#[tokio::test]
async fn snappy_compression_reduces_size() {
    let dir = TempDir::new().unwrap();
    let batch = create_test_batch(1000);
    let metadata = test_metadata();

    let result =
        write_telemetry("compression-test", dir.path(), &batch, metadata).await.unwrap();

    // 1000 rows x 35 cols x 8 bytes avg = ~280KB raw. Snappy should compress.
    // Just verify the file is smaller than a naive estimate.
    let raw_estimate = 1000 * 35 * 8; // ~280KB
    assert!(
        result.size_bytes < raw_estimate as u64,
        "Parquet file should be smaller than raw data (file: {} bytes, raw est: {} bytes)",
        result.size_bytes,
        raw_estimate
    );
}

#[tokio::test]
async fn checksum_validates_correctly() {
    let dir = TempDir::new().unwrap();
    let batch = create_test_batch(50);
    let metadata = test_metadata();

    let result = write_telemetry("checksum-test", dir.path(), &batch, metadata).await.unwrap();

    // Valid checksum
    assert!(validate_checksum(&result.path, &result.checksum).unwrap());

    // Invalid checksum
    assert!(!validate_checksum(&result.path, "wrong-checksum").unwrap());
}

#[tokio::test]
async fn corrupted_file_detected_by_checksum() {
    let dir = TempDir::new().unwrap();
    let batch = create_test_batch(50);
    let metadata = test_metadata();

    let result = write_telemetry("corrupt-test", dir.path(), &batch, metadata).await.unwrap();

    // Corrupt the file by overwriting some bytes
    std::fs::write(&result.path, b"corrupted data").unwrap();

    // Recomputed checksum should not match
    let new_checksum = compute_checksum(&result.path).unwrap();
    assert_ne!(new_checksum, result.checksum);

    // Reading with original checksum should fail
    let read_result = read_telemetry(&result.path, Some(&result.checksum));
    assert!(read_result.is_err());
}

#[tokio::test]
async fn atomic_write_no_corrupt_file_on_readonly_dir() {
    let dir = TempDir::new().unwrap();
    let telemetry_dir = dir.path().join("telemetry");
    std::fs::create_dir_all(&telemetry_dir).unwrap();

    // Make the telemetry directory read-only
    let mut perms = std::fs::metadata(&telemetry_dir).unwrap().permissions();
    perms.set_readonly(true);
    std::fs::set_permissions(&telemetry_dir, perms.clone()).unwrap();

    let batch = create_test_batch(10);
    let metadata = test_metadata();

    // Write should fail (can't create temp file)
    let result = write_telemetry("readonly-test", dir.path(), &batch, metadata).await;
    assert!(result.is_err());

    // Restore permissions for cleanup
    perms.set_readonly(false);
    std::fs::set_permissions(&telemetry_dir, perms).unwrap();

    // No .parquet file should exist
    let parquet_path = telemetry_dir.join("readonly-test.parquet");
    assert!(!parquet_path.exists());

    // No .tmp file should remain
    let tmp_path = telemetry_dir.join("readonly-test.parquet.tmp");
    assert!(!tmp_path.exists());
}

#[test]
fn orphaned_temp_cleanup() {
    let dir = TempDir::new().unwrap();
    let telemetry_dir = dir.path().join("telemetry");
    std::fs::create_dir_all(&telemetry_dir).unwrap();

    // Create some orphaned temp files
    std::fs::write(telemetry_dir.join("session-1.parquet.tmp"), b"temp1").unwrap();
    std::fs::write(telemetry_dir.join("session-2.parquet.tmp"), b"temp2").unwrap();

    // Create a valid parquet file that should NOT be removed
    std::fs::write(telemetry_dir.join("session-3.parquet"), b"valid").unwrap();

    let cleaned = cleanup_orphaned_temps(&telemetry_dir).unwrap();
    assert_eq!(cleaned, 2);

    // Temp files should be gone
    assert!(!telemetry_dir.join("session-1.parquet.tmp").exists());
    assert!(!telemetry_dir.join("session-2.parquet.tmp").exists());

    // Valid parquet should still exist
    assert!(telemetry_dir.join("session-3.parquet").exists());
}

#[tokio::test]
async fn windowed_read_returns_correct_range() {
    let dir = TempDir::new().unwrap();
    let batch = create_test_batch(1000); // lap_distance: 0..999
    let metadata = test_metadata();

    let result = write_telemetry("window-test", dir.path(), &batch, metadata).await.unwrap();

    // Read window: distance 100..200 (inclusive)
    let windowed = read_telemetry_window(&result.path, 100.0, 200.0, Some(&result.checksum)).unwrap();

    // Should have rows where 100 <= lap_distance <= 200 = 101 rows
    assert_eq!(windowed.num_rows(), 101);

    // Verify the first and last lap_distance values
    let lap_distance_idx = windowed.schema().index_of("lap_distance").unwrap();
    let lap_distance = windowed
        .column(lap_distance_idx)
        .as_any()
        .downcast_ref::<Float64Array>()
        .unwrap();

    assert!((lap_distance.value(0) - 100.0).abs() < f64::EPSILON);
    assert!((lap_distance.value(100) - 200.0).abs() < f64::EPSILON);
}

#[tokio::test]
async fn performance_large_session_read() {
    let dir = TempDir::new().unwrap();
    // 60 minutes at 60Hz = 216,000 rows
    let batch = create_test_batch(216_000);
    let metadata = test_metadata();

    let result = write_telemetry("perf-test", dir.path(), &batch, metadata).await.unwrap();

    let start = std::time::Instant::now();
    let read_batch = read_telemetry(&result.path, Some(&result.checksum)).unwrap();
    let elapsed = start.elapsed();

    assert_eq!(read_batch.num_rows(), 216_000);

    // NFR5a: <200ms in release builds. Debug builds are ~10x slower.
    let max_ms = if cfg!(debug_assertions) { 5000 } else { 200 };
    assert!(
        elapsed.as_millis() < max_ms,
        "Read took {}ms, expected <{}ms ({})",
        elapsed.as_millis(),
        max_ms,
        if cfg!(debug_assertions) { "debug" } else { "release" }
    );
}

#[tokio::test]
async fn read_without_checksum_validation() {
    let dir = TempDir::new().unwrap();
    let batch = create_test_batch(50);
    let metadata = test_metadata();

    let result = write_telemetry("no-checksum", dir.path(), &batch, metadata).await.unwrap();

    // Read without checksum validation
    let read_batch = read_telemetry(&result.path, None).unwrap();
    assert_eq!(read_batch.num_rows(), 50);
}

#[test]
fn schema_has_correct_structure() {
    let schema = telemetry_schema();
    assert_eq!(schema.fields().len(), 35);

    // Check a few key fields
    assert_eq!(schema.field(0).name(), "timestamp_ms");
    assert_eq!(*schema.field(0).data_type(), arrow::datatypes::DataType::Int64);

    assert_eq!(schema.field(9).name(), "gear");
    assert_eq!(*schema.field(9).data_type(), arrow::datatypes::DataType::Int32);

    assert_eq!(schema.field(32).name(), "abs_active");
    assert_eq!(*schema.field(32).data_type(), arrow::datatypes::DataType::Boolean);

    assert_eq!(schema.field(34).name(), "track_position");
    assert_eq!(*schema.field(34).data_type(), arrow::datatypes::DataType::Float64);
}
