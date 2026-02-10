//! Session lifecycle state machine.
//!
//! Manages session state transitions (idle → recording → processing → completed),
//! emits state change events, and coordinates with storage for persistence.

use crate::capture_error::{GapMarker as CaptureGapMarker, GapReason as CaptureGapReason};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use storage::types::NewSession;

// Re-export gap types from capture_error for backward compatibility
pub use crate::capture_error::{GapMarker, GapReason};

/// Session state machine states.
#[derive(Debug, Clone, PartialEq)]
pub enum SessionState {
    /// No active session - waiting for session start trigger.
    Idle,
    /// Session is actively recording telemetry.
    Recording {
        session_id: String,
        started_at: SystemTime,
    },
    /// Session recording stopped, processing final data.
    Processing { session_id: String },
    /// Session completed successfully with all data persisted.
    Completed { session_id: String },
    /// Session ended prematurely due to disconnect/crash.
    Partial {
        session_id: String,
        disconnected_at: SystemTime,
    },
}

/// Session manager handles session lifecycle and state transitions.
#[derive(Debug)]
pub struct SessionManager {
    state: SessionState,
    /// Gap markers for the current session.
    gap_markers: Vec<GapMarker>,
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionManager {
    /// Creates a new session manager in the idle state.
    pub fn new() -> Self {
        Self {
            state: SessionState::Idle,
            gap_markers: Vec::new(),
        }
    }

    /// Returns the current session state.
    pub fn current_state(&self) -> &SessionState {
        &self.state
    }

    /// Attempts to start a new session.
    ///
    /// Valid only from `Idle` state.
    /// Returns `Ok(session_id)` on success, `Err(reason)` if transition is invalid.
    pub fn start_session(&mut self, metadata: NewSession) -> Result<String, String> {
        match &self.state {
            SessionState::Idle => {
                let session_id = uuid::Uuid::new_v4().to_string();
                let started_at = SystemTime::now();

                self.state = SessionState::Recording {
                    session_id: session_id.clone(),
                    started_at,
                };

                tracing::info!(
                    session_id = %session_id,
                    track = %metadata.track_name,
                    car = %metadata.car_name,
                    "Session started"
                );

                Ok(session_id)
            }
            _ => Err(format!("Cannot start session from state {:?}", self.state)),
        }
    }

    /// Transitions session to processing state (session end detected).
    ///
    /// Valid only from `Recording` state.
    pub fn begin_processing(&mut self) -> Result<String, String> {
        match &self.state {
            SessionState::Recording { session_id, .. } => {
                let id = session_id.clone();
                self.state = SessionState::Processing {
                    session_id: id.clone(),
                };

                tracing::info!(session_id = %id, "Session processing started");
                Ok(id)
            }
            _ => Err(format!(
                "Cannot begin processing from state {:?}",
                self.state
            )),
        }
    }

    /// Marks session as completed.
    ///
    /// Valid only from `Processing` state.
    pub fn complete_session(&mut self) -> Result<String, String> {
        match &self.state {
            SessionState::Processing { session_id } => {
                let id = session_id.clone();
                self.state = SessionState::Completed {
                    session_id: id.clone(),
                };

                tracing::info!(session_id = %id, "Session completed");
                Ok(id)
            }
            _ => Err(format!(
                "Cannot complete session from state {:?}",
                self.state
            )),
        }
    }

    /// Marks session as partial due to disconnection.
    ///
    /// Valid from `Recording` or `Processing` states.
    pub fn mark_partial(&mut self) -> Result<String, String> {
        match &self.state {
            SessionState::Recording { session_id, .. }
            | SessionState::Processing { session_id } => {
                let id = session_id.clone();
                let disconnected_at = SystemTime::now();

                self.state = SessionState::Partial {
                    session_id: id.clone(),
                    disconnected_at,
                };

                tracing::warn!(session_id = %id, "Session marked as partial");
                Ok(id)
            }
            _ => Err(format!("Cannot mark partial from state {:?}", self.state)),
        }
    }

    /// Resets state machine to idle (after completion or partial).
    ///
    /// Valid from `Completed` or `Partial` states.
    pub fn reset(&mut self) -> Result<(), String> {
        match &self.state {
            SessionState::Completed { .. } | SessionState::Partial { .. } => {
                self.state = SessionState::Idle;
                self.gap_markers.clear();
                tracing::info!("Session manager reset to idle");
                Ok(())
            }
            _ => Err(format!("Cannot reset from state {:?}", self.state)),
        }
    }

    /// Adds a gap marker to the current session.
    ///
    /// Gap markers indicate telemetry stream interruptions.
    pub fn add_gap_marker(&mut self, start_time_ms: i64, end_time_ms: i64, reason: GapReason) {
        let duration_ms = (end_time_ms - start_time_ms).max(0) as u64;
        let marker = GapMarker {
            start_time: start_time_ms,
            end_time: end_time_ms,
            duration_ms,
            reason,
            lap_position: None,
        };

        tracing::warn!(
            start_ms = start_time_ms,
            end_ms = end_time_ms,
            reason = ?marker.reason,
            "Gap marker added"
        );

        self.gap_markers.push(marker);
    }

    /// Returns all gap markers for the current session.
    pub fn gap_markers(&self) -> &[GapMarker] {
        &self.gap_markers
    }

    /// Serializes gap markers to JSON string.
    ///
    /// Returns `None` if there are no gap markers.
    pub fn gap_markers_json(&self) -> Option<String> {
        if self.gap_markers.is_empty() {
            None
        } else {
            serde_json::to_string(&self.gap_markers).ok()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_metadata() -> NewSession {
        NewSession {
            track_name: "Watkins Glen".to_string(),
            car_name: "Mazda MX-5".to_string(),
            session_type: "Practice".to_string(),
            started_at: chrono::Utc::now()
                .format("%Y-%m-%dT%H:%M:%S%.3fZ")
                .to_string(),
        }
    }

    #[test]
    fn test_initial_state_is_idle() {
        let manager = SessionManager::new();
        assert_eq!(*manager.current_state(), SessionState::Idle);
    }

    #[test]
    fn test_valid_transition_idle_to_recording() {
        let mut manager = SessionManager::new();
        let metadata = create_test_metadata();

        let result = manager.start_session(metadata);
        assert!(result.is_ok());

        match manager.current_state() {
            SessionState::Recording { session_id, .. } => {
                assert!(!session_id.is_empty());
            }
            _ => panic!("Expected Recording state"),
        }
    }

    #[test]
    fn test_valid_transition_recording_to_processing() {
        let mut manager = SessionManager::new();
        manager.start_session(create_test_metadata()).unwrap();

        let result = manager.begin_processing();
        assert!(result.is_ok());

        match manager.current_state() {
            SessionState::Processing { session_id } => {
                assert!(!session_id.is_empty());
            }
            _ => panic!("Expected Processing state"),
        }
    }

    #[test]
    fn test_valid_transition_processing_to_completed() {
        let mut manager = SessionManager::new();
        manager.start_session(create_test_metadata()).unwrap();
        manager.begin_processing().unwrap();

        let result = manager.complete_session();
        assert!(result.is_ok());

        match manager.current_state() {
            SessionState::Completed { session_id } => {
                assert!(!session_id.is_empty());
            }
            _ => panic!("Expected Completed state"),
        }
    }

    #[test]
    fn test_valid_transition_recording_to_partial() {
        let mut manager = SessionManager::new();
        manager.start_session(create_test_metadata()).unwrap();

        let result = manager.mark_partial();
        assert!(result.is_ok());

        match manager.current_state() {
            SessionState::Partial {
                session_id,
                disconnected_at,
            } => {
                assert!(!session_id.is_empty());
                // Verify timestamp is recent (within 1 second)
                let elapsed = disconnected_at.elapsed().unwrap();
                assert!(elapsed.as_secs() < 1);
            }
            _ => panic!("Expected Partial state"),
        }
    }

    #[test]
    fn test_valid_transition_completed_to_idle() {
        let mut manager = SessionManager::new();
        manager.start_session(create_test_metadata()).unwrap();
        manager.begin_processing().unwrap();
        manager.complete_session().unwrap();

        let result = manager.reset();
        assert!(result.is_ok());
        assert_eq!(*manager.current_state(), SessionState::Idle);
    }

    #[test]
    fn test_valid_transition_partial_to_idle() {
        let mut manager = SessionManager::new();
        manager.start_session(create_test_metadata()).unwrap();
        manager.mark_partial().unwrap();

        let result = manager.reset();
        assert!(result.is_ok());
        assert_eq!(*manager.current_state(), SessionState::Idle);
    }

    // Invalid transition tests

    #[test]
    fn test_invalid_start_session_from_recording() {
        let mut manager = SessionManager::new();
        manager.start_session(create_test_metadata()).unwrap();

        let result = manager.start_session(create_test_metadata());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Cannot start session from state"));
    }

    #[test]
    fn test_invalid_begin_processing_from_idle() {
        let mut manager = SessionManager::new();

        let result = manager.begin_processing();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Cannot begin processing from state"));
    }

    #[test]
    fn test_invalid_complete_from_recording() {
        let mut manager = SessionManager::new();
        manager.start_session(create_test_metadata()).unwrap();

        let result = manager.complete_session();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Cannot complete session from state"));
    }

    #[test]
    fn test_invalid_reset_from_recording() {
        let mut manager = SessionManager::new();
        manager.start_session(create_test_metadata()).unwrap();

        let result = manager.reset();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Cannot reset from state"));
    }

    #[test]
    fn test_invalid_mark_partial_from_idle() {
        let mut manager = SessionManager::new();

        let result = manager.mark_partial();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Cannot mark partial from state"));
    }

    #[test]
    fn test_invalid_mark_partial_from_completed() {
        let mut manager = SessionManager::new();
        manager.start_session(create_test_metadata()).unwrap();
        manager.begin_processing().unwrap();
        manager.complete_session().unwrap();

        let result = manager.mark_partial();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Cannot mark partial from state"));
    }

    // Gap marker tests

    #[test]
    fn test_add_gap_marker() {
        let mut manager = SessionManager::new();

        manager.add_gap_marker(10000, 10500, GapReason::Disconnect);

        let markers = manager.gap_markers();
        assert_eq!(markers.len(), 1);
        assert_eq!(markers[0].start_time, 10000);
        assert_eq!(markers[0].end_time, 10500);
        assert_eq!(markers[0].duration_ms, 500);
        assert_eq!(markers[0].reason, GapReason::Disconnect);
    }

    #[test]
    fn test_multiple_gap_markers() {
        let mut manager = SessionManager::new();

        manager.add_gap_marker(10000, 10500, GapReason::Disconnect);
        manager.add_gap_marker(25000, 25200, GapReason::Reset);

        let markers = manager.gap_markers();
        assert_eq!(markers.len(), 2);
        assert_eq!(markers[0].reason, GapReason::Disconnect);
        assert_eq!(markers[1].reason, GapReason::Reset);
    }

    #[test]
    fn test_gap_markers_json_serialization() {
        let mut manager = SessionManager::new();

        manager.add_gap_marker(10000, 10500, GapReason::Disconnect);
        manager.add_gap_marker(25000, 25200, GapReason::Reset);

        let json = manager.gap_markers_json();
        assert!(json.is_some());

        let json_str = json.unwrap();
        assert!(json_str.contains("disconnect"));
        assert!(json_str.contains("reset"));
        assert!(json_str.contains("10000"));
        assert!(json_str.contains("25200"));
    }

    #[test]
    fn test_gap_markers_json_empty() {
        let manager = SessionManager::new();

        let json = manager.gap_markers_json();
        assert!(json.is_none());
    }

    #[test]
    fn test_gap_markers_cleared_on_reset() {
        let mut manager = SessionManager::new();
        manager.start_session(create_test_metadata()).unwrap();

        manager.add_gap_marker(10000, 10500, GapReason::Disconnect);
        assert_eq!(manager.gap_markers().len(), 1);

        manager.begin_processing().unwrap();
        manager.complete_session().unwrap();
        manager.reset().unwrap();

        assert_eq!(manager.gap_markers().len(), 0);
    }
}
