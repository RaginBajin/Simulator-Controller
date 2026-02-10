//! Event emission helpers for telemetry capture events
//!
//! This module provides helper functions to emit Tauri events during capture.
//! The actual Tauri app handle is passed from the src-tauri layer.

use crate::capture_error::CaptureError;
use crate::irsdk::reconnection::ReconnectionStatus;

/// Event emission interface for capture events.
/// This trait allows the telemetry engine to emit events without depending on Tauri directly.
pub trait CaptureEventEmitter: Send + Sync {
    /// Emit a reconnection attempt event
    fn emit_reconnecting(
        &self,
        attempt_number: u32,
        max_attempts: u32,
        elapsed_ms: u64,
    ) -> Result<(), String>;

    /// Emit a successful reconnection event
    fn emit_reconnected(&self, gap_duration_ms: u64) -> Result<(), String>;

    /// Emit a capture error event
    fn emit_capture_error(
        &self,
        code: String,
        message: String,
        details: Option<String>,
        retryable: bool,
    ) -> Result<(), String>;

    /// Emit a session completion event
    fn emit_session_completed(
        &self,
        session_id: String,
        lap_count: i32,
        duration_ms: u64,
        status: String,
        gap_count: usize,
    ) -> Result<(), String>;
}

/// Helper to emit reconnection status as an event
pub fn emit_reconnection_status<E: CaptureEventEmitter>(
    emitter: &E,
    status: &ReconnectionStatus,
) -> Result<(), String> {
    match status {
        ReconnectionStatus::Attempting {
            attempt_number,
            max_attempts,
            elapsed_ms,
        } => emitter.emit_reconnecting(*attempt_number, *max_attempts, *elapsed_ms),
        ReconnectionStatus::NotReconnecting => Ok(()), // No event needed
    }
}

/// Helper to emit capture errors as events
pub fn emit_capture_error<E: CaptureEventEmitter>(
    emitter: &E,
    error: &CaptureError,
) -> Result<(), String> {
    let ipc_error = crate::capture_error::IpcError::from(error.clone());
    emitter.emit_capture_error(
        ipc_error.code,
        ipc_error.message,
        ipc_error.details,
        ipc_error.retryable,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    struct MockEmitter {
        events: Arc<Mutex<Vec<String>>>,
    }

    impl MockEmitter {
        fn new() -> Self {
            Self {
                events: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn get_events(&self) -> Vec<String> {
            self.events.lock().unwrap().clone()
        }
    }

    impl CaptureEventEmitter for MockEmitter {
        fn emit_reconnecting(
            &self,
            attempt_number: u32,
            max_attempts: u32,
            elapsed_ms: u64,
        ) -> Result<(), String> {
            self.events.lock().unwrap().push(format!(
                "reconnecting:{}:{}:{}",
                attempt_number, max_attempts, elapsed_ms
            ));
            Ok(())
        }

        fn emit_reconnected(&self, gap_duration_ms: u64) -> Result<(), String> {
            self.events
                .lock()
                .unwrap()
                .push(format!("reconnected:{}", gap_duration_ms));
            Ok(())
        }

        fn emit_capture_error(
            &self,
            code: String,
            message: String,
            _details: Option<String>,
            retryable: bool,
        ) -> Result<(), String> {
            self.events
                .lock()
                .unwrap()
                .push(format!("error:{}:{}:{}", code, message, retryable));
            Ok(())
        }

        fn emit_session_completed(
            &self,
            session_id: String,
            lap_count: i32,
            duration_ms: u64,
            status: String,
            gap_count: usize,
        ) -> Result<(), String> {
            self.events.lock().unwrap().push(format!(
                "session_completed:{}:{}:{}:{}:{}",
                session_id, lap_count, duration_ms, status, gap_count
            ));
            Ok(())
        }
    }

    #[test]
    fn test_emit_reconnection_attempting() {
        let emitter = MockEmitter::new();
        let status = ReconnectionStatus::Attempting {
            attempt_number: 5,
            max_attempts: 30,
            elapsed_ms: 5000,
        };

        emit_reconnection_status(&emitter, &status).unwrap();

        let events = emitter.get_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], "reconnecting:5:30:5000");
    }

    #[test]
    fn test_emit_reconnection_not_reconnecting() {
        let emitter = MockEmitter::new();
        let status = ReconnectionStatus::NotReconnecting;

        emit_reconnection_status(&emitter, &status).unwrap();

        let events = emitter.get_events();
        assert_eq!(events.len(), 0); // No event should be emitted
    }

    #[test]
    fn test_emit_reconnected() {
        let emitter = MockEmitter::new();
        emitter.emit_reconnected(3500).unwrap();

        let events = emitter.get_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], "reconnected:3500");
    }

    #[test]
    fn test_emit_capture_error_stall() {
        let emitter = MockEmitter::new();
        let error = CaptureError::IrsdkStall { duration_ms: 250 };

        emit_capture_error(&emitter, &error).unwrap();

        let events = emitter.get_events();
        assert_eq!(events.len(), 1);
        assert!(events[0].contains("IRSDK_STALL"));
        assert!(events[0].contains("true")); // retryable
    }

    #[test]
    fn test_emit_capture_error_timeout() {
        let emitter = MockEmitter::new();
        let error = CaptureError::IrsdkTimeout { elapsed_s: 30 };

        emit_capture_error(&emitter, &error).unwrap();

        let events = emitter.get_events();
        assert_eq!(events.len(), 1);
        assert!(events[0].contains("IRSDK_CONNECTION_LOST"));
        assert!(events[0].contains("false")); // not retryable
    }

    #[test]
    fn test_emit_session_completed() {
        let emitter = MockEmitter::new();
        emitter
            .emit_session_completed(
                "session-123".to_string(),
                15,
                45000,
                "completed".to_string(),
                2,
            )
            .unwrap();

        let events = emitter.get_events();
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0],
            "session_completed:session-123:15:45000:completed:2"
        );
    }
}
