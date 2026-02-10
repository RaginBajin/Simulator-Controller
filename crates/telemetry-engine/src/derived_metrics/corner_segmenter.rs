//! Corner zone segmentation from telemetry patterns.
//!
//! Identifies corners by detecting braking-turning sequences and computes per-corner metrics.

use super::types::CornerZone;

/// Telemetry sample for corner segmentation.
#[derive(Debug, Clone)]
pub struct TelemetrySample {
    pub distance: f64,
    pub brake: f64,
    pub throttle: f64,
    pub lat_g: f64,
    pub speed: f64,
    pub timestamp_ms: u64,
}

/// Segment corners from the best lap's telemetry.
///
/// Corner detection algorithm:
/// 1. Find braking zones (brake > threshold)
/// 2. Find turning zones (|lat_g| > threshold)
/// 3. Match brake-to-turn pairs
/// 4. Define corner boundaries: brake onset → full throttle (>90%)
/// 5. Name corners sequentially (T1, T2, T3...)
///
/// # Arguments
/// * `best_lap` - Telemetry from the best lap (used as template)
/// * `brake_threshold` - Minimum brake input to detect braking (default 0.05)
/// * `lat_g_threshold` - Minimum lateral G to detect turning (default 0.3)
/// * `throttle_threshold` - Full throttle threshold for corner exit (default 0.9)
///
/// # Returns
/// Vector of corner zones with boundaries and identifiers
pub fn segment_corners(
    best_lap: &[TelemetrySample],
    brake_threshold: f64,
    lat_g_threshold: f64,
    throttle_threshold: f64,
) -> Vec<CornerZone> {
    if best_lap.len() < 10 {
        return vec![]; // Need minimum telemetry for meaningful segmentation
    }

    // Phase 1: Identify braking zones
    let brake_zones = find_braking_zones(best_lap, brake_threshold);

    // Phase 2: Identify turning zones
    let turning_zones = find_turning_zones(best_lap, lat_g_threshold);

    // Phase 3: Match brake zones to turning zones to identify corners
    let mut corners = Vec::new();
    let mut corner_id = 1;

    for brake_zone in &brake_zones {
        // Find next turning zone after brake onset
        if let Some(turn_zone) = find_matching_turn_zone(brake_zone, &turning_zones, best_lap) {
            // Find corner exit (full throttle application)
            let exit_distance =
                find_corner_exit(turn_zone.end_distance, best_lap, throttle_threshold);

            let corner = CornerZone {
                corner_id,
                name: format!("T{}", corner_id),
                start_distance: brake_zone.start_distance,
                end_distance: exit_distance,
                brake_onset_distance: Some(brake_zone.start_distance),
                apex_distance: Some(find_apex_distance(&turn_zone, best_lap)),
            };

            corners.push(corner);
            corner_id += 1;
        }
    }

    corners
}

/// Compute per-corner metrics for a specific corner zone.
///
/// Metrics include:
/// - Brake application count within corner
/// - Average brake pressure
/// - Trail brake average distance
/// - Min speed, apex speed, exit speed
/// - Time in corner
pub fn compute_corner_metrics(
    corner: &CornerZone,
    lap_telemetry: &[TelemetrySample],
) -> CornerMetrics {
    // Filter samples within corner zone
    let corner_samples: Vec<&TelemetrySample> = lap_telemetry
        .iter()
        .filter(|s| s.distance >= corner.start_distance && s.distance <= corner.end_distance)
        .collect();

    if corner_samples.is_empty() {
        return CornerMetrics::default();
    }

    // Count brake applications
    let brake_count = count_brake_apps_in_corner(&corner_samples, 0.05);

    // Average brake pressure (only when brake > threshold)
    let brake_samples: Vec<f64> = corner_samples
        .iter()
        .filter(|s| s.brake > 0.05)
        .map(|s| s.brake)
        .collect();
    let avg_brake_pressure = if !brake_samples.is_empty() {
        brake_samples.iter().sum::<f64>() / brake_samples.len() as f64
    } else {
        0.0
    };

    // Speed metrics
    let min_speed = corner_samples
        .iter()
        .map(|s| s.speed)
        .fold(f64::INFINITY, f64::min);

    let apex_speed = if let Some(apex_dist) = corner.apex_distance {
        find_speed_at_distance(&corner_samples, apex_dist)
    } else {
        min_speed
    };

    let exit_speed = corner_samples.last().map(|s| s.speed).unwrap_or(0.0);

    // Time in corner
    let time_in_corner_ms =
        if let (Some(first), Some(last)) = (corner_samples.first(), corner_samples.last()) {
            last.timestamp_ms.saturating_sub(first.timestamp_ms)
        } else {
            0
        };

    // Trail brake distance (distance from brake onset to apex while braking)
    let trail_brake_avg_distance = if let Some(brake_onset) = corner.brake_onset_distance {
        if let Some(apex) = corner.apex_distance {
            apex - brake_onset
        } else {
            0.0
        }
    } else {
        0.0
    };

    CornerMetrics {
        brake_count,
        avg_brake_pressure,
        trail_brake_avg_distance,
        min_speed,
        apex_speed,
        exit_speed,
        time_in_corner_ms,
    }
}

/// Per-corner metrics structure.
#[derive(Debug, Clone, Default)]
pub struct CornerMetrics {
    pub brake_count: usize,
    pub avg_brake_pressure: f64,
    pub trail_brake_avg_distance: f64,
    pub min_speed: f64,
    pub apex_speed: f64,
    pub exit_speed: f64,
    pub time_in_corner_ms: u64,
}

// Internal zone structures
#[derive(Debug, Clone)]
struct Zone {
    start_distance: f64,
    end_distance: f64,
}

fn find_braking_zones(telemetry: &[TelemetrySample], threshold: f64) -> Vec<Zone> {
    let mut zones = Vec::new();
    let mut in_zone = false;
    let mut zone_start = 0.0;

    for sample in telemetry {
        if sample.brake > threshold && !in_zone {
            in_zone = true;
            zone_start = sample.distance;
        } else if sample.brake <= threshold && in_zone {
            in_zone = false;
            zones.push(Zone {
                start_distance: zone_start,
                end_distance: sample.distance,
            });
        }
    }

    // Handle zone extending to lap end
    if in_zone {
        if let Some(last) = telemetry.last() {
            zones.push(Zone {
                start_distance: zone_start,
                end_distance: last.distance,
            });
        }
    }

    zones
}

fn find_turning_zones(telemetry: &[TelemetrySample], threshold: f64) -> Vec<Zone> {
    let mut zones = Vec::new();
    let mut in_zone = false;
    let mut zone_start = 0.0;

    for sample in telemetry {
        if sample.lat_g.abs() > threshold && !in_zone {
            in_zone = true;
            zone_start = sample.distance;
        } else if sample.lat_g.abs() <= threshold && in_zone {
            in_zone = false;
            zones.push(Zone {
                start_distance: zone_start,
                end_distance: sample.distance,
            });
        }
    }

    if in_zone {
        if let Some(last) = telemetry.last() {
            zones.push(Zone {
                start_distance: zone_start,
                end_distance: last.distance,
            });
        }
    }

    zones
}

fn find_matching_turn_zone(
    brake_zone: &Zone,
    turning_zones: &[Zone],
    _telemetry: &[TelemetrySample],
) -> Option<Zone> {
    // Find first turning zone that starts within reasonable distance after brake onset
    const MAX_DISTANCE_TO_TURN: f64 = 200.0; // meters

    for turn_zone in turning_zones {
        if turn_zone.start_distance >= brake_zone.start_distance
            && turn_zone.start_distance - brake_zone.start_distance < MAX_DISTANCE_TO_TURN
        {
            return Some(turn_zone.clone());
        }
    }

    None
}

fn find_corner_exit(
    turn_end_distance: f64,
    telemetry: &[TelemetrySample],
    throttle_threshold: f64,
) -> f64 {
    // Find first point after turn where throttle > threshold
    for sample in telemetry {
        if sample.distance >= turn_end_distance && sample.throttle > throttle_threshold {
            return sample.distance;
        }
    }

    // Fallback: use turn end distance
    turn_end_distance
}

fn find_apex_distance(turn_zone: &Zone, telemetry: &[TelemetrySample]) -> f64 {
    // Apex = point of minimum speed within turning zone
    let mut min_speed = f64::INFINITY;
    let mut apex_dist = turn_zone.start_distance;

    for sample in telemetry {
        if sample.distance >= turn_zone.start_distance
            && sample.distance <= turn_zone.end_distance
            && sample.speed < min_speed
        {
            min_speed = sample.speed;
            apex_dist = sample.distance;
        }
    }

    apex_dist
}

fn count_brake_apps_in_corner(samples: &[&TelemetrySample], threshold: f64) -> usize {
    let mut count = 0;
    let mut in_brake = false;

    for sample in samples {
        if sample.brake > threshold && !in_brake {
            count += 1;
            in_brake = true;
        } else if sample.brake <= threshold {
            in_brake = false;
        }
    }

    count
}

fn find_speed_at_distance(samples: &[&TelemetrySample], target_distance: f64) -> f64 {
    // Find closest sample to target distance
    samples
        .iter()
        .min_by(|a, b| {
            let dist_a = (a.distance - target_distance).abs();
            let dist_b = (b.distance - target_distance).abs();
            dist_a.partial_cmp(&dist_b).unwrap()
        })
        .map(|s| s.speed)
        .unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_sample(
        distance: f64,
        brake: f64,
        throttle: f64,
        lat_g: f64,
        speed: f64,
        timestamp_ms: u64,
    ) -> TelemetrySample {
        TelemetrySample {
            distance,
            brake,
            throttle,
            lat_g,
            speed,
            timestamp_ms,
        }
    }

    #[test]
    fn test_no_corners_in_empty_telemetry() {
        let telemetry: Vec<TelemetrySample> = vec![];
        let corners = segment_corners(&telemetry, 0.05, 0.3, 0.9);
        assert_eq!(corners.len(), 0);
    }

    #[test]
    fn test_no_corners_in_straight_line() {
        let telemetry = vec![
            create_sample(0.0, 0.0, 1.0, 0.0, 100.0, 0),
            create_sample(100.0, 0.0, 1.0, 0.0, 100.0, 1000),
            create_sample(200.0, 0.0, 1.0, 0.0, 100.0, 2000),
        ];

        let corners = segment_corners(&telemetry, 0.05, 0.3, 0.9);
        assert_eq!(corners.len(), 0, "No corners in straight line");
    }

    #[test]
    fn test_single_corner_detection() {
        let telemetry = vec![
            create_sample(0.0, 0.0, 1.0, 0.0, 150.0, 0),
            create_sample(50.0, 0.0, 1.0, 0.0, 155.0, 500),
            create_sample(80.0, 0.0, 1.0, 0.0, 158.0, 800),
            create_sample(100.0, 0.7, 0.2, 0.1, 140.0, 1000), // brake starts
            create_sample(110.0, 0.8, 0.1, 0.15, 130.0, 1100),
            create_sample(120.0, 0.8, 0.0, 0.2, 120.0, 1200),
            create_sample(130.0, 0.7, 0.0, 0.35, 100.0, 1350),
            create_sample(140.0, 0.6, 0.0, 0.5, 80.0, 1500), // turning + apex
            create_sample(150.0, 0.4, 0.1, 0.45, 85.0, 1650),
            create_sample(160.0, 0.3, 0.2, 0.4, 90.0, 1800),
            create_sample(170.0, 0.1, 0.5, 0.3, 100.0, 1950),
            create_sample(180.0, 0.0, 0.95, 0.2, 110.0, 2100), // full throttle
            create_sample(200.0, 0.0, 1.0, 0.0, 130.0, 2400),
        ];

        let corners = segment_corners(&telemetry, 0.05, 0.3, 0.9);
        assert_eq!(corners.len(), 1, "Should detect one corner");

        let corner = &corners[0];
        assert_eq!(corner.corner_id, 1);
        assert_eq!(corner.name, "T1");
        assert_eq!(corner.start_distance, 100.0); // brake onset
        assert!(corner.end_distance >= 180.0); // full throttle
    }

    #[test]
    fn test_multiple_corners_detection() {
        let telemetry = vec![
            // Straight before corner 1
            create_sample(0.0, 0.0, 1.0, 0.0, 150.0, 0),
            create_sample(50.0, 0.0, 1.0, 0.0, 152.0, 500),
            // Corner 1
            create_sample(100.0, 0.7, 0.2, 0.1, 140.0, 1000),
            create_sample(110.0, 0.8, 0.1, 0.2, 130.0, 1100),
            create_sample(120.0, 0.6, 0.0, 0.5, 80.0, 1500),
            create_sample(130.0, 0.3, 0.3, 0.4, 90.0, 1650),
            create_sample(140.0, 0.0, 0.95, 0.2, 100.0, 1800),
            // Straight
            create_sample(200.0, 0.0, 1.0, 0.0, 155.0, 2400),
            create_sample(250.0, 0.0, 1.0, 0.0, 158.0, 2800),
            create_sample(300.0, 0.0, 1.0, 0.0, 160.0, 3000),
            create_sample(350.0, 0.0, 1.0, 0.0, 162.0, 3400),
            // Corner 2
            create_sample(400.0, 0.8, 0.1, 0.1, 150.0, 4000),
            create_sample(410.0, 0.9, 0.0, 0.3, 120.0, 4200),
            create_sample(420.0, 0.7, 0.0, 0.6, 70.0, 4500),
            create_sample(430.0, 0.4, 0.2, 0.5, 80.0, 4650),
            create_sample(440.0, 0.0, 0.95, 0.3, 95.0, 4800),
            create_sample(500.0, 0.0, 1.0, 0.0, 140.0, 5500),
        ];

        let corners = segment_corners(&telemetry, 0.05, 0.3, 0.9);
        assert_eq!(corners.len(), 2, "Should detect two corners");

        assert_eq!(corners[0].name, "T1");
        assert_eq!(corners[1].name, "T2");
    }

    #[test]
    fn test_corner_metrics_calculation() {
        let corner = CornerZone {
            corner_id: 1,
            name: "T1".to_string(),
            start_distance: 100.0,
            end_distance: 180.0,
            brake_onset_distance: Some(100.0),
            apex_distance: Some(140.0),
        };

        let telemetry = vec![
            create_sample(100.0, 0.7, 0.2, 0.1, 140.0, 1000),
            create_sample(120.0, 0.8, 0.0, 0.2, 120.0, 1200),
            create_sample(140.0, 0.6, 0.0, 0.5, 80.0, 1500), // apex
            create_sample(160.0, 0.3, 0.2, 0.4, 90.0, 1800),
            create_sample(180.0, 0.0, 0.95, 0.2, 110.0, 2100),
        ];

        let metrics = compute_corner_metrics(&corner, &telemetry);

        assert_eq!(metrics.brake_count, 1);
        assert!(metrics.avg_brake_pressure > 0.0);
        assert_eq!(metrics.min_speed, 80.0);
        assert_eq!(metrics.apex_speed, 80.0);
        assert_eq!(metrics.exit_speed, 110.0);
        assert_eq!(metrics.time_in_corner_ms, 1100); // 2100 - 1000
        assert_eq!(metrics.trail_brake_avg_distance, 40.0); // 140 - 100
    }

    #[test]
    fn test_corner_segmentation_deterministic() {
        // Same input should produce identical output (NFR16)
        let telemetry = vec![
            create_sample(0.0, 0.0, 1.0, 0.0, 150.0, 0),
            create_sample(100.0, 0.7, 0.2, 0.1, 140.0, 1000),
            create_sample(140.0, 0.6, 0.0, 0.5, 80.0, 1500),
            create_sample(180.0, 0.0, 0.95, 0.2, 110.0, 2100),
        ];

        let corners1 = segment_corners(&telemetry, 0.05, 0.3, 0.9);
        let corners2 = segment_corners(&telemetry, 0.05, 0.3, 0.9);

        assert_eq!(corners1.len(), corners2.len());
        for (c1, c2) in corners1.iter().zip(corners2.iter()) {
            assert_eq!(c1.corner_id, c2.corner_id);
            assert_eq!(c1.start_distance, c2.start_distance);
            assert_eq!(c1.end_distance, c2.end_distance);
        }
    }
}
