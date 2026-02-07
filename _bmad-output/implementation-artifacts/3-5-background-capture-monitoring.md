# Story 3.5: Background Capture Monitoring

Status: ready-for-dev

## Story

As a sim racer,
I want the capture engine to run with minimal system impact,
so that it doesn't affect my sim racing performance.

## Acceptance Criteria

1. **Resource Consumption Monitoring**
   - While telemetry capture is active during racing, CPU usage remains <2% of total system CPU (NFR1)
   - Memory (RSS) usage remains <200MB (NFR1)
   - Resource measurements are logged every 60 seconds via the `tracing` crate with structured fields: `cpu_percent`, `rss_mb`, `session_id`
   - Resource metrics are exposed as a Tauri event `capture:resource-stats` for optional frontend display

2. **Resource Measurement Implementation**
   - CPU usage is measured as a 5-second rolling average on the capture thread(s)
   - RSS memory is sampled from the OS process info every 10 seconds
   - Measurements use platform-native APIs (Windows: `GetProcessTimes`/`PROCESS_MEMORY_COUNTERS`; cross-platform fallback for development on macOS/Linux)
   - The resource monitor runs on a separate low-priority background thread to avoid impacting capture performance

3. **Resource Limit Alerting**
   - If CPU usage exceeds 2% rolling average for more than 10 consecutive seconds, the system logs a `tracing::warn!` with details
   - If RSS exceeds 200MB, the system logs a `tracing::warn!` with current usage
   - Resource alerts are emitted as `capture:resource-warning` Tauri event with `{ metric, current, threshold, sessionId }`
   - Resource warnings do NOT stop capture - they are informational only

4. **Background Capture While Minimized**
   - When the user minimizes the app to the system tray, the window closes but the capture engine continues running
   - No UI rendering occurs while the window is minimized (no wasted GPU/CPU on invisible frontend)
   - The system tray icon and tooltip continue updating with capture status (lap count, session state)
   - Capture continues at the same fidelity (60Hz, all 35 channels) regardless of window state

5. **Debrief Ready Event Emission**
   - When post-session processing completes (pre-processing pipeline finishes), the system emits `debrief:ready` event with `{ sessionId, trackName, carName, lapCount, bestLapTimeMs }`
   - The `debrief:ready` event triggers the system tray badge update (amber icon + debrief count)
   - The event follows architecture conventions: includes `type`, `timestamp`, `version` fields
   - If the window is minimized when `debrief:ready` fires, the tray badge updates silently (Story 2.2 dependency)

6. **Parquet Flush on Background Thread**
   - Telemetry data is flushed to Parquet files on a dedicated background I/O thread, not the capture thread
   - The capture thread writes to in-memory ring buffers; the I/O thread periodically drains and persists
   - Parquet flush frequency is configurable (default: every 30 seconds or at session end)
   - Parquet writes use the atomic write pattern from Story 1.4 (write to temp file, rename)

## Tasks / Subtasks

- [ ] Task 1: Implement resource monitor module (AC: #1, #2, #3)
  - [ ] 1.1 Create `crates/telemetry-engine/src/resource_monitor.rs`
  - [ ] 1.2 Implement CPU measurement: track process CPU time delta over 5-second windows, compute percentage
  - [ ] 1.3 Implement RSS measurement: read process memory info from OS APIs
  - [ ] 1.4 Platform abstraction: use `cfg(target_os)` for Windows-native APIs with cross-platform fallback (e.g., `sysinfo` crate or `/proc/self/stat` on Linux, `mach` APIs on macOS)
  - [ ] 1.5 Spawn resource monitor on a low-priority background thread with configurable interval (default: 10s for RSS, 5s for CPU)
  - [ ] 1.6 Log resource metrics every 60 seconds via `tracing::info!` with structured fields

- [ ] Task 2: Implement resource alerting (AC: #3)
  - [ ] 2.1 Define threshold constants: `CPU_THRESHOLD_PERCENT: f64 = 2.0`, `RSS_THRESHOLD_MB: u64 = 200`, `CPU_ALERT_DURATION_SECS: u64 = 10`
  - [ ] 2.2 Track consecutive CPU threshold breaches; warn after 10 seconds sustained
  - [ ] 2.3 Warn immediately on RSS threshold breach
  - [ ] 2.4 Define `ResourceWarning` event payload: `{ metric: "cpu"|"rss", current: f64, threshold: f64, sessionId: String, type, timestamp, version }`
  - [ ] 2.5 Emit `capture:resource-warning` Tauri event on threshold breach

- [ ] Task 3: Implement background I/O thread for Parquet flush (AC: #6)
  - [ ] 3.1 Create `crates/telemetry-engine/src/capture_io.rs`
  - [ ] 3.2 Implement a channel-based I/O worker: capture thread sends `FlushRequest` messages, I/O thread processes them
  - [ ] 3.3 `FlushRequest` contains: session_id, buffered telemetry data (Arrow RecordBatch), flush_type (periodic/session_end)
  - [ ] 3.4 I/O thread writes to Parquet using existing `write_telemetry` API from storage crate
  - [ ] 3.5 Periodic flush every 30 seconds (configurable via `FLUSH_INTERVAL_SECS`)
  - [ ] 3.6 Final flush on session end ensures all buffered data is written before `session:completed` event

- [ ] Task 4: Implement resource stats Tauri event (AC: #1)
  - [ ] 4.1 Define `ResourceStats` event payload: `{ cpuPercent, rssMb, sessionId, uptimeSeconds, type, timestamp, version }`
  - [ ] 4.2 Emit `capture:resource-stats` event every 60 seconds during active capture
  - [ ] 4.3 Event follows architecture conventions with `type`, `timestamp`, `version`

- [ ] Task 5: Implement debrief ready event emission (AC: #5)
  - [ ] 5.1 Define `DebriefReady` event payload: `{ sessionId, trackName, carName, lapCount, bestLapTimeMs, type, timestamp, version }`
  - [ ] 5.2 Emit `debrief:ready` Tauri event when pre-processing pipeline completes
  - [ ] 5.3 Wire event to trigger system tray badge update (coordinate with Story 2.2 tray integration)

- [ ] Task 6: Ensure capture continues while window is minimized (AC: #4)
  - [ ] 6.1 Verify that the Tauri window close-to-tray behavior does not terminate the Rust capture thread
  - [ ] 6.2 Ensure tray icon and tooltip updates continue when window is hidden
  - [ ] 6.3 Verify no frontend rendering occurs when window is hidden (Tauri handles this natively)
  - [ ] 6.4 Test: minimize window during active capture, verify capture continues at full fidelity

- [ ] Task 7: Write unit and integration tests (AC: all)
  - [ ] 7.1 Test: resource monitor reports CPU and RSS values (smoke test - values > 0)
  - [ ] 7.2 Test: resource alerting triggers warn log when threshold exceeded
  - [ ] 7.3 Test: I/O thread processes flush requests and writes Parquet data
  - [ ] 7.4 Test: periodic flush fires at configured interval
  - [ ] 7.5 Test: session end triggers final flush before completion event
  - [ ] 7.6 Test: debrief ready event contains correct session metadata
  - [ ] 7.7 Run `cargo test` and `cargo build` - all pass

## Dev Notes

### Architecture Compliance

**Crate Boundaries:**
- Resource monitoring lives in `crates/telemetry-engine/src/resource_monitor.rs` - telemetry-engine owns the capture pipeline
- Background I/O thread lives in `crates/telemetry-engine/src/capture_io.rs` - coordinates between capture and storage
- Parquet writes use existing `storage` crate APIs (`write_telemetry`) - storage remains a leaf crate
- Event payload definitions live in `src-tauri/src/events.rs` - Tauri app shell orchestrates events

**Threading Model:**
The capture pipeline uses three threads:
1. **Capture thread** - Reads IRSDK shared memory at 60Hz, writes to in-memory ring buffers (Stories 3.1-3.3)
2. **I/O thread** - Periodically drains ring buffers and flushes to Parquet via storage crate (this story)
3. **Resource monitor thread** - Low-priority sampling of CPU/RSS metrics (this story)

All inter-thread communication uses Rust `std::sync::mpsc` channels or `crossbeam` channels. No shared mutable state between threads.

**Event Contract:**
All events follow architecture conventions:
- Event names: `domain:action` in kebab-case
- Event payloads include: `type`, `timestamp`, `version` fields
- Events: `capture:resource-stats`, `capture:resource-warning`, `debrief:ready`

### Existing Code to Build On

From Story 1.4 (done):
- `crates/storage/src/parquet/writer.rs` - `write_telemetry` API with atomic write + checksum
- Parquet schema from `crates/storage/src/parquet/schema.rs`

From Story 3.1-3.3 (backlog - must be implemented first):
- IRSDK connection and polling loop
- Capture thread with ring buffers
- Session lifecycle state machine

From Story 2.2 (ready-for-dev):
- System tray icon state management
- Tray tooltip updates
- Close-to-tray window behavior

### Platform Considerations

**Cross-Platform Resource Monitoring:**
- Windows (production): Use `GetProcessTimes` for CPU, `GetProcessMemoryInfo` for RSS
- macOS (development): Use `mach_task_info` or `sysinfo` crate as fallback
- Linux (CI): Use `/proc/self/stat` or `sysinfo` crate
- Recommended: Use the `sysinfo` crate for cross-platform support during development, with option to replace with platform-native APIs for production optimization

**NFR1 Measurement Methodology:**
Per PRD, NFR1 targets <2% CPU on a 5-second rolling average on a 6-core reference rig (i5-12400 / Ryzen 5 5600). The resource monitor implements this specific measurement: 5-second rolling average of process CPU time divided by total available CPU time.

### File Structure for This Story

```
crates/telemetry-engine/
  Cargo.toml                    # UPDATED - add sysinfo or platform-native deps
  src/
    lib.rs                      # UPDATED - declare new modules
    resource_monitor.rs         # NEW - CPU/RSS monitoring, alerting
    capture_io.rs               # NEW - Background I/O thread, Parquet flush

src-tauri/
  src/
    events.rs                   # UPDATED - ResourceStats, ResourceWarning, DebriefReady event structs
```

### Naming Conventions (Enforced)

| Zone | Convention | Example |
|------|-----------|---------|
| Rust functions | `snake_case` | `start_resource_monitor`, `flush_telemetry`, `emit_debrief_ready` |
| Rust types | `PascalCase` | `ResourceMonitor`, `ResourceStats`, `FlushRequest`, `DebriefReady` |
| Tauri events | `kebab-case` with namespace | `capture:resource-stats`, `capture:resource-warning`, `debrief:ready` |
| IPC JSON fields | `camelCase` | `cpuPercent`, `rssMb`, `sessionId`, `bestLapTimeMs` |
| Constants | `SCREAMING_SNAKE_CASE` | `CPU_THRESHOLD_PERCENT`, `RSS_THRESHOLD_MB`, `FLUSH_INTERVAL_SECS` |

### Cross-Story Dependencies

- **Story 3.1** (backlog): IRSDK connection - provides the connection that capture runs on
- **Story 3.2** (backlog): Telemetry channel capture - provides the capture loop and ring buffers
- **Story 3.3** (backlog): Session lifecycle - provides session start/end detection that triggers flush
- **Story 3.4** (ready-for-dev): Capture error handling - error states may trigger resource alerts
- **Story 1.4** (done): Parquet storage - provides `write_telemetry` API for the I/O thread
- **Story 2.2** (ready-for-dev): System tray - consumes `debrief:ready` event for badge updates

### NFR Compliance

| NFR | Target | How This Story Addresses It |
|-----|--------|-----------------------------|
| NFR1 | <2% CPU, <200MB RSS | Direct implementation: resource monitor measures and alerts on breach |
| NFR7 | 99%+ sample completeness | Background I/O thread prevents capture thread blocking on writes |
| NFR9 | Zero data corruption on crash | Periodic flush (30s) limits data loss window; atomic write pattern for each flush |
| NFR11 | Recovery from iRacing shutdown | Periodic flush ensures most data persisted even on sudden crash |

## References

- [Source: _bmad-output/planning-artifacts/epics-and-stories.md#Story 3.5: Background Capture Monitoring]
- [Source: _bmad-output/planning-artifacts/architecture.md#API & Communication Patterns - Event System]
- [Source: _bmad-output/planning-artifacts/architecture.md#Infrastructure & Deployment - Logging]
- [Source: _bmad-output/planning-artifacts/prd.md#NFR1] (telemetry capture resource consumption - <2% CPU, <200MB RSS)
- [Source: _bmad-output/planning-artifacts/prd.md#NFR7] (telemetry data completeness - 99%+ samples)
- [Source: _bmad-output/planning-artifacts/prd.md#NFR9] (data protection on crash - zero corruption)
- [Source: _bmad-output/planning-artifacts/prd.md#FR34] (run as background process via system tray)
- [Source: _bmad-output/planning-artifacts/ux-design-specification.md#System Tray Icon State System]
- [Source: _bmad-output/planning-artifacts/ux-design-specification.md#Experience Principles - Race first, analyze after]

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List
