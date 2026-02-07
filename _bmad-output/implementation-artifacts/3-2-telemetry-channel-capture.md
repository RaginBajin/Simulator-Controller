# Story 3.2: Telemetry Channel Capture

Status: ready-for-dev

## Story

As a sim racer,
I want the app to capture all essential telemetry channels from iRacing,
so that AI coaching has complete data to analyze my driving.

## Acceptance Criteria

1. **60Hz Sampling Rate**
   - When IRSDK connection is established (Story 3.1), the app captures telemetry at 60Hz default sampling rate
   - Capture reads from IRSDK shared memory using the data-valid event to sync with iRacing's tick rate
   - Capture latency (time from IRSDK update to internal buffer write) is <10ms p99 (NFR2)

2. **All 35 Canonical Channels Captured**
   - The app captures all 35 required channels defined in the Parquet schema: `timestamp_ms`, `session_time`, `lap_distance`, `lap_time`, `speed`, `throttle`, `brake`, `steering`, `clutch`, `gear`, `rpm`, `lat_g`, `long_g`, `yaw`, `pitch`, `roll`, `velocity_x`, `velocity_y`, `velocity_z`, `tire_temp_lf`, `tire_temp_rf`, `tire_temp_lr`, `tire_temp_rr`, `tire_pressure_lf`, `tire_pressure_rf`, `tire_pressure_lr`, `tire_pressure_rr`, `fuel_level`, `fuel_usage`, `oil_temp`, `water_temp`, `brake_bias`, `abs_active`, `tc_active`, `track_position`
   - Channel mapping from IRSDK variable names to canonical names follows the existing mapping in `ibt_importer.rs`
   - Missing channels (not available from IRSDK for a given car) are recorded as NULL

3. **In-Memory Ring Buffer Storage**
   - Raw channel data is stored in an in-memory ring buffer during capture
   - Ring buffer capacity is configurable (default: 5 minutes at 60Hz = 18,000 samples)
   - Buffer supports concurrent reads (for periodic flush) and writes (from capture thread)
   - Memory usage stays within NFR1 budget (<200MB RSS total application)

4. **Session State Tracking**
   - The app continues capturing all channels regardless of session state (driving, pitting, spectating)
   - Session state transitions are flagged with timestamps in the telemetry stream
   - Environmental conditions captured: track temperature, air temperature, weather state
   - Session context captured: session type, car class, track configuration

5. **Configurable Sample Rate**
   - Telemetry sample rate is configurable (default: 60Hz, valid range: 10-120Hz)
   - Configuration is stored in app settings
   - Rate changes take effect on next capture session (not mid-session)

## Tasks / Subtasks

- [ ] Task 1: Implement telemetry data reader from IRSDK shared memory (AC: #1, #2)
  - [ ] 1.1 Create `crates/telemetry-engine/src/capture.rs` with `CaptureEngine` struct
  - [ ] 1.2 Implement `read_telemetry_frame()` that reads a single frame of all variable values from IRSDK shared memory buffer
  - [ ] 1.3 Reuse channel mapping from `ibt_importer.rs::ibt_channel_mapping()` -- extract to shared location in `irsdk/types.rs` or `irsdk/mod.rs`
  - [ ] 1.4 Map IRSDK variable values to the 35 canonical channels; set missing channels to `None`
  - [ ] 1.5 Compute `timestamp_ms` from `SessionTime` (seconds -> milliseconds) or from sample index if `SessionTime` is unavailable
  - [ ] 1.6 Wait on IRSDK data-valid event (`Local\\IRSDKDataValidEvent`) to sync with iRacing tick rate, with timeout fallback to polling interval

- [ ] Task 2: Implement ring buffer for telemetry samples (AC: #3)
  - [ ] 2.1 Create `crates/telemetry-engine/src/ring_buffer.rs` with `TelemetryRingBuffer` struct
  - [ ] 2.2 Implement lock-free or mutex-guarded circular buffer with configurable capacity
  - [ ] 2.3 Implement `push(sample: TelemetrySample)` for writing from capture thread
  - [ ] 2.4 Implement `drain()` or `take_batch()` for flushing to storage (returns all buffered samples and clears)
  - [ ] 2.5 Implement `len()` and `capacity()` for monitoring
  - [ ] 2.6 Use `Arc<Mutex<VecDeque<TelemetrySample>>>` for MVP simplicity (upgrade to lock-free if profiling shows contention)

- [ ] Task 3: Define telemetry sample data structures (AC: #2, #4)
  - [ ] 3.1 Create `crates/telemetry-engine/src/sample.rs` with `TelemetrySample` struct
  - [ ] 3.2 `TelemetrySample` holds all 35 channels as `Option<f64>` (or typed: `Option<i32>` for gear, `Option<bool>` for abs/tc)
  - [ ] 3.3 Include `session_state` field (enum: `Driving`, `Pitting`, `Spectating`, `Invalid`)
  - [ ] 3.4 Include environmental fields: `track_temp`, `air_temp`, `weather`
  - [ ] 3.5 Include session context fields: `session_type`, `car_class`, `track_config`
  - [ ] 3.6 Implement conversion from `TelemetrySample` batch to Arrow `RecordBatch` (for Parquet write)

- [ ] Task 4: Implement capture loop (AC: #1, #4, #5)
  - [ ] 4.1 Implement `CaptureEngine::start_capture()` that spawns a capture thread
  - [ ] 4.2 Capture loop: wait for data-valid event -> read frame -> push to ring buffer -> repeat
  - [ ] 4.3 Respect configured sample rate: if IRSDK tick rate > configured rate, skip frames (e.g., 60Hz IRSDK but 30Hz configured = read every other frame)
  - [ ] 4.4 Track session state changes: read `SessionState` IRSDK variable, log transitions, flag in sample
  - [ ] 4.5 Implement `CaptureEngine::stop_capture()` for graceful shutdown
  - [ ] 4.6 Implement `CaptureEngine::is_capturing()` status query
  - [ ] 4.7 Emit `session:capture-started` Tauri event when capture loop begins
  - [ ] 4.8 Emit `capture:progress` Tauri event periodically (every 60 seconds) with sample count and buffer utilization

- [ ] Task 5: Integrate with ConnectionManager from Story 3.1 (AC: #1)
  - [ ] 5.1 `CaptureEngine` takes a `ConnectionManager` reference (or its IRSDK reader handle)
  - [ ] 5.2 Start capture automatically when `ConnectionManager` signals `Connected`
  - [ ] 5.3 Stop capture when `ConnectionManager` signals `Disconnected`
  - [ ] 5.4 Handle reconnection: if IRSDK reconnects mid-capture, resume capture (mark gap)

- [ ] Task 6: Periodic flush to Parquet storage (AC: #3)
  - [ ] 6.1 Implement periodic flush from ring buffer to Parquet via `storage::parquet::writer`
  - [ ] 6.2 Flush interval: every 30 seconds or when ring buffer reaches 80% capacity
  - [ ] 6.3 Convert `TelemetrySample` batch to Arrow `RecordBatch` using the canonical `telemetry_schema()`
  - [ ] 6.4 Append to session's Parquet file (or create new file on session start)
  - [ ] 6.5 Use atomic write pattern: write to temp file, rename on success (NFR9 crash safety)

- [ ] Task 7: Unit tests and build verification (AC: all)
  - [ ] 7.1 Write unit tests for `TelemetrySample` to `RecordBatch` conversion
  - [ ] 7.2 Write unit tests for ring buffer push/drain/capacity
  - [ ] 7.3 Write unit tests for sample rate downsampling logic
  - [ ] 7.4 Write unit tests for channel mapping (verify all 35 channels map correctly)
  - [ ] 7.5 Run `cargo build` -- must pass on current platform
  - [ ] 7.6 Run `cargo test` -- all new and existing tests pass
  - [ ] 7.7 Run `npm run build` -- frontend builds clean

## Dev Notes

### What's Already Built

The `telemetry-engine` crate has:
- `irsdk/mod.rs` -- placeholder module (populated by Story 3.1 with reader + types)
- `ibt_importer.rs` -- full channel mapping in `ibt_channel_mapping()` function (maps IRSDK var names like `"Speed"`, `"Throttle"`, `"Brake"` to canonical names like `"speed"`, `"throttle"`, `"brake"`)
- `import_orchestrator.rs` -- import pipeline that writes to storage

The `storage` crate has:
- `parquet/schema.rs` -- canonical 35-channel `telemetry_schema()` with Arrow types
- `parquet/writer.rs` -- Parquet file writer
- `types.rs` -- `Session`, `NewSession`, `NewLap`, `SessionUpdate` structs

### Architecture Patterns to Follow

- **Channel Mapping:** Reuse the IRSDK-to-canonical mapping from `ibt_importer.rs`. Consider extracting to a shared location (`irsdk/mod.rs` or `irsdk/types.rs`) so both the .ibt importer and live capture use the same mapping
- **Arrow Schema:** Use `storage::parquet::schema::telemetry_schema()` as the single source of truth for all 35 channels
- **Nullable Fields:** All channels except `timestamp_ms` are nullable in the Parquet schema. Use `Option<T>` in the sample struct
- **IPC Events:** `capture:progress` with `{ type, timestamp, version, sampleCount, bufferUtilization }`
- **Error Contract:** `{ code, message, details?, retryable? }` for IPC errors
- **Crash Safety:** Atomic write pattern (temp file + rename) for Parquet flush, per NFR9

### IRSDK Data Reading Pattern

Reading from IRSDK shared memory follows this sequence:
1. Wait for data-valid event (kernel event `Local\\IRSDKDataValidEvent`)
2. Read header to get latest buffer index (double-buffered)
3. Read tick count from latest buffer, then read data, then re-read tick count
4. If tick counts match, data is consistent; if not, retry (torn read)
5. Parse variable values from buffer using var header offsets and types

This is the same format as .ibt files but read from live shared memory instead of a file.

### Ring Buffer Design

For MVP, use a simple `Arc<Mutex<VecDeque<TelemetrySample>>>`:
- Write side: capture thread pushes samples
- Read side: flush thread drains samples periodically
- Lock contention is minimal at 60Hz with 30-second flush intervals
- Upgrade to `crossbeam::ArrayQueue` or similar lock-free structure only if profiling shows issues

### Key Files to Create/Modify

| File | Action | Purpose |
|------|--------|---------|
| `crates/telemetry-engine/src/capture.rs` | Create | Capture engine with start/stop and capture loop |
| `crates/telemetry-engine/src/ring_buffer.rs` | Create | Thread-safe ring buffer for telemetry samples |
| `crates/telemetry-engine/src/sample.rs` | Create | TelemetrySample struct and RecordBatch conversion |
| `crates/telemetry-engine/src/lib.rs` | Modify | Export capture, ring_buffer, sample modules |
| `crates/telemetry-engine/src/irsdk/mod.rs` | Modify | Extract shared channel mapping (if refactoring) |

### Performance Budget (NFR1)

- CPU: <2% total system CPU during capture
- Memory: <200MB RSS total application
- At 60Hz with 35 channels: ~2,100 values/second
- Each `TelemetrySample` is ~35 * 8 bytes = 280 bytes -> 16.8 KB/second -> ~504 KB for 30-second flush
- 5-minute ring buffer: ~5 MB (well within budget)

### Relationship to Other Stories

- **Depends on Story 3.1** (IRSDK Connection) for the IRSDK reader and connection events
- **Story 3.3 (Session Lifecycle)** consumes captured data and manages session boundaries
- **Story 3.4 (Error Handling)** adds gap detection and reconnection logic on top of capture
- **Story 3.5 (Background Monitoring)** adds resource usage monitoring for the capture engine

## References

- [Source: _bmad-output/planning-artifacts/epics-and-stories.md#Story 3.2: Telemetry Channel Capture]
- [Source: _bmad-output/planning-artifacts/architecture.md#Data Architecture]
- [Source: _bmad-output/planning-artifacts/prd.md FR2: Record driver input, vehicle dynamics, tire/brake, environmental, session context telemetry]
- [Source: _bmad-output/planning-artifacts/prd.md NFR1: <2% CPU, <200MB RSS]
- [Source: _bmad-output/planning-artifacts/prd.md NFR2: <10ms capture latency p99]
- [Source: crates/telemetry-engine/src/ibt_importer.rs -- channel mapping and data parsing patterns]
- [Source: crates/storage/src/parquet/schema.rs -- canonical 35-channel telemetry schema]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### Change Log

### File List
