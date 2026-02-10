//! iRacing SDK connection and telemetry capture

pub mod reader;
pub mod types;

pub use reader::IrsdkReader;
pub use types::{ConnectionEvent, ConnectionStatus, IrsdkHeader, IrsdkVarHeader};
