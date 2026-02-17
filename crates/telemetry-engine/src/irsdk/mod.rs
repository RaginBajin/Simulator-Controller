//! iRacing SDK connection and telemetry capture

use std::collections::HashMap;

pub mod reader;
pub mod reconnection;
pub mod types;

pub use reader::IrsdkReader;
pub use reconnection::{
    ConnectionState, ReconnectionManager, ReconnectionStatus, MAX_RECONNECTION_ATTEMPTS,
    RECONNECTION_INTERVAL,
};
pub use types::{ConnectionEvent, ConnectionStatus, IrsdkHeader, IrsdkVarHeader};

/// Channel name mapping from IRSDK variable names to canonical schema names.
/// Shared between .ibt importer and live capture engine.
pub fn irsdk_channel_mapping() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("Speed", "speed");
    m.insert("Throttle", "throttle");
    m.insert("Brake", "brake");
    m.insert("SteeringWheelAngle", "steering");
    m.insert("Clutch", "clutch");
    m.insert("Gear", "gear");
    m.insert("RPM", "rpm");
    m.insert("LatAccel", "lat_g");
    m.insert("LongAccel", "long_g");
    m.insert("Yaw", "yaw");
    m.insert("Pitch", "pitch");
    m.insert("Roll", "roll");
    m.insert("VelocityX", "velocity_x");
    m.insert("VelocityY", "velocity_y");
    m.insert("VelocityZ", "velocity_z");
    m.insert("LFtempCL", "tire_temp_lf");
    m.insert("RFtempCL", "tire_temp_rf");
    m.insert("LRtempCL", "tire_temp_lr");
    m.insert("RRtempCL", "tire_temp_rr");
    m.insert("LFpressure", "tire_pressure_lf");
    m.insert("RFpressure", "tire_pressure_rf");
    m.insert("LRpressure", "tire_pressure_lr");
    m.insert("RRpressure", "tire_pressure_rr");
    m.insert("FuelLevel", "fuel_level");
    m.insert("FuelUsePerHour", "fuel_usage");
    m.insert("OilTemp", "oil_temp");
    m.insert("WaterTemp", "water_temp");
    m.insert("dcBrakeBias", "brake_bias");
    m.insert("BrakeABSactive", "abs_active");
    m.insert("dcTractionControl", "tc_active");
    m.insert("LapDistPct", "track_position");
    m.insert("LapDist", "lap_distance");
    m.insert("LapCurrentLapTime", "lap_time");
    m.insert("SessionTime", "session_time");
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_mapping_has_all_channels() {
        let mapping = irsdk_channel_mapping();

        // Verify we have mappings for all essential channels
        assert!(mapping.contains_key("Speed"));
        assert!(mapping.contains_key("Throttle"));
        assert!(mapping.contains_key("Brake"));
        assert!(mapping.contains_key("Gear"));
        assert!(mapping.contains_key("SessionTime"));

        // Should have at least 28 mappings (we have 35 total channels, some might be computed)
        assert!(mapping.len() >= 28);
    }

    #[test]
    fn test_channel_mapping_values_match_schema() {
        let mapping = irsdk_channel_mapping();

        // Verify canonical names match expected schema
        assert_eq!(mapping.get("Speed"), Some(&"speed"));
        assert_eq!(mapping.get("Throttle"), Some(&"throttle"));
        assert_eq!(mapping.get("Brake"), Some(&"brake"));
        assert_eq!(mapping.get("SteeringWheelAngle"), Some(&"steering"));
    }
}
