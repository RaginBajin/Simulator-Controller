use std::collections::HashMap;

use arrow::array::{Array, Float64Array, Int32Array};
use arrow::record_batch::RecordBatch;

use super::{ChannelRange, RangeViolation};

/// Violation threshold: >5% of samples out-of-range for any channel flags the session.
pub const RANGE_VIOLATION_THRESHOLD_PERCENT: f64 = 5.0;

/// Returns the valid ranges for known telemetry channels.
pub fn channel_ranges() -> HashMap<&'static str, ChannelRange> {
    let mut ranges = HashMap::new();
    ranges.insert("brake", ChannelRange { min: 0.0, max: 1.0 });
    ranges.insert("throttle", ChannelRange { min: 0.0, max: 1.0 });
    ranges.insert("clutch", ChannelRange { min: 0.0, max: 1.0 });
    ranges.insert("speed", ChannelRange { min: 0.0, max: 120.0 });
    ranges.insert("steering", ChannelRange { min: -6.28, max: 6.28 });
    ranges.insert("rpm", ChannelRange { min: 0.0, max: 20000.0 });
    ranges.insert("lat_g", ChannelRange { min: -10.0, max: 10.0 });
    ranges.insert("long_g", ChannelRange { min: -10.0, max: 10.0 });
    ranges.insert("brake_bias", ChannelRange { min: 0.0, max: 1.0 });
    ranges.insert("track_position", ChannelRange { min: 0.0, max: 1.0 });
    ranges.insert("fuel_level", ChannelRange { min: 0.0, max: 200.0 });
    ranges.insert("oil_temp", ChannelRange { min: 0.0, max: 250.0 });
    ranges.insert("water_temp", ChannelRange { min: 0.0, max: 250.0 });
    for channel in ["tire_temp_lf", "tire_temp_rf", "tire_temp_lr", "tire_temp_rr"] {
        ranges.insert(channel, ChannelRange { min: 0.0, max: 200.0 });
    }
    for channel in [
        "tire_pressure_lf",
        "tire_pressure_rf",
        "tire_pressure_lr",
        "tire_pressure_rr",
    ] {
        ranges.insert(channel, ChannelRange { min: 50.0, max: 350.0 });
    }
    ranges
}

/// Validate all channels in a RecordBatch against their defined ranges.
/// Channels without defined ranges are checked for NaN/Infinity only.
pub fn validate_channel_ranges(data: &RecordBatch) -> HashMap<String, RangeViolation> {
    let ranges = channel_ranges();
    let mut violations = HashMap::new();
    let total_samples = data.num_rows();

    if total_samples == 0 {
        return violations;
    }

    for field in data.schema().fields() {
        let name = field.name().as_str();

        // Handle gear separately (Int32)
        if name == "gear" {
            if let Some(arr) = data
                .column_by_name(name)
                .and_then(|col| col.as_any().downcast_ref::<Int32Array>())
            {
                let mut out_of_range = 0usize;
                let mut min_seen = i32::MAX;
                let mut max_seen = i32::MIN;
                for i in 0..arr.len() {
                    if arr.is_null(i) {
                        continue;
                    }
                    let v = arr.value(i);
                    if v < min_seen {
                        min_seen = v;
                    }
                    if v > max_seen {
                        max_seen = v;
                    }
                    if v < -1 || v > 8 {
                        out_of_range += 1;
                    }
                }
                if out_of_range > 0 {
                    violations.insert(
                        name.to_string(),
                        RangeViolation {
                            channel_name: name.to_string(),
                            out_of_range_count: out_of_range,
                            total_samples,
                            min_seen: min_seen as f64,
                            max_seen: max_seen as f64,
                        },
                    );
                }
            }
            continue;
        }

        // Skip non-Float64 columns (Boolean, Int64, etc.)
        let arr = match data
            .column_by_name(name)
            .and_then(|col| col.as_any().downcast_ref::<Float64Array>())
        {
            Some(a) => a,
            None => continue,
        };

        let mut out_of_range = 0usize;
        let mut min_seen = f64::INFINITY;
        let mut max_seen = f64::NEG_INFINITY;

        if let Some(range) = ranges.get(name) {
            // Validate against defined range
            for i in 0..arr.len() {
                if arr.is_null(i) {
                    continue;
                }
                let v = arr.value(i);
                if v < min_seen {
                    min_seen = v;
                }
                if v > max_seen {
                    max_seen = v;
                }
                if v.is_nan() || v.is_infinite() || v < range.min || v > range.max {
                    out_of_range += 1;
                }
            }
        } else {
            // No defined range -- check only for NaN/Infinity
            for i in 0..arr.len() {
                if arr.is_null(i) {
                    continue;
                }
                let v = arr.value(i);
                if v < min_seen {
                    min_seen = v;
                }
                if v > max_seen {
                    max_seen = v;
                }
                if v.is_nan() || v.is_infinite() {
                    out_of_range += 1;
                }
            }
        }

        if out_of_range > 0 {
            violations.insert(
                name.to_string(),
                RangeViolation {
                    channel_name: name.to_string(),
                    out_of_range_count: out_of_range,
                    total_samples,
                    min_seen,
                    max_seen,
                },
            );
        }
    }

    violations
}

/// Check if any channel exceeds the range violation threshold.
pub fn has_excessive_range_violations(violations: &HashMap<String, RangeViolation>) -> bool {
    violations.values().any(|v| {
        if v.total_samples == 0 {
            return false;
        }
        let percent = (v.out_of_range_count as f64 / v.total_samples as f64) * 100.0;
        percent > RANGE_VIOLATION_THRESHOLD_PERCENT
    })
}
