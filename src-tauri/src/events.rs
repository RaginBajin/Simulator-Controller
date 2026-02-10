//! Tauri event definitions

use serde::{Deserialize, Serialize};

/// Emitted when a debrief notification is clicked by the user.
/// Payload: `DebriefClickedPayload { session_id }`
#[allow(dead_code)]
pub const NOTIFICATION_DEBRIEF_CLICKED: &str = "notification:debrief-clicked";

/// Emitted to request tray badge count update.
/// Payload: `u32` (unread count)
pub const TRAY_UPDATE_BADGE: &str = "tray:update-badge";

/// Emitted when the tray state changes. Frontend listens for this to mirror state.
#[allow(dead_code)]
pub const TRAY_STATUS_CHANGED: &str = "tray:status-changed";

/// Emitted when user clicks tray icon while a debrief is ready.
#[allow(dead_code)]
pub const TRAY_NAVIGATE_TO_DEBRIEF: &str = "tray:navigate-to-debrief";

/// Emitted when IRSDK is attempting to reconnect after a disconnection.
#[allow(dead_code)] // Will be used in Story 3.5 for background capture monitoring
pub const CAPTURE_RECONNECTING: &str = "capture:reconnecting";

/// Emitted when IRSDK reconnection succeeds.
#[allow(dead_code)] // Will be used in Story 3.5 for background capture monitoring
pub const CAPTURE_RECONNECTED: &str = "capture:reconnected";

/// Emitted when a capture error occurs (unrecoverable).
#[allow(dead_code)] // Will be used in Story 3.5 for background capture monitoring
pub const CAPTURE_ERROR: &str = "capture:error";

/// Emitted when a session completes (normal or partial).
#[allow(dead_code)] // Will be used in Story 3.5 for background capture monitoring
pub const SESSION_COMPLETED: &str = "session:completed";

/// Payload for `tray:status-changed` events.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrayStatusPayload {
    /// Event type identifier (e.g., "tray:status-changed").
    #[serde(rename = "type")]
    pub event_type: String,
    /// ISO-8601 timestamp of when the event was emitted.
    pub timestamp: String,
    /// Event schema version (semver string, e.g. "1.0").
    pub version: String,
    pub state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<TrayStatusDetails>,
}

/// Optional details accompanying a tray status change.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrayStatusDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lap: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lap_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub best_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

/// Payload for `capture:reconnecting` events.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)] // Will be used in Story 3.5 for background capture monitoring
pub struct CaptureReconnecting {
    #[serde(rename = "type")]
    pub event_type: String,
    pub timestamp: String,
    pub version: String,
    pub attempt_number: u32,
    pub max_attempts: u32,
    pub elapsed_ms: u64,
}

/// Payload for `capture:reconnected` events.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)] // Will be used in Story 3.5 for background capture monitoring
pub struct CaptureReconnected {
    #[serde(rename = "type")]
    pub event_type: String,
    pub timestamp: String,
    pub version: String,
    pub gap_duration_ms: u64,
}

/// Payload for `capture:error` events.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)] // Will be used in Story 3.5 for background capture monitoring
pub struct CaptureError {
    #[serde(rename = "type")]
    pub event_type: String,
    pub timestamp: String,
    pub version: String,
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    pub retryable: bool,
}

/// Payload for `session:completed` events.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)] // Will be used in Story 3.5 for background capture monitoring
pub struct SessionCompleted {
    #[serde(rename = "type")]
    pub event_type: String,
    pub timestamp: String,
    pub version: String,
    pub session_id: String,
    pub lap_count: i32,
    pub duration_ms: u64,
    pub status: String,
    pub gap_count: usize,
}

// Session lifecycle events (Story 3.3)
// NOTE: These event definitions are not yet integrated (pending Stories 3.1/3.2 merge).
// They are marked with #[allow(dead_code)] to pass clippy checks until integration is complete.

/// Emitted when a telemetry capture session starts.
/// Payload: `SessionCaptureEvent { type, timestamp, version, sessionId }`
#[allow(dead_code)]
pub const SESSION_CAPTURE_STARTED: &str = "session:capture-started";

/// Emitted when a telemetry capture session stops (normal end).
/// Payload: `SessionCaptureEvent { type, timestamp, version, sessionId }`
#[allow(dead_code)]
pub const SESSION_CAPTURE_STOPPED: &str = "session:capture-stopped";

/// Emitted when a lap is completed during a session.
/// Payload: `LapCompletedEvent { type, timestamp, version, sessionId, lapNumber, lapTimeMs }`
#[allow(dead_code)]
pub const SESSION_LAP_COMPLETED: &str = "session:lap-completed";

/// Emitted when session state changes in the state machine.
/// Payload: `SessionStateChangedEvent { type, timestamp, version, state, sessionId? }`
#[allow(dead_code)]
pub const SESSION_STATE_CHANGED: &str = "session:state-changed";

/// Emitted when IRSDK connection is lost (from Story 3.1).
/// Payload: `IrsdkDisconnectedEvent { type, timestamp, version }`
#[allow(dead_code)]
pub const CAPTURE_IRSDK_DISCONNECTED: &str = "capture:irsdk-disconnected";

/// Payload for session capture start/stop events.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionCaptureEvent {
    /// Event type identifier (e.g., "capture-started").
    #[serde(rename = "type")]
    pub event_type: String,
    /// ISO-8601 timestamp of when the event was emitted.
    pub timestamp: String,
    /// Event schema version (e.g., "1").
    pub version: i32,
    /// Session ID.
    pub session_id: String,
}

/// Payload for lap completed events.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LapCompletedEvent {
    /// Event type identifier ("lap-completed").
    #[serde(rename = "type")]
    pub event_type: String,
    /// ISO-8601 timestamp.
    pub timestamp: String,
    /// Event schema version.
    pub version: i32,
    /// Session ID.
    pub session_id: String,
    /// Lap number that was completed.
    pub lap_number: i32,
    /// Lap time in milliseconds.
    pub lap_time_ms: i64,
}

/// Payload for session state changed events.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionStateChangedEvent {
    /// Event type identifier ("state-changed").
    #[serde(rename = "type")]
    pub event_type: String,
    /// ISO-8601 timestamp.
    pub timestamp: String,
    /// Event schema version.
    pub version: i32,
    /// New state ("idle", "recording", "processing", "completed", "partial").
    pub state: String,
    /// Session ID (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

impl CaptureReconnecting {
    /// Creates a new reconnecting event payload
    #[allow(dead_code)] // Will be used in Story 3.5 for background capture monitoring
    pub fn new(attempt_number: u32, max_attempts: u32, elapsed_ms: u64) -> Self {
        Self {
            event_type: CAPTURE_RECONNECTING.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            version: "1.0".to_string(),
            attempt_number,
            max_attempts,
            elapsed_ms,
        }
    }
}

impl CaptureReconnected {
    /// Creates a new reconnected event payload
    #[allow(dead_code)] // Will be used in Story 3.5 for background capture monitoring
    pub fn new(gap_duration_ms: u64) -> Self {
        Self {
            event_type: CAPTURE_RECONNECTED.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            version: "1.0".to_string(),
            gap_duration_ms,
        }
    }
}

impl CaptureError {
    /// Creates a new capture error event payload
    #[allow(dead_code)] // Will be used in Story 3.5 for background capture monitoring
    pub fn new(code: String, message: String, details: Option<String>, retryable: bool) -> Self {
        Self {
            event_type: CAPTURE_ERROR.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            version: "1.0".to_string(),
            code,
            message,
            details,
            retryable,
        }
    }
}

impl SessionCompleted {
    /// Creates a new session completed event payload
    #[allow(dead_code)] // Will be used in Story 3.5 for background capture monitoring
    pub fn new(
        session_id: String,
        lap_count: i32,
        duration_ms: u64,
        status: String,
        gap_count: usize,
    ) -> Self {
        Self {
            event_type: SESSION_COMPLETED.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            version: "1.0".to_string(),
            session_id,
            lap_count,
            duration_ms,
            status,
            gap_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_reconnecting_creation() {
        let event = CaptureReconnecting::new(5, 30, 5000);

        assert_eq!(event.event_type, CAPTURE_RECONNECTING);
        assert_eq!(event.version, "1.0");
        assert_eq!(event.attempt_number, 5);
        assert_eq!(event.max_attempts, 30);
        assert_eq!(event.elapsed_ms, 5000);
        assert!(!event.timestamp.is_empty());
    }

    #[test]
    fn test_capture_reconnected_creation() {
        let event = CaptureReconnected::new(3500);

        assert_eq!(event.event_type, CAPTURE_RECONNECTED);
        assert_eq!(event.version, "1.0");
        assert_eq!(event.gap_duration_ms, 3500);
        assert!(!event.timestamp.is_empty());
    }

    #[test]
    fn test_capture_error_creation() {
        let event = CaptureError::new(
            "IRSDK_CONNECTION_LOST".to_string(),
            "Telemetry connection lost".to_string(),
            Some("No response after 30s".to_string()),
            false,
        );

        assert_eq!(event.event_type, CAPTURE_ERROR);
        assert_eq!(event.version, "1.0");
        assert_eq!(event.code, "IRSDK_CONNECTION_LOST");
        assert_eq!(event.message, "Telemetry connection lost");
        assert_eq!(event.details, Some("No response after 30s".to_string()));
        assert!(!event.retryable);
        assert!(!event.timestamp.is_empty());
    }

    #[test]
    fn test_session_completed_creation() {
        let event = SessionCompleted::new(
            "session-123".to_string(),
            15,
            45000,
            "completed".to_string(),
            2,
        );

        assert_eq!(event.event_type, SESSION_COMPLETED);
        assert_eq!(event.version, "1.0");
        assert_eq!(event.session_id, "session-123");
        assert_eq!(event.lap_count, 15);
        assert_eq!(event.duration_ms, 45000);
        assert_eq!(event.status, "completed");
        assert_eq!(event.gap_count, 2);
        assert!(!event.timestamp.is_empty());
    }

    #[test]
    fn test_event_serialization() {
        let event = CaptureReconnecting::new(1, 30, 1000);
        let json = serde_json::to_string(&event).unwrap();

        assert!(json.contains("\"type\":\"capture:reconnecting\""));
        assert!(json.contains("\"attemptNumber\":1"));
        assert!(json.contains("\"maxAttempts\":30"));
        assert!(json.contains("\"version\":\"1.0\""));
    }
}
