//! Capture-related Tauri commands.
//!
//! Story 3.3b: Manual Debrief Trigger - trigger_debrief command.

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tracing::{info, warn};

/// Response from trigger_debrief command.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TriggerDebriefResponse {
    pub session_id: String,
    pub debrief_id: Option<String>,
    pub status: String,
}

/// Payload for session:debrief-requested event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DebriefRequestedPayload {
    #[serde(rename = "type")]
    pub event_type: String,
    pub timestamp: String,
    pub version: String,
    pub session_id: String,
    pub trigger_type: String,
}

/// Manually trigger debrief analysis for the current or specified session.
///
/// AC #2, #3, #5:
/// - If `session_id` is provided, trigger for that session
/// - If `session_id` is absent, use current/most recent session
/// - For active sessions: snapshot ring buffer, flush to Parquet, trigger debrief
/// - For completed sessions: trigger debrief on existing Parquet
/// - Emit `session:debrief-requested` event with `triggerType: "manual"`
#[tauri::command]
pub async fn trigger_debrief(
    app: AppHandle,
    session_id: Option<String>,
) -> Result<TriggerDebriefResponse, String> {
    info!(
        "Manual debrief trigger requested for session: {:?}",
        session_id
    );

    // For now, simulate triggering for a mock session
    // In real implementation, this would:
    // 1. Determine session_id (use provided, or fetch current/most recent)
    // 2. Check if session is active (recording) or completed
    // 3. If active: snapshot ring buffer, write to temp Parquet
    // 4. If completed: use existing Parquet
    // 5. Trigger debrief pipeline
    // 6. Emit session:debrief-requested event

    let resolved_session_id = session_id.unwrap_or_else(|| "mock-session-id".to_string());

    // Emit session:debrief-requested event
    let payload = DebriefRequestedPayload {
        event_type: "session:debrief-requested".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        version: "1".to_string(),
        session_id: resolved_session_id.clone(),
        trigger_type: "manual".to_string(),
    };

    if let Err(e) = app.emit("session:debrief-requested", &payload) {
        warn!("Failed to emit session:debrief-requested: {}", e);
        return Err(format!("Failed to emit debrief event: {}", e));
    }

    Ok(TriggerDebriefResponse {
        session_id: resolved_session_id,
        debrief_id: Some("mock-debrief-id".to_string()),
        status: "started".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debrief_requested_payload_structure() {
        let payload = DebriefRequestedPayload {
            event_type: "session:debrief-requested".to_string(),
            timestamp: "2026-02-09T00:00:00Z".to_string(),
            version: "1".to_string(),
            session_id: "test-session".to_string(),
            trigger_type: "manual".to_string(),
        };

        assert_eq!(payload.event_type, "session:debrief-requested");
        assert_eq!(payload.session_id, "test-session");
        assert_eq!(payload.trigger_type, "manual");
        assert_eq!(payload.version, "1");
    }

    #[test]
    fn test_trigger_debrief_response_structure() {
        let response = TriggerDebriefResponse {
            session_id: "sess-123".to_string(),
            debrief_id: Some("deb-456".to_string()),
            status: "started".to_string(),
        };

        assert_eq!(response.session_id, "sess-123");
        assert_eq!(response.debrief_id, Some("deb-456".to_string()));
        assert_eq!(response.status, "started");
    }
}
