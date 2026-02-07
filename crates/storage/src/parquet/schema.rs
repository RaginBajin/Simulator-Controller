use arrow::datatypes::{DataType, Field, Schema};
use std::collections::HashMap;

pub const SCHEMA_VERSION: &str = "1";

/// All 35 telemetry channel names in canonical order.
pub const TELEMETRY_CHANNELS: &[&str] = &[
    "timestamp_ms",
    "session_time",
    "lap_distance",
    "lap_time",
    "speed",
    "throttle",
    "brake",
    "steering",
    "clutch",
    "gear",
    "rpm",
    "lat_g",
    "long_g",
    "yaw",
    "pitch",
    "roll",
    "velocity_x",
    "velocity_y",
    "velocity_z",
    "tire_temp_lf",
    "tire_temp_rf",
    "tire_temp_lr",
    "tire_temp_rr",
    "tire_pressure_lf",
    "tire_pressure_rf",
    "tire_pressure_lr",
    "tire_pressure_rr",
    "fuel_level",
    "fuel_usage",
    "oil_temp",
    "water_temp",
    "brake_bias",
    "abs_active",
    "tc_active",
    "track_position",
];

/// Returns the canonical Arrow schema for telemetry data with all 35 channels.
pub fn telemetry_schema() -> Schema {
    let metadata = HashMap::from([
        ("schema_version".to_string(), SCHEMA_VERSION.to_string()),
    ]);

    Schema::new_with_metadata(
        vec![
            Field::new("timestamp_ms", DataType::Int64, false),
            Field::new("session_time", DataType::Float64, true),
            Field::new("lap_distance", DataType::Float64, true),
            Field::new("lap_time", DataType::Float64, true),
            Field::new("speed", DataType::Float64, true),
            Field::new("throttle", DataType::Float64, true),
            Field::new("brake", DataType::Float64, true),
            Field::new("steering", DataType::Float64, true),
            Field::new("clutch", DataType::Float64, true),
            Field::new("gear", DataType::Int32, true),
            Field::new("rpm", DataType::Float64, true),
            Field::new("lat_g", DataType::Float64, true),
            Field::new("long_g", DataType::Float64, true),
            Field::new("yaw", DataType::Float64, true),
            Field::new("pitch", DataType::Float64, true),
            Field::new("roll", DataType::Float64, true),
            Field::new("velocity_x", DataType::Float64, true),
            Field::new("velocity_y", DataType::Float64, true),
            Field::new("velocity_z", DataType::Float64, true),
            Field::new("tire_temp_lf", DataType::Float64, true),
            Field::new("tire_temp_rf", DataType::Float64, true),
            Field::new("tire_temp_lr", DataType::Float64, true),
            Field::new("tire_temp_rr", DataType::Float64, true),
            Field::new("tire_pressure_lf", DataType::Float64, true),
            Field::new("tire_pressure_rf", DataType::Float64, true),
            Field::new("tire_pressure_lr", DataType::Float64, true),
            Field::new("tire_pressure_rr", DataType::Float64, true),
            Field::new("fuel_level", DataType::Float64, true),
            Field::new("fuel_usage", DataType::Float64, true),
            Field::new("oil_temp", DataType::Float64, true),
            Field::new("water_temp", DataType::Float64, true),
            Field::new("brake_bias", DataType::Float64, true),
            Field::new("abs_active", DataType::Boolean, true),
            Field::new("tc_active", DataType::Boolean, true),
            Field::new("track_position", DataType::Float64, true),
        ],
        metadata,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_has_35_channels() {
        let schema = telemetry_schema();
        assert_eq!(schema.fields().len(), 35);
        assert_eq!(TELEMETRY_CHANNELS.len(), 35);
    }

    #[test]
    fn schema_has_version_metadata() {
        let schema = telemetry_schema();
        assert_eq!(
            schema.metadata().get("schema_version"),
            Some(&"1".to_string())
        );
    }

    #[test]
    fn channel_names_match_schema_fields() {
        let schema = telemetry_schema();
        for (i, name) in TELEMETRY_CHANNELS.iter().enumerate() {
            assert_eq!(schema.field(i).name(), *name);
        }
    }
}
