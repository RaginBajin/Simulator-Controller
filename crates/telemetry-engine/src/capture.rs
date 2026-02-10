//! Telemetry capture engine
//!
//! Captures telemetry from IRSDK at configurable sample rate (default 60Hz)
//! and stores in ring buffer for periodic flush to storage.

use crate::connection::ConnectionManager;
use crate::irsdk::{ConnectionEvent, ConnectionStatus, IrsdkReader};
use crate::ring_buffer::TelemetryRingBuffer;
use crate::sample::{SessionState, TelemetrySample};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
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

/// Flush callback type - called when ring buffer needs to be flushed
pub type FlushCallback = Arc<dyn Fn(Vec<TelemetrySample>) + Send + Sync>;

/// Telemetry capture engine
pub struct CaptureEngine {
    config: CaptureConfig,
    ring_buffer: TelemetryRingBuffer,
    is_capturing: Arc<AtomicBool>,
    capture_thread: Option<JoinHandle<()>>,
    flush_thread: Option<JoinHandle<()>>,
    connection_manager: Option<ConnectionManager>,
    connection_event_rx: Option<Receiver<ConnectionEvent>>,
    auto_start_on_connect: bool,
    flush_callback: Option<FlushCallback>,
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
            flush_thread: None,
            connection_manager: None,
            connection_event_rx: None,
            auto_start_on_connect: false,
            flush_callback: None,
        }
    }

    /// Set ConnectionManager for automatic capture start/stop on connect/disconnect
    pub fn with_connection_manager(mut self, mut manager: ConnectionManager) -> Self {
        let rx = manager.start();
        self.connection_manager = Some(manager);
        self.connection_event_rx = Some(rx);
        self.auto_start_on_connect = true;
        self
    }

    /// Enable/disable automatic capture start on IRSDK connection
    pub fn set_auto_start_on_connect(&mut self, enabled: bool) {
        self.auto_start_on_connect = enabled;
    }

    /// Set flush callback for periodic buffer flushing
    pub fn set_flush_callback<F>(&mut self, callback: F)
    where
        F: Fn(Vec<TelemetrySample>) + Send + Sync + 'static,
    {
        self.flush_callback = Some(Arc::new(callback));
    }

    /// Check if capture is currently running
    pub fn is_capturing(&self) -> bool {
        self.is_capturing.load(Ordering::Relaxed)
    }

    /// Get reference to the ring buffer
    pub fn ring_buffer(&self) -> &TelemetryRingBuffer {
        &self.ring_buffer
    }

    /// Start capture manually or automatically on connection
    pub fn start_capture(&mut self) -> Result<(), String> {
        if self.is_capturing() {
            return Err("Capture already running".to_string());
        }

        info!("Starting capture at {}Hz", self.config.sample_rate);

        let is_capturing = Arc::clone(&self.is_capturing);
        let ring_buffer = self.ring_buffer.clone();
        let sample_rate = self.config.sample_rate;

        is_capturing.store(true, Ordering::Relaxed);

        // Start capture thread
        let capture_handle = thread::spawn(move || {
            Self::capture_loop(is_capturing, ring_buffer, sample_rate);
        });

        self.capture_thread = Some(capture_handle);

        // Start flush thread if callback is set
        if let Some(callback) = self.flush_callback.clone() {
            let is_capturing_flush = Arc::clone(&self.is_capturing);
            let ring_buffer_flush = self.ring_buffer.clone();
            let buffer_capacity = self.config.buffer_capacity;

            let flush_handle = thread::spawn(move || {
                Self::flush_loop(
                    is_capturing_flush,
                    ring_buffer_flush,
                    callback,
                    buffer_capacity,
                );
            });

            self.flush_thread = Some(flush_handle);
        }

        Ok(())
    }

    /// Process connection events and automatically start/stop capture
    /// This should be called periodically or in a monitoring thread
    pub fn process_connection_events(&mut self) {
        if !self.auto_start_on_connect {
            return;
        }

        // Collect all pending events first (to avoid borrow conflicts)
        let mut events = Vec::new();
        if let Some(rx) = &self.connection_event_rx {
            while let Ok(event) = rx.try_recv() {
                events.push(event);
            }
        }

        // Process collected events
        for event in events {
            match event.status {
                ConnectionStatus::Connected => {
                    info!("IRSDK connected - auto-starting capture");
                    if let Err(e) = self.start_capture() {
                        warn!("Failed to auto-start capture: {}", e);
                    }
                }
                ConnectionStatus::Disconnected => {
                    info!("IRSDK disconnected - stopping capture");
                    self.stop_capture();
                }
            }
        }
    }

    /// Stop capture
    pub fn stop_capture(&mut self) {
        if !self.is_capturing() {
            return;
        }

        info!("Stopping capture");
        self.is_capturing.store(false, Ordering::Relaxed);

        // Stop capture thread
        if let Some(handle) = self.capture_thread.take() {
            let _ = handle.join();
        }

        // Stop flush thread
        if let Some(handle) = self.flush_thread.take() {
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
        let mut frame_counter: u64 = 0;

        // Get IRSDK tick rate if available (default to 60Hz)
        let irsdk_tick_rate = 60; // TODO: Read from header when connected
        let downsample_ratio = if irsdk_tick_rate > sample_rate {
            irsdk_tick_rate / sample_rate
        } else {
            1
        };

        info!(
            "Capture loop started: {}Hz target, IRSDK {}Hz, downsample ratio: {}",
            sample_rate, irsdk_tick_rate, downsample_ratio
        );

        while is_capturing.load(Ordering::Relaxed) {
            let now = Instant::now();
            let elapsed = now.duration_since(last_sample);

            if elapsed >= interval {
                frame_counter += 1;

                // Smart downsampling: only read every Nth frame if IRSDK tick rate > configured rate
                if downsample_ratio > 1 && !frame_counter.is_multiple_of(downsample_ratio as u64) {
                    // Skip this frame
                    last_sample = now;
                    continue;
                }

                // Read telemetry frame
                match Self::read_telemetry_frame(&mut reader) {
                    Ok(Some(sample)) => {
                        let current_len = ring_buffer.len();
                        let capacity = ring_buffer.capacity();
                        let utilization = current_len as f64 / capacity as f64;

                        // Backpressure warning at 80% capacity
                        if utilization >= 0.8 {
                            warn!(
                                "Ring buffer high utilization: {}/{} ({:.1}%) - urgent flush needed",
                                current_len,
                                capacity,
                                utilization * 100.0
                            );
                        }

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

    /// Periodic flush loop - drains ring buffer every 30s or at 80% capacity
    fn flush_loop(
        is_capturing: Arc<AtomicBool>,
        ring_buffer: TelemetryRingBuffer,
        callback: FlushCallback,
        capacity: usize,
    ) {
        let flush_interval = Duration::from_secs(30);
        let capacity_threshold = (capacity as f64 * 0.8) as usize;
        let mut last_flush = Instant::now();

        info!(
            "Flush loop started (30s interval or 80% capacity = {} samples)",
            capacity_threshold
        );

        while is_capturing.load(Ordering::Relaxed) {
            let now = Instant::now();
            let elapsed = now.duration_since(last_flush);
            let current_len = ring_buffer.len();

            // Check if we should flush (30s timer OR 80% capacity)
            let should_flush = elapsed >= flush_interval || current_len >= capacity_threshold;

            if should_flush && current_len > 0 {
                debug!(
                    "Flushing ring buffer: {} samples (elapsed: {:.1}s, capacity: {:.1}%)",
                    current_len,
                    elapsed.as_secs_f64(),
                    (current_len as f64 / capacity as f64) * 100.0
                );

                // Drain all samples from ring buffer
                let samples = ring_buffer.drain();

                if !samples.is_empty() {
                    // Call flush callback
                    callback(samples);
                }

                last_flush = now;
            }

            // Sleep for 1 second before next check
            thread::sleep(Duration::from_secs(1));
        }

        // Final flush on shutdown if any samples remain
        let remaining = ring_buffer.drain();
        if !remaining.is_empty() {
            info!("Final flush on shutdown: {} samples", remaining.len());
            callback(remaining);
        }

        debug!("Flush loop exited");
    }

    /// Read a single telemetry frame from IRSDK
    /// Returns Ok(Some(sample)) if data available, Ok(None) if not connected, Err on error
    fn read_telemetry_frame(reader: &mut IrsdkReader) -> Result<Option<TelemetrySample>, String> {
        // Check if connected
        if !reader.is_connected() {
            return Ok(None);
        }

        #[cfg(target_os = "windows")]
        {
            Self::read_telemetry_frame_windows(reader)
        }

        #[cfg(not(target_os = "windows"))]
        {
            Self::read_telemetry_frame_mock()
        }
    }

    /// Read telemetry frame on Windows from IRSDK shared memory
    #[cfg(target_os = "windows")]
    fn read_telemetry_frame_windows(
        reader: &mut IrsdkReader,
    ) -> Result<Option<TelemetrySample>, String> {
        use crate::irsdk::irsdk_channel_mapping;
        use std::collections::HashMap;

        // Read variable headers to get offsets and types
        let var_headers = reader
            .read_var_headers()
            .map_err(|e| format!("Failed to read var headers: {}", e))?;

        // Read telemetry data buffer
        let data = reader
            .read_telemetry_data()
            .map_err(|e| format!("Failed to read telemetry data: {}", e))?;

        // Build lookup map: IRSDK variable name -> (offset, type)
        let mut var_lookup: HashMap<String, (usize, i32)> = HashMap::new();
        for var_header in &var_headers {
            // Extract null-terminated name
            let name_end = var_header
                .name
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(var_header.name.len());
            if let Ok(name) = std::str::from_utf8(&var_header.name[..name_end]) {
                var_lookup.insert(
                    name.to_string(),
                    (var_header.offset as usize, var_header.var_type),
                );
            }
        }

        // Get channel mapping
        let channel_mapping = irsdk_channel_mapping();

        // Read SessionTime for timestamp (or use chrono if not available)
        let timestamp_ms = if let Some(&(offset, var_type)) = var_lookup.get("SessionTime") {
            match var_type {
                5 => {
                    // double
                    if let Some(session_time_sec) = reader.read_double(&data, offset) {
                        (session_time_sec * 1000.0) as i64
                    } else {
                        chrono::Utc::now().timestamp_millis()
                    }
                }
                4 => {
                    // float
                    if let Some(session_time_sec) = reader.read_float(&data, offset) {
                        (session_time_sec as f64 * 1000.0) as i64
                    } else {
                        chrono::Utc::now().timestamp_millis()
                    }
                }
                _ => chrono::Utc::now().timestamp_millis(),
            }
        } else {
            chrono::Utc::now().timestamp_millis()
        };

        let mut sample = TelemetrySample::new(timestamp_ms);

        // Helper macro to read a channel value
        macro_rules! read_channel {
            ($irsdk_name:expr, $sample_field:ident, $read_method:ident, $convert:expr) => {
                if let Some(canonical_name) = channel_mapping.get($irsdk_name) {
                    if *canonical_name == stringify!($sample_field).trim_start_matches("r#") {
                        if let Some(&(offset, _)) = var_lookup.get($irsdk_name) {
                            sample.$sample_field = reader.$read_method(&data, offset).map($convert);
                        }
                    }
                }
            };
        }

        // Map all 35 channels
        read_channel!("SessionTime", session_time, read_double, |v| v);
        read_channel!("LapDist", lap_distance, read_double, |v| v);
        read_channel!("LapCurrentLapTime", lap_time, read_float, |v: f32| v as f64);
        read_channel!("Speed", speed, read_float, |v: f32| v as f64);
        read_channel!("Throttle", throttle, read_float, |v: f32| v as f64);
        read_channel!("Brake", brake, read_float, |v: f32| v as f64);
        read_channel!("SteeringWheelAngle", steering, read_float, |v: f32| v
            as f64);
        read_channel!("Clutch", clutch, read_float, |v: f32| v as f64);
        read_channel!("Gear", gear, read_int, |v| v);
        read_channel!("RPM", rpm, read_float, |v: f32| v as f64);
        read_channel!("LatAccel", lat_g, read_float, |v: f32| v as f64);
        read_channel!("LongAccel", long_g, read_float, |v: f32| v as f64);
        read_channel!("Yaw", yaw, read_float, |v: f32| v as f64);
        read_channel!("Pitch", pitch, read_float, |v: f32| v as f64);
        read_channel!("Roll", roll, read_float, |v: f32| v as f64);
        read_channel!("VelocityX", velocity_x, read_float, |v: f32| v as f64);
        read_channel!("VelocityY", velocity_y, read_float, |v: f32| v as f64);
        read_channel!("VelocityZ", velocity_z, read_float, |v: f32| v as f64);
        read_channel!("LFtempCL", tire_temp_lf, read_float, |v: f32| v as f64);
        read_channel!("RFtempCL", tire_temp_rf, read_float, |v: f32| v as f64);
        read_channel!("LRtempCL", tire_temp_lr, read_float, |v: f32| v as f64);
        read_channel!("RRtempCL", tire_temp_rr, read_float, |v: f32| v as f64);
        read_channel!("LFpressure", tire_pressure_lf, read_float, |v: f32| v
            as f64);
        read_channel!("RFpressure", tire_pressure_rf, read_float, |v: f32| v
            as f64);
        read_channel!("LRpressure", tire_pressure_lr, read_float, |v: f32| v
            as f64);
        read_channel!("RRpressure", tire_pressure_rr, read_float, |v: f32| v
            as f64);
        read_channel!("FuelLevel", fuel_level, read_float, |v: f32| v as f64);
        read_channel!("FuelUsePerHour", fuel_usage, read_float, |v: f32| v as f64);
        read_channel!("OilTemp", oil_temp, read_float, |v: f32| v as f64);
        read_channel!("WaterTemp", water_temp, read_float, |v: f32| v as f64);
        read_channel!("dcBrakeBias", brake_bias, read_float, |v: f32| v as f64);
        read_channel!("BrakeABSactive", abs_active, read_bool, |v| v);
        read_channel!("dcTractionControl", tc_active, read_int, |v| v != 0);
        read_channel!("LapDistPct", track_position, read_float, |v: f32| v as f64);

        // Read session state from SessionState IRSDK variable
        if let Some(&(offset, _)) = var_lookup.get("SessionState") {
            if let Some(state_int) = reader.read_int(&data, offset) {
                sample.session_state = match state_int {
                    0 => SessionState::Invalid,
                    4 => SessionState::Driving,    // StateRacing
                    5 => SessionState::Pitting,    // StateParadeLaps
                    6 => SessionState::Spectating, // StateWarmup (treat as spectating)
                    _ => SessionState::Driving,
                };
            }
        }

        // Read environmental and session context fields (MEDIUM priority fix)
        // Track temp, air temp, weather
        if let Some(&(offset, _)) = var_lookup.get("TrackTemp") {
            sample.track_temp = reader.read_float(&data, offset).map(|v| v as f64);
        }
        if let Some(&(offset, _)) = var_lookup.get("AirTemp") {
            sample.air_temp = reader.read_float(&data, offset).map(|v| v as f64);
        }
        if let Some(&(offset, _)) = var_lookup.get("Skies") {
            sample.weather = reader
                .read_int(&data, offset)
                .map(|v| format!("skies_{}", v));
        }

        // Session context would normally come from session info YAML parsing
        // For now, leave as None (deferred to future stories)

        Ok(Some(sample))
    }

    /// Mock telemetry frame for non-Windows testing
    #[cfg(not(target_os = "windows"))]
    fn read_telemetry_frame_mock() -> Result<Option<TelemetrySample>, String> {
        use rand::Rng;

        let mut rng = rand::thread_rng();
        let timestamp_ms = chrono::Utc::now().timestamp_millis();
        let mut sample = TelemetrySample::new(timestamp_ms);

        // Generate synthetic telemetry data for testing
        sample.session_state = SessionState::Driving;
        sample.speed = Some(rng.gen_range(0.0..200.0));
        sample.throttle = Some(rng.gen_range(0.0..1.0));
        sample.brake = Some(rng.gen_range(0.0..1.0));
        sample.steering = Some(rng.gen_range(-1.0..1.0));
        sample.gear = Some(rng.gen_range(1..7));
        sample.rpm = Some(rng.gen_range(1000.0..8000.0));
        sample.lat_g = Some(rng.gen_range(-2.0..2.0));
        sample.long_g = Some(rng.gen_range(-2.0..2.0));
        sample.fuel_level = Some(rng.gen_range(0.0..100.0));

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
