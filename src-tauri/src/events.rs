//! Tauri event definitions

use serde::{Deserialize, Serialize};

/// Emitted when a debrief notification is clicked by the user.
/// Payload: `DebriefClickedPayload { session_id }`
pub const NOTIFICATION_DEBRIEF_CLICKED: &str = "notification:debrief-clicked";

/// Emitted to request tray badge count update.
/// Payload: `u32` (unread count)
pub const TRAY_UPDATE_BADGE: &str = "tray:update-badge";

/// Emitted when the tray state changes. Frontend listens for this to mirror state.
pub const TRAY_STATUS_CHANGED: &str = "tray:status-changed";

/// Emitted when user clicks tray icon while a debrief is ready.
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
