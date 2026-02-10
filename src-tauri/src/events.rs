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
pub const CAPTURE_RECONNECTING: &str = "capture:reconnecting";

/// Emitted when IRSDK reconnection succeeds.
pub const CAPTURE_RECONNECTED: &str = "capture:reconnected";

/// Emitted when a capture error occurs (unrecoverable).
pub const CAPTURE_ERROR: &str = "capture:error";

/// Emitted when a session completes (normal or partial).
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

impl CaptureReconnecting {
    /// Creates a new reconnecting event payload
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

