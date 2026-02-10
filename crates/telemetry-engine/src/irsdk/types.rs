//! IRSDK data structures and types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Connection status of the IRSDK connection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionStatus {
    /// Not connected to IRSDK
    Disconnected,
    /// Connected to IRSDK
    Connected,
}

/// Connection event with timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionEvent {
    /// Current connection status
    pub status: ConnectionStatus,
    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,
}

/// IRSDK shared memory header structure
/// Matches the layout from iRacing's shared memory
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct IrsdkHeader {
    /// Version of the IRSDK header
    pub version: i32,
    /// Status flags
    pub status: i32,
    /// Tick rate in Hz
    pub tick_rate: i32,
    /// Session info update count
    pub session_info_update: i32,
    /// Length of session info YAML
    pub session_info_len: i32,
    /// Offset to session info YAML
    pub session_info_offset: i32,
    /// Number of variable headers
    pub num_vars: i32,
    /// Offset to variable headers
    pub var_header_offset: i32,
    /// Number of telemetry buffers
    pub num_buf: i32,
    /// Length of each buffer
    pub buf_len: i32,
    /// Padding to ensure alignment
    _pad1: [i32; 2],
    /// Buffer info array (triple buffered)
    pub var_buf: [VarBuf; 4],
}

/// Variable buffer information
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VarBuf {
    /// Tick count for this buffer
    pub tick_count: i32,
    /// Offset to buffer data
    pub buf_offset: i32,
    /// Padding for alignment
    _pad: [i32; 2],
}

/// IRSDK variable header
#[repr(C)]
#[derive(Debug, Clone)]
pub struct IrsdkVarHeader {
    /// Variable type (0=char, 1=bool, 2=int, 3=bitfield, 4=float, 5=double)
    pub var_type: i32,
    /// Offset to data in telemetry buffer
    pub offset: i32,
    /// Number of entries (array size)
    pub count: i32,
    /// Count as time flag
    pub count_as_time: bool,
    /// Padding
    _pad: [u8; 3],
    /// Variable name (null-terminated)
    pub name: [u8; 32],
    /// Variable description (null-terminated)
    pub desc: [u8; 64],
    /// Variable unit (null-terminated)
    pub unit: [u8; 32],
}
