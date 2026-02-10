//! Capture-related Tauri commands

use tauri::State;
use telemetry_engine::ConnectionStatus;

use crate::state::AppState;

/// Get current capture/connection status
#[tauri::command]
pub fn get_capture_status(state: State<AppState>) -> Result<String, String> {
    let manager = state
        .connection_manager
        .lock()
        .map_err(|e| format!("Failed to lock connection manager: {}", e))?;

    let status = manager.status();

    match status {
        ConnectionStatus::Connected => Ok("connected".to_string()),
        ConnectionStatus::Disconnected => Ok("disconnected".to_string()),
    }
}
