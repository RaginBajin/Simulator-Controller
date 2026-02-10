//! IRSDK shared memory reader
//!
//! Provides access to iRacing's shared memory for real-time telemetry capture.
//! Windows-only implementation with cross-platform stubs for development.

use super::types::IrsdkHeader;
use std::io;

#[cfg(target_os = "windows")]
use windows::Win32::{
    Foundation::{CloseHandle, HANDLE},
    System::Memory::{
        MapViewOfFile, OpenFileMappingW, UnmapViewOfFile, FILE_MAP_READ, MEMORY_MAPPED_VIEW_ADDRESS,
    },
};

#[cfg(target_os = "windows")]
use std::ffi::c_void;

/// IRSDK shared memory file name
#[cfg(target_os = "windows")]
const IRSDK_MEM_MAP_FILE_NAME: &str = "Local\\IRSDKMemMapFileName";

/// IRSDK shared memory reader
pub struct IrsdkReader {
    #[cfg(target_os = "windows")]
    handle: Option<HANDLE>,
    #[cfg(target_os = "windows")]
    mapped_view: Option<MEMORY_MAPPED_VIEW_ADDRESS>,
}

impl IrsdkReader {
    /// Create a new IRSDK reader
    pub fn new() -> Self {
        Self {
            #[cfg(target_os = "windows")]
            handle: None,
            #[cfg(target_os = "windows")]
            mapped_view: None,
        }
    }

    /// Check if connected to IRSDK shared memory
    #[cfg(target_os = "windows")]
    pub fn is_connected(&mut self) -> bool {
        // Try to open shared memory if not already open
        if self.handle.is_none() {
            if let Ok(_) = self.open_shared_memory() {
                return true;
            }
            return false;
        }

        // Already have a handle, verify it's still valid by reading header
        if self.read_header().is_ok() {
            true
        } else {
            // Connection lost - cleanup handles to prevent leak
            self.close();
            false
        }
    }

    /// Close the shared memory connection and cleanup handles
    #[cfg(target_os = "windows")]
    fn close(&mut self) {
        unsafe {
            if let Some(mapped_view) = self.mapped_view.take() {
                UnmapViewOfFile(mapped_view).ok();
            }
            if let Some(handle) = self.handle.take() {
                CloseHandle(handle).ok();
            }
        }
    }

    /// Check if connected to IRSDK shared memory (stub for non-Windows)
    #[cfg(not(target_os = "windows"))]
    pub fn is_connected(&mut self) -> bool {
        false
    }

    /// Open the IRSDK shared memory
    #[cfg(target_os = "windows")]
    fn open_shared_memory(&mut self) -> io::Result<()> {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;

        // Convert name to wide string
        let wide_name: Vec<u16> = OsStr::new(IRSDK_MEM_MAP_FILE_NAME)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        unsafe {
            // Open existing memory-mapped file
            let handle = OpenFileMappingW(
                FILE_MAP_READ.0,
                false,
                windows::core::PCWSTR(wide_name.as_ptr()),
            )
            .map_err(|e| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Failed to open shared memory: {}", e),
                )
            })?;

            // Map view of file
            let mapped_view = MapViewOfFile(handle, FILE_MAP_READ, 0, 0, 0);

            if mapped_view.Value.is_null() {
                CloseHandle(handle).ok();
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Failed to map view of file",
                ));
            }

            self.handle = Some(handle);
            self.mapped_view = Some(mapped_view);
        }

        Ok(())
    }

    /// Read the IRSDK header from shared memory
    #[cfg(target_os = "windows")]
    pub fn read_header(&self) -> io::Result<IrsdkHeader> {
        let mapped_view = self.mapped_view.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotConnected,
                "Not connected to shared memory",
            )
        })?;

        unsafe {
            let header_ptr = mapped_view.Value as *const IrsdkHeader;
            let header = *header_ptr;

            // Validate header
            if header.version < 1 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Invalid IRSDK header version",
                ));
            }

            Ok(header)
        }
    }

    /// Read the IRSDK header (stub for non-Windows)
    #[cfg(not(target_os = "windows"))]
    pub fn read_header(&self) -> io::Result<IrsdkHeader> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "IRSDK is only supported on Windows",
        ))
    }

    /// Read session info YAML from shared memory
    #[cfg(target_os = "windows")]
    pub fn read_session_info(&self) -> io::Result<String> {
        let header = self.read_header()?;
        let mapped_view = self.mapped_view.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotConnected,
                "Not connected to shared memory",
            )
        })?;

        unsafe {
            let base_ptr = mapped_view.Value as *const u8;
            let session_info_ptr = base_ptr.add(header.session_info_offset as usize);
            let session_info_slice =
                std::slice::from_raw_parts(session_info_ptr, header.session_info_len as usize);

            // Find null terminator
            let len = session_info_slice
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(session_info_slice.len());

            String::from_utf8(session_info_slice[..len].to_vec()).map_err(|e| {
                io::Error::new(io::ErrorKind::InvalidData, format!("Invalid UTF-8: {}", e))
            })
        }
    }

    /// Read session info YAML (stub for non-Windows)
    #[cfg(not(target_os = "windows"))]
    pub fn read_session_info(&self) -> io::Result<String> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "IRSDK is only supported on Windows",
        ))
    }

    /// Read variable headers from shared memory
    #[cfg(target_os = "windows")]
    pub fn read_var_headers(&self) -> io::Result<Vec<super::types::IrsdkVarHeader>> {
        let header = self.read_header()?;
        let mapped_view = self.mapped_view.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotConnected,
                "Not connected to shared memory",
            )
        })?;

        let mut var_headers = Vec::with_capacity(header.num_vars as usize);

        unsafe {
            let base_ptr = mapped_view.Value as *const u8;
            let var_header_ptr = base_ptr.add(header.var_header_offset as usize);

            for i in 0..header.num_vars {
                let offset = i as usize * std::mem::size_of::<super::types::IrsdkVarHeader>();
                let header_ptr = var_header_ptr.add(offset) as *const super::types::IrsdkVarHeader;
                var_headers.push(*header_ptr);
            }
        }

        Ok(var_headers)
    }

    /// Read telemetry data from the latest buffer
    #[cfg(target_os = "windows")]
    pub fn read_telemetry_data(&self) -> io::Result<Vec<u8>> {
        let header = self.read_header()?;
        let mapped_view = self.mapped_view.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotConnected,
                "Not connected to shared memory",
            )
        })?;

        // Find the latest buffer using tick counts
        let mut latest_buf_idx = 0;
        let mut latest_tick = 0;

        for i in 0..header.num_buf.min(4) {
            let tick = header.var_buf[i as usize].tick_count;
            if tick > latest_tick {
                latest_tick = tick;
                latest_buf_idx = i;
            }
        }

        let buf_info = &header.var_buf[latest_buf_idx as usize];
        let buf_len = header.buf_len as usize;

        unsafe {
            let base_ptr = mapped_view.Value as *const u8;
            let buf_ptr = base_ptr.add(buf_info.buf_offset as usize);
            let buf_slice = std::slice::from_raw_parts(buf_ptr, buf_len);
            Ok(buf_slice.to_vec())
        }
    }

    /// Read a float value from telemetry data at given offset
    #[cfg(target_os = "windows")]
    pub fn read_float(&self, data: &[u8], offset: usize) -> Option<f32> {
        if offset + 4 > data.len() {
            return None;
        }
        let bytes: [u8; 4] = data[offset..offset + 4].try_into().ok()?;
        Some(f32::from_le_bytes(bytes))
    }

    /// Read a double value from telemetry data at given offset
    #[cfg(target_os = "windows")]
    pub fn read_double(&self, data: &[u8], offset: usize) -> Option<f64> {
        if offset + 8 > data.len() {
            return None;
        }
        let bytes: [u8; 8] = data[offset..offset + 8].try_into().ok()?;
        Some(f64::from_le_bytes(bytes))
    }

    /// Read an int value from telemetry data at given offset
    #[cfg(target_os = "windows")]
    pub fn read_int(&self, data: &[u8], offset: usize) -> Option<i32> {
        if offset + 4 > data.len() {
            return None;
        }
        let bytes: [u8; 4] = data[offset..offset + 4].try_into().ok()?;
        Some(i32::from_le_bytes(bytes))
    }

    /// Read a bool value from telemetry data at given offset
    #[cfg(target_os = "windows")]
    pub fn read_bool(&self, data: &[u8], offset: usize) -> Option<bool> {
        if offset >= data.len() {
            return None;
        }
        Some(data[offset] != 0)
    }
}

impl Default for IrsdkReader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_os = "windows")]
impl Drop for IrsdkReader {
    fn drop(&mut self) {
        unsafe {
            if let Some(mapped_view) = self.mapped_view {
                UnmapViewOfFile(mapped_view).ok();
            }
            if let Some(handle) = self.handle {
                CloseHandle(handle).ok();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reader_creation() {
        let _reader = IrsdkReader::new();
        // Should be able to create reader on any platform
        assert!(true);
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_not_connected_on_non_windows() {
        let mut reader = IrsdkReader::new();
        assert!(!reader.is_connected());
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_read_header_fails_on_non_windows() {
        let reader = IrsdkReader::new();
        assert!(reader.read_header().is_err());
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_read_session_info_fails_on_non_windows() {
        let reader = IrsdkReader::new();
        assert!(reader.read_session_info().is_err());
    }
}
