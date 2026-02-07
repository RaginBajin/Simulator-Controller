use crate::types::LapSummary;
use super::{LapTimeViolation, MAX_LAP_TIME_MS};

/// Validate lap times: must be positive and under 10 minutes.
pub fn validate_lap_times(laps: &[LapSummary]) -> Vec<LapTimeViolation> {
    let mut violations = Vec::new();

    for lap in laps {
        if lap.lap_time_ms <= 0 {
            violations.push(LapTimeViolation {
                lap_number: lap.lap_number,
                lap_time_ms: lap.lap_time_ms,
                reason: "Lap time is zero or negative".to_string(),
            });
        } else if lap.lap_time_ms > MAX_LAP_TIME_MS {
            violations.push(LapTimeViolation {
                lap_number: lap.lap_number,
                lap_time_ms: lap.lap_time_ms,
                reason: format!("Lap time exceeds {}ms (10 minutes)", MAX_LAP_TIME_MS),
            });
        }
    }

    violations
}
