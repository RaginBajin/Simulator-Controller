//! Background I/O thread for non-blocking Parquet writes.
//!
//! This module provides a background I/O worker that receives flush requests from
//! the capture thread and writes telemetry data to Parquet files without blocking
//! the capture loop.
//!
//! # Architecture
//!
//! - Runs on a dedicated background thread with tokio runtime
//! - Receives `FlushRequest` messages via crossbeam channel
//! - Writes to Parquet using storage crate's `write_telemetry` API
//! - Supports periodic flush (default: 30s) and session-end flush
//! - Ensures all data is flushed before session completion

use arrow::record_batch::RecordBatch;
use crossbeam::channel;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::thread;
use storage::{write_telemetry, TelemetryWriteResult};
use tracing::{error, info, warn};

/// Default interval for periodic flushes (seconds)
pub const FLUSH_INTERVAL_SECS: u64 = 30;

/// Type of flush operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlushType {
    /// Periodic flush during active capture
    Periodic,
    /// Final flush at session end
    SessionEnd,
}

/// Request to flush telemetry data to Parquet
#[derive(Debug)]
pub struct FlushRequest {
    /// Session ID for the telemetry data
    pub session_id: String,
    /// Buffered telemetry data as Arrow RecordBatch
    pub batch: RecordBatch,
    /// Type of flush (periodic or session end)
    pub flush_type: FlushType,
    /// File metadata to include in Parquet file
    pub file_metadata: HashMap<String, String>,
}

/// Commands for controlling the I/O worker
#[derive(Debug)]
pub enum IoCommand {
    /// Process a flush request
    Flush(FlushRequest),
    /// Stop the I/O worker and shut down the thread
    Stop,
}

/// Handle for the background I/O worker thread
pub struct CaptureIo {
    command_tx: channel::Sender<IoCommand>,
    result_rx: channel::Receiver<Result<TelemetryWriteResult, String>>,
    thread_handle: Option<thread::JoinHandle<()>>,
}

impl CaptureIo {
    /// Start the background I/O worker thread
    ///
    /// # Arguments
    /// * `data_dir` - Base directory for telemetry storage
    ///
    /// # Returns
    /// A `CaptureIo` handle for sending flush requests and receiving results
    pub fn start(data_dir: PathBuf) -> Self {
        let (command_tx, command_rx) = channel::unbounded();
        let (result_tx, result_rx) = channel::unbounded();

        let thread_handle = thread::Builder::new()
            .name("capture-io".to_string())
            .spawn(move || {
                Self::io_loop(data_dir, command_rx, result_tx);
            })
            .expect("Failed to spawn capture I/O thread");

        Self {
            command_tx,
            result_rx,
            thread_handle: Some(thread_handle),
        }
    }

    /// Send a flush request to the I/O worker
    ///
    /// # Arguments
    /// * `request` - The flush request containing session data and batch
    ///
    /// # Returns
    /// `Ok(())` if the request was queued, `Err` if the channel is disconnected
    pub fn flush(&self, request: FlushRequest) -> Result<(), String> {
        self.command_tx
            .send(IoCommand::Flush(request))
            .map_err(|e| format!("Failed to send flush request: {}", e))
    }

    /// Try to receive a flush result (non-blocking)
    ///
    /// # Returns
    /// `Some(result)` if a result is available, `None` otherwise
    pub fn try_recv_result(&self) -> Option<Result<TelemetryWriteResult, String>> {
        self.result_rx.try_recv().ok()
    }

    /// Stop the I/O worker and wait for thread to complete
    pub fn stop(mut self) {
        let _ = self.command_tx.send(IoCommand::Stop);
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }

    /// Main I/O loop that runs on the background thread
    fn io_loop(
        data_dir: PathBuf,
        command_rx: channel::Receiver<IoCommand>,
        result_tx: channel::Sender<Result<TelemetryWriteResult, String>>,
    ) {
        // Create tokio runtime for async write_telemetry calls
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime");

        info!("Capture I/O worker started");

        loop {
            match command_rx.recv() {
                Ok(IoCommand::Flush(request)) => {
                    let session_id = request.session_id.clone();
                    let flush_type = request.flush_type;
                    let row_count = request.batch.num_rows();

                    info!(
                        session_id = %session_id,
                        flush_type = ?flush_type,
                        rows = row_count,
                        "Processing flush request"
                    );

                    // Execute async write_telemetry on tokio runtime
                    let result = runtime.block_on(write_telemetry(
                        &request.session_id,
                        &data_dir,
                        &request.batch,
                        request.file_metadata,
                    ));

                    match result {
                        Ok(write_result) => {
                            info!(
                                session_id = %session_id,
                                path = %write_result.path.display(),
                                size_bytes = write_result.size_bytes,
                                rows = write_result.row_count,
                                checksum = %write_result.checksum,
                                "Flush completed successfully"
                            );
                            let _ = result_tx.send(Ok(write_result));
                        }
                        Err(e) => {
                            error!(
                                session_id = %session_id,
                                error = %e,
                                "Flush failed"
                            );
                            let _ = result_tx.send(Err(format!("Parquet write failed: {}", e)));
                        }
                    }
                }
                Ok(IoCommand::Stop) => {
                    info!("Capture I/O worker stopping");
                    break;
                }
                Err(_) => {
                    // Channel closed, exit loop
                    warn!("Command channel closed, stopping I/O worker");
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::{Float64Array, Int32Array, StringArray};
    use arrow::datatypes::{DataType, Field, Schema};
    use std::sync::Arc;
    use std::time::Duration;
    use tempfile::TempDir;

    fn create_test_batch() -> RecordBatch {
        let schema = Arc::new(Schema::new(vec![
            Field::new("session_time", DataType::Float64, false),
            Field::new("lap_number", DataType::Int32, false),
            Field::new("car_name", DataType::Utf8, false),
        ]));

        let session_time = Float64Array::from(vec![1.0, 2.0, 3.0]);
        let lap_number = Int32Array::from(vec![1, 1, 1]);
        let car_name = StringArray::from(vec!["Test Car", "Test Car", "Test Car"]);

        RecordBatch::try_new(
            schema,
            vec![
                Arc::new(session_time),
                Arc::new(lap_number),
                Arc::new(car_name),
            ],
        )
        .expect("Failed to create test batch")
    }

    #[test]
    fn test_capture_io_processes_flush_request() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let data_dir = temp_dir.path().to_path_buf();

        let capture_io = CaptureIo::start(data_dir.clone());

        let batch = create_test_batch();
        let mut metadata = HashMap::new();
        metadata.insert("test_key".to_string(), "test_value".to_string());

        let request = FlushRequest {
            session_id: "test-session-1".to_string(),
            batch,
            flush_type: FlushType::Periodic,
            file_metadata: metadata,
        };

        // Send flush request
        capture_io.flush(request).expect("Failed to send flush");

        // Wait for result
        thread::sleep(Duration::from_secs(2));

        // Check result
        let result = capture_io.try_recv_result();
        assert!(result.is_some(), "Should receive flush result");

        let result = result.unwrap();
        assert!(result.is_ok(), "Flush should succeed: {:?}", result);

        let write_result = result.unwrap();
        assert_eq!(write_result.row_count, 3);
        assert!(write_result.size_bytes > 0);
        assert!(write_result.path.exists());

        capture_io.stop();
    }

    #[test]
    fn test_capture_io_handles_session_end_flush() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let data_dir = temp_dir.path().to_path_buf();

        let capture_io = CaptureIo::start(data_dir.clone());

        let batch = create_test_batch();
        let metadata = HashMap::new();

        let request = FlushRequest {
            session_id: "test-session-2".to_string(),
            batch,
            flush_type: FlushType::SessionEnd,
            file_metadata: metadata,
        };

        // Send session end flush
        capture_io.flush(request).expect("Failed to send flush");

        // Wait for result
        thread::sleep(Duration::from_secs(2));

        // Verify flush completed
        let result = capture_io.try_recv_result();
        assert!(result.is_some());
        assert!(result.unwrap().is_ok());

        capture_io.stop();
    }

    #[test]
    fn test_capture_io_stops_cleanly() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let data_dir = temp_dir.path().to_path_buf();

        let capture_io = CaptureIo::start(data_dir);

        // Stop should complete without hanging
        capture_io.stop();
        // If we reach here, stop completed successfully
    }
}
