use arrow::array::{Array, Float64Array};
use arrow::record_batch::RecordBatch;

use super::MonotonicityViolation;

/// Threshold: lap resets are detected when distance drops to within 50m of zero
/// while the previous value was above 100m.
const LAP_RESET_THRESHOLD: f64 = 50.0;
const LAP_RESET_PREV_MIN: f64 = 100.0;

/// Violation threshold: a lap with >2% violations is flagged.
pub const MONOTONICITY_VIOLATION_THRESHOLD_PERCENT: f64 = 2.0;

/// Validate that lap_distance is monotonically increasing within each lap.
/// Returns a list of violations, excluding known lap reset events.
pub fn validate_lap_distance_monotonicity(data: &RecordBatch) -> Vec<MonotonicityViolation> {
    let lap_distance = match data
        .column_by_name("lap_distance")
        .and_then(|col| col.as_any().downcast_ref::<Float64Array>())
    {
        Some(arr) => arr,
        None => return Vec::new(),
    };

    if lap_distance.is_empty() {
        return Vec::new();
    }

    let mut violations = Vec::new();
    let mut prev_value = f64::NEG_INFINITY;
    let mut current_lap: u32 = 1;

    for i in 0..lap_distance.len() {
        if lap_distance.is_null(i) {
            continue;
        }
        let value = lap_distance.value(i);

        // Detect lap reset: distance drops near zero from a meaningful distance
        if value < LAP_RESET_THRESHOLD && prev_value > LAP_RESET_PREV_MIN {
            current_lap += 1;
            prev_value = value;
            continue;
        }

        if value < prev_value {
            violations.push(MonotonicityViolation {
                lap_number: current_lap,
                sample_index: i,
                previous_value: prev_value,
                violating_value: value,
            });
        }
        prev_value = value;
    }

    violations
}

/// Check if the violation rate exceeds the 2% threshold for any individual lap.
///
/// `lap_sample_counts` maps lap_number -> number of samples in that lap, derived from
/// the actual RecordBatch data. If a lap has no entry, its violations are checked against
/// a fallback of `total_samples / max(lap_count, 1)`.
pub fn has_excessive_violations(
    violations: &[MonotonicityViolation],
    lap_sample_counts: &std::collections::HashMap<u32, usize>,
    total_samples: usize,
) -> bool {
    if total_samples == 0 || violations.is_empty() {
        return false;
    }

    // Group violations by lap
    let mut lap_violations: std::collections::HashMap<u32, usize> = std::collections::HashMap::new();
    for v in violations {
        *lap_violations.entry(v.lap_number).or_default() += 1;
    }

    // Check if any single lap exceeds the 2% threshold
    let lap_count = lap_sample_counts.len().max(1);
    let fallback_samples_per_lap = total_samples / lap_count;

    for (&lap_number, &violation_count) in &lap_violations {
        let samples_in_lap = lap_sample_counts
            .get(&lap_number)
            .copied()
            .unwrap_or(fallback_samples_per_lap);
        if samples_in_lap == 0 {
            continue;
        }
        let violation_percent = (violation_count as f64 / samples_in_lap as f64) * 100.0;
        if violation_percent > MONOTONICITY_VIOLATION_THRESHOLD_PERCENT {
            return true;
        }
    }

    false
}

/// Compute per-lap sample counts from the RecordBatch by detecting lap boundaries
/// in the `lap_distance` channel (same logic as violation detection).
pub fn compute_lap_sample_counts(data: &RecordBatch) -> std::collections::HashMap<u32, usize> {
    let mut counts: std::collections::HashMap<u32, usize> = std::collections::HashMap::new();

    let lap_distance = match data
        .column_by_name("lap_distance")
        .and_then(|col| col.as_any().downcast_ref::<Float64Array>())
    {
        Some(arr) => arr,
        None => return counts,
    };

    if lap_distance.is_empty() {
        return counts;
    }

    let mut prev_value = f64::NEG_INFINITY;
    let mut current_lap: u32 = 1;

    for i in 0..lap_distance.len() {
        if lap_distance.is_null(i) {
            continue;
        }
        let value = lap_distance.value(i);

        // Detect lap reset (same logic as validate_lap_distance_monotonicity)
        if value < LAP_RESET_THRESHOLD && prev_value > LAP_RESET_PREV_MIN {
            current_lap += 1;
            prev_value = value;
            *counts.entry(current_lap).or_default() += 1;
            continue;
        }

        *counts.entry(current_lap).or_default() += 1;
        prev_value = value;
    }

    counts
}
