//! Brake application counting and analysis.
//!
//! Identifies discrete brake applications from telemetry and computes per-lap summaries.

use super::types::BrakeApplication;

/// Telemetry sample for brake analysis.
///
/// Minimal struct containing fields needed for brake counting.
/// In production, this would map to the full 35-channel telemetry schema.
#[derive(Debug, Clone)]
pub struct TelemetrySample {
    pub distance: f64,
    pub brake: f64,
    pub timestamp_ms: u64,
}

/// Count and analyze brake applications in a lap's telemetry.
///
/// A brake application is defined as:
/// - Brake input crosses above `threshold` (default 0.05 = 5%)
/// - Remains above threshold for at least 2 consecutive samples (debounce)
/// - Each application records: start_distance, peak_pressure, duration_ms, release_distance
///
/// # Arguments
/// * `lap_telemetry` - Time-ordered telemetry samples for a single lap
/// * `threshold` - Minimum brake pressure to register (0.0-1.0, default 0.05)
///
/// # Returns
/// Vector of brake applications with detailed metrics
pub fn count_brake_applications(
    lap_telemetry: &[TelemetrySample],
    threshold: f64,
) -> Vec<BrakeApplication> {
    if lap_telemetry.len() < 2 {
        return vec![];
    }

    let mut applications = Vec::new();
    let mut in_brake_zone = false;
    let mut brake_start_idx: usize = 0;
    let mut peak_pressure = 0.0;
    let mut consecutive_above_threshold = 0;

    for (idx, sample) in lap_telemetry.iter().enumerate() {
        if sample.brake > threshold {
            consecutive_above_threshold += 1;

            if !in_brake_zone && consecutive_above_threshold >= 2 {
                // Start of new brake application (debounced)
                in_brake_zone = true;
                brake_start_idx = idx - 1; // Start at first sample above threshold
                peak_pressure = sample.brake;
            } else if in_brake_zone {
                // Update peak pressure during brake application
                if sample.brake > peak_pressure {
                    peak_pressure = sample.brake;
                }
            }
        } else {
            // Brake released
            if in_brake_zone {
                // End of brake application
                let start_sample = &lap_telemetry[brake_start_idx];
                let end_sample = if idx > 0 {
                    &lap_telemetry[idx - 1]
                } else {
                    sample
                };

                let duration_ms = end_sample
                    .timestamp_ms
                    .saturating_sub(start_sample.timestamp_ms);

                applications.push(BrakeApplication {
                    start_distance: start_sample.distance,
                    peak_pressure,
                    duration_ms,
                    release_distance: sample.distance,
                });

                in_brake_zone = false;
                peak_pressure = 0.0;
            }
            consecutive_above_threshold = 0;
        }
    }

    // Handle case where lap ends while still braking
    if in_brake_zone && brake_start_idx < lap_telemetry.len() {
        let start_sample = &lap_telemetry[brake_start_idx];
        let end_sample = lap_telemetry.last().unwrap();

        let duration_ms = end_sample
            .timestamp_ms
            .saturating_sub(start_sample.timestamp_ms);

        applications.push(BrakeApplication {
            start_distance: start_sample.distance,
            peak_pressure,
            duration_ms,
            release_distance: end_sample.distance,
        });
    }

    applications
}

/// Per-lap brake summary statistics.
#[derive(Debug, Clone)]
pub struct BrakeSummary {
    pub brake_count: usize,
    pub avg_peak_pressure: f64,
    pub total_braking_distance: f64,
}

/// Compute summary statistics for brake applications in a lap.
pub fn compute_brake_summary(applications: &[BrakeApplication]) -> BrakeSummary {
    if applications.is_empty() {
        return BrakeSummary {
            brake_count: 0,
            avg_peak_pressure: 0.0,
            total_braking_distance: 0.0,
        };
    }

    let total_pressure: f64 = applications.iter().map(|app| app.peak_pressure).sum();
    let avg_peak_pressure = total_pressure / applications.len() as f64;

    let total_braking_distance: f64 = applications
        .iter()
        .map(|app| app.release_distance - app.start_distance)
        .sum();

    BrakeSummary {
        brake_count: applications.len(),
        avg_peak_pressure,
        total_braking_distance,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_sample(distance: f64, brake: f64, timestamp_ms: u64) -> TelemetrySample {
        TelemetrySample {
            distance,
            brake,
            timestamp_ms,
        }
    }

    #[test]
    fn test_no_brake_applications() {
        let telemetry = vec![
            create_sample(0.0, 0.0, 0),
            create_sample(10.0, 0.0, 100),
            create_sample(20.0, 0.0, 200),
        ];

        let apps = count_brake_applications(&telemetry, 0.05);
        assert_eq!(apps.len(), 0);
    }

    #[test]
    fn test_single_brake_application() {
        let telemetry = vec![
            create_sample(0.0, 0.0, 0),
            create_sample(10.0, 0.1, 100), // brake starts
            create_sample(20.0, 0.8, 200), // peak
            create_sample(30.0, 0.5, 300),
            create_sample(40.0, 0.0, 400), // brake released
            create_sample(50.0, 0.0, 500),
        ];

        let apps = count_brake_applications(&telemetry, 0.05);
        assert_eq!(apps.len(), 1);

        let app = &apps[0];
        assert_eq!(app.start_distance, 10.0);
        assert_eq!(app.peak_pressure, 0.8);
        assert_eq!(app.duration_ms, 200); // 300ms - 100ms
        assert_eq!(app.release_distance, 40.0);
    }

    #[test]
    fn test_debounce_filters_single_spike() {
        let telemetry = vec![
            create_sample(0.0, 0.0, 0),
            create_sample(10.0, 0.1, 100), // single spike - should be filtered
            create_sample(20.0, 0.0, 200),
            create_sample(30.0, 0.0, 300),
        ];

        let apps = count_brake_applications(&telemetry, 0.05);
        assert_eq!(apps.len(), 0, "Single-sample spike should be debounced");
    }

    #[test]
    fn test_debounce_allows_two_consecutive_samples() {
        let telemetry = vec![
            create_sample(0.0, 0.0, 0),
            create_sample(10.0, 0.1, 100), // brake starts
            create_sample(20.0, 0.2, 200), // consecutive - passes debounce
            create_sample(30.0, 0.0, 300), // brake released
        ];

        let apps = count_brake_applications(&telemetry, 0.05);
        assert_eq!(
            apps.len(),
            1,
            "Two consecutive samples should pass debounce"
        );
    }

    #[test]
    fn test_multiple_brake_applications() {
        let telemetry = vec![
            create_sample(0.0, 0.0, 0),
            create_sample(50.0, 0.7, 100), // brake 1 start
            create_sample(60.0, 0.8, 200),
            create_sample(70.0, 0.0, 300), // brake 1 end
            create_sample(100.0, 0.0, 400),
            create_sample(200.0, 0.6, 500), // brake 2 start
            create_sample(210.0, 0.9, 600),
            create_sample(220.0, 0.0, 700), // brake 2 end
        ];

        let apps = count_brake_applications(&telemetry, 0.05);
        assert_eq!(apps.len(), 2);

        assert_eq!(apps[0].start_distance, 50.0);
        assert_eq!(apps[0].peak_pressure, 0.8);

        assert_eq!(apps[1].start_distance, 200.0);
        assert_eq!(apps[1].peak_pressure, 0.9);
    }

    #[test]
    fn test_brake_application_ending_at_lap_end() {
        let telemetry = vec![
            create_sample(0.0, 0.0, 0),
            create_sample(500.0, 0.7, 500), // brake starts
            create_sample(510.0, 0.8, 600), // still braking at end
        ];

        let apps = count_brake_applications(&telemetry, 0.05);
        assert_eq!(apps.len(), 1);

        let app = &apps[0];
        assert_eq!(app.start_distance, 500.0);
        assert_eq!(app.release_distance, 510.0);
    }

    #[test]
    fn test_compute_brake_summary_empty() {
        let summary = compute_brake_summary(&[]);
        assert_eq!(summary.brake_count, 0);
        assert_eq!(summary.avg_peak_pressure, 0.0);
        assert_eq!(summary.total_braking_distance, 0.0);
    }

    #[test]
    fn test_compute_brake_summary() {
        let apps = vec![
            BrakeApplication {
                start_distance: 100.0,
                peak_pressure: 0.8,
                duration_ms: 1000,
                release_distance: 120.0,
            },
            BrakeApplication {
                start_distance: 300.0,
                peak_pressure: 0.9,
                duration_ms: 1200,
                release_distance: 325.0,
            },
        ];

        let summary = compute_brake_summary(&apps);
        assert_eq!(summary.brake_count, 2);
        assert!((summary.avg_peak_pressure - 0.85).abs() < 0.001); // (0.8 + 0.9) / 2
        assert_eq!(summary.total_braking_distance, 45.0); // (120-100) + (325-300)
    }
}
