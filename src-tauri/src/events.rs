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
