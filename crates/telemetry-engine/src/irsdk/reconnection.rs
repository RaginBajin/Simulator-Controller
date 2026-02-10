//! IRSDK reconnection state machine and logic

use crate::capture_error::CaptureError;
use std::time::{Duration, Instant};

/// Maximum reconnection attempts (30 seconds at 1 attempt/second)
pub const MAX_RECONNECTION_ATTEMPTS: u32 = 30;

/// Delay between reconnection attempts
pub const RECONNECTION_INTERVAL: Duration = Duration::from_secs(1);

/// Connection state for the IRSDK connection manager
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    /// Successfully connected to IRSDK
    Connected,

    /// Attempting to reconnect after a disconnection
    Reconnecting { attempt: u32, started_at: Instant },

    /// Disconnected and not attempting to reconnect
    Disconnected,
}

/// Manages IRSDK connection lifecycle and reconnection logic
#[derive(Debug)]
pub struct ReconnectionManager {
    state: ConnectionState,
}

impl ReconnectionManager {
    /// Creates a new reconnection manager in disconnected state
    pub fn new() -> Self {
        Self {
            state: ConnectionState::Disconnected,
        }
    }

    /// Returns the current connection state
    pub fn state(&self) -> &ConnectionState {
        &self.state
    }

    /// Transitions to connected state
    pub fn mark_connected(&mut self) {
        self.state = ConnectionState::Connected;
    }

    /// Begins reconnection attempts, transitioning to Reconnecting state
    pub fn begin_reconnection(&mut self) {
        self.state = ConnectionState::Reconnecting {
            attempt: 1,
            started_at: Instant::now(),
        };
    }

    /// Attempts the next reconnection.
    /// Returns Ok(()) if should attempt connection, Err(CaptureError) if timeout exceeded.
    pub fn tick_reconnection(&mut self) -> Result<ReconnectionStatus, CaptureError> {
        match &self.state {
            ConnectionState::Reconnecting {
                attempt,
                started_at,
            } => {
                let elapsed = started_at.elapsed();
                let attempt = *attempt;

                if attempt >= MAX_RECONNECTION_ATTEMPTS {
                    self.state = ConnectionState::Disconnected;
                    return Err(CaptureError::IrsdkTimeout {
                        elapsed_s: elapsed.as_secs() as u32,
                    });
                }

                // Increment attempt for next tick
                self.state = ConnectionState::Reconnecting {
                    attempt: attempt + 1,
                    started_at: *started_at,
                };

                Ok(ReconnectionStatus::Attempting {
                    attempt_number: attempt,
                    max_attempts: MAX_RECONNECTION_ATTEMPTS,
                    elapsed_ms: elapsed.as_millis() as u64,
                })
            }
            _ => {
                // Not in reconnecting state, should not tick
                Ok(ReconnectionStatus::NotReconnecting)
            }
        }
    }

    /// Called when reconnection succeeds, transitions to Connected
    pub fn on_reconnection_success(&mut self) -> Option<Duration> {
        if let ConnectionState::Reconnecting { started_at, .. } = &self.state {
            let gap_duration = started_at.elapsed();
            self.state = ConnectionState::Connected;
            Some(gap_duration)
        } else {
            None
        }
    }

    /// Marks the connection as disconnected (gives up on reconnection)
    pub fn mark_disconnected(&mut self) {
        self.state = ConnectionState::Disconnected;
    }

    /// Returns true if currently attempting to reconnect
    pub fn is_reconnecting(&self) -> bool {
        matches!(self.state, ConnectionState::Reconnecting { .. })
    }

    /// Returns true if connected
    pub fn is_connected(&self) -> bool {
        matches!(self.state, ConnectionState::Connected)
    }
}

/// Status information from a reconnection tick
#[derive(Debug, Clone, PartialEq)]
pub enum ReconnectionStatus {
    /// Currently attempting reconnection
    Attempting {
        attempt_number: u32,
        max_attempts: u32,
        elapsed_ms: u64,
    },

    /// Not in reconnecting state
    NotReconnecting,
}

impl Default for ReconnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_new_manager_is_disconnected() {
        let manager = ReconnectionManager::new();
        assert_eq!(manager.state(), &ConnectionState::Disconnected);
        assert!(!manager.is_connected());
        assert!(!manager.is_reconnecting());
    }

    #[test]
    fn test_mark_connected() {
        let mut manager = ReconnectionManager::new();
        manager.mark_connected();

        assert_eq!(manager.state(), &ConnectionState::Connected);
        assert!(manager.is_connected());
        assert!(!manager.is_reconnecting());
    }

    #[test]
    fn test_begin_reconnection() {
        let mut manager = ReconnectionManager::new();
        manager.begin_reconnection();

        assert!(manager.is_reconnecting());
        assert!(!manager.is_connected());

        if let ConnectionState::Reconnecting { attempt, .. } = manager.state() {
            assert_eq!(*attempt, 1);
        } else {
            panic!("Expected Reconnecting state");
        }
    }

    #[test]
    fn test_tick_reconnection_increments_attempt() {
        let mut manager = ReconnectionManager::new();
        manager.begin_reconnection();

        let result = manager.tick_reconnection();
        assert!(result.is_ok());

        if let Ok(ReconnectionStatus::Attempting { attempt_number, .. }) = result {
            assert_eq!(attempt_number, 1);
        } else {
            panic!("Expected Attempting status");
        }

        // After tick, attempt should be incremented
        if let ConnectionState::Reconnecting { attempt, .. } = manager.state() {
            assert_eq!(*attempt, 2);
        } else {
            panic!("Expected Reconnecting state");
        }
    }

    #[test]
    fn test_tick_reconnection_max_attempts_timeout() {
        let mut manager = ReconnectionManager::new();
        manager.state = ConnectionState::Reconnecting {
            attempt: MAX_RECONNECTION_ATTEMPTS,
            started_at: Instant::now(),
        };

        let result = manager.tick_reconnection();
        assert!(result.is_err());

        match result {
            Err(CaptureError::IrsdkTimeout { elapsed_s: _ }) => {
                // Successfully got timeout error
            }
            _ => panic!("Expected IrsdkTimeout error"),
        }

        // Should transition to Disconnected
        assert_eq!(manager.state(), &ConnectionState::Disconnected);
    }

    #[test]
    fn test_tick_when_not_reconnecting() {
        let mut manager = ReconnectionManager::new();
        manager.mark_connected();

        let result = manager.tick_reconnection();
        assert!(matches!(result, Ok(ReconnectionStatus::NotReconnecting)));
    }

    #[test]
    fn test_on_reconnection_success() {
        let mut manager = ReconnectionManager::new();
        manager.begin_reconnection();

        // Sleep briefly to ensure gap duration > 0
        sleep(Duration::from_millis(10));

        let gap_duration = manager.on_reconnection_success();
        assert!(gap_duration.is_some());
        assert!(gap_duration.unwrap().as_millis() >= 10);

        // Should be connected now
        assert!(manager.is_connected());
        assert_eq!(manager.state(), &ConnectionState::Connected);
    }

    #[test]
    fn test_on_reconnection_success_when_not_reconnecting() {
        let mut manager = ReconnectionManager::new();
        manager.mark_connected();

        let gap_duration = manager.on_reconnection_success();
        assert!(gap_duration.is_none());
    }

    #[test]
    fn test_mark_disconnected() {
        let mut manager = ReconnectionManager::new();
        manager.begin_reconnection();

        manager.mark_disconnected();
        assert_eq!(manager.state(), &ConnectionState::Disconnected);
        assert!(!manager.is_reconnecting());
        assert!(!manager.is_connected());
    }

    #[test]
    fn test_reconnection_status_fields() {
        let mut manager = ReconnectionManager::new();
        manager.begin_reconnection();

        let result = manager.tick_reconnection();
        if let Ok(ReconnectionStatus::Attempting {
            attempt_number,
            max_attempts,
            elapsed_ms: _,
        }) = result
        {
            assert_eq!(attempt_number, 1);
            assert_eq!(max_attempts, MAX_RECONNECTION_ATTEMPTS);
        } else {
            panic!("Expected Attempting status");
        }
    }

    #[test]
    fn test_multiple_reconnection_attempts() {
        let mut manager = ReconnectionManager::new();
        manager.begin_reconnection();

        for expected_attempt in 1..=5 {
            let result = manager.tick_reconnection();

            if let Ok(ReconnectionStatus::Attempting { attempt_number, .. }) = result {
                assert_eq!(attempt_number, expected_attempt);
            } else {
                panic!("Expected Attempting status on attempt {}", expected_attempt);
            }
        }

        // After 5 ticks, attempt should be 6
        if let ConnectionState::Reconnecting { attempt, .. } = manager.state() {
            assert_eq!(*attempt, 6);
        } else {
            panic!("Expected Reconnecting state");
        }
    }
}
