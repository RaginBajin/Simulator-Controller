//! Tauri event definitions

use serde::{Deserialize, Serialize};

/// Emitted periodically (every 60s) with resource usage statistics during active capture.
/// Payload: `ResourceStatsPayload`
#[allow(dead_code)]
pub const CAPTURE_RESOURCE_STATS: &str = "capture:resource-stats";

/// Emitted when resource usage exceeds configured thresholds.
/// Payload: `ResourceWarningPayload`
#[allow(dead_code)]
pub const CAPTURE_RESOURCE_WARNING: &str = "capture:resource-warning";

/// Emitted when session pre-processing completes and debrief is ready.
/// Payload: `DebriefReadyPayload`
#[allow(dead_code)]
pub const DEBRIEF_READY: &str = "debrief:ready";

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

/// Payload for `capture:resource-stats` events (AC: Story 3.5, Task 4).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct ResourceStatsPayload {
    /// Event type identifier ("capture:resource-stats").
    #[serde(rename = "type")]
    pub event_type: String,
    /// ISO-8601 timestamp of when the event was emitted.
    pub timestamp: String,
    /// Event schema version (semver string, e.g. "1.0").
    pub version: String,
    /// CPU usage as percentage (0-100).
    pub cpu_percent: f64,
    /// Resident Set Size (RSS) memory in megabytes.
    pub rss_mb: u64,
    /// Session ID associated with this measurement.
    pub session_id: String,
    /// Uptime in seconds since monitoring started.
    pub uptime_seconds: u64,
}

/// Payload for `capture:resource-warning` events (AC: Story 3.5, Task 2).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct ResourceWarningPayload {
    /// Event type identifier ("capture:resource-warning").
    #[serde(rename = "type")]
    pub event_type: String,
    /// ISO-8601 timestamp of when the event was emitted.
    pub timestamp: String,
    /// Event schema version (semver string, e.g. "1.0").
    pub version: String,
    /// Metric that breached threshold ("cpu" or "rss").
    pub metric: String,
    /// Current value of the metric.
    pub current: f64,
    /// Threshold value that was exceeded.
    pub threshold: f64,
    /// Session ID associated with the breach.
    pub session_id: String,
}

/// Payload for `debrief:ready` events (AC: Story 3.5, Task 5).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct DebriefReadyPayload {
    /// Event type identifier ("debrief:ready").
    #[serde(rename = "type")]
    pub event_type: String,
    /// ISO-8601 timestamp of when the event was emitted.
    pub timestamp: String,
    /// Event schema version (semver string, e.g. "1.0").
    pub version: String,
    /// Session ID of the completed session.
    pub session_id: String,
    /// Track name where the session took place.
    pub track_name: String,
    /// Car name used during the session.
    pub car_name: String,
    /// Total number of laps completed.
    pub lap_count: u32,
    /// Best lap time in milliseconds.
    pub best_lap_time_ms: Option<u32>,
}
