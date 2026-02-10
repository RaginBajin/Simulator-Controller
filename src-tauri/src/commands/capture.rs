//! Capture-related Tauri commands.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

/// Response for get_capture_status command.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureStatus {
    /// Current session state ("idle", "recording", "processing", "completed", "partial").
    pub state: String,
    /// Active session ID (if in recording/processing/completed/partial state).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Session start timestamp (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    /// Is IRSDK currently connected.
    pub irsdk_connected: bool,
}

/// Gets the current capture status.
///
/// Returns information about the active session state and IRSDK connection status.
#[tauri::command]
pub async fn get_capture_status(_state: State<'_, AppState>) -> Result<CaptureStatus, AppError> {
    // TODO: Once SessionManager is integrated into AppState, read actual state
    // For now, return default idle state
    Ok(CaptureStatus {
        state: "idle".to_string(),
        session_id: None,
        started_at: None,
        irsdk_connected: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_status_serialization() {
        let status = CaptureStatus {
            state: "recording".to_string(),
            session_id: Some("test-123".to_string()),
            started_at: Some("2026-02-09T19:00:00.000Z".to_string()),
            irsdk_connected: true,
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("recording"));
        assert!(json.contains("test-123"));
        assert!(json.contains("irsdk"));
    }

    #[test]
    fn test_capture_status_idle_serialization() {
        let status = CaptureStatus {
            state: "idle".to_string(),
            session_id: None,
            started_at: None,
            irsdk_connected: false,
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("idle"));
        assert!(!json.contains("sessionId")); // Should be omitted when None
    }
}
