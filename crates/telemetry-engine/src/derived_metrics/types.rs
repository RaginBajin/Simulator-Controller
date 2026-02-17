//! Type definitions for derived metrics.

use serde::{Deserialize, Serialize};

/// Metric type enumeration for derived metrics storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricType {
    BrakeCount,
    TrailBraking,
    TireDegradation,
    CornerSegmentation,
}

/// Container for all derived metrics computed for a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DerivedMetrics {
    pub session_id: String,
    pub brake_applications: Vec<BrakeApplication>,
    pub trail_braking_phases: Vec<TrailBrakingPhase>,
    pub tire_degradation: Option<TireDegradation>,
    pub corner_zones: Vec<CornerZone>,
}

/// A single brake application event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrakeApplication {
    /// Lap distance (meters) where brake input crossed threshold
    pub start_distance: f64,
    /// Maximum brake pressure during this application (0.0-1.0)
    pub peak_pressure: f64,
    /// Duration of brake application in milliseconds
    pub duration_ms: u64,
    /// Lap distance where brake was released
    pub release_distance: f64,
}

/// Trail braking phase within a brake application.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrailBrakingPhase {
    /// Duration of trail braking in milliseconds
    pub trail_duration_ms: u64,
    /// Distance covered while trail braking (meters)
    pub trail_distance_m: f64,
    /// Average brake pressure during trail braking phase
    pub trail_pressure_avg: f64,
    /// Lap distance where turn-in was detected
    pub turn_in_distance: f64,
}

/// Tire degradation metrics for a stint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TireDegradation {
    /// Left front tire temperature delta (°C)
    pub lf_temp_delta_c: f64,
    /// Right front tire temperature delta (°C)
    pub rf_temp_delta_c: f64,
    /// Left rear tire temperature delta (°C)
    pub lr_temp_delta_c: f64,
    /// Right rear tire temperature delta (°C)
    pub rr_temp_delta_c: f64,
    /// Pressure change per lap (linear regression slope)
    pub pressure_slope_per_lap: f64,
    /// Overall degradation severity classification
    pub severity: DegradationSeverity,
}

/// Tire degradation severity classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DegradationSeverity {
    /// Temperature delta < 2°C
    None,
    /// Temperature delta 2-5°C
    Mild,
    /// Temperature delta 5-10°C
    Moderate,
    /// Temperature delta > 10°C
    Severe,
}

/// A corner zone identified from telemetry patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CornerZone {
    /// Sequential corner identifier (1, 2, 3...)
    pub corner_id: u32,
    /// Corner name (T1, T2, T3...)
    pub name: String,
    /// Lap distance where corner starts (meters)
    pub start_distance: f64,
    /// Lap distance where corner ends (meters)
    pub end_distance: f64,
    /// Lap distance where braking begins (optional)
    pub brake_onset_distance: Option<f64>,
    /// Lap distance of corner apex (optional)
    pub apex_distance: Option<f64>,
}
