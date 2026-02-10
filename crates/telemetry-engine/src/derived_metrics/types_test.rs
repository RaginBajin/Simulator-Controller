//! Unit tests for derived metrics types.

#[cfg(test)]
mod tests {
    use super::super::types::*;

    #[test]
    fn test_metric_type_serialization() {
        // Verify MetricType enum serializes to snake_case
        let brake_count = MetricType::BrakeCount;
        let json = serde_json::to_string(&brake_count).unwrap();
        assert_eq!(json, "\"brake_count\"");

        let corner_seg = MetricType::CornerSegmentation;
        let json = serde_json::to_string(&corner_seg).unwrap();
        assert_eq!(json, "\"corner_segmentation\"");
    }

    #[test]
    fn test_degradation_severity_classification() {
        // Verify all severity levels are distinct
        assert_ne!(DegradationSeverity::None, DegradationSeverity::Mild);
        assert_ne!(DegradationSeverity::Mild, DegradationSeverity::Moderate);
        assert_ne!(DegradationSeverity::Moderate, DegradationSeverity::Severe);
    }

    #[test]
    fn test_brake_application_creation() {
        let brake_app = BrakeApplication {
            start_distance: 100.0,
            peak_pressure: 0.85,
            duration_ms: 1500,
            release_distance: 120.0,
        };

        assert_eq!(brake_app.start_distance, 100.0);
        assert_eq!(brake_app.peak_pressure, 0.85);
        assert_eq!(brake_app.duration_ms, 1500);
        assert_eq!(brake_app.release_distance, 120.0);
    }

    #[test]
    fn test_trail_braking_phase_creation() {
        let trail_phase = TrailBrakingPhase {
            trail_duration_ms: 500,
            trail_distance_m: 15.0,
            trail_pressure_avg: 0.45,
            turn_in_distance: 110.0,
        };

        assert_eq!(trail_phase.trail_duration_ms, 500);
        assert_eq!(trail_phase.trail_distance_m, 15.0);
        assert_eq!(trail_phase.trail_pressure_avg, 0.45);
        assert_eq!(trail_phase.turn_in_distance, 110.0);
    }

    #[test]
    fn test_tire_degradation_creation() {
        let tire_deg = TireDegradation {
            lf_temp_delta_c: 3.5,
            rf_temp_delta_c: 4.0,
            lr_temp_delta_c: 2.8,
            rr_temp_delta_c: 3.2,
            pressure_slope_per_lap: 0.05,
            severity: DegradationSeverity::Mild,
        };

        assert_eq!(tire_deg.lf_temp_delta_c, 3.5);
        assert_eq!(tire_deg.severity, DegradationSeverity::Mild);
    }

    #[test]
    fn test_corner_zone_creation() {
        let corner = CornerZone {
            corner_id: 1,
            name: "T1".to_string(),
            start_distance: 50.0,
            end_distance: 150.0,
            brake_onset_distance: Some(60.0),
            apex_distance: Some(100.0),
        };

        assert_eq!(corner.corner_id, 1);
        assert_eq!(corner.name, "T1");
        assert_eq!(corner.start_distance, 50.0);
        assert_eq!(corner.brake_onset_distance, Some(60.0));
    }

    #[test]
    fn test_derived_metrics_container() {
        let metrics = DerivedMetrics {
            session_id: "test-session-123".to_string(),
            brake_applications: vec![],
            trail_braking_phases: vec![],
            tire_degradation: None,
            corner_zones: vec![],
        };

        assert_eq!(metrics.session_id, "test-session-123");
        assert_eq!(metrics.brake_applications.len(), 0);
        assert!(metrics.tire_degradation.is_none());
    }

    #[test]
    fn test_derived_metrics_json_serialization() {
        // Verify DerivedMetrics serializes to camelCase
        let metrics = DerivedMetrics {
            session_id: "session-1".to_string(),
            brake_applications: vec![BrakeApplication {
                start_distance: 100.0,
                peak_pressure: 0.9,
                duration_ms: 1000,
                release_distance: 115.0,
            }],
            trail_braking_phases: vec![],
            tire_degradation: None,
            corner_zones: vec![],
        };

        let json = serde_json::to_value(&metrics).unwrap();
        assert!(json.get("sessionId").is_some());
        assert!(json.get("brakeApplications").is_some());
        assert!(json.get("trailBrakingPhases").is_some());
    }
}
