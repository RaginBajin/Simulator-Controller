# Story 3.1: IRSDK Connection & Auto-Detection

Status: ready-for-dev

## Story

As a sim racer,
I want the app to automatically detect when iRacing is running and connect to telemetry,
so that I never have to manually configure or start capture.

## Acceptance Criteria

1. **Polling for IRSDK Availability**
   - When iRacing is NOT running, the app polls for IRSDK shared memory availability every 2 seconds
   - Polling runs on a dedicated background thread (not blocking the UI or Tauri event loop)
   - Polling uses minimal CPU (<0.1% idle polling overhead, NFR1)

2. **Auto-Connect When iRacing Launches**
   - When iRacing launches while Pitwall is running, the app detects IRSDK shared memory within 2 seconds
   - On detection, the app opens a connection to IRSDK shared memory for reading telemetry data
   - System emits `capture:irsdk-connected` Tauri event with payload `{ type: "irsdk-connected", timestamp: <ISO8601>, version: 1 }`
   - Connection status changes are logged via `tracing::info!`

3. **Immediate Detection When iRacing Is Already Running**
   - When iRacing is already running at Pitwall launch, the app detects IRSDK on first poll cycle (within 1 second)
   - The app connects and begins capture without user action
   - No user-facing "Connect" button or manual configuration required

4. **Connection Loss Detection**
   - When iRacing exits or IRSDK shared memory becomes unavailable, the app detects disconnection within 2 seconds
   - System emits `capture:irsdk-disconnected` Tauri event with payload `{ type: "irsdk-disconnected", timestamp: <ISO8601>, version: 1 }`
   - After disconnection, the app resumes polling for reconnection (same 2-second interval)

5. **Connection Status State Machine**
   - Connection state follows: `disconnected` -> `connected` -> `disconnected` (cycle)
   - System emits `connection_status_changed` internal event for other engine modules to consume
   - State is queryable by frontend via IPC (for tray icon / status display)

## Tasks / Subtasks

- [ ] Task 1: Create IRSDK reader module with shared memory access (AC: #1, #2, #3)
  - [ ] 1.1 Create `crates/telemetry-engine/src/irsdk/reader.rs` with IRSDK shared memory reader struct
  - [ ] 1.2 Implement Windows shared memory open via `windows` crate (`OpenFileMappingW` / `MapViewOfFile` for `"Local\\IRSDKMemMapFileName"`)
  - [ ] 1.3 Implement `is_connected()` check (shared memory exists and header is valid)
  - [ ] 1.4 Implement `read_header()` to parse IRSDK header from shared memory (version, tick rate, session info offset/length, var header offset, num vars, buffer info)
  - [ ] 1.5 Implement `read_session_info()` to extract YAML session info from shared memory
  - [ ] 1.6 Add cross-platform compilation support: use `#[cfg(target_os = "windows")]` for real IRSDK, provide a mock/stub for non-Windows builds

- [ ] Task 2: Create IRSDK types module (AC: #5)
  - [ ] 2.1 Create `crates/telemetry-engine/src/irsdk/types.rs` with IRSDK data structures
  - [ ] 2.2 Define `ConnectionStatus` enum: `Disconnected`, `Connected`
  - [ ] 2.3 Define `IrsdkHeader` struct matching the IRSDK shared memory header layout
  - [ ] 2.4 Define `IrsdkVarHeader` struct for variable definitions
  - [ ] 2.5 Define `ConnectionEvent` struct: `{ status: ConnectionStatus, timestamp: DateTime<Utc> }`

- [ ] Task 3: Create connection manager with polling loop (AC: #1, #2, #3, #4, #5)
  - [ ] 3.1 Create `crates/telemetry-engine/src/connection.rs` with `ConnectionManager` struct
  - [ ] 3.2 Implement `start()` method that spawns a polling thread (2-second interval)
  - [ ] 3.3 Implement state machine: track current `ConnectionStatus`, detect transitions
  - [ ] 3.4 On connect: log `tracing::info!("IRSDK connected")`, send event via callback/channel
  - [ ] 3.5 On disconnect: log `tracing::info!("IRSDK disconnected")`, send event via callback/channel
  - [ ] 3.6 Implement `stop()` method for graceful shutdown of polling thread
  - [ ] 3.7 Implement `status()` method returning current `ConnectionStatus`
  - [ ] 3.8 Use `std::sync::mpsc` or `tokio::sync::watch` channel for broadcasting status changes

- [ ] Task 4: Wire connection events to Tauri events (AC: #2, #4, #5)
  - [ ] 4.1 Create `src-tauri/src/commands/capture.rs` with `get_capture_status` command
  - [ ] 4.2 Register the `ConnectionManager` as Tauri managed state in `lib.rs`
  - [ ] 4.3 On connection status change, emit Tauri events: `capture:irsdk-connected` / `capture:irsdk-disconnected`
  - [ ] 4.4 Event payloads follow contract: `{ type, timestamp, version: 1 }`
  - [ ] 4.5 Register new Tauri command in `lib.rs` invoke handler

- [ ] Task 5: Add Windows IRSDK dependencies (AC: #1)
  - [ ] 5.1 Add `windows` crate to `telemetry-engine/Cargo.toml` with features: `Win32_Foundation`, `Win32_System_Memory`
  - [ ] 5.2 Add conditional compilation flags in `Cargo.toml` for platform-specific dependencies
  - [ ] 5.3 Ensure non-Windows builds compile cleanly with stub implementation

- [ ] Task 6: Unit tests and build verification (AC: all)
  - [ ] 6.1 Write unit tests for `ConnectionStatus` state machine transitions
  - [ ] 6.2 Write unit tests for IRSDK header parsing (using mock byte buffers)
  - [ ] 6.3 Write tests for connection manager start/stop lifecycle
  - [ ] 6.4 Run `cargo build` -- must pass on current platform (macOS/Linux with stubs)
  - [ ] 6.5 Run `cargo test` -- all new and existing tests pass
  - [ ] 6.6 Run `npm run build` -- frontend builds clean (no regressions)

## Dev Notes

### What's Already Built

The `telemetry-engine` crate exists with:
- `crates/telemetry-engine/src/lib.rs` -- exports `ibt_importer`, `import_orchestrator`, `irsdk` modules
- `crates/telemetry-engine/src/irsdk/mod.rs` -- empty module placeholder (`//! iRacing SDK connection and telemetry capture`)
- `crates/telemetry-engine/src/ibt_importer.rs` -- full .ibt file parser with channel mapping, lap detection, Arrow RecordBatch generation
- `crates/telemetry-engine/Cargo.toml` -- depends on `storage`, `arrow`, `tracing`, `chrono`, `uuid`, `serde`, `serde_json`

The `irsdk/mod.rs` is the entry point for this story. The `ibt_importer.rs` already demonstrates the channel mapping and data parsing patterns to follow.

### Architecture Patterns to Follow

- **IPC Events:** Use `domain:action` kebab-case naming: `capture:irsdk-connected`, `capture:irsdk-disconnected`
- **Event Payload:** Always include `{ type, timestamp, version }` per architecture spec
- **Error Contract:** `{ code, message, details?, retryable? }` for any IPC errors
- **Rust Naming:** `snake_case` functions, `PascalCase` types, `SCREAMING_SNAKE_CASE` constants
- **Crate Boundaries:** `telemetry-engine` depends on `storage` (leaf crate). `src-tauri` orchestrates both
- **Logging:** Use `tracing` crate (`info!`, `warn!`, `error!`) for all diagnostic output

### IRSDK Shared Memory Reference

iRacing exposes telemetry via Windows shared memory:
- **Memory-mapped file name:** `Local\\IRSDKMemMapFileName`
- **Data event name:** `Local\\IRSDKDataValidEvent` (signaled when new data is available)
- **Header structure:** 112 bytes at offset 0 with version, status, tick rate, session info location, var headers location, buffer info
- **Session info:** YAML string at `session_info_offset` for `session_info_length` bytes
- **Variable headers:** Array of variable definitions starting at `var_header_offset`
- **Telemetry buffers:** Double-buffered data at offsets specified in header buffer info

The existing `ibt_importer.rs` already parses the same header format from .ibt files (which are IRSDK shared memory dumps). Reuse the same header structure and channel mapping.

### Cross-Platform Considerations

iRacing (and IRSDK) is Windows-only. For development on macOS/Linux:
- Use `#[cfg(target_os = "windows")]` for real shared memory access
- Provide `#[cfg(not(target_os = "windows"))]` stubs that return `Disconnected` status
- This allows the crate to compile and test on all platforms while the real implementation targets Windows

### Key Files to Create/Modify

| File | Action | Purpose |
|------|--------|---------|
| `crates/telemetry-engine/src/irsdk/reader.rs` | Create | IRSDK shared memory reader |
| `crates/telemetry-engine/src/irsdk/types.rs` | Create | IRSDK data structures and enums |
| `crates/telemetry-engine/src/irsdk/mod.rs` | Modify | Export reader and types modules |
| `crates/telemetry-engine/src/connection.rs` | Create | Connection manager with polling loop |
| `crates/telemetry-engine/src/lib.rs` | Modify | Export connection module |
| `crates/telemetry-engine/Cargo.toml` | Modify | Add `windows` crate dependency |
| `src-tauri/src/commands/capture.rs` | Create | Tauri capture IPC commands |
| `src-tauri/src/commands/mod.rs` | Modify | Export capture commands |
| `src-tauri/src/lib.rs` | Modify | Register ConnectionManager state and commands |

### Dependencies to Add

```toml
# crates/telemetry-engine/Cargo.toml
[target.'cfg(windows)'.dependencies]
windows = { version = "0.58", features = [
    "Win32_Foundation",
    "Win32_System_Memory",
    "Win32_System_Threading",
] }
```

### Relationship to Other Stories

- **Story 3.2 (Telemetry Channel Capture)** depends on this story for the IRSDK connection
- **Story 3.3 (Session Lifecycle Management)** depends on connection events to detect session boundaries
- **Story 2.2 (System Tray Status Indicator)** consumes `capture:irsdk-connected/disconnected` events for tray icon state

## References

- [Source: _bmad-output/planning-artifacts/epics-and-stories.md#Story 3.1: IRSDK Connection & Auto-Detection]
- [Source: _bmad-output/planning-artifacts/architecture.md#API & Communication Patterns]
- [Source: _bmad-output/planning-artifacts/architecture.md#Project Structure]
- [Source: _bmad-output/planning-artifacts/prd.md FR1: Auto-detect iRacing and begin capture]
- [Source: crates/telemetry-engine/src/ibt_importer.rs -- existing IRSDK header/channel parsing patterns]
- [Source: crates/telemetry-engine/src/irsdk/mod.rs -- placeholder module for this story]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### Change Log

### File List
