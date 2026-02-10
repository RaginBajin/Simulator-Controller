//! Tire degradation calculation and severity classification.
//!
//! Computes tire degradation metrics across a stint based on temperature and pressure changes.

use super::types::{DegradationSeverity, TireDegradation};

/// Telemetry sample for tire degradation analysis.
#[derive(Debug, Clone)]
pub struct LapTelemetry {
    pub lap_number: i32,
    pub lf_temp_avg: f64,
    pub rf_temp_avg: f64,
    pub lr_temp_avg: f64,
    pub rr_temp_avg: f64,
    pub lf_pressure: f64,
    pub rf_pressure: f64,
    pub lr_pressure: f64,
    pub rr_pressure: f64,
}

/// Compute tire degradation metrics for a stint.
///
/// A stint is a sequence of consecutive laps between pit stops or session boundaries.
/// Degradation is measured as:
/// - Temperature delta from stint start to stint end for each tire
/// - Pressure change per lap (linear regression slope)
/// - Severity classification based on temperature change
///
/// # Arguments
/// * `stint_laps` - Consecutive laps in the stint, chronologically ordered
///
/// # Returns
/// `TireDegradation` with temp deltas, pressure slope, and severity classification
pub fn compute_tire_degradation(stint_laps: &[LapTelemetry]) -> Option<TireDegradation> {
    if stint_laps.len() < 2 {
        return None; // Need at least 2 laps to measure degradation
    }

    let first_lap = stint_laps.first().unwrap();
    let last_lap = stint_laps.last().unwrap();

    // Temperature deltas (end - start)
    let lf_temp_delta_c = last_lap.lf_temp_avg - first_lap.lf_temp_avg;
    let rf_temp_delta_c = last_lap.rf_temp_avg - first_lap.rf_temp_avg;
    let lr_temp_delta_c = last_lap.lr_temp_avg - first_lap.lr_temp_avg;
    let rr_temp_delta_c = last_lap.rr_temp_avg - first_lap.rr_temp_avg;

    // Compute linear regression for average pressure across all tires
    let pressure_slope_per_lap = compute_pressure_slope(stint_laps);

    // Classify severity based on maximum absolute temperature delta
    let max_temp_delta = [
        lf_temp_delta_c.abs(),
        rf_temp_delta_c.abs(),
        lr_temp_delta_c.abs(),
        rr_temp_delta_c.abs(),
    ]
    .iter()
    .cloned()
    .fold(f64::NAN, f64::max);

    let severity = classify_degradation_severity(max_temp_delta);

    Some(TireDegradation {
        lf_temp_delta_c,
        rf_temp_delta_c,
        lr_temp_delta_c,
        rr_temp_delta_c,
        pressure_slope_per_lap,
        severity,
    })
}

/// Compute linear regression slope for pressure change across stint.
///
/// Uses average pressure across all four tires per lap.
fn compute_pressure_slope(stint_laps: &[LapTelemetry]) -> f64 {
    let n = stint_laps.len() as f64;

    // Compute average pressure per lap
    let pressures: Vec<f64> = stint_laps
        .iter()
        .map(|lap| (lap.lf_pressure + lap.rf_pressure + lap.lr_pressure + lap.rr_pressure) / 4.0)
        .collect();

    // Linear regression: y = mx + b
    // We use lap index as x (0, 1, 2, ..., n-1)
    let x_mean = (n - 1.0) / 2.0; // Mean of [0, 1, 2, ..., n-1]
    let y_mean = pressures.iter().sum::<f64>() / n;

    let mut numerator = 0.0;
    let mut denominator = 0.0;

    for (i, pressure) in pressures.iter().enumerate() {
        let x_dev = i as f64 - x_mean;
        let y_dev = pressure - y_mean;
        numerator += x_dev * y_dev;
        denominator += x_dev * x_dev;
    }

    if denominator.abs() < 1e-10 {
        return 0.0; // Avoid division by zero
    }

    numerator / denominator
}

/// Classify degradation severity based on temperature delta.
fn classify_degradation_severity(temp_delta_c: f64) -> DegradationSeverity {
    match temp_delta_c {
        delta if delta < 2.0 => DegradationSeverity::None,
        delta if delta < 5.0 => DegradationSeverity::Mild,
        delta if delta < 10.0 => DegradationSeverity::Moderate,
        _ => DegradationSeverity::Severe,
    }
}

/// Group consecutive laps into stints.
///
/// A stint ends when there's a gap in lap numbers > threshold (indicating a pit stop).
///
/// # Arguments
/// * `all_laps` - All laps in the session, chronologically ordered
/// * `gap_threshold` - Max lap number gap before starting new stint (default: 1)
///
/// # Returns
/// Vector of stints, each containing consecutive laps
pub fn segment_into_stints(
    all_laps: &[LapTelemetry],
    gap_threshold: i32,
) -> Vec<Vec<LapTelemetry>> {
    if all_laps.is_empty() {
        return vec![];
    }

    let mut stints: Vec<Vec<LapTelemetry>> = Vec::new();
    let mut current_stint = Vec::new();

    for (idx, lap) in all_laps.iter().enumerate() {
        if idx == 0 {
            current_stint.push(lap.clone());
        } else {
            let prev_lap = &all_laps[idx - 1];
            let gap = lap.lap_number - prev_lap.lap_number;

            if gap > gap_threshold {
                // Start new stint
                if !current_stint.is_empty() {
                    stints.push(current_stint.clone());
                }
                current_stint = vec![lap.clone()];
            } else {
                current_stint.push(lap.clone());
            }
        }
    }

    // Add final stint
    if !current_stint.is_empty() {
        stints.push(current_stint);
    }

    stints
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_lap(
        lap_number: i32,
        lf_temp: f64,
        rf_temp: f64,
        lr_temp: f64,
        rr_temp: f64,
        avg_pressure: f64,
    ) -> LapTelemetry {
        LapTelemetry {
            lap_number,
            lf_temp_avg: lf_temp,
            rf_temp_avg: rf_temp,
            lr_temp_avg: lr_temp,
            rr_temp_avg: rr_temp,
            lf_pressure: avg_pressure,
            rf_pressure: avg_pressure,
            lr_pressure: avg_pressure,
            rr_pressure: avg_pressure,
        }
    }

    #[test]
    fn test_no_degradation_with_single_lap() {
        let laps = vec![create_lap(1, 85.0, 85.0, 83.0, 83.0, 30.0)];
        let result = compute_tire_degradation(&laps);
        assert!(result.is_none(), "Single lap should return None");
    }

    #[test]
    fn test_degradation_severity_none() {
        let laps = vec![
            create_lap(1, 85.0, 85.0, 83.0, 83.0, 30.0),
            create_lap(2, 85.5, 85.5, 83.5, 83.5, 30.1),
        ];

        let deg = compute_tire_degradation(&laps).unwrap();
        assert_eq!(deg.severity, DegradationSeverity::None);
        assert!((deg.lf_temp_delta_c - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_degradation_severity_mild() {
        let laps = vec![
            create_lap(1, 80.0, 80.0, 78.0, 78.0, 30.0),
            create_lap(2, 82.5, 83.0, 80.5, 81.0, 29.9),
        ];

        let deg = compute_tire_degradation(&laps).unwrap();
        assert_eq!(deg.severity, DegradationSeverity::Mild);
        assert!((deg.rf_temp_delta_c - 3.0).abs() < 0.01);
    }

    #[test]
    fn test_degradation_severity_moderate() {
        let laps = vec![
            create_lap(1, 75.0, 75.0, 73.0, 73.0, 30.5),
            create_lap(5, 82.0, 83.0, 80.0, 81.0, 29.8),
        ];

        let deg = compute_tire_degradation(&laps).unwrap();
        assert_eq!(deg.severity, DegradationSeverity::Moderate);
        assert!((deg.rf_temp_delta_c - 8.0).abs() < 0.01);
    }

    #[test]
    fn test_degradation_severity_severe() {
        let laps = vec![
            create_lap(1, 70.0, 70.0, 68.0, 68.0, 31.0),
            create_lap(10, 82.0, 85.0, 80.0, 82.0, 28.5),
        ];

        let deg = compute_tire_degradation(&laps).unwrap();
        assert_eq!(deg.severity, DegradationSeverity::Severe);
        assert!((deg.rf_temp_delta_c - 15.0).abs() < 0.01);
    }

    #[test]
    fn test_pressure_slope_decreasing() {
        let laps = vec![
            create_lap(1, 80.0, 80.0, 78.0, 78.0, 30.0),
            create_lap(2, 81.0, 81.0, 79.0, 79.0, 29.8),
            create_lap(3, 82.0, 82.0, 80.0, 80.0, 29.6),
        ];

        let deg = compute_tire_degradation(&laps).unwrap();
        assert!(
            deg.pressure_slope_per_lap < 0.0,
            "Pressure slope should be negative (decreasing)"
        );
        assert!((deg.pressure_slope_per_lap - (-0.2)).abs() < 0.01);
    }

    #[test]
    fn test_pressure_slope_increasing() {
        let laps = vec![
            create_lap(1, 80.0, 80.0, 78.0, 78.0, 29.0),
            create_lap(2, 81.0, 81.0, 79.0, 79.0, 29.5),
            create_lap(3, 82.0, 82.0, 80.0, 80.0, 30.0),
        ];

        let deg = compute_tire_degradation(&laps).unwrap();
        assert!(
            deg.pressure_slope_per_lap > 0.0,
            "Pressure slope should be positive (increasing)"
        );
        assert!((deg.pressure_slope_per_lap - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_segment_into_stints_no_pit_stops() {
        let laps = vec![
            create_lap(1, 80.0, 80.0, 78.0, 78.0, 30.0),
            create_lap(2, 81.0, 81.0, 79.0, 79.0, 29.9),
            create_lap(3, 82.0, 82.0, 80.0, 80.0, 29.8),
        ];

        let stints = segment_into_stints(&laps, 1);
        assert_eq!(stints.len(), 1);
        assert_eq!(stints[0].len(), 3);
    }

    #[test]
    fn test_segment_into_stints_with_pit_stop() {
        let laps = vec![
            create_lap(1, 80.0, 80.0, 78.0, 78.0, 30.0),
            create_lap(2, 81.0, 81.0, 79.0, 79.0, 29.9),
            // Pit stop here (lap 3 missing)
            create_lap(4, 75.0, 75.0, 73.0, 73.0, 31.0),
            create_lap(5, 76.0, 76.0, 74.0, 74.0, 30.9),
        ];

        let stints = segment_into_stints(&laps, 1);
        assert_eq!(stints.len(), 2);
        assert_eq!(stints[0].len(), 2); // Laps 1-2
        assert_eq!(stints[1].len(), 2); // Laps 4-5
    }

    #[test]
    fn test_segment_into_stints_multiple_pit_stops() {
        let laps = vec![
            create_lap(1, 80.0, 80.0, 78.0, 78.0, 30.0),
            create_lap(2, 81.0, 81.0, 79.0, 79.0, 29.9),
            create_lap(5, 75.0, 75.0, 73.0, 73.0, 31.0),
            create_lap(6, 76.0, 76.0, 74.0, 74.0, 30.9),
            create_lap(10, 78.0, 78.0, 76.0, 76.0, 30.5),
        ];

        let stints = segment_into_stints(&laps, 1);
        assert_eq!(stints.len(), 3);
        assert_eq!(stints[0][0].lap_number, 1);
        assert_eq!(stints[1][0].lap_number, 5);
        assert_eq!(stints[2][0].lap_number, 10);
    }
}
