//! Capture error types and gap marker models for telemetry error handling

use serde::{Deserialize, Serialize};

/// Reason for a telemetry gap
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GapReason {
    Disconnect,
    Stall,
    Reset,
    Unknown,
}

/// Marker for a gap in telemetry data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GapMarker {
    pub start_time: i64,
    pub end_time: i64,
    pub duration_ms: u64,
    pub reason: GapReason,
    pub lap_position: Option<f64>,
}

/// Capture-specific errors that can occur during telemetry collection
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
pub enum CaptureError {
    #[error("IRSDK data stalled for {duration_ms}ms")]
    IrsdkStall { duration_ms: u64 },

    #[error("IRSDK connection lost")]
    IrsdkDisconnect,

    #[error("IRSDK reconnection timeout after {elapsed_s}s")]
    IrsdkTimeout { elapsed_s: u32 },

    #[error("Storage write failed: {details}")]
    StorageError { details: String },

    #[error("Resource limit exceeded: {resource}")]
    ResourceExhausted { resource: String },
}

/// IPC error contract struct for cross-process communication
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct IpcError {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
    pub retryable: bool,
}

impl From<CaptureError> for IpcError {
    fn from(err: CaptureError) -> Self {
        match err {
            CaptureError::IrsdkStall { duration_ms } => IpcError {
                code: "IRSDK_STALL".to_string(),
                message: "Telemetry data gap detected".to_string(),
                details: Some(format!("Data stalled for {}ms", duration_ms)),
                retryable: true,
            },
            CaptureError::IrsdkDisconnect => IpcError {
                code: "IRSDK_DISCONNECT".to_string(),
                message: "Telemetry connection lost".to_string(),
                details: Some("IRSDK shared memory disconnected".to_string()),
                retryable: true,
            },
            CaptureError::IrsdkTimeout { elapsed_s } => IpcError {
                code: "IRSDK_CONNECTION_LOST".to_string(),
                message: "Telemetry connection lost".to_string(),
                details: Some(format!(
                    "No response from iRacing after {} seconds",
                    elapsed_s
                )),
                retryable: false,
            },
            CaptureError::StorageError { details } => IpcError {
                code: "STORAGE_ERROR".to_string(),
                message: "Failed to save telemetry data".to_string(),
                details: Some(details),
                retryable: false,
            },
            CaptureError::ResourceExhausted { resource } => IpcError {
                code: "RESOURCE_EXHAUSTED".to_string(),
                message: "System resource limit exceeded".to_string(),
                details: Some(format!("Resource: {}", resource)),
                retryable: false,
            },
        }
    }
}

impl GapMarker {
    /// Creates a new gap marker
    pub fn new(
        start_time: i64,
        end_time: i64,
        reason: GapReason,
        lap_position: Option<f64>,
    ) -> Self {
        let duration_ms = (end_time - start_time).max(0) as u64;
        Self {
            start_time,
            end_time,
            duration_ms,
            reason,
            lap_position,
        }
    }

    /// Returns true if this gap is significant (>500ms per NFR7)
    pub fn is_significant(&self) -> bool {
        self.duration_ms > 500
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gap_reason_serialization() {
        assert_eq!(
            serde_json::to_string(&GapReason::Disconnect).unwrap(),
            "\"disconnect\""
        );
        assert_eq!(
            serde_json::to_string(&GapReason::Stall).unwrap(),
            "\"stall\""
        );
    }

    #[test]
    fn test_gap_marker_creation() {
        let marker = GapMarker::new(1000, 1500, GapReason::Stall, Some(1234.5));

        assert_eq!(marker.start_time, 1000);
        assert_eq!(marker.end_time, 1500);
        assert_eq!(marker.duration_ms, 500);
        assert_eq!(marker.reason, GapReason::Stall);
        assert_eq!(marker.lap_position, Some(1234.5));
    }

    #[test]
    fn test_gap_marker_duration_calculation() {
        let marker = GapMarker::new(1000, 1250, GapReason::Disconnect, None);
        assert_eq!(marker.duration_ms, 250);
    }

    #[test]
    fn test_gap_marker_negative_duration_clamped() {
        // Invalid input: end before start should be clamped to 0
        let marker = GapMarker::new(2000, 1000, GapReason::Unknown, None);
        assert_eq!(marker.duration_ms, 0);
    }

    #[test]
    fn test_gap_marker_is_significant() {
        let small_gap = GapMarker::new(1000, 1400, GapReason::Stall, None);
        assert!(!small_gap.is_significant()); // 400ms < 500ms

        let large_gap = GapMarker::new(1000, 1600, GapReason::Disconnect, None);
        assert!(large_gap.is_significant()); // 600ms > 500ms

        let exact_threshold = GapMarker::new(1000, 1500, GapReason::Reset, None);
        assert!(!exact_threshold.is_significant()); // 500ms is NOT significant (>500ms required)
    }

    #[test]
    fn test_capture_error_to_ipc_error_stall() {
        let err = CaptureError::IrsdkStall { duration_ms: 250 };
        let ipc = IpcError::from(err);

        assert_eq!(ipc.code, "IRSDK_STALL");
        assert_eq!(ipc.message, "Telemetry data gap detected");
        assert!(ipc.details.unwrap().contains("250ms"));
        assert!(ipc.retryable);
    }

    #[test]
    fn test_capture_error_to_ipc_error_disconnect() {
        let err = CaptureError::IrsdkDisconnect;
        let ipc = IpcError::from(err);

        assert_eq!(ipc.code, "IRSDK_DISCONNECT");
        assert_eq!(ipc.message, "Telemetry connection lost");
        assert!(ipc.retryable);
    }

    #[test]
    fn test_capture_error_to_ipc_error_timeout() {
        let err = CaptureError::IrsdkTimeout { elapsed_s: 30 };
        let ipc = IpcError::from(err);

        assert_eq!(ipc.code, "IRSDK_CONNECTION_LOST");
        assert_eq!(ipc.message, "Telemetry connection lost");
        assert!(ipc.details.unwrap().contains("30 seconds"));
        assert!(!ipc.retryable); // Timeout is not retryable
    }

    #[test]
    fn test_capture_error_to_ipc_error_storage() {
        let err = CaptureError::StorageError {
            details: "Disk full".to_string(),
        };
        let ipc = IpcError::from(err);

        assert_eq!(ipc.code, "STORAGE_ERROR");
        assert_eq!(ipc.message, "Failed to save telemetry data");
        assert_eq!(ipc.details, Some("Disk full".to_string()));
        assert!(!ipc.retryable);
    }

    #[test]
    fn test_capture_error_to_ipc_error_resource_exhausted() {
        let err = CaptureError::ResourceExhausted {
            resource: "memory".to_string(),
        };
        let ipc = IpcError::from(err);

        assert_eq!(ipc.code, "RESOURCE_EXHAUSTED");
        assert_eq!(ipc.message, "System resource limit exceeded");
        assert!(ipc.details.unwrap().contains("memory"));
        assert!(!ipc.retryable);
    }

    #[test]
    fn test_ipc_error_serialization() {
        let ipc = IpcError {
            code: "TEST_ERROR".to_string(),
            message: "Test message".to_string(),
            details: Some("Test details".to_string()),
            retryable: true,
        };

        let json = serde_json::to_string(&ipc).unwrap();
        assert!(json.contains("\"code\":\"TEST_ERROR\""));
        assert!(json.contains("\"message\":\"Test message\""));
        assert!(json.contains("\"retryable\":true"));
    }
}
