//! Notification service for debrief-ready OS notifications.
//!
//! Handles sending, deduplication, fullscreen suppression, and permission fallback.

use std::sync::Mutex;
use std::time::Instant;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;
use tracing::{info, warn};

use crate::events;

/// Deduplication window: suppress duplicate notifications within 5 minutes.
const DEDUP_WINDOW_SECS: u64 = 5 * 60;

/// Payload emitted when a debrief is ready for notification.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DebriefReadyPayload {
    pub session_id: String,
    pub car: String,
    pub track: String,
    pub lap_count: u32,
    pub best_lap_time: String,
}

/// Payload emitted when user clicks a debrief notification.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct DebriefClickedPayload {
    pub session_id: String,
}

/// Notification state managed via Tauri State for deduplication and queuing.
pub struct NotificationState {
    inner: Mutex<NotificationStateInner>,
}

struct NotificationStateInner {
    /// When the last notification was sent (for deduplication).
    last_notification_time: Option<Instant>,
    /// Pending notification queued while fullscreen was active.
    pending_notification: Option<DebriefReadyPayload>,
    /// Count of unread debriefs (for tray badge).
    unread_count: u32,
    /// Whether OS notification permission is available.
    notifications_enabled: bool,
}

impl NotificationState {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(NotificationStateInner {
                last_notification_time: None,
                pending_notification: None,
                unread_count: 0,
                notifications_enabled: true,
            }),
        }
    }

    /// Increment the unread debrief count and return the new count.
    pub fn increment_unread(&self) -> u32 {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        inner.unread_count += 1;
        inner.unread_count
    }

    /// Get the current unread count.
    #[cfg(test)]
    pub fn unread_count(&self) -> u32 {
        let inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        inner.unread_count
    }

    /// Reset the unread count (e.g., when user views session list).
    pub fn reset_unread(&self) -> u32 {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let prev = inner.unread_count;
        inner.unread_count = 0;
        prev
    }

    /// Check if we should send a notification (deduplication window).
    /// Returns true if enough time has passed since the last notification.
    fn should_send(&self) -> bool {
        let inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        match inner.last_notification_time {
            None => true,
            Some(last) => last.elapsed().as_secs() >= DEDUP_WINDOW_SECS,
        }
    }

    /// Record that a notification was just sent.
    fn mark_sent(&self) {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        inner.last_notification_time = Some(Instant::now());
    }

    /// Store a pending notification for when fullscreen ends.
    fn queue_pending(&self, payload: DebriefReadyPayload) {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        // Always replace with the most recent debrief (latest wins).
        inner.pending_notification = Some(payload);
    }

    /// Take any pending notification out of the queue.
    fn take_pending(&self) -> Option<DebriefReadyPayload> {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        inner.pending_notification.take()
    }

    /// Check whether notifications are enabled.
    fn is_enabled(&self) -> bool {
        let inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        inner.notifications_enabled
    }

    /// Disable notifications (e.g., permission denied).
    pub fn disable(&self) {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        inner.notifications_enabled = false;
    }

    /// Enable notifications.
    #[allow(dead_code)]
    pub fn enable(&self) {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        inner.notifications_enabled = true;
    }
}

/// Check if iRacing is running in fullscreen.
///
/// On Windows, checks for the iRacing process as the foreground window.
/// On non-Windows platforms, always returns false (never suppresses).
#[cfg(windows)]
pub fn is_iracing_fullscreen() -> bool {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowTextW};

    unsafe {
        let hwnd: HWND = GetForegroundWindow();
        if hwnd.0.is_null() {
            return false;
        }

        let mut title = [0u16; 512];
        let len = GetWindowTextW(hwnd, &mut title);
        if len == 0 {
            return false;
        }

        let title_str = OsString::from_wide(&title[..len as usize])
            .to_string_lossy()
            .to_string();

        title_str.contains("iRacing")
    }
}

#[cfg(not(windows))]
pub fn is_iracing_fullscreen() -> bool {
    false
}

/// Handle an incoming debrief-ready event: decide whether to show an OS notification,
/// queue it (if fullscreen), or just update the badge.
pub fn handle_debrief_ready(app: &AppHandle, payload: DebriefReadyPayload) {
    let state = app.state::<NotificationState>();

    // Always increment unread count and update tray badge.
    let count = state.increment_unread();
    update_tray_badge(app, count);

    // Check if notifications are enabled (permission granted).
    if !state.is_enabled() {
        info!(
            "Notifications disabled — debrief ready for session {} (badge updated to {})",
            payload.session_id, count
        );
        return;
    }

    // Check fullscreen: if racing, queue and suppress.
    if is_iracing_fullscreen() {
        info!(
            "iRacing fullscreen detected — queueing notification for session {}",
            payload.session_id
        );
        state.queue_pending(payload);
        start_fullscreen_poll(app.clone());
        return;
    }

    // Check deduplication: within 5 min window?
    if !state.should_send() {
        info!(
            "Deduplication: skipping notification for session {} (< 5 min since last)",
            payload.session_id
        );
        return;
    }

    send_notification(app, &payload);
}

/// Send the OS notification for a debrief.
fn send_notification(app: &AppHandle, payload: &DebriefReadyPayload) {
    let state = app.state::<NotificationState>();
    let body = format!(
        "{} @ {} \u{2014} {} laps, best {}",
        payload.car, payload.track, payload.lap_count, payload.best_lap_time
    );

    match app
        .notification()
        .builder()
        .title("Debrief Ready")
        .body(&body)
        .show()
    {
        Ok(_) => {
            info!("Notification sent for session {}", payload.session_id);
            state.mark_sent();
            // Note: OS notification click opens the app window by default (OS behavior).
            // Navigation to the specific debrief is handled via tray click (Story 2.2):
            // when the user clicks the tray icon while DebriefReady state is active,
            // `tray:navigate-to-debrief` is emitted with the session_id.
        }
        Err(e) => {
            warn!("Failed to send notification: {}", e);
            // If sending fails, it might be a permission issue.
            state.disable();
        }
    }
}

/// Update the system tray badge with the unread count.
///
/// This is a placeholder that emits a tray badge event.
/// Story 2.2's tray module will handle the actual tray icon update.
fn update_tray_badge(app: &AppHandle, count: u32) {
    // Emit event for tray module to pick up.
    if let Err(e) = app.emit(events::TRAY_UPDATE_BADGE, count) {
        warn!("Failed to emit tray badge update: {}", e);
    }
}

/// Poll for fullscreen exit every 5 seconds when there's a pending notification.
fn start_fullscreen_poll(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;

            if is_iracing_fullscreen() {
                continue;
            }

            // Fullscreen ended — send pending notification if any.
            let state = app.state::<NotificationState>();
            if let Some(pending) = state.take_pending() {
                info!(
                    "Fullscreen ended — sending queued notification for session {}",
                    pending.session_id
                );
                if state.should_send() {
                    send_notification(&app, &pending);
                }
            }
            break;
        }
    });
}

/// Check notification permission status.
/// Returns "enabled", "disabled", or "not-requested".
pub fn get_permission_status(app: &AppHandle) -> String {
    let state = app.state::<NotificationState>();
    if state.is_enabled() {
        "enabled".to_string()
    } else {
        "disabled".to_string()
    }
}

/// IPC command: trigger a debrief-ready notification (called from pipeline or for testing).
#[tauri::command]
pub fn notify_debrief_ready(app: AppHandle, payload: DebriefReadyPayload) {
    handle_debrief_ready(&app, payload);
}

/// IPC command: get notification permission status.
#[tauri::command]
pub fn get_notification_status(app: AppHandle) -> String {
    get_permission_status(&app)
}

/// IPC command: reset unread count (user viewed session list).
#[tauri::command]
pub fn reset_notification_count(app: AppHandle) -> u32 {
    let state = app.state::<NotificationState>();
    let prev = state.reset_unread();
    update_tray_badge(&app, 0);
    prev
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_state_unread_count() {
        let state = NotificationState::new();
        assert_eq!(state.unread_count(), 0);
        assert_eq!(state.increment_unread(), 1);
        assert_eq!(state.increment_unread(), 2);
        assert_eq!(state.unread_count(), 2);
        assert_eq!(state.reset_unread(), 2);
        assert_eq!(state.unread_count(), 0);
    }

    #[test]
    fn test_deduplication_window() {
        let state = NotificationState::new();
        // First time should always allow sending.
        assert!(state.should_send());
        state.mark_sent();
        // Immediately after should NOT allow (within 5 min window).
        assert!(!state.should_send());
    }

    #[test]
    fn test_pending_notification_queue() {
        let state = NotificationState::new();
        assert!(state.take_pending().is_none());

        let payload = DebriefReadyPayload {
            session_id: "test-1".to_string(),
            car: "McLaren 720S GT3".to_string(),
            track: "Lime Rock Park".to_string(),
            lap_count: 25,
            best_lap_time: "57.823".to_string(),
        };

        state.queue_pending(payload.clone());
        let taken = state.take_pending();
        assert!(taken.is_some());
        assert_eq!(taken.unwrap().session_id, "test-1");
        assert!(state.take_pending().is_none());
    }

    #[test]
    fn test_queue_replaces_with_latest() {
        let state = NotificationState::new();

        let payload1 = DebriefReadyPayload {
            session_id: "test-1".to_string(),
            car: "Car A".to_string(),
            track: "Track A".to_string(),
            lap_count: 10,
            best_lap_time: "60.000".to_string(),
        };
        let payload2 = DebriefReadyPayload {
            session_id: "test-2".to_string(),
            car: "Car B".to_string(),
            track: "Track B".to_string(),
            lap_count: 15,
            best_lap_time: "58.000".to_string(),
        };

        state.queue_pending(payload1);
        state.queue_pending(payload2);

        let taken = state.take_pending().unwrap();
        assert_eq!(taken.session_id, "test-2");
    }

    #[test]
    fn test_enable_disable() {
        let state = NotificationState::new();
        assert!(state.is_enabled());
        state.disable();
        assert!(!state.is_enabled());
        state.enable();
        assert!(state.is_enabled());
    }

    #[test]
    fn test_is_iracing_fullscreen_non_windows() {
        // On non-Windows (macOS/Linux), should always return false.
        assert!(!is_iracing_fullscreen());
    }
}
