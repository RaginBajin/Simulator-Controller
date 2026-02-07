use std::collections::HashMap;
use std::sync::Arc;

use arrow::array::{BooleanArray, Float64Array, Int32Array, Int64Array};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;

use storage::{Database, NewLap, NewSession};
use storage::validation::{IntegrityReport, STATUS_VALID};
use storage::validation::lap_time::validate_lap_times;
use storage::validation::monotonicity::{compute_lap_sample_counts, has_excessive_violations, validate_lap_distance_monotonicity};
use storage::validation::range_check::{has_excessive_range_violations, validate_channel_ranges};

fn new_session() -> NewSession {
    NewSession {
        track_name: "Spa-Francorchamps".to_string(),
        car_name: "Porsche 911 GT3 R".to_string(),
        session_type: "practice".to_string(),
        started_at: chrono::Utc::now()
            .format("%Y-%m-%dT%H:%M:%S%.3fZ")
            .to_string(),
    }
}

/// Build a valid full-schema telemetry batch where every channel value is within its defined range.
fn make_valid_telemetry_batch(schema: &arrow::datatypes::Schema, num_rows: usize) -> RecordBatch {
    let mut columns: Vec<Arc<dyn arrow::array::Array>> = Vec::new();
    for field in schema.fields() {
        let name = field.name().as_str();
        match field.data_type() {
            DataType::Float64 => {
                let values: Vec<f64> = (0..num_rows)
                    .map(|i| {
                        let t = i as f64 / num_rows.max(1) as f64;
                        match name {
                            // 0-1 range channels
                            "throttle" | "brake" | "clutch" | "brake_bias" | "track_position" => t,
                            // Monotonically increasing lap distance
                            "lap_distance" => i as f64 * 50.0,
                            "speed" => 60.0 + t * 40.0,         // 60-100 m/s
                            "rpm" => 3000.0 + t * 5000.0,       // 3000-8000
                            "steering" => (t - 0.5) * 2.0,      // -1.0 to 1.0
                            "lat_g" | "long_g" => (t - 0.5) * 4.0, // -2 to 2
                            "tire_temp_lf" | "tire_temp_rf" | "tire_temp_lr" | "tire_temp_rr" => {
                                80.0 + t * 20.0
                            }
                            "tire_pressure_lf" | "tire_pressure_rf" | "tire_pressure_lr"
                            | "tire_pressure_rr" => 170.0 + t * 10.0,
                            "fuel_level" => 50.0 - t * 5.0,
                            "oil_temp" | "water_temp" => 90.0 + t * 10.0,
                            "session_time" | "lap_time" => i as f64 * 0.1,
                            _ => t * 10.0, // yaw, pitch, roll, velocity_*, fuel_usage
                        }
                    })
                    .collect();
                columns.push(Arc::new(Float64Array::from(values)));
            }
            DataType::Int32 => {
                columns.push(Arc::new(Int32Array::from(vec![3i32; num_rows])));
            }
            DataType::Int64 => {
                let values: Vec<i64> = (0..num_rows).map(|i| i as i64 * 100).collect();
                columns.push(Arc::new(Int64Array::from(values)));
            }
            DataType::Boolean => {
                columns.push(Arc::new(BooleanArray::from(vec![false; num_rows])));
            }
            _ => panic!("Unexpected data type: {:?}", field.data_type()),
        }
    }
    RecordBatch::try_new(Arc::new(schema.clone()), columns).unwrap()
}

/// Create a minimal telemetry RecordBatch with only the columns needed for testing.
fn make_telemetry_batch(
    lap_distances: Vec<f64>,
    throttles: Vec<f64>,
    brakes: Vec<f64>,
    speeds: Vec<f64>,
) -> RecordBatch {
    let schema = Arc::new(Schema::new(vec![
        Field::new("lap_distance", DataType::Float64, false),
        Field::new("throttle", DataType::Float64, false),
        Field::new("brake", DataType::Float64, false),
        Field::new("speed", DataType::Float64, false),
    ]));

    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(Float64Array::from(lap_distances)),
            Arc::new(Float64Array::from(throttles)),
            Arc::new(Float64Array::from(brakes)),
            Arc::new(Float64Array::from(speeds)),
        ],
    )
    .unwrap()
}

// -- Unit tests for lap time validation --

#[test]
fn test_validate_lap_times_all_valid() {
    let laps = vec![
        storage::LapSummary {
            id: "1".to_string(),
            session_id: "s1".to_string(),
            lap_number: 1,
            lap_time_ms: 120000,
            is_valid: true,
            completion_status: "complete".to_string(),
            created_at: "2025-01-01T00:00:00.000Z".to_string(),
        },
        storage::LapSummary {
            id: "2".to_string(),
            session_id: "s1".to_string(),
            lap_number: 2,
            lap_time_ms: 119500,
            is_valid: true,
            completion_status: "complete".to_string(),
            created_at: "2025-01-01T00:00:00.000Z".to_string(),
        },
    ];

    let violations = validate_lap_times(&laps);
    assert!(violations.is_empty(), "Expected no violations for valid lap times");
}

#[test]
fn test_validate_lap_times_zero_time() {
    let laps = vec![storage::LapSummary {
        id: "1".to_string(),
        session_id: "s1".to_string(),
        lap_number: 1,
        lap_time_ms: 0,
        is_valid: true,
        completion_status: "complete".to_string(),
        created_at: "2025-01-01T00:00:00.000Z".to_string(),
    }];

    let violations = validate_lap_times(&laps);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].lap_number, 1);
    assert!(violations[0].reason.contains("zero or negative"));
}

#[test]
fn test_validate_lap_times_exceeds_max() {
    let laps = vec![storage::LapSummary {
        id: "1".to_string(),
        session_id: "s1".to_string(),
        lap_number: 1,
        lap_time_ms: 700_000, // > 10 minutes
        is_valid: true,
        completion_status: "complete".to_string(),
        created_at: "2025-01-01T00:00:00.000Z".to_string(),
    }];

    let violations = validate_lap_times(&laps);
    assert_eq!(violations.len(), 1);
    assert!(violations[0].reason.contains("exceeds"));
}

// -- Unit tests for monotonicity validation --

#[test]
fn test_monotonicity_clean_data() {
    let batch = make_telemetry_batch(
        vec![0.0, 100.0, 200.0, 300.0, 400.0],
        vec![0.5; 5],
        vec![0.0; 5],
        vec![50.0; 5],
    );

    let violations = validate_lap_distance_monotonicity(&batch);
    assert!(violations.is_empty(), "Monotonically increasing data should have no violations");
}

#[test]
fn test_monotonicity_with_lap_reset() {
    // Simulate lap reset: distance goes from high value back to near-zero
    let batch = make_telemetry_batch(
        vec![0.0, 200.0, 400.0, 600.0, 10.0, 200.0, 400.0],
        vec![0.5; 7],
        vec![0.0; 7],
        vec![50.0; 7],
    );

    let violations = validate_lap_distance_monotonicity(&batch);
    assert!(violations.is_empty(), "Lap resets should not be counted as violations");
}

#[test]
fn test_monotonicity_with_violation() {
    // Distance decreases without a lap reset pattern
    let batch = make_telemetry_batch(
        vec![0.0, 100.0, 200.0, 150.0, 300.0],
        vec![0.5; 5],
        vec![0.0; 5],
        vec![50.0; 5],
    );

    let violations = validate_lap_distance_monotonicity(&batch);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].sample_index, 3);
    assert_eq!(violations[0].previous_value, 200.0);
    assert_eq!(violations[0].violating_value, 150.0);
}

#[test]
fn test_excessive_violations_threshold() {
    // 3 violations in lap 1 (100 samples) = 3% > 2% threshold
    let mut lap_samples = HashMap::new();
    lap_samples.insert(1u32, 100usize);

    assert!(has_excessive_violations(
        &vec![
            storage::validation::MonotonicityViolation {
                lap_number: 1,
                sample_index: 10,
                previous_value: 100.0,
                violating_value: 90.0,
            },
            storage::validation::MonotonicityViolation {
                lap_number: 1,
                sample_index: 20,
                previous_value: 200.0,
                violating_value: 190.0,
            },
            storage::validation::MonotonicityViolation {
                lap_number: 1,
                sample_index: 30,
                previous_value: 300.0,
                violating_value: 290.0,
            },
        ],
        &lap_samples,
        100,
    ));

    // 1 violation in lap 1 (100 samples) = 1% < 2% threshold
    assert!(!has_excessive_violations(
        &vec![storage::validation::MonotonicityViolation {
            lap_number: 1,
            sample_index: 10,
            previous_value: 100.0,
            violating_value: 90.0,
        }],
        &lap_samples,
        100,
    ));
}

#[test]
fn test_excessive_violations_per_lap_check() {
    // 3 violations spread across 2 laps (200 total samples)
    // Lap 1: 100 samples, 1 violation = 1% (below 2%)
    // Lap 2: 100 samples, 2 violations = 2% (at threshold, not exceeded)
    let mut lap_samples = HashMap::new();
    lap_samples.insert(1u32, 100usize);
    lap_samples.insert(2u32, 100usize);

    assert!(!has_excessive_violations(
        &vec![
            storage::validation::MonotonicityViolation {
                lap_number: 1,
                sample_index: 10,
                previous_value: 100.0,
                violating_value: 90.0,
            },
            storage::validation::MonotonicityViolation {
                lap_number: 2,
                sample_index: 110,
                previous_value: 200.0,
                violating_value: 190.0,
            },
            storage::validation::MonotonicityViolation {
                lap_number: 2,
                sample_index: 120,
                previous_value: 300.0,
                violating_value: 290.0,
            },
        ],
        &lap_samples,
        200,
    ));

    // Lap 2 has 3 violations in 100 samples = 3% > 2% -- should flag
    assert!(has_excessive_violations(
        &vec![
            storage::validation::MonotonicityViolation {
                lap_number: 2,
                sample_index: 110,
                previous_value: 200.0,
                violating_value: 190.0,
            },
            storage::validation::MonotonicityViolation {
                lap_number: 2,
                sample_index: 120,
                previous_value: 300.0,
                violating_value: 290.0,
            },
            storage::validation::MonotonicityViolation {
                lap_number: 2,
                sample_index: 130,
                previous_value: 400.0,
                violating_value: 390.0,
            },
        ],
        &lap_samples,
        200,
    ));
}

// -- Unit tests for range validation --

#[test]
fn test_range_check_clean_data() {
    let batch = make_telemetry_batch(
        vec![0.0, 100.0, 200.0],
        vec![0.5, 0.8, 1.0],
        vec![0.0, 0.3, 0.5],
        vec![50.0, 80.0, 100.0],
    );

    let violations = validate_channel_ranges(&batch);
    assert!(violations.is_empty(), "In-range data should have no violations");
}

#[test]
fn test_range_check_out_of_range() {
    let batch = make_telemetry_batch(
        vec![0.0, 100.0, 200.0],
        vec![0.5, 1.5, 0.8], // throttle 1.5 is out of 0-1 range
        vec![0.0, 0.3, 0.5],
        vec![50.0, 80.0, 130.0], // speed 130 is out of 0-120 range
    );

    let violations = validate_channel_ranges(&batch);
    assert!(violations.contains_key("throttle"), "Should flag throttle out-of-range");
    assert!(violations.contains_key("speed"), "Should flag speed out-of-range");
    assert_eq!(violations["throttle"].out_of_range_count, 1);
    assert_eq!(violations["speed"].out_of_range_count, 1);
}

#[test]
fn test_range_check_nan_values() {
    let batch = make_telemetry_batch(
        vec![0.0, f64::NAN, 200.0],
        vec![0.5, 0.8, 1.0],
        vec![0.0, 0.3, 0.5],
        vec![50.0, 80.0, 100.0],
    );

    let violations = validate_channel_ranges(&batch);
    // lap_distance has no defined range, but NaN should still be flagged
    assert!(
        violations.contains_key("lap_distance"),
        "Should flag NaN values even without defined range"
    );
}

#[test]
fn test_excessive_range_violations() {
    let mut violations = HashMap::new();
    violations.insert(
        "throttle".to_string(),
        storage::validation::RangeViolation {
            channel_name: "throttle".to_string(),
            out_of_range_count: 6, // 6% > 5% threshold
            total_samples: 100,
            min_seen: -0.1,
            max_seen: 1.5,
        },
    );
    assert!(has_excessive_range_violations(&violations));

    // Below threshold
    let mut violations2 = HashMap::new();
    violations2.insert(
        "throttle".to_string(),
        storage::validation::RangeViolation {
            channel_name: "throttle".to_string(),
            out_of_range_count: 4, // 4% < 5% threshold
            total_samples: 100,
            min_seen: -0.1,
            max_seen: 1.5,
        },
    );
    assert!(!has_excessive_range_violations(&violations2));
}

// -- Integration tests for session validation orchestrator --

#[tokio::test]
async fn test_validate_session_without_telemetry() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db = Database::init(&db_path).await.unwrap();

    let session = db.insert_session(&new_session()).await.unwrap();

    // Add valid laps
    db.insert_lap(&NewLap {
        session_id: session.id.clone(),
        lap_number: 1,
        lap_time_ms: 120000,
        is_valid: true,
        completion_status: "complete".to_string(),
    })
    .await
    .unwrap();

    let report = db.validate_session_integrity(&session.id).await.unwrap();
    assert_eq!(report.status, STATUS_VALID);
    assert!(report.checksum_valid.is_none(), "No telemetry means no checksum check");
    assert!(report.monotonicity_violations.is_empty());
    assert!(report.range_violations.is_empty());
    assert!(report.lap_time_violations.is_empty());
}

#[tokio::test]
async fn test_validate_session_with_invalid_laps() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db = Database::init(&db_path).await.unwrap();

    let session = db.insert_session(&new_session()).await.unwrap();

    // Add a lap with zero time (invalid)
    db.insert_lap(&NewLap {
        session_id: session.id.clone(),
        lap_number: 1,
        lap_time_ms: 0,
        is_valid: true,
        completion_status: "complete".to_string(),
    })
    .await
    .unwrap();

    let report = db.validate_session_integrity(&session.id).await.unwrap();
    assert_eq!(report.lap_time_violations.len(), 1);
    assert_eq!(report.lap_time_violations[0].lap_number, 1);

    // Verify the lap was flagged as invalid in the database
    let laps = db.get_laps_for_session(&session.id).await.unwrap();
    assert_eq!(laps[0].completion_status, "invalid");
}

#[tokio::test]
async fn test_validate_session_persists_integrity_status() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db = Database::init(&db_path).await.unwrap();

    let session = db.insert_session(&new_session()).await.unwrap();

    // Session should start as not validated
    assert!(
        session.integrity_status.is_none()
            || session.integrity_status.as_deref() == Some("not_validated")
    );

    let report = db.validate_session_integrity(&session.id).await.unwrap();
    assert_eq!(report.status, STATUS_VALID);

    // Re-fetch and verify persisted status
    let updated = db.get_session(&session.id).await.unwrap().unwrap();
    assert_eq!(updated.integrity_status.as_deref(), Some(STATUS_VALID));
    assert!(updated.integrity_details.is_some());
    assert!(updated.integrity_validated_at.is_some());
}

#[tokio::test]
async fn test_validate_session_with_telemetry() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db = Database::init(&db_path).await.unwrap();

    let session = db.insert_session(&new_session()).await.unwrap();

    // Write valid telemetry with channel-appropriate values
    let schema = storage::telemetry_schema();
    let num_rows = 10;
    let telemetry_batch = make_valid_telemetry_batch(&schema, num_rows);

    let telemetry_dir = dir.path().join("telemetry");
    std::fs::create_dir_all(&telemetry_dir).unwrap();

    let _write_result = db
        .write_telemetry(&session.id, &telemetry_dir, &telemetry_batch, HashMap::new())
        .await
        .unwrap();

    // Add a valid lap
    db.insert_lap(&NewLap {
        session_id: session.id.clone(),
        lap_number: 1,
        lap_time_ms: 120000,
        is_valid: true,
        completion_status: "complete".to_string(),
    })
    .await
    .unwrap();

    let report = db.validate_session_integrity(&session.id).await.unwrap();
    assert_eq!(report.checksum_valid, Some(true));
    assert_eq!(report.status, STATUS_VALID);
}

#[tokio::test]
async fn test_validate_unvalidated_sessions() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db = Database::init(&db_path).await.unwrap();

    // Create 3 sessions
    for _ in 0..3 {
        let session = db.insert_session(&new_session()).await.unwrap();
        db.insert_lap(&NewLap {
            session_id: session.id.clone(),
            lap_number: 1,
            lap_time_ms: 120000,
            is_valid: true,
            completion_status: "complete".to_string(),
        })
        .await
        .unwrap();
    }

    // Validate all unvalidated
    let count = db.validate_unvalidated_sessions().await.unwrap();
    // Sessions without telemetry_path won't be picked up by validate_unvalidated_sessions
    // since the query filters on telemetry_path IS NOT NULL
    assert_eq!(count, 0, "Sessions without telemetry are excluded from batch validation");
}

#[tokio::test]
async fn test_validate_session_not_found() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db = Database::init(&db_path).await.unwrap();

    let result = db.validate_session_integrity("nonexistent-id").await;
    assert!(result.is_err(), "Should error for nonexistent session");
}

#[test]
fn test_empty_batch_range_check() {
    let schema = Arc::new(Schema::new(vec![
        Field::new("throttle", DataType::Float64, false),
    ]));
    let batch = RecordBatch::try_new(
        schema,
        vec![Arc::new(Float64Array::from(Vec::<f64>::new()))],
    )
    .unwrap();

    let violations = validate_channel_ranges(&batch);
    assert!(violations.is_empty(), "Empty batch should have no violations");
}

#[test]
fn test_empty_batch_monotonicity() {
    let schema = Arc::new(Schema::new(vec![
        Field::new("lap_distance", DataType::Float64, false),
    ]));
    let batch = RecordBatch::try_new(
        schema,
        vec![Arc::new(Float64Array::from(Vec::<f64>::new()))],
    )
    .unwrap();

    let violations = validate_lap_distance_monotonicity(&batch);
    assert!(violations.is_empty(), "Empty batch should have no violations");
}

#[test]
fn test_integrity_report_serialization() {
    let report = IntegrityReport {
        status: STATUS_VALID.to_string(),
        checksum_valid: Some(true),
        monotonicity_violations: vec![],
        range_violations: HashMap::new(),
        lap_time_violations: vec![],
        validated_at: "2025-01-01T00:00:00.000Z".to_string(),
    };

    let json = serde_json::to_string(&report).unwrap();
    assert!(json.contains("\"status\":\"valid\""));
    assert!(json.contains("\"checksumValid\":true"));

    // Roundtrip
    let deserialized: IntegrityReport = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.status, STATUS_VALID);
}
