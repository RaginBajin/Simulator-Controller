//! Capture-related Tauri commands

use tauri::State;
use telemetry_engine::ConnectionStatus;

use crate::state::AppState;

/// Get current capture/connection status
#[tauri::command]
pub fn get_capture_status(state: State<AppState>) -> Result<String, String> {
    // Handle both poisoned locks (from thread panic) and normal lock errors
    let manager = match state.connection_manager.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            // If lock is poisoned, recover the guard but log the issue
            tracing::warn!("Connection manager lock was poisoned, recovering");
            poisoned.into_inner()
        }
    };

    let status = manager.status();

    match status {
        ConnectionStatus::Connected => Ok("connected".to_string()),
        ConnectionStatus::Disconnected => Ok("disconnected".to_string()),
    }
}
