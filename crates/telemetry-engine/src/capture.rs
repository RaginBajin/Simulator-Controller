//! Telemetry capture engine
//!
//! Captures telemetry from IRSDK at configurable sample rate (default 60Hz)
//! and stores in ring buffer for periodic flush to storage.

use crate::irsdk::IrsdkReader;
use crate::ring_buffer::TelemetryRingBuffer;
use crate::sample::{SessionState, TelemetrySample};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Configuration for capture engine
#[derive(Debug, Clone)]
pub struct CaptureConfig {
    /// Sample rate in Hz (10-120 valid range, default 60)
    pub sample_rate: u32,
    /// Ring buffer capacity (default 18000 = 5 minutes at 60Hz)
    pub buffer_capacity: usize,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            sample_rate: 60,
            buffer_capacity: 18_000,
        }
    }
}

/// Telemetry capture engine
pub struct CaptureEngine {
    config: CaptureConfig,
    ring_buffer: TelemetryRingBuffer,
    is_capturing: Arc<AtomicBool>,
    capture_thread: Option<JoinHandle<()>>,
}

impl CaptureEngine {
    /// Create a new capture engine with default configuration
    pub fn new() -> Self {
        Self::with_config(CaptureConfig::default())
    }

    /// Create a new capture engine with custom configuration
    pub fn with_config(config: CaptureConfig) -> Self {
        Self {
            ring_buffer: TelemetryRingBuffer::new(config.buffer_capacity),
            config,
            is_capturing: Arc::new(AtomicBool::new(false)),
            capture_thread: None,
        }
    }

    /// Check if capture is currently running
    pub fn is_capturing(&self) -> bool {
        self.is_capturing.load(Ordering::Relaxed)
    }

    /// Get reference to the ring buffer
    pub fn ring_buffer(&self) -> &TelemetryRingBuffer {
        &self.ring_buffer
    }

    /// Start capture (stub for MVP - full implementation in Task 4)
    /// This is a placeholder that will be fully implemented in subsequent tasks
    pub fn start_capture(&mut self) -> Result<(), String> {
        if self.is_capturing() {
            return Err("Capture already running".to_string());
        }

        info!("Starting capture at {}Hz", self.config.sample_rate);

        let is_capturing = Arc::clone(&self.is_capturing);
        let ring_buffer = self.ring_buffer.clone();
        let sample_rate = self.config.sample_rate;

        is_capturing.store(true, Ordering::Relaxed);

        let handle = thread::spawn(move || {
            Self::capture_loop(is_capturing, ring_buffer, sample_rate);
        });

        self.capture_thread = Some(handle);

        Ok(())
    }

    /// Stop capture
    pub fn stop_capture(&mut self) {
        if !self.is_capturing() {
            return;
        }

        info!("Stopping capture");
        self.is_capturing.store(false, Ordering::Relaxed);

        if let Some(handle) = self.capture_thread.take() {
            let _ = handle.join();
        }
    }

    /// Capture loop (runs in background thread)
    fn capture_loop(
        is_capturing: Arc<AtomicBool>,
        ring_buffer: TelemetryRingBuffer,
        sample_rate: u32,
    ) {
        let interval = Duration::from_micros(1_000_000 / sample_rate as u64);
        let mut reader = IrsdkReader::new();
        let mut last_sample = Instant::now();

        while is_capturing.load(Ordering::Relaxed) {
            let now = Instant::now();
            let elapsed = now.duration_since(last_sample);

            if elapsed >= interval {
                // Read telemetry frame
                match Self::read_telemetry_frame(&mut reader) {
                    Ok(Some(sample)) => {
                        ring_buffer.push(sample);
                        last_sample = now;
                    }
                    Ok(None) => {
                        // No data available, continue
                    }
                    Err(e) => {
                        warn!("Failed to read telemetry frame: {}", e);
                    }
                }
            }

            // Sleep for a fraction of the interval to avoid busy-waiting
            thread::sleep(Duration::from_millis(1));
        }

        debug!("Capture loop exited");
    }

    /// Read a single telemetry frame from IRSDK
    /// Returns Ok(Some(sample)) if data available, Ok(None) if not connected, Err on error
    fn read_telemetry_frame(reader: &mut IrsdkReader) -> Result<Option<TelemetrySample>, String> {
        // Check if connected
        if !reader.is_connected() {
            return Ok(None);
        }

        // For MVP, create a minimal sample with timestamp
        // Full implementation will read all channels from IRSDK shared memory
        let timestamp_ms = chrono::Utc::now().timestamp_millis();
        let mut sample = TelemetrySample::new(timestamp_ms);
        sample.session_state = SessionState::Driving;

        // TODO: Task 1.2-1.6 - Read actual telemetry data from IRSDK shared memory
        // This is a stub that will be implemented in the full Task 1 completion

        Ok(Some(sample))
    }
}

impl Default for CaptureEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for CaptureEngine {
    fn drop(&mut self) {
        self.stop_capture();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_engine_creation() {
        let engine = CaptureEngine::new();
        assert!(!engine.is_capturing());
        assert_eq!(engine.ring_buffer().capacity(), 18_000);
    }

    #[test]
    fn test_capture_engine_with_custom_config() {
        let config = CaptureConfig {
            sample_rate: 30,
            buffer_capacity: 1000,
        };
        let engine = CaptureEngine::with_config(config);
        assert_eq!(engine.ring_buffer().capacity(), 1000);
    }

    #[test]
    fn test_capture_start_stop() {
        let mut engine = CaptureEngine::new();

        assert!(!engine.is_capturing());

        engine.start_capture().unwrap();
        assert!(engine.is_capturing());

        // Give it a moment to actually start
        std::thread::sleep(Duration::from_millis(50));

        engine.stop_capture();
        assert!(!engine.is_capturing());
    }

    #[test]
    fn test_capture_prevents_double_start() {
        let mut engine = CaptureEngine::new();

        engine.start_capture().unwrap();
        let result = engine.start_capture();
        assert!(result.is_err());

        engine.stop_capture();
    }

    #[test]
    fn test_capture_config_defaults() {
        let config = CaptureConfig::default();
        assert_eq!(config.sample_rate, 60);
        assert_eq!(config.buffer_capacity, 18_000);
    }
}
