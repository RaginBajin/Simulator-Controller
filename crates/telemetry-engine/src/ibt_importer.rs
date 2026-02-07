//! iRacing .ibt binary telemetry file parser.
//!
//! Parses the binary .ibt format to extract session metadata, lap boundaries,
//! and telemetry channel data, mapping them to the canonical 35-channel Parquet schema.

use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::Arc;

use arrow::array::{
    ArrayRef, BooleanBuilder, Float64Builder, Int32Builder, Int64Builder, RecordBatch,
};
use arrow::datatypes::DataType;
use tracing::{info, warn};

use storage::error::StorageError;
use storage::import::types::{ImportedSession, ImportedSessionMetadata};
use storage::parquet::schema::telemetry_schema;
use storage::types::NewLap;

// .ibt file header constants (used in tests for synthetic file generation)
#[cfg(test)]
const IBT_HEADER_SIZE: usize = 112;
#[cfg(test)]
const VAR_HEADER_SIZE: usize = 144;
const VAR_NAME_LEN: usize = 32;
const VAR_DESC_LEN: usize = 64;
const VAR_UNIT_LEN: usize = 32;

/// iRacing variable data types
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(i32)]
enum IbtVarType {
    Char = 0,
    Bool = 1,
    Int = 2,
    BitField = 3,
    Float = 4,
    Double = 5,
}

impl IbtVarType {
    fn from_i32(val: i32) -> Option<Self> {
        match val {
            0 => Some(Self::Char),
            1 => Some(Self::Bool),
            2 => Some(Self::Int),
            3 => Some(Self::BitField),
            4 => Some(Self::Float),
            5 => Some(Self::Double),
            _ => None,
        }
    }

    fn byte_size(&self) -> usize {
        match self {
            Self::Char | Self::Bool => 1,
            Self::Int | Self::BitField | Self::Float => 4,
            Self::Double => 8,
        }
    }
}

/// Parsed .ibt file header.
#[derive(Debug)]
struct IbtHeader {
    version: i32,
    _tick_rate: i32,
    session_info_offset: i32,
    session_info_length: i32,
    num_vars: i32,
    var_header_offset: i32,
    num_buf: i32,
    buf_len: i32,
    buf_offset: i32,
}

/// A single variable definition from the header.
#[derive(Debug, Clone)]
struct IbtVarHeader {
    name: String,
    _description: String,
    _unit: String,
    var_type: IbtVarType,
    #[allow(dead_code)]
    count: i32,
    offset: i32,
}

/// Channel name mapping from .ibt variable names to canonical schema names.
fn ibt_channel_mapping() -> HashMap<&'static str, &'static str> {
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

/// Parse an iRacing .ibt binary telemetry file and return an ImportedSession.
pub fn parse_ibt_file(path: &Path) -> Result<ImportedSession, StorageError> {
    info!(path = %path.display(), "Parsing .ibt file");

    let mut file = std::fs::File::open(path).map_err(|e| {
        StorageError::ParseError(format!("Failed to open .ibt file {}: {}", path.display(), e))
    })?;

    // Parse file header
    let header = parse_header(&mut file)?;
    info!(
        version = header.version,
        num_vars = header.num_vars,
        num_buf = header.num_buf,
        "Parsed .ibt header"
    );

    // Parse session info YAML
    let session_info = parse_session_info(&mut file, &header)?;
    let metadata = extract_metadata(&session_info, path)?;

    // Parse variable headers
    let var_headers = parse_var_headers(&mut file, &header)?;

    // Parse telemetry data records
    let (telemetry_batch, raw_laps_data) = parse_data_records(
        &mut file,
        &header,
        &var_headers,
    )?;

    // Extract lap boundaries and compute lap summaries
    let laps = compute_laps(&raw_laps_data);

    info!(
        track = %metadata.track_name,
        car = %metadata.car_name,
        rows = telemetry_batch.num_rows(),
        laps = laps.len(),
        "Parsed .ibt file successfully"
    );

    Ok(ImportedSession {
        metadata,
        laps,
        telemetry: telemetry_batch,
    })
}

fn read_i32<R: Read>(reader: &mut R) -> Result<i32, StorageError> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf).map_err(|e| {
        StorageError::ParseError(format!("Failed to read i32: {}", e))
    })?;
    Ok(i32::from_le_bytes(buf))
}

fn read_fixed_string<R: Read>(reader: &mut R, len: usize) -> Result<String, StorageError> {
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf).map_err(|e| {
        StorageError::ParseError(format!("Failed to read string: {}", e))
    })?;
    let end = buf.iter().position(|&b| b == 0).unwrap_or(len);
    Ok(String::from_utf8_lossy(&buf[..end]).to_string())
}

fn parse_header<R: Read + Seek>(reader: &mut R) -> Result<IbtHeader, StorageError> {
    reader.seek(SeekFrom::Start(0)).map_err(|e| {
        StorageError::ParseError(format!("Failed to seek to header: {}", e))
    })?;

    let version = read_i32(reader)?;
    let _status = read_i32(reader)?;
    let tick_rate = read_i32(reader)?;

    // Session info
    let session_info_update = read_i32(reader)?;
    let _ = session_info_update;
    let session_info_length = read_i32(reader)?;
    let session_info_offset = read_i32(reader)?;

    // Variable headers
    let num_vars = read_i32(reader)?;
    let var_header_offset = read_i32(reader)?;

    // Data buffers
    let num_buf = read_i32(reader)?;
    let buf_len = read_i32(reader)?;

    // Skip padding to get to buffer offset
    // The buffer info array starts at offset 48 in the header
    // Each buffer entry is: tick_count (4) + buf_offset (4) = 8 bytes
    reader.seek(SeekFrom::Start(48)).map_err(|e| {
        StorageError::ParseError(format!("Failed to seek to buffer info: {}", e))
    })?;
    let _tick_count = read_i32(reader)?;
    let buf_offset = read_i32(reader)?;

    // Validate header fields are non-negative before they get cast to u64/usize
    if session_info_offset < 0 {
        return Err(StorageError::ParseError(
            "Invalid .ibt file: session_info_offset is negative".to_string(),
        ));
    }
    if session_info_length < 0 {
        return Err(StorageError::ParseError(
            "Invalid .ibt file: session_info_length is negative".to_string(),
        ));
    }
    if num_vars < 0 {
        return Err(StorageError::ParseError(
            "Invalid .ibt file: num_vars is negative".to_string(),
        ));
    }
    if var_header_offset < 0 {
        return Err(StorageError::ParseError(
            "Invalid .ibt file: var_header_offset is negative".to_string(),
        ));
    }
    if num_buf < 0 {
        return Err(StorageError::ParseError(
            "Invalid .ibt file: num_buf is negative".to_string(),
        ));
    }
    if buf_len < 0 {
        return Err(StorageError::ParseError(
            "Invalid .ibt file: buf_len is negative".to_string(),
        ));
    }
    if buf_offset < 0 {
        return Err(StorageError::ParseError(
            "Invalid .ibt file: buf_offset is negative".to_string(),
        ));
    }

    Ok(IbtHeader {
        version,
        _tick_rate: tick_rate,
        session_info_offset,
        session_info_length,
        num_vars,
        var_header_offset,
        num_buf,
        buf_len,
        buf_offset,
    })
}

fn parse_session_info<R: Read + Seek>(
    reader: &mut R,
    header: &IbtHeader,
) -> Result<String, StorageError> {
    reader
        .seek(SeekFrom::Start(header.session_info_offset as u64))
        .map_err(|e| {
            StorageError::ParseError(format!("Failed to seek to session info: {}", e))
        })?;

    let mut buf = vec![0u8; header.session_info_length as usize];
    reader.read_exact(&mut buf).map_err(|e| {
        StorageError::ParseError(format!("Failed to read session info: {}", e))
    })?;

    let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    Ok(String::from_utf8_lossy(&buf[..end]).to_string())
}

fn extract_metadata(
    session_info: &str,
    file_path: &Path,
) -> Result<ImportedSessionMetadata, StorageError> {
    // Extract fields from YAML session info using simple line parsing.
    // The YAML structure is not deeply nested for the fields we need.
    let mut track_name = String::from("Unknown Track");
    let mut car_name = String::from("Unknown Car");
    let mut session_type = String::from("practice");
    let mut date_str = None;

    for line in session_info.lines() {
        let trimmed = line.trim().trim_start_matches("- ");
        if let Some(val) = trimmed.strip_prefix("TrackDisplayName: ") {
            track_name = val.trim().to_string();
        } else if let Some(val) = trimmed.strip_prefix("TrackDisplayShortName: ") {
            if track_name == "Unknown Track" {
                track_name = val.trim().to_string();
            }
        } else if let Some(val) = trimmed.strip_prefix("CarScreenName: ") {
            car_name = val.trim().to_string();
        } else if let Some(val) = trimmed.strip_prefix("CarScreenNameShort: ") {
            if car_name == "Unknown Car" {
                car_name = val.trim().to_string();
            }
        } else if let Some(val) = trimmed.strip_prefix("SessionType: ") {
            let st = val.trim().to_lowercase();
            session_type = match st.as_str() {
                "race" => "race".to_string(),
                "qualify" | "qualifying" | "lone qualify" | "open qualify" => {
                    "qualifying".to_string()
                }
                "warmup" | "warm up" => "warmup".to_string(),
                _ => "practice".to_string(),
            };
        } else if date_str.is_none() {
            // Try to find date from WeekendInfo section
            if let Some(val) = trimmed.strip_prefix("SimSetupDate: ") {
                date_str = Some(val.trim().to_string());
            }
        }
    }

    // Build started_at timestamp
    let started_at = if let Some(date) = date_str {
        // Try parsing the date and formatting as ISO 8601
        if let Ok(dt) = chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d") {
            dt.and_hms_opt(0, 0, 0)
                .map(|ndt| format!("{}Z", ndt.format("%Y-%m-%dT%H:%M:%S")))
                .unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string())
        } else {
            chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
        }
    } else {
        // Fallback: use file modification time or current time
        if let Ok(metadata) = std::fs::metadata(file_path) {
            if let Ok(modified) = metadata.modified() {
                let dt: chrono::DateTime<chrono::Utc> = modified.into();
                dt.format("%Y-%m-%dT%H:%M:%SZ").to_string()
            } else {
                chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
            }
        } else {
            chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
        }
    };

    Ok(ImportedSessionMetadata {
        track_name,
        car_name,
        session_type,
        started_at,
        ended_at: None,
        import_source: file_path.display().to_string(),
        import_format: "ibt".to_string(),
    })
}

fn parse_var_headers<R: Read + Seek>(
    reader: &mut R,
    header: &IbtHeader,
) -> Result<Vec<IbtVarHeader>, StorageError> {
    reader
        .seek(SeekFrom::Start(header.var_header_offset as u64))
        .map_err(|e| {
            StorageError::ParseError(format!("Failed to seek to var headers: {}", e))
        })?;

    let mut var_headers = Vec::with_capacity(header.num_vars as usize);

    for _ in 0..header.num_vars {
        let var_type_raw = read_i32(reader)?;
        let offset = read_i32(reader)?;
        let count = read_i32(reader)?;
        // Skip: countAsTime (1 byte) + pad (3 bytes) = 4 bytes
        let mut _pad = [0u8; 4];
        reader.read_exact(&mut _pad).map_err(|e| {
            StorageError::ParseError(format!("Failed to read var header pad: {}", e))
        })?;

        let name = read_fixed_string(reader, VAR_NAME_LEN)?;
        let description = read_fixed_string(reader, VAR_DESC_LEN)?;
        let unit = read_fixed_string(reader, VAR_UNIT_LEN)?;

        let var_type = match IbtVarType::from_i32(var_type_raw) {
            Some(t) => t,
            None => {
                warn!(
                    var_type = var_type_raw,
                    name = %name,
                    "Skipping .ibt variable with unknown type"
                );
                continue;
            }
        };

        var_headers.push(IbtVarHeader {
            name,
            _description: description,
            _unit: unit,
            var_type,
            count,
            offset,
        });
    }

    Ok(var_headers)
}

/// Raw data we need for lap boundary detection.
struct LapRawData {
    session_times: Vec<f64>,
    lap_dist_pcts: Vec<f64>,
}

fn parse_data_records<R: Read + Seek>(
    reader: &mut R,
    header: &IbtHeader,
    var_headers: &[IbtVarHeader],
) -> Result<(RecordBatch, LapRawData), StorageError> {
    let channel_map = ibt_channel_mapping();

    // Build index: canonical_name -> (var_header_index, ibt_var_header)
    let mut canonical_vars: HashMap<String, &IbtVarHeader> = HashMap::new();
    for vh in var_headers {
        if let Some(&canonical) = channel_map.get(vh.name.as_str()) {
            canonical_vars.insert(canonical.to_string(), vh);
        }
    }

    // Also find the Lap variable for boundary detection
    let lap_var = var_headers.iter().find(|vh| vh.name == "Lap");

    // Calculate number of data records
    let file_size = reader.seek(SeekFrom::End(0)).map_err(|e| {
        StorageError::ParseError(format!("Failed to get file size: {}", e))
    })?;

    let data_start = header.buf_offset as u64;
    let record_len = header.buf_len as u64;

    if record_len == 0 {
        return Err(StorageError::ParseError(
            "Invalid .ibt file: record length is 0".to_string(),
        ));
    }

    if data_start > file_size {
        return Err(StorageError::ParseError(format!(
            "Invalid .ibt file: data offset ({}) exceeds file size ({})",
            data_start, file_size
        )));
    }

    let available_bytes = file_size.checked_sub(data_start).ok_or_else(|| {
        StorageError::ParseError("Invalid .ibt file: data offset arithmetic overflow".to_string())
    })?;

    let num_records = (available_bytes / record_len) as usize;
    if num_records == 0 {
        return Err(StorageError::ParseError(
            "No telemetry data records found in .ibt file".to_string(),
        ));
    }

    info!(num_records, "Reading telemetry data records");

    // Pre-allocate arrays for all canonical channels
    let schema = telemetry_schema();
    let num_fields = schema.fields().len();
    let mut columns: Vec<ColumnBuilder> = Vec::with_capacity(num_fields);
    for field in schema.fields() {
        columns.push(ColumnBuilder::new(field.data_type().clone(), num_records));
    }

    // Track raw data for lap detection
    let mut session_times = Vec::with_capacity(num_records);
    let mut lap_dist_pcts = Vec::with_capacity(num_records);
    let mut lap_numbers = Vec::with_capacity(num_records);

    // Read record buffer
    let mut record_buf = vec![0u8; record_len as usize];

    // Compute tick_rate for timestamp_ms calculation
    let tick_rate = if header._tick_rate > 0 {
        header._tick_rate as f64
    } else {
        60.0 // Default 60Hz
    };
    let sample_period_ms = 1000.0 / tick_rate;

    reader.seek(SeekFrom::Start(data_start)).map_err(|e| {
        StorageError::ParseError(format!("Failed to seek to data start: {}", e))
    })?;

    for sample_idx in 0..num_records {
        reader.read_exact(&mut record_buf).map_err(|e| {
            StorageError::ParseError(format!(
                "Failed to read data record {}: {}",
                sample_idx, e
            ))
        })?;

        // For each canonical channel in the schema, either read from .ibt or fill with default
        for (col_idx, field) in schema.fields().iter().enumerate() {
            let field_name = field.name().as_str();

            if field_name == "timestamp_ms" {
                // Derive from session_time or sample index
                let ts_ms = if let Some(vh) = canonical_vars.get("session_time") {
                    let val = read_var_as_f64(&record_buf, vh)?;
                    (val * 1000.0) as i64
                } else {
                    (sample_idx as f64 * sample_period_ms) as i64
                };
                columns[col_idx].push_i64(ts_ms);
                continue;
            }

            if let Some(vh) = canonical_vars.get(field_name) {
                match field.data_type() {
                    DataType::Float64 => {
                        let val = read_var_as_f64(&record_buf, vh)?;
                        columns[col_idx].push_f64(val);
                    }
                    DataType::Int32 => {
                        let val = read_var_as_i32(&record_buf, vh)?;
                        columns[col_idx].push_i32(val);
                    }
                    DataType::Int64 => {
                        let val = read_var_as_f64(&record_buf, vh)?;
                        columns[col_idx].push_i64(val as i64);
                    }
                    DataType::Boolean => {
                        let val = read_var_as_bool(&record_buf, vh)?;
                        columns[col_idx].push_bool(val);
                    }
                    _ => {
                        columns[col_idx].push_null();
                    }
                }
            } else {
                columns[col_idx].push_null();
            }
        }

        // Collect raw data for lap detection
        let session_time = if let Some(vh) = canonical_vars.get("session_time") {
            read_var_as_f64(&record_buf, vh)?
        } else {
            sample_idx as f64 / tick_rate
        };
        session_times.push(session_time);

        let lap_dist_pct = if let Some(vh) = canonical_vars.get("track_position") {
            read_var_as_f64(&record_buf, vh)?
        } else {
            0.0
        };
        lap_dist_pcts.push(lap_dist_pct);

        if let Some(vh) = lap_var {
            let lap_num = read_var_as_i32(&record_buf, vh)?;
            lap_numbers.push(lap_num);
        }
    }

    // Build Arrow arrays
    let mut arrow_columns: Vec<ArrayRef> = Vec::with_capacity(num_fields);
    for col in columns {
        arrow_columns.push(col.build());
    }

    let batch = RecordBatch::try_new(Arc::new(schema), arrow_columns).map_err(|e| {
        StorageError::ParseError(format!("Failed to build RecordBatch: {}", e))
    })?;

    let raw_data = LapRawData {
        session_times,
        lap_dist_pcts,
    };

    Ok((batch, raw_data))
}

fn read_var_as_f64(record: &[u8], vh: &IbtVarHeader) -> Result<f64, StorageError> {
    let offset = vh.offset as usize;
    if offset + vh.var_type.byte_size() > record.len() {
        return Ok(0.0);
    }

    match vh.var_type {
        IbtVarType::Float => {
            let bytes: [u8; 4] = record[offset..offset + 4].try_into().unwrap();
            Ok(f32::from_le_bytes(bytes) as f64)
        }
        IbtVarType::Double => {
            let bytes: [u8; 8] = record[offset..offset + 8].try_into().unwrap();
            Ok(f64::from_le_bytes(bytes))
        }
        IbtVarType::Int => {
            let bytes: [u8; 4] = record[offset..offset + 4].try_into().unwrap();
            Ok(i32::from_le_bytes(bytes) as f64)
        }
        IbtVarType::Bool | IbtVarType::Char => Ok(record[offset] as f64),
        IbtVarType::BitField => {
            let bytes: [u8; 4] = record[offset..offset + 4].try_into().unwrap();
            Ok(i32::from_le_bytes(bytes) as f64)
        }
    }
}

fn read_var_as_i32(record: &[u8], vh: &IbtVarHeader) -> Result<i32, StorageError> {
    let offset = vh.offset as usize;
    if offset + vh.var_type.byte_size() > record.len() {
        return Ok(0);
    }

    match vh.var_type {
        IbtVarType::Int | IbtVarType::BitField => {
            let bytes: [u8; 4] = record[offset..offset + 4].try_into().unwrap();
            Ok(i32::from_le_bytes(bytes))
        }
        IbtVarType::Float => {
            let bytes: [u8; 4] = record[offset..offset + 4].try_into().unwrap();
            Ok(f32::from_le_bytes(bytes) as i32)
        }
        IbtVarType::Double => {
            let bytes: [u8; 8] = record[offset..offset + 8].try_into().unwrap();
            Ok(f64::from_le_bytes(bytes) as i32)
        }
        IbtVarType::Bool | IbtVarType::Char => Ok(record[offset] as i32),
    }
}

fn read_var_as_bool(record: &[u8], vh: &IbtVarHeader) -> Result<bool, StorageError> {
    let offset = vh.offset as usize;
    if offset >= record.len() {
        return Ok(false);
    }

    match vh.var_type {
        IbtVarType::Bool | IbtVarType::Char => Ok(record[offset] != 0),
        IbtVarType::Int | IbtVarType::BitField => {
            if offset + 4 > record.len() {
                return Ok(false);
            }
            let bytes: [u8; 4] = record[offset..offset + 4].try_into().unwrap();
            Ok(i32::from_le_bytes(bytes) != 0)
        }
        IbtVarType::Float => {
            if offset + 4 > record.len() {
                return Ok(false);
            }
            let bytes: [u8; 4] = record[offset..offset + 4].try_into().unwrap();
            Ok(f32::from_le_bytes(bytes) != 0.0)
        }
        IbtVarType::Double => {
            if offset + 8 > record.len() {
                return Ok(false);
            }
            let bytes: [u8; 8] = record[offset..offset + 8].try_into().unwrap();
            Ok(f64::from_le_bytes(bytes) != 0.0)
        }
    }
}

/// Compute lap summaries from LapDistPct transitions (crossing 0.0 from near 1.0).
fn compute_laps(raw: &LapRawData) -> Vec<NewLap> {
    if raw.session_times.is_empty() {
        return Vec::new();
    }

    let mut laps = Vec::new();
    let mut lap_number = 1i32;
    let mut lap_start_time = raw.session_times[0];

    for i in 1..raw.lap_dist_pcts.len() {
        let prev = raw.lap_dist_pcts[i - 1];
        let curr = raw.lap_dist_pcts[i];

        // Detect lap crossing: previous was near end of track, current is near start
        if prev > 0.9 && curr < 0.1 {
            let lap_end_time = raw.session_times[i];
            let lap_time_secs = lap_end_time - lap_start_time;
            let lap_time_ms = (lap_time_secs * 1000.0) as i64;

            // Only record laps with reasonable time (> 5 seconds, < 10 minutes)
            if lap_time_ms > 5000 && lap_time_ms < 600_000 {
                laps.push(NewLap {
                    session_id: String::new(), // Set by orchestrator
                    lap_number,
                    lap_time_ms,
                    is_valid: true,
                    completion_status: "complete".to_string(),
                });
                lap_number += 1;
            }

            lap_start_time = lap_end_time;
        }
    }

    laps
}

/// Column builder that accumulates values for a single Arrow column.
/// Uses Arrow's nullable builders so missing channels can be filled with NULL.
enum ColumnBuilder {
    Float64(Float64Builder),
    Int32(Int32Builder),
    Int64(Int64Builder),
    Boolean(BooleanBuilder),
}

impl ColumnBuilder {
    fn new(data_type: DataType, capacity: usize) -> Self {
        match data_type {
            DataType::Float64 => Self::Float64(Float64Builder::with_capacity(capacity)),
            DataType::Int32 => Self::Int32(Int32Builder::with_capacity(capacity)),
            DataType::Int64 => Self::Int64(Int64Builder::with_capacity(capacity)),
            DataType::Boolean => Self::Boolean(BooleanBuilder::with_capacity(capacity)),
            _ => Self::Float64(Float64Builder::with_capacity(capacity)),
        }
    }

    fn push_f64(&mut self, val: f64) {
        if let Self::Float64(b) = self {
            b.append_value(val);
        } else {
            warn!("Type mismatch: push_f64 called on non-Float64 column builder");
        }
    }

    fn push_i32(&mut self, val: i32) {
        if let Self::Int32(b) = self {
            b.append_value(val);
        } else {
            warn!("Type mismatch: push_i32 called on non-Int32 column builder");
        }
    }

    fn push_i64(&mut self, val: i64) {
        if let Self::Int64(b) = self {
            b.append_value(val);
        } else {
            warn!("Type mismatch: push_i64 called on non-Int64 column builder");
        }
    }

    fn push_bool(&mut self, val: bool) {
        if let Self::Boolean(b) = self {
            b.append_value(val);
        } else {
            warn!("Type mismatch: push_bool called on non-Boolean column builder");
        }
    }

    fn push_null(&mut self) {
        match self {
            Self::Float64(b) => b.append_null(),
            Self::Int32(b) => b.append_null(),
            Self::Int64(b) => b.append_null(),
            Self::Boolean(b) => b.append_null(),
        }
    }

    fn build(mut self) -> ArrayRef {
        match &mut self {
            Self::Float64(b) => Arc::new(b.finish()) as ArrayRef,
            Self::Int32(b) => Arc::new(b.finish()) as ArrayRef,
            Self::Int64(b) => Arc::new(b.finish()) as ArrayRef,
            Self::Boolean(b) => Arc::new(b.finish()) as ArrayRef,
        }
    }
}

/// Create a synthetic .ibt file for testing purposes.
/// This generates a minimal valid .ibt binary with known channel data.
#[cfg(test)]
pub fn create_test_ibt_file(path: &Path, num_samples: usize) -> Result<(), StorageError> {
    use std::io::Write;

    let session_info = b"---\nWeekendInfo:\n  TrackDisplayName: Lime Rock Park\n  TrackDisplayShortName: Lime Rock\nDriverInfo:\n  Drivers:\n    - CarScreenName: Mazda MX-5 Cup\n      CarScreenNameShort: MX-5\nSessionInfo:\n  Sessions:\n    - SessionType: Practice\n      SimSetupDate: 2026-02-07\n---\n\0";

    // Define test variables: Speed, Throttle, Brake, Gear, SessionTime, LapDistPct
    let test_vars = vec![
        ("Speed", IbtVarType::Float, 0),
        ("Throttle", IbtVarType::Float, 4),
        ("Brake", IbtVarType::Float, 8),
        ("Gear", IbtVarType::Int, 12),
        ("SessionTime", IbtVarType::Double, 16),
        ("LapDistPct", IbtVarType::Float, 24),
        ("LapDist", IbtVarType::Float, 28),
        ("RPM", IbtVarType::Float, 32),
        ("LapCurrentLapTime", IbtVarType::Float, 36),
        ("SteeringWheelAngle", IbtVarType::Float, 40),
    ];

    let num_vars = test_vars.len() as i32;
    let record_len = 48i32; // All vars fit within 48 bytes

    let session_info_offset = IBT_HEADER_SIZE as i32;
    let session_info_length = session_info.len() as i32;
    let var_header_offset = session_info_offset + session_info_length;
    let data_offset = var_header_offset + (num_vars * VAR_HEADER_SIZE as i32);

    let mut file = std::fs::File::create(path).map_err(|e| {
        StorageError::ParseError(format!("Failed to create test .ibt: {}", e))
    })?;

    // Write header (112 bytes)
    let mut header_buf = vec![0u8; IBT_HEADER_SIZE];
    // version
    header_buf[0..4].copy_from_slice(&1i32.to_le_bytes());
    // status
    header_buf[4..8].copy_from_slice(&1i32.to_le_bytes());
    // tick_rate
    header_buf[8..12].copy_from_slice(&60i32.to_le_bytes());
    // session_info_update
    header_buf[12..16].copy_from_slice(&0i32.to_le_bytes());
    // session_info_length
    header_buf[16..20].copy_from_slice(&session_info_length.to_le_bytes());
    // session_info_offset
    header_buf[20..24].copy_from_slice(&session_info_offset.to_le_bytes());
    // num_vars
    header_buf[24..28].copy_from_slice(&num_vars.to_le_bytes());
    // var_header_offset
    header_buf[28..32].copy_from_slice(&var_header_offset.to_le_bytes());
    // num_buf
    header_buf[32..36].copy_from_slice(&1i32.to_le_bytes());
    // buf_len
    header_buf[36..40].copy_from_slice(&record_len.to_le_bytes());
    // Pad to 48 then buffer info
    // buf[48]: tick_count
    header_buf[48..52].copy_from_slice(&0i32.to_le_bytes());
    // buf[52]: buf_offset
    header_buf[52..56].copy_from_slice(&data_offset.to_le_bytes());

    file.write_all(&header_buf).map_err(|e| {
        StorageError::ParseError(format!("Failed to write test header: {}", e))
    })?;

    // Write session info
    file.write_all(session_info).map_err(|e| {
        StorageError::ParseError(format!("Failed to write session info: {}", e))
    })?;

    // Write variable headers (144 bytes each)
    for (name, var_type, offset) in &test_vars {
        let mut vh_buf = vec![0u8; VAR_HEADER_SIZE];
        // var_type
        vh_buf[0..4].copy_from_slice(&(*var_type as i32).to_le_bytes());
        // offset
        vh_buf[4..8].copy_from_slice(&(*offset as i32).to_le_bytes());
        // count
        vh_buf[8..12].copy_from_slice(&1i32.to_le_bytes());
        // pad (4 bytes)
        // name (32 bytes starting at offset 16)
        let name_bytes = name.as_bytes();
        vh_buf[16..16 + name_bytes.len()].copy_from_slice(name_bytes);
        // description (64 bytes starting at offset 48) - leave empty
        // unit (32 bytes starting at offset 112) - leave empty

        file.write_all(&vh_buf).map_err(|e| {
            StorageError::ParseError(format!("Failed to write var header: {}", e))
        })?;
    }

    // Write data records
    for i in 0..num_samples {
        let mut record = vec![0u8; record_len as usize];
        let t = i as f64 / 60.0; // 60Hz
        let lap_pct = (i % 3600) as f32 / 3600.0; // One lap every 60 seconds at 60Hz

        // Speed (float, offset 0): 40.0 m/s
        record[0..4].copy_from_slice(&40.0f32.to_le_bytes());
        // Throttle (float, offset 4): 0.8
        record[4..8].copy_from_slice(&0.8f32.to_le_bytes());
        // Brake (float, offset 8): 0.0
        record[8..12].copy_from_slice(&0.0f32.to_le_bytes());
        // Gear (int, offset 12): 4
        record[12..16].copy_from_slice(&4i32.to_le_bytes());
        // SessionTime (double, offset 16)
        record[16..24].copy_from_slice(&t.to_le_bytes());
        // LapDistPct (float, offset 24)
        record[24..28].copy_from_slice(&lap_pct.to_le_bytes());
        // LapDist (float, offset 28): lap_pct * 2400m
        let lap_dist = lap_pct * 2400.0;
        record[28..32].copy_from_slice(&lap_dist.to_le_bytes());
        // RPM (float, offset 32): 5000
        record[32..36].copy_from_slice(&5000.0f32.to_le_bytes());
        // LapCurrentLapTime (float, offset 36)
        let lap_time = (i % 3600) as f32 / 60.0;
        record[36..40].copy_from_slice(&lap_time.to_le_bytes());
        // SteeringWheelAngle (float, offset 40): 0.1 rad
        record[40..44].copy_from_slice(&0.1f32.to_le_bytes());

        file.write_all(&record).map_err(|e| {
            StorageError::ParseError(format!("Failed to write test record: {}", e))
        })?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::{Array, Float64Array};
    use tempfile::TempDir;

    #[test]
    fn test_parse_ibt_extracts_metadata() {
        let dir = TempDir::new().unwrap();
        let ibt_path = dir.path().join("test.ibt");
        // 7200 samples = 2 minutes at 60Hz => should produce ~2 laps
        create_test_ibt_file(&ibt_path, 7200).unwrap();

        let result = parse_ibt_file(&ibt_path).unwrap();

        assert_eq!(result.metadata.track_name, "Lime Rock Park");
        assert_eq!(result.metadata.car_name, "Mazda MX-5 Cup");
        assert_eq!(result.metadata.import_format, "ibt");
    }

    #[test]
    fn test_parse_ibt_maps_channels_to_schema() {
        let dir = TempDir::new().unwrap();
        let ibt_path = dir.path().join("test.ibt");
        create_test_ibt_file(&ibt_path, 600).unwrap();

        let result = parse_ibt_file(&ibt_path).unwrap();
        let schema = result.telemetry.schema();

        // Should have all 35 canonical channels
        assert_eq!(schema.fields().len(), 35);
        assert_eq!(result.telemetry.num_rows(), 600);

        // Verify mapped channels have non-zero data
        let speed_idx = schema.index_of("speed").unwrap();
        let speed = result
            .telemetry
            .column(speed_idx)
            .as_any()
            .downcast_ref::<Float64Array>()
            .unwrap();
        assert!(speed.value(0) > 0.0, "Speed should be > 0");

        let throttle_idx = schema.index_of("throttle").unwrap();
        let throttle = result
            .telemetry
            .column(throttle_idx)
            .as_any()
            .downcast_ref::<Float64Array>()
            .unwrap();
        assert!(throttle.value(0) > 0.0, "Throttle should be > 0");
    }

    #[test]
    fn test_missing_channels_produce_nulls() {
        let dir = TempDir::new().unwrap();
        let ibt_path = dir.path().join("test.ibt");
        create_test_ibt_file(&ibt_path, 100).unwrap();

        let result = parse_ibt_file(&ibt_path).unwrap();
        let schema = result.telemetry.schema();

        // Channels not in our test .ibt should have NULL values
        let oil_temp_idx = schema.index_of("oil_temp").unwrap();
        let oil_temp = result
            .telemetry
            .column(oil_temp_idx)
            .as_any()
            .downcast_ref::<Float64Array>()
            .unwrap();
        assert!(oil_temp.is_null(0), "Missing channel should be NULL");
    }

    #[test]
    fn test_lap_detection_from_lap_dist_pct() {
        let dir = TempDir::new().unwrap();
        let ibt_path = dir.path().join("test.ibt");
        // 7200 samples at 60Hz = 120 seconds, with lap crossing at every 3600 samples (60s)
        create_test_ibt_file(&ibt_path, 7200).unwrap();

        let result = parse_ibt_file(&ibt_path).unwrap();

        // Should detect 1 complete lap (the crossing at 3600 samples)
        assert!(
            !result.laps.is_empty(),
            "Should detect at least one lap"
        );

        // First detected lap should have reasonable time (~60 seconds = 60000ms)
        if !result.laps.is_empty() {
            let first_lap = &result.laps[0];
            assert!(
                first_lap.lap_time_ms > 50000 && first_lap.lap_time_ms < 70000,
                "Lap time should be ~60s, got {}ms",
                first_lap.lap_time_ms
            );
        }
    }
}
