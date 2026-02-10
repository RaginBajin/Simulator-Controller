//! iRacing SDK connection and telemetry capture

pub mod reconnection;

pub use reconnection::{ConnectionState, ReconnectionManager, ReconnectionStatus, MAX_RECONNECTION_ATTEMPTS, RECONNECTION_INTERVAL};
