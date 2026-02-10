//! Tauri event emitter implementation for capture events

use tauri::{AppHandle, Emitter};
use telemetry_engine::event_emitter::CaptureEventEmitter;
use crate::events::{CaptureReconnecting, CaptureReconnected, CaptureError, SessionCompleted};

/// Tauri implementation of the CaptureEventEmitter trait
#[derive(Clone)]
pub struct TauriEventEmitter {
    app_handle: AppHandle,
}

impl TauriEventEmitter {
    /// Creates a new Tauri event emitter
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }
}

impl CaptureEventEmitter for TauriEventEmitter {
    fn emit_reconnecting(
        &self,
        attempt_number: u32,
        max_attempts: u32,
        elapsed_ms: u64,
    ) -> Result<(), String> {
        let payload = CaptureReconnecting::new(attempt_number, max_attempts, elapsed_ms);

        self.app_handle
            .emit(crate::events::CAPTURE_RECONNECTING, &payload)
            .map_err(|e| format!("Failed to emit reconnecting event: {}", e))
    }

    fn emit_reconnected(&self, gap_duration_ms: u64) -> Result<(), String> {
        let payload = CaptureReconnected::new(gap_duration_ms);

        self.app_handle
            .emit(crate::events::CAPTURE_RECONNECTED, &payload)
            .map_err(|e| format!("Failed to emit reconnected event: {}", e))
    }

    fn emit_capture_error(
        &self,
        code: String,
        message: String,
        details: Option<String>,
        retryable: bool,
    ) -> Result<(), String> {
        let payload = CaptureError::new(code, message, details, retryable);

        self.app_handle
            .emit(crate::events::CAPTURE_ERROR, &payload)
            .map_err(|e| format!("Failed to emit capture error event: {}", e))
    }

    fn emit_session_completed(
        &self,
        session_id: String,
        lap_count: i32,
        duration_ms: u64,
        status: String,
        gap_count: usize,
    ) -> Result<(), String> {
        let payload = SessionCompleted::new(session_id, lap_count, duration_ms, status, gap_count);

        self.app_handle
            .emit(crate::events::SESSION_COMPLETED, &payload)
            .map_err(|e| format!("Failed to emit session completed event: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Full integration tests for Tauri event emission require a running Tauri app.
    // These tests verify the struct can be constructed and implements the trait.

    #[test]
    fn test_tauri_event_emitter_implements_trait() {
        // This test ensures the trait is properly implemented
        // Actual emission testing requires integration tests with a Tauri runtime
        fn _assert_implements_trait<T: CaptureEventEmitter>() {}
        _assert_implements_trait::<TauriEventEmitter>();
    }
}
