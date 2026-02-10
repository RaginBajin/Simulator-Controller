//! Trail braking phase identification and quantification.
//!
//! Analyzes brake applications to identify portions occurring after turn-in.

use super::types::{BrakeApplication, TrailBrakingPhase};

/// Analyze a brake application to identify trail braking phase.
///
/// Trail braking is the portion of braking that occurs after the turn-in point.
/// Turn-in is detected when lateral G exceeds threshold AND steering angle increases.
///
/// # Arguments
/// * `brake_application` - The brake application to analyze
/// * `lap_telemetry` - Full lap telemetry (needed to find turn-in point)
/// * `lat_g_threshold` - Lateral G threshold for turn-in detection (default: 0.3G)
/// * `steering_threshold` - Minimum steering angle for turn-in (radians or normalized)
///
/// # Returns
/// `Some(TrailBrakingPhase)` if trail braking detected, `None` otherwise
pub fn analyze_trail_braking(
    brake_application: &BrakeApplication,
    lap_telemetry: &[TelemetrySampleExtended],
    lat_g_threshold: f64,
    steering_threshold: f64,
) -> Option<TrailBrakingPhase> {
    // Find turn-in point within the braking zone
    let mut turn_in_idx: Option<usize> = None;

    for (idx, sample) in lap_telemetry.iter().enumerate() {
        // Check if we're within the brake application zone
        if sample.distance >= brake_application.start_distance
            && sample.distance <= brake_application.release_distance
        {
            // Turn-in detection: lateral G exceeds threshold AND steering is active
            if sample.lat_g.abs() > lat_g_threshold && sample.steering.abs() > steering_threshold {
                turn_in_idx = Some(idx);
                break;
            }
        }
    }

    let turn_in_idx = turn_in_idx?;
    let turn_in_sample = &lap_telemetry[turn_in_idx];

    // Find where braking ends (or brake application release point)
    let mut brake_end_idx = turn_in_idx;
    let mut total_brake_pressure = 0.0;
    let mut sample_count = 0;

    for (idx, sample) in lap_telemetry.iter().enumerate().skip(turn_in_idx) {
        if sample.distance <= brake_application.release_distance && sample.brake > 0.05 {
            brake_end_idx = idx;
            total_brake_pressure += sample.brake;
            sample_count += 1;
        } else if sample.distance > brake_application.release_distance {
            break;
        }
    }

    if sample_count == 0 {
        return None; // No trail braking detected
    }

    let brake_end_sample = &lap_telemetry[brake_end_idx];

    let trail_duration_ms = brake_end_sample
        .timestamp_ms
        .saturating_sub(turn_in_sample.timestamp_ms);
    let trail_distance_m = brake_end_sample.distance - turn_in_sample.distance;
    let trail_pressure_avg = total_brake_pressure / sample_count as f64;

    Some(TrailBrakingPhase {
        trail_duration_ms,
        trail_distance_m,
        trail_pressure_avg,
        turn_in_distance: turn_in_sample.distance,
    })
}

/// Extended telemetry sample with lateral G and steering for trail braking analysis.
#[derive(Debug, Clone)]
pub struct TelemetrySampleExtended {
    pub distance: f64,
    pub brake: f64,
    pub timestamp_ms: u64,
    pub lat_g: f64,
    pub steering: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_sample(
        distance: f64,
        brake: f64,
        timestamp_ms: u64,
        lat_g: f64,
        steering: f64,
    ) -> TelemetrySampleExtended {
        TelemetrySampleExtended {
            distance,
            brake,
            timestamp_ms,
            lat_g,
            steering,
        }
    }

    #[test]
    fn test_no_trail_braking_when_no_turn_in() {
        let brake_app = BrakeApplication {
            start_distance: 100.0,
            peak_pressure: 0.8,
            duration_ms: 1000,
            release_distance: 120.0,
        };

        // Telemetry with braking but no turn-in (low lat_g and steering)
        let telemetry = vec![
            create_sample(100.0, 0.7, 100, 0.1, 0.0),
            create_sample(110.0, 0.8, 200, 0.1, 0.0),
            create_sample(120.0, 0.0, 300, 0.0, 0.0),
        ];

        let result = analyze_trail_braking(&brake_app, &telemetry, 0.3, 0.1);
        assert!(
            result.is_none(),
            "Should be None when no turn-in detected"
        );
    }

    #[test]
    fn test_trail_braking_detected() {
        let brake_app = BrakeApplication {
            start_distance: 100.0,
            peak_pressure: 0.8,
            duration_ms: 1500,
            release_distance: 125.0,
        };

        // Telemetry with turn-in at 110m (lat_g crosses threshold + steering active)
        let telemetry = vec![
            create_sample(100.0, 0.7, 100, 0.1, 0.0), // brake starts, no turn
            create_sample(105.0, 0.8, 200, 0.2, 0.05), // still no turn
            create_sample(110.0, 0.6, 300, 0.4, 0.15), // turn-in! (lat_g > 0.3, steering > 0.1)
            create_sample(115.0, 0.5, 400, 0.5, 0.2), // trail braking continues
            create_sample(120.0, 0.3, 500, 0.6, 0.25), // trail braking continues
            create_sample(125.0, 0.0, 600, 0.5, 0.2), // brake released
        ];

        let result = analyze_trail_braking(&brake_app, &telemetry, 0.3, 0.1);
        assert!(result.is_some(), "Should detect trail braking");

        let phase = result.unwrap();
        assert_eq!(phase.turn_in_distance, 110.0);
        assert_eq!(phase.trail_duration_ms, 200); // 500ms - 300ms
        assert_eq!(phase.trail_distance_m, 10.0); // 120m - 110m
        assert!((phase.trail_pressure_avg - 0.467).abs() < 0.01); // avg of 0.6, 0.5, 0.3
    }

    #[test]
    fn test_turn_in_before_brake_zone_ignored() {
        let brake_app = BrakeApplication {
            start_distance: 100.0,
            peak_pressure: 0.8,
            duration_ms: 1000,
            release_distance: 120.0,
        };

        // Turn-in happens before brake zone starts
        let telemetry = vec![
            create_sample(90.0, 0.0, 50, 0.5, 0.2), // turn-in before braking
            create_sample(100.0, 0.7, 100, 0.6, 0.25), // brake starts
            create_sample(110.0, 0.8, 200, 0.7, 0.3),
            create_sample(120.0, 0.0, 300, 0.5, 0.2), // brake released
        ];

        let result = analyze_trail_braking(&brake_app, &telemetry, 0.3, 0.1);
        // Should detect turn-in at 100m (first sample in brake zone with high lat_g)
        assert!(result.is_some());
    }

    #[test]
    fn test_no_trail_braking_when_turn_in_at_release() {
        let brake_app = BrakeApplication {
            start_distance: 100.0,
            peak_pressure: 0.8,
            duration_ms: 1000,
            release_distance: 120.0,
        };

        // Turn-in happens exactly when brake is released
        let telemetry = vec![
            create_sample(100.0, 0.7, 100, 0.1, 0.0),
            create_sample(110.0, 0.8, 200, 0.1, 0.0),
            create_sample(120.0, 0.0, 300, 0.4, 0.15), // turn-in at release point
        ];

        let result = analyze_trail_braking(&brake_app, &telemetry, 0.3, 0.1);
        assert!(
            result.is_none(),
            "Should be None when turn-in coincides with brake release"
        );
    }

    #[test]
    fn test_trail_braking_calculation_accuracy() {
        let brake_app = BrakeApplication {
            start_distance: 200.0,
            peak_pressure: 0.9,
            duration_ms: 2000,
            release_distance: 240.0,
        };

        let telemetry = vec![
            create_sample(200.0, 0.9, 1000, 0.1, 0.0),
            create_sample(210.0, 0.8, 1200, 0.2, 0.05),
            create_sample(220.0, 0.7, 1400, 0.4, 0.15), // turn-in at 220m
            create_sample(225.0, 0.6, 1600, 0.5, 0.2),
            create_sample(230.0, 0.5, 1800, 0.6, 0.25),
            create_sample(235.0, 0.4, 2000, 0.6, 0.25),
            create_sample(240.0, 0.0, 2200, 0.5, 0.2),
        ];

        let result = analyze_trail_braking(&brake_app, &telemetry, 0.3, 0.1).unwrap();

        assert_eq!(result.turn_in_distance, 220.0);
        assert_eq!(result.trail_duration_ms, 600); // 2000ms - 1400ms
        assert_eq!(result.trail_distance_m, 15.0); // 235m - 220m (last brake sample - turn-in)
        assert!((result.trail_pressure_avg - 0.55).abs() < 0.01); // avg of 0.7, 0.6, 0.5, 0.4
    }
}
