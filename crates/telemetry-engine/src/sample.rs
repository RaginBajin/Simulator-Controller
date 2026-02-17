//! Telemetry sample data structures
//!
//! Defines TelemetrySample struct holding all 35 channels and session context,
//! plus conversion utilities to Arrow RecordBatch for Parquet storage.

use arrow::array::{
    ArrayRef, BooleanBuilder, Float64Builder, Int32Builder, Int64Builder, RecordBatch,
};
use arrow::datatypes::Schema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Session state enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionState {
    /// Actively driving
    Driving,
    /// In pit lane or pit box
    Pitting,
    /// Spectating
    Spectating,
    /// Invalid or unknown state
    Invalid,
}

/// Telemetry sample containing all 35 canonical channels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetrySample {
    // Core telemetry (35 channels per AC #2)
    pub timestamp_ms: i64,
    pub session_time: Option<f64>,
    pub lap_distance: Option<f64>,
    pub lap_time: Option<f64>,
    pub speed: Option<f64>,
    pub throttle: Option<f64>,
    pub brake: Option<f64>,
    pub steering: Option<f64>,
    pub clutch: Option<f64>,
    pub gear: Option<i32>,
    pub rpm: Option<f64>,
    pub lat_g: Option<f64>,
    pub long_g: Option<f64>,
    pub yaw: Option<f64>,
    pub pitch: Option<f64>,
    pub roll: Option<f64>,
    pub velocity_x: Option<f64>,
    pub velocity_y: Option<f64>,
    pub velocity_z: Option<f64>,
    pub tire_temp_lf: Option<f64>,
    pub tire_temp_rf: Option<f64>,
    pub tire_temp_lr: Option<f64>,
    pub tire_temp_rr: Option<f64>,
    pub tire_pressure_lf: Option<f64>,
    pub tire_pressure_rf: Option<f64>,
    pub tire_pressure_lr: Option<f64>,
    pub tire_pressure_rr: Option<f64>,
    pub fuel_level: Option<f64>,
    pub fuel_usage: Option<f64>,
    pub oil_temp: Option<f64>,
    pub water_temp: Option<f64>,
    pub brake_bias: Option<f64>,
    pub abs_active: Option<bool>,
    pub tc_active: Option<bool>,
    pub track_position: Option<f64>,

    // Session context (AC #4)
    pub session_state: SessionState,
    pub track_temp: Option<f64>,
    pub air_temp: Option<f64>,
    pub weather: Option<String>,
    pub session_type: Option<String>,
    pub car_class: Option<String>,
    pub track_config: Option<String>,
}

impl TelemetrySample {
    /// Create a new sample with minimal required fields
    pub fn new(timestamp_ms: i64) -> Self {
        Self {
            timestamp_ms,
            session_time: None,
            lap_distance: None,
            lap_time: None,
            speed: None,
            throttle: None,
            brake: None,
            steering: None,
            clutch: None,
            gear: None,
            rpm: None,
            lat_g: None,
            long_g: None,
            yaw: None,
            pitch: None,
            roll: None,
            velocity_x: None,
            velocity_y: None,
            velocity_z: None,
            tire_temp_lf: None,
            tire_temp_rf: None,
            tire_temp_lr: None,
            tire_temp_rr: None,
            tire_pressure_lf: None,
            tire_pressure_rf: None,
            tire_pressure_lr: None,
            tire_pressure_rr: None,
            fuel_level: None,
            fuel_usage: None,
            oil_temp: None,
            water_temp: None,
            brake_bias: None,
            abs_active: None,
            tc_active: None,
            track_position: None,
            session_state: SessionState::Invalid,
            track_temp: None,
            air_temp: None,
            weather: None,
            session_type: None,
            car_class: None,
            track_config: None,
        }
    }

    /// Convert a batch of samples to an Arrow RecordBatch
    /// This is used for writing to Parquet via the storage crate
    pub fn samples_to_record_batch(
        samples: &[TelemetrySample],
        schema: &Schema,
    ) -> Result<RecordBatch, arrow::error::ArrowError> {
        if samples.is_empty() {
            return Err(arrow::error::ArrowError::InvalidArgumentError(
                "Cannot convert empty sample batch".to_string(),
            ));
        }

        let num_samples = samples.len();

        // Build timestamp_ms (non-nullable)
        let mut timestamp_builder = Int64Builder::with_capacity(num_samples);
        for sample in samples {
            timestamp_builder.append_value(sample.timestamp_ms);
        }

        // Build all nullable float64 channels
        let mut session_time_builder = Float64Builder::with_capacity(num_samples);
        let mut lap_distance_builder = Float64Builder::with_capacity(num_samples);
        let mut lap_time_builder = Float64Builder::with_capacity(num_samples);
        let mut speed_builder = Float64Builder::with_capacity(num_samples);
        let mut throttle_builder = Float64Builder::with_capacity(num_samples);
        let mut brake_builder = Float64Builder::with_capacity(num_samples);
        let mut steering_builder = Float64Builder::with_capacity(num_samples);
        let mut clutch_builder = Float64Builder::with_capacity(num_samples);
        let mut rpm_builder = Float64Builder::with_capacity(num_samples);
        let mut lat_g_builder = Float64Builder::with_capacity(num_samples);
        let mut long_g_builder = Float64Builder::with_capacity(num_samples);
        let mut yaw_builder = Float64Builder::with_capacity(num_samples);
        let mut pitch_builder = Float64Builder::with_capacity(num_samples);
        let mut roll_builder = Float64Builder::with_capacity(num_samples);
        let mut velocity_x_builder = Float64Builder::with_capacity(num_samples);
        let mut velocity_y_builder = Float64Builder::with_capacity(num_samples);
        let mut velocity_z_builder = Float64Builder::with_capacity(num_samples);
        let mut tire_temp_lf_builder = Float64Builder::with_capacity(num_samples);
        let mut tire_temp_rf_builder = Float64Builder::with_capacity(num_samples);
        let mut tire_temp_lr_builder = Float64Builder::with_capacity(num_samples);
        let mut tire_temp_rr_builder = Float64Builder::with_capacity(num_samples);
        let mut tire_pressure_lf_builder = Float64Builder::with_capacity(num_samples);
        let mut tire_pressure_rf_builder = Float64Builder::with_capacity(num_samples);
        let mut tire_pressure_lr_builder = Float64Builder::with_capacity(num_samples);
        let mut tire_pressure_rr_builder = Float64Builder::with_capacity(num_samples);
        let mut fuel_level_builder = Float64Builder::with_capacity(num_samples);
        let mut fuel_usage_builder = Float64Builder::with_capacity(num_samples);
        let mut oil_temp_builder = Float64Builder::with_capacity(num_samples);
        let mut water_temp_builder = Float64Builder::with_capacity(num_samples);
        let mut brake_bias_builder = Float64Builder::with_capacity(num_samples);
        let mut track_position_builder = Float64Builder::with_capacity(num_samples);

        // Build gear (nullable int32)
        let mut gear_builder = Int32Builder::with_capacity(num_samples);

        // Build abs_active and tc_active (nullable boolean)
        let mut abs_active_builder = BooleanBuilder::with_capacity(num_samples);
        let mut tc_active_builder = BooleanBuilder::with_capacity(num_samples);

        for sample in samples {
            session_time_builder.append_option(sample.session_time);
            lap_distance_builder.append_option(sample.lap_distance);
            lap_time_builder.append_option(sample.lap_time);
            speed_builder.append_option(sample.speed);
            throttle_builder.append_option(sample.throttle);
            brake_builder.append_option(sample.brake);
            steering_builder.append_option(sample.steering);
            clutch_builder.append_option(sample.clutch);
            gear_builder.append_option(sample.gear);
            rpm_builder.append_option(sample.rpm);
            lat_g_builder.append_option(sample.lat_g);
            long_g_builder.append_option(sample.long_g);
            yaw_builder.append_option(sample.yaw);
            pitch_builder.append_option(sample.pitch);
            roll_builder.append_option(sample.roll);
            velocity_x_builder.append_option(sample.velocity_x);
            velocity_y_builder.append_option(sample.velocity_y);
            velocity_z_builder.append_option(sample.velocity_z);
            tire_temp_lf_builder.append_option(sample.tire_temp_lf);
            tire_temp_rf_builder.append_option(sample.tire_temp_rf);
            tire_temp_lr_builder.append_option(sample.tire_temp_lr);
            tire_temp_rr_builder.append_option(sample.tire_temp_rr);
            tire_pressure_lf_builder.append_option(sample.tire_pressure_lf);
            tire_pressure_rf_builder.append_option(sample.tire_pressure_rf);
            tire_pressure_lr_builder.append_option(sample.tire_pressure_lr);
            tire_pressure_rr_builder.append_option(sample.tire_pressure_rr);
            fuel_level_builder.append_option(sample.fuel_level);
            fuel_usage_builder.append_option(sample.fuel_usage);
            oil_temp_builder.append_option(sample.oil_temp);
            water_temp_builder.append_option(sample.water_temp);
            brake_bias_builder.append_option(sample.brake_bias);
            abs_active_builder.append_option(sample.abs_active);
            tc_active_builder.append_option(sample.tc_active);
            track_position_builder.append_option(sample.track_position);
        }

        // Build column arrays in schema order
        let columns: Vec<ArrayRef> = vec![
            Arc::new(timestamp_builder.finish()),
            Arc::new(session_time_builder.finish()),
            Arc::new(lap_distance_builder.finish()),
            Arc::new(lap_time_builder.finish()),
            Arc::new(speed_builder.finish()),
            Arc::new(throttle_builder.finish()),
            Arc::new(brake_builder.finish()),
            Arc::new(steering_builder.finish()),
            Arc::new(clutch_builder.finish()),
            Arc::new(gear_builder.finish()),
            Arc::new(rpm_builder.finish()),
            Arc::new(lat_g_builder.finish()),
            Arc::new(long_g_builder.finish()),
            Arc::new(yaw_builder.finish()),
            Arc::new(pitch_builder.finish()),
            Arc::new(roll_builder.finish()),
            Arc::new(velocity_x_builder.finish()),
            Arc::new(velocity_y_builder.finish()),
            Arc::new(velocity_z_builder.finish()),
            Arc::new(tire_temp_lf_builder.finish()),
            Arc::new(tire_temp_rf_builder.finish()),
            Arc::new(tire_temp_lr_builder.finish()),
            Arc::new(tire_temp_rr_builder.finish()),
            Arc::new(tire_pressure_lf_builder.finish()),
            Arc::new(tire_pressure_rf_builder.finish()),
            Arc::new(tire_pressure_lr_builder.finish()),
            Arc::new(tire_pressure_rr_builder.finish()),
            Arc::new(fuel_level_builder.finish()),
            Arc::new(fuel_usage_builder.finish()),
            Arc::new(oil_temp_builder.finish()),
            Arc::new(water_temp_builder.finish()),
            Arc::new(brake_bias_builder.finish()),
            Arc::new(abs_active_builder.finish()),
            Arc::new(tc_active_builder.finish()),
            Arc::new(track_position_builder.finish()),
        ];

        RecordBatch::try_new(Arc::new(schema.clone()), columns)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_creation() {
        let sample = TelemetrySample::new(1000);
        assert_eq!(sample.timestamp_ms, 1000);
        assert_eq!(sample.session_state, SessionState::Invalid);
        assert!(sample.speed.is_none());
    }

    #[test]
    fn test_sample_with_data() {
        let mut sample = TelemetrySample::new(2000);
        sample.speed = Some(120.5);
        sample.throttle = Some(0.85);
        sample.gear = Some(4);
        sample.session_state = SessionState::Driving;

        assert_eq!(sample.timestamp_ms, 2000);
        assert_eq!(sample.speed, Some(120.5));
        assert_eq!(sample.throttle, Some(0.85));
        assert_eq!(sample.gear, Some(4));
        assert_eq!(sample.session_state, SessionState::Driving);
    }

    #[test]
    fn test_session_state_variants() {
        assert_ne!(SessionState::Driving, SessionState::Pitting);
        assert_ne!(SessionState::Driving, SessionState::Spectating);
        assert_ne!(SessionState::Driving, SessionState::Invalid);
    }

    #[test]
    fn test_samples_to_record_batch_empty() {
        use storage::parquet::schema::telemetry_schema;
        let schema = telemetry_schema();
        let result = TelemetrySample::samples_to_record_batch(&[], &schema);
        assert!(result.is_err());
    }

    #[test]
    fn test_samples_to_record_batch_single() {
        use storage::parquet::schema::telemetry_schema;
        let schema = telemetry_schema();

        let mut sample = TelemetrySample::new(1000);
        sample.speed = Some(120.5);
        sample.throttle = Some(0.85);
        sample.gear = Some(4);

        let batch = TelemetrySample::samples_to_record_batch(&[sample], &schema).unwrap();
        assert_eq!(batch.num_rows(), 1);
        assert_eq!(batch.num_columns(), 35);
    }

    #[test]
    fn test_samples_to_record_batch_multiple() {
        use storage::parquet::schema::telemetry_schema;
        let schema = telemetry_schema();

        let samples = vec![
            {
                let mut s = TelemetrySample::new(1000);
                s.speed = Some(120.5);
                s.throttle = Some(0.85);
                s
            },
            {
                let mut s = TelemetrySample::new(1016); // ~60Hz = 16ms
                s.speed = Some(121.0);
                s.throttle = Some(0.90);
                s
            },
            {
                let mut s = TelemetrySample::new(1032);
                s.speed = Some(121.5);
                s.throttle = Some(0.95);
                s
            },
        ];

        let batch = TelemetrySample::samples_to_record_batch(&samples, &schema).unwrap();
        assert_eq!(batch.num_rows(), 3);
        assert_eq!(batch.num_columns(), 35);

        // Verify schema matches
        assert_eq!(batch.schema().fields().len(), 35);
    }

    #[test]
    fn test_samples_to_record_batch_with_nulls() {
        use storage::parquet::schema::telemetry_schema;
        let schema = telemetry_schema();

        let mut sample = TelemetrySample::new(2000);
        sample.speed = Some(100.0);
        // Leave other fields as None

        let batch = TelemetrySample::samples_to_record_batch(&[sample], &schema).unwrap();
        assert_eq!(batch.num_rows(), 1);
        assert_eq!(batch.num_columns(), 35);
    }
}
