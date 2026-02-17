//! Integration tests for capture error handling and recovery
//!
//! These tests verify the complete error handling flow including gap detection,
//! reconnection state machine, event emission, and session status updates.

use std::sync::{Arc, Mutex};
use telemetry_engine::{
    capture_error::{CaptureError, GapMarker, GapReason},
    event_emitter::CaptureEventEmitter,
    gap_handler::{GapDetectionConfig, GapHandler},
    irsdk::reconnection::{ReconnectionManager, ReconnectionStatus, MAX_RECONNECTION_ATTEMPTS},
};

/// Mock event emitter for testing
#[derive(Clone)]
struct MockEventEmitter {
    events: Arc<Mutex<Vec<EmittedEvent>>>,
}

#[derive(Debug, Clone, PartialEq)]
enum EmittedEvent {
    Reconnecting {
        attempt: u32,
        max_attempts: u32,
        elapsed_ms: u64,
    },
    Reconnected {
        gap_duration_ms: u64,
    },
    Error {
        code: String,
        message: String,
        retryable: bool,
    },
    SessionCompleted {
        session_id: String,
        lap_count: i32,
        status: String,
        gap_count: usize,
    },
}

impl MockEventEmitter {
    fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn get_events(&self) -> Vec<EmittedEvent> {
        self.events.lock().unwrap().clone()
    }

    fn event_count(&self) -> usize {
        self.events.lock().unwrap().len()
    }
}

impl CaptureEventEmitter for MockEventEmitter {
    fn emit_reconnecting(
        &self,
        attempt_number: u32,
        max_attempts: u32,
        elapsed_ms: u64,
    ) -> Result<(), String> {
        self.events
            .lock()
            .unwrap()
            .push(EmittedEvent::Reconnecting {
                attempt: attempt_number,
                max_attempts,
                elapsed_ms,
            });
        Ok(())
    }

    fn emit_reconnected(&self, gap_duration_ms: u64) -> Result<(), String> {
        self.events
            .lock()
            .unwrap()
            .push(EmittedEvent::Reconnected { gap_duration_ms });
        Ok(())
    }

    fn emit_capture_error(
        &self,
        code: String,
        message: String,
        _details: Option<String>,
        retryable: bool,
    ) -> Result<(), String> {
        self.events.lock().unwrap().push(EmittedEvent::Error {
            code,
            message,
            retryable,
        });
        Ok(())
    }

    fn emit_session_completed(
        &self,
        session_id: String,
        lap_count: i32,
        _duration_ms: u64,
        status: String,
        gap_count: usize,
    ) -> Result<(), String> {
        self.events
            .lock()
            .unwrap()
            .push(EmittedEvent::SessionCompleted {
                session_id,
                lap_count,
                status,
                gap_count,
            });
        Ok(())
    }
}

#[test]
fn test_gap_detection_flow() {
    let mut handler = GapHandler::new();

    // First sample - no gap
    let gap1 = handler.check_gap(1000, Some(500.0));
    assert!(gap1.is_none());

    // Normal sample within threshold - no gap
    let gap2 = handler.check_gap(1016, Some(520.0)); // 16ms delta < 100ms
    assert!(gap2.is_none());

    // Sample exceeds threshold - gap detected
    let gap3 = handler.check_gap(1400, Some(800.0)); // 384ms delta > 100ms
    assert!(gap3.is_some());

    let gap = gap3.unwrap();
    assert_eq!(gap.start_time, 1016);
    assert_eq!(gap.end_time, 1400);
    assert_eq!(gap.duration_ms, 384);
    assert_eq!(gap.reason, GapReason::Stall); // 384ms -> Stall
    assert_eq!(gap.lap_position, Some(800.0));

    // Verify gap is tracked
    assert_eq!(handler.gap_count(), 1);
    assert_eq!(handler.total_gap_duration_ms(), 384);
}

#[test]
fn test_reconnection_state_machine_flow() {
    let mut manager = ReconnectionManager::new();

    // Initial state: Disconnected
    assert!(!manager.is_connected());
    assert!(!manager.is_reconnecting());

    // Connect
    manager.mark_connected();
    assert!(manager.is_connected());

    // Begin reconnection (simulates disconnection)
    manager.begin_reconnection();
    assert!(manager.is_reconnecting());
    assert!(!manager.is_connected());

    // Tick reconnection attempts
    for expected_attempt in 1..=5 {
        let result = manager.tick_reconnection();
        assert!(result.is_ok());

        if let Ok(ReconnectionStatus::Attempting { attempt_number, .. }) = result {
            assert_eq!(attempt_number, expected_attempt);
        } else {
            panic!("Expected Attempting status");
        }
    }

    // Simulate successful reconnection
    let gap_duration = manager.on_reconnection_success();
    assert!(gap_duration.is_some());
    assert!(manager.is_connected());
    assert!(!manager.is_reconnecting());
}

#[test]
fn test_reconnection_timeout_after_max_attempts() {
    let mut manager = ReconnectionManager::new();
    manager.begin_reconnection();

    // Tick until we reach max attempts
    for _ in 0..MAX_RECONNECTION_ATTEMPTS {
        let result = manager.tick_reconnection();
        if result.is_err() {
            // Should fail on the 30th attempt
            match result {
                Err(CaptureError::IrsdkTimeout { .. }) => {
                    // Expected timeout error
                    assert!(!manager.is_reconnecting());
                    assert!(!manager.is_connected());
                    return;
                }
                _ => panic!("Expected IrsdkTimeout error"),
            }
        }
    }

    panic!("Expected timeout error after max attempts");
}

#[test]
fn test_event_emission_on_reconnection_success() {
    let emitter = MockEventEmitter::new();

    // Simulate successful reconnection
    emitter.emit_reconnected(3500).unwrap();

    let events = emitter.get_events();
    assert_eq!(events.len(), 1);

    match &events[0] {
        EmittedEvent::Reconnected { gap_duration_ms } => {
            assert_eq!(*gap_duration_ms, 3500);
        }
        _ => panic!("Expected Reconnected event"),
    }
}

#[test]
fn test_event_emission_on_reconnection_attempts() {
    let emitter = MockEventEmitter::new();

    // Simulate reconnection attempts
    emitter.emit_reconnecting(1, 30, 1000).unwrap();
    emitter.emit_reconnecting(2, 30, 2000).unwrap();
    emitter.emit_reconnecting(3, 30, 3000).unwrap();

    let events = emitter.get_events();
    assert_eq!(events.len(), 3);

    for (i, event) in events.iter().enumerate() {
        match event {
            EmittedEvent::Reconnecting {
                attempt,
                max_attempts,
                elapsed_ms,
            } => {
                assert_eq!(*attempt, (i + 1) as u32);
                assert_eq!(*max_attempts, 30);
                assert_eq!(*elapsed_ms, ((i + 1) * 1000) as u64);
            }
            _ => panic!("Expected Reconnecting event"),
        }
    }
}

#[test]
fn test_event_emission_on_capture_error() {
    let emitter = MockEventEmitter::new();

    // Emit different error types
    emitter
        .emit_capture_error(
            "IRSDK_STALL".to_string(),
            "Telemetry data gap detected".to_string(),
            Some("Data stalled for 250ms".to_string()),
            true,
        )
        .unwrap();

    emitter
        .emit_capture_error(
            "IRSDK_CONNECTION_LOST".to_string(),
            "Telemetry connection lost".to_string(),
            Some("No response from iRacing after 30 seconds".to_string()),
            false,
        )
        .unwrap();

    let events = emitter.get_events();
    assert_eq!(events.len(), 2);

    match &events[0] {
        EmittedEvent::Error {
            code,
            message,
            retryable,
        } => {
            assert_eq!(code, "IRSDK_STALL");
            assert_eq!(message, "Telemetry data gap detected");
            assert!(retryable);
        }
        _ => panic!("Expected Error event"),
    }

    match &events[1] {
        EmittedEvent::Error {
            code,
            message,
            retryable,
        } => {
            assert_eq!(code, "IRSDK_CONNECTION_LOST");
            assert_eq!(message, "Telemetry connection lost");
            assert!(!retryable);
        }
        _ => panic!("Expected Error event"),
    }
}

#[test]
fn test_event_emission_on_session_completion() {
    let emitter = MockEventEmitter::new();

    // Emit session completed events for normal and partial sessions
    emitter
        .emit_session_completed(
            "session-1".to_string(),
            15,
            45000,
            "completed".to_string(),
            2,
        )
        .unwrap();

    emitter
        .emit_session_completed("session-2".to_string(), 8, 20000, "partial".to_string(), 5)
        .unwrap();

    let events = emitter.get_events();
    assert_eq!(events.len(), 2);

    match &events[0] {
        EmittedEvent::SessionCompleted {
            session_id,
            lap_count,
            status,
            gap_count,
        } => {
            assert_eq!(session_id, "session-1");
            assert_eq!(*lap_count, 15);
            assert_eq!(status, "completed");
            assert_eq!(*gap_count, 2);
        }
        _ => panic!("Expected SessionCompleted event"),
    }

    match &events[1] {
        EmittedEvent::SessionCompleted {
            session_id,
            lap_count,
            status,
            gap_count,
        } => {
            assert_eq!(session_id, "session-2");
            assert_eq!(*lap_count, 8);
            assert_eq!(status, "partial");
            assert_eq!(*gap_count, 5);
        }
        _ => panic!("Expected SessionCompleted event"),
    }
}

#[test]
fn test_complete_error_recovery_flow() {
    let mut handler = GapHandler::new();
    let mut manager = ReconnectionManager::new();
    let emitter = MockEventEmitter::new();

    // Scenario: Normal capture -> Gap detected -> Disconnection -> Reconnection attempts -> Success

    // 1. Normal capture
    manager.mark_connected();
    handler.check_gap(1000, Some(500.0));
    handler.check_gap(1016, Some(520.0));

    // 2. Gap detected (simulates brief stall)
    let gap = handler.check_gap(1400, Some(800.0));
    assert!(gap.is_some());
    assert_eq!(handler.gap_count(), 1);

    // 3. Connection lost - begin reconnection
    manager.begin_reconnection();
    assert!(manager.is_reconnecting());

    // 4. Multiple reconnection attempts with events
    for _ in 0..3 {
        let status = manager.tick_reconnection().unwrap();
        telemetry_engine::emit_reconnection_status(&emitter, &status).unwrap();
    }

    assert_eq!(emitter.event_count(), 3); // 3 reconnecting events

    // 5. Successful reconnection
    let gap_duration = manager.on_reconnection_success().unwrap();
    emitter
        .emit_reconnected(gap_duration.as_millis() as u64)
        .unwrap();

    // 6. Record the disconnection gap
    let disconnect_gap = GapMarker::new(
        1400,
        1400 + gap_duration.as_millis() as i64,
        GapReason::Disconnect,
        Some(800.0),
    );
    handler.record_gap(disconnect_gap);

    assert_eq!(handler.gap_count(), 2); // Original stall + disconnect gap
    assert!(manager.is_connected());
    assert_eq!(emitter.event_count(), 4); // 3 reconnecting + 1 reconnected
}

#[test]
fn test_partial_session_on_connection_timeout() {
    let mut manager = ReconnectionManager::new();
    let emitter = MockEventEmitter::new();

    manager.begin_reconnection();

    // Tick until timeout
    let mut timeout_occurred = false;
    for _ in 0..MAX_RECONNECTION_ATTEMPTS + 1 {
        match manager.tick_reconnection() {
            Ok(_) => {}
            Err(err) => {
                // Timeout occurred
                telemetry_engine::emit_capture_error(&emitter, &err).unwrap();
                timeout_occurred = true;
                break;
            }
        }
    }

    assert!(timeout_occurred);
    assert!(!manager.is_connected());
    assert!(!manager.is_reconnecting());

    // Verify error event was emitted
    let events = emitter.get_events();
    assert_eq!(events.len(), 1);

    match &events[0] {
        EmittedEvent::Error {
            code,
            message,
            retryable,
        } => {
            assert_eq!(code, "IRSDK_CONNECTION_LOST");
            assert_eq!(message, "Telemetry connection lost");
            assert!(!retryable);
        }
        _ => panic!("Expected Error event for timeout"),
    }

    // Emit session completed event for partial session
    emitter
        .emit_session_completed(
            "session-partial".to_string(),
            10,
            30000,
            "partial".to_string(),
            3,
        )
        .unwrap();

    let events = emitter.get_events();
    assert_eq!(events.len(), 2); // Error + SessionCompleted
}

#[test]
fn test_gap_handler_with_custom_threshold() {
    let config = GapDetectionConfig { threshold_ms: 200 };
    let mut handler = GapHandler::with_config(config);

    handler.check_gap(1000, None);

    // 150ms delta - below 200ms threshold
    let gap1 = handler.check_gap(1150, None);
    assert!(gap1.is_none());

    // 300ms delta - above 200ms threshold
    let gap2 = handler.check_gap(1450, None);
    assert!(gap2.is_some());
    assert_eq!(gap2.unwrap().duration_ms, 300);
}

#[test]
fn test_significant_gap_detection() {
    let mut handler = GapHandler::new();

    handler.check_gap(1000, None);

    // Small gap (400ms) - not significant (<500ms)
    handler.check_gap(1400, None);

    // Large gap (600ms) - significant (>500ms)
    handler.check_gap(1450, None);
    handler.check_gap(2100, None);

    assert_eq!(handler.gap_count(), 2);
    assert_eq!(handler.significant_gap_count(), 1); // Only the 600ms gap
}
