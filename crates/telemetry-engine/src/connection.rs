//! Connection manager for IRSDK polling and state management

use crate::irsdk::{ConnectionEvent, ConnectionStatus, IrsdkReader};
use chrono::Utc;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::{self, Sender},
    Arc, Mutex,
};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use tracing::{info, warn};

/// Connection manager for IRSDK
///
/// Manages automatic connection detection and polling for IRSDK shared memory.
/// Implements a state machine that transitions between Connected/Disconnected states
/// and broadcasts state changes via a channel.
pub struct ConnectionManager {
    /// Current connection status
    status: Arc<Mutex<ConnectionStatus>>,
    /// Channel sender for broadcasting connection events
    event_tx: Option<Sender<ConnectionEvent>>,
    /// Polling thread handle
    poll_thread: Option<JoinHandle<()>>,
    /// Flag to stop polling thread
    stop_flag: Arc<AtomicBool>,
}

impl ConnectionManager {
    /// Create a new connection manager
    pub fn new() -> Self {
        Self {
            status: Arc::new(Mutex::new(ConnectionStatus::Disconnected)),
            event_tx: None,
            poll_thread: None,
            stop_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Start the polling loop
    ///
    /// Spawns a background thread that polls for IRSDK connection every 2 seconds.
    /// State changes are broadcast via the returned receiver channel.
    ///
    /// Returns a receiver channel for connection events.
    pub fn start(&mut self) -> mpsc::Receiver<ConnectionEvent> {
        // Create channel for broadcasting events
        let (tx, rx) = mpsc::channel();
        self.event_tx = Some(tx.clone());

        // Reset stop flag
        self.stop_flag.store(false, Ordering::SeqCst);

        // Clone Arc references for thread
        let status = Arc::clone(&self.status);
        let stop_flag = Arc::clone(&self.stop_flag);

        // Spawn polling thread
        let handle = thread::spawn(move || {
            let mut reader = IrsdkReader::new();
            let mut last_status = ConnectionStatus::Disconnected;

            info!("IRSDK polling loop started (2-second interval)");

            while !stop_flag.load(Ordering::SeqCst) {
                // Check connection status
                let is_connected = reader.is_connected();
                let new_status = if is_connected {
                    ConnectionStatus::Connected
                } else {
                    ConnectionStatus::Disconnected
                };

                // Detect state change
                if new_status != last_status {
                    // Update shared status
                    {
                        let mut status_guard = status.lock().unwrap();
                        *status_guard = new_status;
                    }

                    // Log state change
                    match new_status {
                        ConnectionStatus::Connected => {
                            info!("IRSDK connected");
                        }
                        ConnectionStatus::Disconnected => {
                            info!("IRSDK disconnected");
                        }
                    }

                    // Broadcast event
                    let event = ConnectionEvent {
                        status: new_status,
                        timestamp: Utc::now(),
                    };

                    if let Err(e) = tx.send(event) {
                        warn!("Failed to send connection event: {}", e);
                        break;
                    }

                    last_status = new_status;
                }

                // Sleep for 2 seconds
                thread::sleep(Duration::from_secs(2));
            }

            info!("IRSDK polling loop stopped");
        });

        self.poll_thread = Some(handle);
        rx
    }

    /// Stop the polling loop
    ///
    /// Gracefully shuts down the polling thread.
    pub fn stop(&mut self) {
        info!("Stopping connection manager");
        self.stop_flag.store(true, Ordering::SeqCst);

        if let Some(handle) = self.poll_thread.take() {
            if let Err(e) = handle.join() {
                warn!("Error joining poll thread: {:?}", e);
            }
        }

        self.event_tx = None;
    }

    /// Get current connection status
    pub fn status(&self) -> ConnectionStatus {
        *self.status.lock().unwrap()
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ConnectionManager {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_connection_manager_creation() {
        let manager = ConnectionManager::new();
        assert_eq!(manager.status(), ConnectionStatus::Disconnected);
    }

    #[test]
    fn test_connection_manager_start_stop() {
        let mut manager = ConnectionManager::new();

        // Start polling
        let _rx = manager.start();

        // Should still be able to query status
        let _status = manager.status();

        // Stop polling
        manager.stop();

        // Should be able to stop again without error
        manager.stop();
    }

    #[test]
    fn test_state_machine_transitions() {
        // Test that the manager can track state transitions
        let mut manager = ConnectionManager::new();

        // Initial state should be disconnected
        assert_eq!(manager.status(), ConnectionStatus::Disconnected);

        // Start polling
        let _rx = manager.start();

        // Give it a moment to poll (on non-Windows, will stay disconnected)
        thread::sleep(Duration::from_millis(100));

        // Should still be disconnected on non-Windows
        #[cfg(not(target_os = "windows"))]
        assert_eq!(manager.status(), ConnectionStatus::Disconnected);

        manager.stop();
    }

    #[test]
    fn test_event_channel() {
        let mut manager = ConnectionManager::new();
        let rx = manager.start();

        // On non-Windows, we won't get any connection events
        // Just verify the channel is working
        #[cfg(not(target_os = "windows"))]
        {
            thread::sleep(Duration::from_millis(100));
            // Channel should be open but empty
            assert!(rx.try_recv().is_err());
        }

        manager.stop();
    }
}
