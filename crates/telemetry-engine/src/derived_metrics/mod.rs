//! Derived metrics computation pipeline for telemetry analysis.
//!
//! This module processes raw telemetry data to compute advanced metrics:
//! - Brake application counting and analysis
//! - Trail braking phase identification
//! - Tire degradation tracking
//! - Corner segmentation
//!
//! All computations are deterministic (same input → same output) per NFR14/NFR16.
//! Performance target: <10ms per lap (NFR2).

pub mod brake_counter;
pub mod corner_segmenter;
pub mod pipeline;
pub mod tire_degradation;
pub mod trail_braking;
pub mod types;
#[cfg(test)]
mod types_test;

pub use brake_counter::{compute_brake_summary, count_brake_applications, BrakeSummary};
pub use corner_segmenter::{compute_corner_metrics, segment_corners, CornerMetrics};
pub use pipeline::compute_derived_metrics;
pub use tire_degradation::{compute_tire_degradation, segment_into_stints};
pub use trail_braking::analyze_trail_braking;
pub use types::{
    BrakeApplication, CornerZone, DegradationSeverity, DerivedMetrics, MetricType,
    TireDegradation, TrailBrakingPhase,
};
