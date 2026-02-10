//! Comprehensive integration tests for capture error handling and recovery

use telemetry_engine::{
    capture_error::{CaptureError, GapMarker, GapReason, IpcError},
    event_emitter::{emit_capture_error, emit_reconnection_status, CaptureEventEmitter},
    gap_handler::{GapDetectionConfig, GapHandler},
    irsdk::reconnection::{ConnectionState, ReconnectionManager, ReconnectionStatus},
};
use std::sync::{Arc, Mutex};

// Mock event emitter for testing
#[derive(Clone)]
struct TestEventEmitter {
    events: Arc<Mutex<Vec<String>>>,
}

impl TestEventEmitter {
    fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn get_events(&self) -> Vec<String> {
        self.events.lock().unwrap().clone()
    }

    fn clear(&self) {
        self.events.lock().unwrap().clear();
    }
}

impl CaptureEventEmitter for TestEventEmitter {
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
        self.events.lock().unwrap().push(format!(
            "error:{}:{}:{}",
            code, message, retryable
        ));
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
fn test_gap_detection_and_reporting_workflow() {
    let mut gap_handler = GapHandler::new();

    // Simulate telemetry samples
    gap_handler.check_gap(1000, Some(100.0)); // First sample
    gap_handler.check_gap(1050, Some(150.0)); // Normal gap (50ms)

    let gap = gap_handler.check_gap(1300, Some(200.0)); // Gap detected (250ms)
    assert!(gap.is_some());

    let gap = gap.unwrap();
    assert_eq!(gap.duration_ms, 250);
    assert_eq!(gap.reason, GapReason::Stall);

    assert_eq!(gap_handler.gap_count(), 1);
    assert_eq!(gap_handler.total_gap_duration_ms(), 250);
}

#[test]
fn test_reconnection_state_machine_full_cycle() {
    let mut manager = ReconnectionManager::new();
    let emitter = TestEventEmitter::new();

    // Start disconnected
    assert_eq!(manager.state(), &ConnectionState::Disconnected);

    // Connect
    manager.mark_connected();
    assert!(manager.is_connected());

    // Begin reconnection after disconnect
    manager.begin_reconnection();
    assert!(manager.is_reconnecting());

    // Tick reconnection attempts
    for attempt in 1..=5 {
        let status = manager.tick_reconnection().unwrap();
        emit_reconnection_status(&emitter, &status).unwrap();

        if let ReconnectionStatus::Attempting { attempt_number, .. } = status {
            assert_eq!(attempt_number, attempt);
        } else {
            panic!("Expected Attempting status");
        }
    }

    // Verify events were emitted
    let events = emitter.get_events();
    assert_eq!(events.len(), 5);
    assert!(events[0].contains("reconnecting:1:30:"));
    assert!(events[4].contains("reconnecting:5:30:"));

    // Successful reconnection
    let gap_duration = manager.on_reconnection_success().unwrap();
    assert!(manager.is_connected());

    emitter.emit_reconnected(gap_duration.as_millis() as u64).unwrap();
    let events = emitter.get_events();
    assert_eq!(events.len(), 6);
    assert!(events[5].contains("reconnected:"));
}

#[test]
fn test_reconnection_timeout_after_max_attempts() {
    let mut manager = ReconnectionManager::new();
    let emitter = TestEventEmitter::new();

    manager.begin_reconnection();

    // Tick through all 30 attempts
    for _ in 1..30 {
        let result = manager.tick_reconnection();
        assert!(result.is_ok());
    }

    // The 30th tick should timeout
    let result = manager.tick_reconnection();
    assert!(result.is_err());

    match result {
        Err(CaptureError::IrsdkTimeout { .. }) => {
            emit_capture_error(&emitter, &CaptureError::IrsdkTimeout { elapsed_s: 30 }).unwrap();
        }
        _ => panic!("Expected timeout error"),
    }

    assert_eq!(manager.state(), &ConnectionState::Disconnected);

    let events = emitter.get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].contains("IRSDK_CONNECTION_LOST"));
    assert!(events[0].contains("false")); // not retryable
}

#[test]
fn test_capture_error_to_ipc_error_conversion() {
    let errors = vec![
        CaptureError::IrsdkStall { duration_ms: 250 },
        CaptureError::IrsdkDisconnect,
        CaptureError::IrsdkTimeout { elapsed_s: 30 },
        CaptureError::StorageError {
            details: "Disk full".to_string(),
        },
        CaptureError::ResourceExhausted {
            resource: "memory".to_string(),
        },
    ];

    for error in errors {
        let ipc_error = IpcError::from(error.clone());

        // Verify IPC contract fields
        assert!(!ipc_error.code.is_empty());
        assert!(!ipc_error.message.is_empty());

        // Check retryable flags
        match error {
            CaptureError::IrsdkStall { .. } => assert!(ipc_error.retryable),
            CaptureError::IrsdkDisconnect => assert!(ipc_error.retryable),
            CaptureError::IrsdkTimeout { .. } => assert!(!ipc_error.retryable),
            CaptureError::StorageError { .. } => assert!(!ipc_error.retryable),
            CaptureError::ResourceExhausted { .. } => assert!(!ipc_error.retryable),
        }
    }
}

#[test]
fn test_gap_marker_significant_detection() {
    let mut gap_handler = GapHandler::new();

    gap_handler.check_gap(1000, None);
    gap_handler.check_gap(1400, None); // 400ms - not significant
    gap_handler.check_gap(1450, None);
    gap_handler.check_gap(2100, None); // 650ms - significant

    assert_eq!(gap_handler.gap_count(), 2);
    assert_eq!(gap_handler.significant_gap_count(), 1);

    let gaps = gap_handler.gaps();
    assert!(!gaps[0].is_significant()); // 400ms
    assert!(gaps[1].is_significant()); // 650ms
}

#[test]
fn test_session_completion_event_emission() {
    let emitter = TestEventEmitter::new();

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
    assert_eq!(events[0], "session_completed:session-123:15:45000:completed:2");
}

#[test]
fn test_session_completion_event_partial_session() {
    let emitter = TestEventEmitter::new();

    emitter
        .emit_session_completed(
            "session-456".to_string(),
            8,
            30000,
            "partial".to_string(),
            3,
        )
        .unwrap();

    let events = emitter.get_events();
    assert_eq!(events.len(), 1);
    assert!(events[0].contains("partial"));
    assert!(events[0].contains(":3")); // gap_count
}

#[test]
fn test_gap_handler_custom_threshold() {
    let config = GapDetectionConfig { threshold_ms: 200 };
    let mut gap_handler = GapHandler::with_config(config);

    gap_handler.check_gap(1000, None);
    let gap1 = gap_handler.check_gap(1150, None); // 150ms < 200ms
    assert!(gap1.is_none());

    let gap2 = gap_handler.check_gap(1400, None); // 250ms > 200ms
    assert!(gap2.is_some());
}

#[test]
fn test_gap_classification_by_duration() {
    let mut gap_handler = GapHandler::new();

    gap_handler.check_gap(1000, None);

    // Stall: 100-500ms
    let gap = gap_handler.check_gap(1250, None).unwrap();
    assert_eq!(gap.reason, GapReason::Stall);

    gap_handler.check_gap(1300, None);

    // Disconnect: 501-5000ms
    let gap = gap_handler.check_gap(2300, None).unwrap();
    assert_eq!(gap.reason, GapReason::Disconnect);

    gap_handler.check_gap(2350, None);

    // Unknown: >5000ms
    let gap = gap_handler.check_gap(8000, None).unwrap();
    assert_eq!(gap.reason, GapReason::Unknown);
}

#[test]
fn test_manual_gap_recording() {
    let mut gap_handler = GapHandler::new();

    let manual_gap = GapMarker::new(5000, 8000, GapReason::Disconnect, Some(999.9));
    gap_handler.record_gap(manual_gap);

    assert_eq!(gap_handler.gap_count(), 1);
    assert_eq!(gap_handler.gaps()[0].duration_ms, 3000);
}

#[test]
fn test_gap_handler_reset() {
    let mut gap_handler = GapHandler::new();

    gap_handler.check_gap(1000, None);
    gap_handler.check_gap(1300, None);
    assert_eq!(gap_handler.gap_count(), 1);

    gap_handler.reset();
    assert_eq!(gap_handler.gap_count(), 0);
    assert_eq!(gap_handler.total_gap_duration_ms(), 0);

    // Can start fresh capture
    gap_handler.check_gap(2000, None);
    let gap = gap_handler.check_gap(2250, None);
    assert!(gap.is_some());
}

#[test]
fn test_complete_error_handling_workflow() {
    // Simulates a complete capture session with gaps and reconnection
    let mut gap_handler = GapHandler::new();
    let mut reconnection_manager = ReconnectionManager::new();
    let emitter = TestEventEmitter::new();

    // Initial connection
    reconnection_manager.mark_connected();

    // Normal telemetry capture
    gap_handler.check_gap(1000, Some(100.0));
    gap_handler.check_gap(1050, Some(150.0));

    // Brief gap detected (stall)
    let gap = gap_handler.check_gap(1300, Some(200.0));
    assert!(gap.is_some());
    assert_eq!(gap_handler.gap_count(), 1);

    // Continue normal capture
    gap_handler.check_gap(1350, Some(250.0));

    // Connection lost - begin reconnection
    reconnection_manager.begin_reconnection();
    assert!(reconnection_manager.is_reconnecting());

    // Reconnection attempts
    for _ in 1..=3 {
        let status = reconnection_manager.tick_reconnection().unwrap();
        emit_reconnection_status(&emitter, &status).unwrap();
    }

    // Successful reconnection
    let gap_duration = reconnection_manager.on_reconnection_success().unwrap();
    emitter.emit_reconnected(gap_duration.as_millis() as u64).unwrap();

    // Record the disconnection gap
    let disconnect_gap = GapMarker::new(
        1350,
        1350 + gap_duration.as_millis() as i64,
        GapReason::Disconnect,
        Some(250.0),
    );
    gap_handler.record_gap(disconnect_gap);

    // Session completes
    emitter
        .emit_session_completed(
            "session-test".to_string(),
            10,
            20000,
            "completed".to_string(),
            gap_handler.gap_count(),
        )
        .unwrap();

    // Verify final state
    assert_eq!(gap_handler.gap_count(), 2); // 1 stall + 1 disconnect
    assert!(gap_handler.total_gap_duration_ms() >= 250); // At least the stall duration
    assert!(reconnection_manager.is_connected());

    let events = emitter.get_events();
    assert!(events.len() >= 5); // 3 reconnecting + 1 reconnected + 1 session_completed
}
