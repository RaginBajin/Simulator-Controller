//! System tray icon management with state-driven icon/tooltip updates.
//!
//! Story 2.1: Basic tray icon setup (idle state).
//! Story 2.2: Full state management, minimize-to-tray, click-to-open, badge count.

use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::image::Image;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager};
use tracing::{info, warn};

use crate::events::TrayStatusPayload;

/// Application states reflected in the tray icon.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum TrayState {
    #[default]
    Idle,
    Recording {
        track: String,
        lap: u32,
    },
    DebriefReady {
        session_id: String,
        track: String,
        lap_count: u32,
        best_time: String,
    },
    Error {
        message: String,
    },
}

impl TrayState {
    /// Short string key used for event payload `state` field.
    #[allow(dead_code)]
    pub fn state_key(&self) -> &'static str {
        match self {
            TrayState::Idle => "idle",
            TrayState::Recording { .. } => "recording",
            TrayState::DebriefReady { .. } => "ready",
            TrayState::Error { .. } => "error",
        }
    }

    /// Tooltip text for the current state.
    #[allow(dead_code)]
    pub fn tooltip(&self) -> String {
        match self {
            TrayState::Idle => "Pitwall \u{2014} Waiting for iRacing".to_string(),
            TrayState::Recording { track, lap } => {
                format!("Recording \u{2014} {}, lap {}...", track, lap)
            }
            TrayState::DebriefReady {
                track,
                lap_count,
                best_time,
                ..
            } => {
                format!(
                    "Debrief ready \u{2014} {}, {} laps, best {}",
                    track, lap_count, best_time
                )
            }
            TrayState::Error { message } => format!("Capture error \u{2014} {}", message),
        }
    }

    /// Build the event payload for `tray:status-changed`.
    pub fn to_payload(&self) -> TrayStatusPayload {
        let now = chrono::Utc::now().to_rfc3339();
        match self {
            TrayState::Idle => TrayStatusPayload {
                event_type: "tray:status-changed".to_string(),
                timestamp: now,
                version: "1.0".to_string(),
                state: "idle".to_string(),
                details: None,
            },
            TrayState::Recording { track, lap } => TrayStatusPayload {
                event_type: "tray:status-changed".to_string(),
                timestamp: now,
                version: "1.0".to_string(),
                state: "recording".to_string(),
                details: Some(crate::events::TrayStatusDetails {
                    track: Some(track.clone()),
                    lap: Some(*lap),
                    session_id: None,
                    lap_count: None,
                    best_time: None,
                    error_message: None,
                }),
            },
            TrayState::DebriefReady {
                session_id,
                track,
                lap_count,
                best_time,
            } => TrayStatusPayload {
                event_type: "tray:status-changed".to_string(),
                timestamp: now,
                version: "1.0".to_string(),
                state: "ready".to_string(),
                details: Some(crate::events::TrayStatusDetails {
                    track: Some(track.clone()),
                    lap: None,
                    session_id: Some(session_id.clone()),
                    lap_count: Some(*lap_count),
                    best_time: Some(best_time.clone()),
                    error_message: None,
                }),
            },
            TrayState::Error { message } => TrayStatusPayload {
                event_type: "tray:status-changed".to_string(),
                timestamp: now,
                version: "1.0".to_string(),
                state: "error".to_string(),
                details: Some(crate::events::TrayStatusDetails {
                    track: None,
                    lap: None,
                    session_id: None,
                    lap_count: None,
                    best_time: None,
                    error_message: Some(message.clone()),
                }),
            },
        }
    }
}

/// Manages the system tray icon, state, and badge count.
pub struct TrayManager {
    state: Mutex<TrayState>,
    badge_count: Mutex<u32>,
}

impl TrayManager {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(TrayState::default()),
            badge_count: Mutex::new(0),
        }
    }

    /// Get a clone of the current tray state.
    pub fn current_state(&self) -> TrayState {
        self.state.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    /// Get the current badge count.
    #[allow(dead_code)]
    pub fn badge_count(&self) -> u32 {
        *self.badge_count.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Update the tray state and apply icon/tooltip changes.
    #[allow(dead_code)]
    pub fn update_state(&self, app: &AppHandle, new_state: TrayState) {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if *state == new_state {
            return;
        }
        info!(
            "Tray state: {} -> {}",
            state.state_key(),
            new_state.state_key()
        );
        *state = new_state.clone();
        drop(state);

        if let Err(e) = self.apply_icon(app, &new_state) {
            warn!("Failed to update tray icon: {}", e);
        }

        if let Err(e) = self.apply_tooltip(app, &new_state) {
            warn!("Failed to update tray tooltip: {}", e);
        }

        let payload = new_state.to_payload();
        if let Err(e) = app.emit("tray:status-changed", &payload) {
            warn!("Failed to emit tray:status-changed event: {}", e);
        }
    }

    /// Convenience: set idle state.
    #[allow(dead_code)]
    pub fn set_idle(&self, app: &AppHandle) {
        self.update_state(app, TrayState::Idle);
    }

    /// Convenience: set recording state.
    #[allow(dead_code)]
    pub fn set_recording(&self, app: &AppHandle, track: String, lap: u32) {
        self.update_state(app, TrayState::Recording { track, lap });
    }

    /// Convenience: set debrief-ready state and increment badge.
    #[allow(dead_code)]
    pub fn set_debrief_ready(
        &self,
        app: &AppHandle,
        session_id: String,
        track: String,
        lap_count: u32,
        best_time: String,
    ) {
        {
            let mut count = self.badge_count.lock().unwrap_or_else(|e| e.into_inner());
            *count = count.saturating_add(1);
        }
        self.update_state(
            app,
            TrayState::DebriefReady {
                session_id,
                track,
                lap_count,
                best_time,
            },
        );
    }

    /// Convenience: set error state.
    #[allow(dead_code)]
    pub fn set_error(&self, app: &AppHandle, message: String) {
        self.update_state(app, TrayState::Error { message });
    }

    /// Clear the debrief badge count (e.g., when user views the debrief).
    pub fn clear_badge(&self) {
        let mut count = self.badge_count.lock().unwrap_or_else(|e| e.into_inner());
        *count = 0;
    }

    /// Apply the correct icon for the given state.
    #[allow(dead_code)]
    fn apply_icon(&self, app: &AppHandle, state: &TrayState) -> Result<(), String> {
        let icon_bytes: &[u8] = match state {
            TrayState::Idle => include_bytes!("../icons/tray/idle.png"),
            TrayState::Recording { .. } => include_bytes!("../icons/tray/recording.png"),
            TrayState::DebriefReady { .. } => include_bytes!("../icons/tray/ready.png"),
            TrayState::Error { .. } => include_bytes!("../icons/tray/error.png"),
        };

        let icon =
            Image::from_bytes(icon_bytes).map_err(|e| format!("Failed to load icon: {}", e))?;

        if let Some(tray) = app.tray_by_id("pitwall-tray") {
            tray.set_icon(Some(icon))
                .map_err(|e| format!("Failed to set icon: {}", e))?;
        }
        Ok(())
    }

    /// Apply the correct tooltip for the given state.
    #[allow(dead_code)]
    fn apply_tooltip(&self, app: &AppHandle, state: &TrayState) -> Result<(), String> {
        if let Some(tray) = app.tray_by_id("pitwall-tray") {
            tray.set_tooltip(Some(&state.tooltip()))
                .map_err(|e| format!("Failed to set tooltip: {}", e))?;
        }
        Ok(())
    }
}

/// Build and register the system tray icon during app setup.
pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let idle_icon = Image::from_bytes(include_bytes!("../icons/tray/idle.png"))?;

    // Build context menu
    let open_item = MenuItem::with_id(app, "open", "Open", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open_item, &quit_item])?;

    let app_handle = app.clone();
    TrayIconBuilder::with_id("pitwall-tray")
        .icon(idle_icon)
        .tooltip("Pitwall \u{2014} Waiting for iRacing")
        .menu(&menu)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "open" => {
                show_main_window(app);
            }
            "quit" => {
                info!("Quit requested from tray menu");
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event({
            let app_handle = app_handle.clone();
            move |_tray, event| {
                if let TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } = event
                {
                    handle_tray_click(&app_handle);
                }
            }
        })
        .build(app)?;

    // Emit initial idle status
    let idle_payload = TrayState::Idle.to_payload();
    if let Err(e) = app.emit("tray:status-changed", &idle_payload) {
        warn!("Failed to emit initial tray:status-changed event: {}", e);
    }

    info!("System tray icon initialized");
    Ok(())
}

/// Handle left-click on the tray icon: show window and optionally navigate to debrief.
fn handle_tray_click(app: &AppHandle) {
    let tray_manager = app.state::<TrayManager>();
    let state = tray_manager.current_state();

    // If debrief is ready, emit navigation event and clear badge
    if let TrayState::DebriefReady { ref session_id, .. } = state {
        let session_id = session_id.clone();
        tray_manager.clear_badge();
        if let Err(e) = app.emit("tray:navigate-to-debrief", &session_id) {
            warn!("Failed to emit navigate-to-debrief: {}", e);
        }
    }

    show_main_window(app);
}

/// Show the main window and bring it to focus.
fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if let Err(e) = window.show() {
            warn!("Failed to show window: {}", e);
        }
        if let Err(e) = window.unminimize() {
            warn!("Failed to unminimize window: {}", e);
        }
        if let Err(e) = window.set_focus() {
            warn!("Failed to set focus on window: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tray_state_default_is_idle() {
        let state = TrayState::default();
        assert_eq!(state.state_key(), "idle");
    }

    #[test]
    fn test_idle_tooltip() {
        let state = TrayState::Idle;
        assert_eq!(state.tooltip(), "Pitwall \u{2014} Waiting for iRacing");
    }

    #[test]
    fn test_recording_tooltip() {
        let state = TrayState::Recording {
            track: "Spa-Francorchamps".to_string(),
            lap: 14,
        };
        assert_eq!(
            state.tooltip(),
            "Recording \u{2014} Spa-Francorchamps, lap 14..."
        );
    }

    #[test]
    fn test_debrief_ready_tooltip() {
        let state = TrayState::DebriefReady {
            session_id: "abc123".to_string(),
            track: "Lime Rock".to_string(),
            lap_count: 25,
            best_time: "57.8s".to_string(),
        };
        assert_eq!(
            state.tooltip(),
            "Debrief ready \u{2014} Lime Rock, 25 laps, best 57.8s"
        );
    }

    #[test]
    fn test_error_tooltip() {
        let state = TrayState::Error {
            message: "Connection lost".to_string(),
        };
        assert_eq!(state.tooltip(), "Capture error \u{2014} Connection lost");
    }

    #[test]
    fn test_state_keys() {
        assert_eq!(TrayState::Idle.state_key(), "idle");
        assert_eq!(
            TrayState::Recording {
                track: "t".into(),
                lap: 1
            }
            .state_key(),
            "recording"
        );
        assert_eq!(
            TrayState::DebriefReady {
                session_id: "s".into(),
                track: "t".into(),
                lap_count: 1,
                best_time: "1:00".into(),
            }
            .state_key(),
            "ready"
        );
        assert_eq!(
            TrayState::Error {
                message: "err".into()
            }
            .state_key(),
            "error"
        );
    }

    #[test]
    fn test_to_payload_idle() {
        let state = TrayState::Idle;
        let payload = state.to_payload();
        assert_eq!(payload.event_type, "tray:status-changed");
        assert_eq!(payload.version, "1.0");
        assert!(!payload.timestamp.is_empty());
        assert_eq!(payload.state, "idle");
        assert!(payload.details.is_none());
    }

    #[test]
    fn test_to_payload_recording() {
        let state = TrayState::Recording {
            track: "Monza".to_string(),
            lap: 5,
        };
        let payload = state.to_payload();
        assert_eq!(payload.state, "recording");
        let details = payload.details.unwrap();
        assert_eq!(details.track.as_deref(), Some("Monza"));
        assert_eq!(details.lap, Some(5));
    }

    #[test]
    fn test_to_payload_debrief_ready() {
        let state = TrayState::DebriefReady {
            session_id: "sess-1".to_string(),
            track: "Spa".to_string(),
            lap_count: 10,
            best_time: "2:15.000".to_string(),
        };
        let payload = state.to_payload();
        assert_eq!(payload.state, "ready");
        let details = payload.details.unwrap();
        assert_eq!(details.session_id.as_deref(), Some("sess-1"));
        assert_eq!(details.lap_count, Some(10));
    }

    #[test]
    fn test_to_payload_error() {
        let state = TrayState::Error {
            message: "timeout".to_string(),
        };
        let payload = state.to_payload();
        assert_eq!(payload.state, "error");
        let details = payload.details.unwrap();
        assert_eq!(details.error_message.as_deref(), Some("timeout"));
    }

    #[test]
    fn test_tray_manager_initial_state() {
        let mgr = TrayManager::new();
        assert_eq!(mgr.current_state(), TrayState::Idle);
        assert_eq!(mgr.badge_count(), 0);
    }

    #[test]
    fn test_tray_manager_badge_clear() {
        let mgr = TrayManager::new();
        {
            let mut count = mgr.badge_count.lock().unwrap_or_else(|e| e.into_inner());
            *count = 5;
        }
        assert_eq!(mgr.badge_count(), 5);
        mgr.clear_badge();
        assert_eq!(mgr.badge_count(), 0);
    }
}
