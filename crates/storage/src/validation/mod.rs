pub mod checksum;
pub mod lap_time;
pub mod monotonicity;
pub mod orchestrator;
pub mod range_check;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Integrity status values (priority order: worst wins).
pub const STATUS_CHECKSUM_FAILED: &str = "checksum_failed";
pub const STATUS_RANGE_VIOLATION: &str = "range_violation";
pub const STATUS_DISTANCE_ANOMALY: &str = "distance_anomaly";
pub const STATUS_VALID: &str = "valid";
pub const STATUS_NOT_VALIDATED: &str = "not_validated";

/// Maximum valid lap time: 10 minutes in milliseconds.
pub const MAX_LAP_TIME_MS: i64 = 600_000;

/// Full integrity validation report for a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrityReport {
    pub status: String,
    pub checksum_valid: Option<bool>,
    pub monotonicity_violations: Vec<MonotonicityViolation>,
    pub range_violations: HashMap<String, RangeViolation>,
    pub lap_time_violations: Vec<LapTimeViolation>,
    pub validated_at: String,
}

/// A single monotonicity violation in lap distance data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonotonicityViolation {
    pub lap_number: u32,
    pub sample_index: usize,
    pub previous_value: f64,
    pub violating_value: f64,
}

/// Summary of out-of-range values for a single channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RangeViolation {
    pub channel_name: String,
    pub out_of_range_count: usize,
    pub total_samples: usize,
    pub min_seen: f64,
    pub max_seen: f64,
}

/// A single lap time validation violation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LapTimeViolation {
    pub lap_number: i32,
    pub lap_time_ms: i64,
    pub reason: String,
}

/// Defines the valid range for a telemetry channel.
pub struct ChannelRange {
    pub min: f64,
    pub max: f64,
}
