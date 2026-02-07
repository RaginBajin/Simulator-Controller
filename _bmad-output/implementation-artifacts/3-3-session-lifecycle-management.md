# Story 3.3: Session Lifecycle Management

Status: ready-for-dev

## Story

As a sim racer,
I want the app to automatically detect session start/end and preserve incomplete sessions,
so that I never lose data even if I disconnect or crash.

## Acceptance Criteria

1. **Automatic Session Start Detection**
   - When the user enters a driving session in iRacing and `SessionState` changes to "Racing" or "Testing", the app marks session start with a timestamp
   - A new session record is created in SQLite via `storage::sqlite::queries::sessions::create_session()`
   - System emits `session:capture-started` Tauri event with payload `{ type: "capture-started", timestamp: <ISO8601>, version: 1, sessionId: <uuid> }`
   - System emits internal `session_state_changed` event with state: `recording`

2. **Normal Session End Detection**
   - When `SessionState` changes to "ParadeLap", "CoolDown", or session timer expires, the app marks session end with timestamp
   - System emits `session:capture-stopped` Tauri event with payload `{ type: "capture-stopped", timestamp: <ISO8601>, version: 1, sessionId: <uuid> }`
   - System triggers data flush: all buffered telemetry written to Parquet, session metadata updated in SQLite
   - Session status transitions to `completed` in SQLite
   - System emits `session_completed` internal event with `session_id` to trigger downstream debrief pipeline

3. **Disconnection Handling (Partial Sessions)**
   - When IRSDK connection is lost (alt-F4, crash, network drop), the app preserves all captured data up to disconnection
   - Session is marked as `partial` with disconnection timestamp in SQLite
   - All buffered telemetry is flushed to Parquet immediately on disconnect
   - System emits `capture:irsdk-disconnected` Tauri event (from Story 3.1)
   - Partial sessions remain available for analysis and coaching

4. **Lap Boundary Detection**
   - System detects lap boundaries using iRacing `LapDistPct` crossing logic (value drops from >0.9 to <0.1)
   - Lap boundary timestamps are recorded with <50ms precision
   - Lap summaries (lap number, lap time, validity) are computed immediately upon boundary detection
   - Each completed lap is inserted into SQLite via `storage::sqlite::queries::laps::create_lap()`
   - System emits `session:lap-completed` Tauri event with payload `{ type: "lap-completed", timestamp: <ISO8601>, version: 1, sessionId, lapNumber, lapTimeMs }`

5. **Incomplete Lap Preservation**
   - Incomplete laps (spins, resets, disconnects) are preserved as diagnostic data
   - Incomplete laps are flagged with `completion_status` metadata: `"complete"`, `"incomplete_spin"`, `"incomplete_reset"`, `"incomplete_disconnect"`, `"incomplete_pit"`
   - System inserts gap markers on telemetry stream interruptions
   - Gap markers record `start_time`, `end_time`, and `reason` (disconnect/reset/crash)
   - Analysis correctly handles gaps without misinterpreting data continuity

6. **Session State Machine**
   - Session state follows: `idle` -> `recording` -> `processing` -> `completed`
   - State also supports: `idle` -> `recording` -> `partial` (on disconnect)
   - System emits `session:state-changed` Tauri event on every state transition
   - State is queryable by frontend via the `get_capture_status` IPC command

## Tasks / Subtasks

- [ ] Task 1: Implement session state machine (AC: #1, #2, #6)
  - [ ] 1.1 Create `crates/telemetry-engine/src/session_manager.rs` with `SessionManager` struct
  - [ ] 1.2 Define `SessionState` enum: `Idle`, `Recording { session_id, started_at }`, `Processing { session_id }`, `Completed { session_id }`, `Partial { session_id, disconnected_at }`
  - [ ] 1.3 Implement state transition validation (only valid transitions allowed, log invalid attempts)
  - [ ] 1.4 On transition to `Recording`: create session in SQLite, start telemetry flush timer
  - [ ] 1.5 On transition to `Processing`: stop capture, flush remaining buffer, compute final stats
  - [ ] 1.6 On transition to `Completed`: update session status in SQLite, emit `session_completed` event
  - [ ] 1.7 On transition to `Partial`: flush buffer, mark session as partial in SQLite

- [ ] Task 2: Implement session start/end detection from IRSDK (AC: #1, #2)
  - [ ] 2.1 Read `SessionState` variable from IRSDK shared memory (values: 0=Invalid, 1=GetInCar, 2=Warmup, 3=ParadeLaps, 4=Racing, 5=Checkered, 6=CoolDown)
  - [ ] 2.2 Detect session start: transition to `Racing` (4) or `Warmup` (2) from any other state
  - [ ] 2.3 Detect session end: transition from `Racing`/`Warmup` to `CoolDown` (6), `Checkered` (5), or `ParadeLaps` (3)
  - [ ] 2.4 Also detect session end: `SessionState` stays at `Invalid` (0) for >10 seconds after being in active state
  - [ ] 2.5 Extract session metadata on start: track name, car name, session type from IRSDK session info YAML

- [ ] Task 3: Implement lap boundary detection (AC: #4)
  - [ ] 3.1 Create `crates/telemetry-engine/src/lap_detector.rs` with `LapDetector` struct
  - [ ] 3.2 Implement crossing detection: `LapDistPct` drops from >0.9 to <0.1 between consecutive samples
  - [ ] 3.3 Also use iRacing `Lap` variable as cross-reference (lap number increments)
  - [ ] 3.4 Record lap boundary timestamp from `SessionTime` (precision <50ms at 60Hz = 16.7ms per sample)
  - [ ] 3.5 Compute lap summary on boundary: `lap_time_ms`, `lap_number`, `is_valid` (use iRacing `LapLastLapTime` for official time if available, fall back to computed time)
  - [ ] 3.6 Insert completed lap into SQLite via storage API
  - [ ] 3.7 Update session `lap_count` and `best_lap_time_ms` in SQLite

- [ ] Task 4: Implement incomplete lap and gap marker handling (AC: #5)
  - [ ] 4.1 Define `GapMarker` struct: `{ start_time_ms: i64, end_time_ms: i64, reason: GapReason }`
  - [ ] 4.2 Define `GapReason` enum: `Disconnect`, `Reset`, `Crash`, `Pit`, `Unknown`
  - [ ] 4.3 Detect incomplete laps: sudden large jump in `LapDistPct` (>0.3 in single sample), `SessionState` change mid-lap, IRSDK disconnect mid-lap
  - [ ] 4.4 On incomplete lap detection: insert lap with appropriate `completion_status` into SQLite
  - [ ] 4.5 Insert gap markers into a `gap_markers` field in session metadata (store as JSON in SQLite)
  - [ ] 4.6 When converting to Parquet, include gap marker annotations (metadata or separate column)

- [ ] Task 5: Wire session events to Tauri (AC: #1, #2, #4, #6)
  - [ ] 5.1 Emit `session:capture-started` on session start with `{ type, timestamp, version, sessionId }`
  - [ ] 5.2 Emit `session:capture-stopped` on session end with `{ type, timestamp, version, sessionId }`
  - [ ] 5.3 Emit `session:lap-completed` on each lap with `{ type, timestamp, version, sessionId, lapNumber, lapTimeMs }`
  - [ ] 5.4 Emit `session:state-changed` on every state machine transition with `{ type, timestamp, version, state, sessionId? }`
  - [ ] 5.5 Add `get_capture_status` IPC command returning current session state and active session info
  - [ ] 5.6 Emit `session_completed` internal event (not Tauri event) to trigger debrief generation pipeline

- [ ] Task 6: Implement data flush on session end (AC: #2, #3)
  - [ ] 6.1 On session end (normal or partial): drain ring buffer completely
  - [ ] 6.2 Write all remaining telemetry to Parquet using atomic write (temp + rename)
  - [ ] 6.3 Compute SHA-256 checksum of final Parquet file, store in session metadata (NFR12)
  - [ ] 6.4 Update session record: `ended_at`, `lap_count`, `best_lap_time_ms`, `status`, `telemetry_checksum`, `telemetry_path`
  - [ ] 6.5 On IRSDK disconnect: flush immediately, do not wait for normal end detection

- [ ] Task 7: Unit tests and build verification (AC: all)
  - [ ] 7.1 Write unit tests for session state machine transitions (all valid paths)
  - [ ] 7.2 Write unit tests for session state machine invalid transitions (should reject/log)
  - [ ] 7.3 Write unit tests for lap boundary detection with synthetic `LapDistPct` data
  - [ ] 7.4 Write unit tests for incomplete lap detection (spin, reset, disconnect scenarios)
  - [ ] 7.5 Write unit tests for gap marker creation
  - [ ] 7.6 Run `cargo build` -- must pass
  - [ ] 7.7 Run `cargo test` -- all new and existing tests pass
  - [ ] 7.8 Run `npm run build` -- frontend builds clean

## Dev Notes

### What's Already Built

From Epic 1 (storage crate):
- `storage::sqlite::queries::sessions` -- `create_session()`, `update_session()`, `get_session()` etc.
- `storage::sqlite::queries::laps` -- `create_lap()`, `list_laps_for_session()`
- `storage::parquet::writer` -- Parquet file writer with atomic write support
- `storage::types` -- `Session`, `NewSession`, `SessionUpdate`, `NewLap`, `LapSummary`

From Story 3.1 (IRSDK connection):
- `ConnectionManager` with connect/disconnect events
- IRSDK shared memory reader

From Story 3.2 (telemetry capture):
- `CaptureEngine` with ring buffer and 60Hz capture loop
- `TelemetrySample` struct and RecordBatch conversion

The existing `ibt_importer.rs::compute_laps()` function already implements `LapDistPct` crossing detection. The same algorithm should be used for live lap detection but adapted for streaming (sample-by-sample) rather than batch processing.

### Architecture Patterns to Follow

- **State Machine:** Use a Rust enum for session state with explicit transition methods. Invalid transitions should log a warning and return an error, not panic
- **Event-Based Decoupling:** Session lifecycle emits events that other modules (debrief pipeline, tray icon) consume. No direct coupling between session manager and AI provider
- **IPC Events:** `session:capture-started`, `session:capture-stopped`, `session:lap-completed`, `session:state-changed` per architecture event naming
- **Error Contract:** `{ code, message, details?, retryable? }` for IPC errors
- **Data Integrity:** Atomic Parquet writes (temp + rename), SHA-256 checksums per NFR9/NFR12

### IRSDK SessionState Values

```
0 = Invalid
1 = GetInCar
2 = Warmup
3 = ParadeLaps
4 = Racing
5 = Checkered
6 = CoolDown
```

Session start triggers: transition TO `Racing` (4) or `Warmup` (2)
Session end triggers: transition FROM active state TO `CoolDown` (6) or `Checkered` (5)

### Lap Detection Algorithm (Streaming)

The existing `compute_laps()` in `ibt_importer.rs` uses:
```
if prev_lap_dist_pct > 0.9 && curr_lap_dist_pct < 0.1 -> lap boundary
```

For live streaming, the `LapDetector` maintains state between calls:
- Track `prev_lap_dist_pct` from the last sample
- On each new sample, check for crossing
- Also cross-reference with iRacing's `Lap` variable (integer lap count)
- Immediately compute lap summary and emit event

### Gap Marker Design

Gap markers are stored as JSON in the session record (or a separate table):
```json
{
  "gaps": [
    { "start_time_ms": 120000, "end_time_ms": 120500, "reason": "disconnect" },
    { "start_time_ms": 185000, "end_time_ms": 185200, "reason": "reset" }
  ]
}
```

This avoids schema changes to the Parquet file while preserving gap information.

### Key Files to Create/Modify

| File | Action | Purpose |
|------|--------|---------|
| `crates/telemetry-engine/src/session_manager.rs` | Create | Session state machine and lifecycle management |
| `crates/telemetry-engine/src/lap_detector.rs` | Create | Streaming lap boundary detection |
| `crates/telemetry-engine/src/lib.rs` | Modify | Export session_manager, lap_detector modules |
| `src-tauri/src/commands/capture.rs` | Modify | Add `get_capture_status` command, session events |
| `src-tauri/src/lib.rs` | Modify | Register SessionManager state |

### Relationship to Other Stories

- **Depends on Story 3.1** (IRSDK Connection) for connect/disconnect events
- **Depends on Story 3.2** (Telemetry Capture) for the capture engine and ring buffer
- **Story 3.3b (Manual Debrief Trigger)** adds manual override to trigger debrief from any session state
- **Story 3.4 (Error Handling)** adds reconnection retry logic and more sophisticated gap handling
- **Story 2.2 (System Tray)** consumes session state events for tray icon updates
- **Story 4.3 (Coaching Prompt)** consumes `session_completed` event to start AI analysis

## References

- [Source: _bmad-output/planning-artifacts/epics-and-stories.md#Story 3.3: Session Lifecycle Management]
- [Source: _bmad-output/planning-artifacts/architecture.md#API & Communication Patterns]
- [Source: _bmad-output/planning-artifacts/prd.md FR1a: Detect session end conditions and trigger debrief]
- [Source: _bmad-output/planning-artifacts/prd.md FR3: Detect lap boundaries and compute per-lap statistics]
- [Source: _bmad-output/planning-artifacts/prd.md FR4: Preserve incomplete laps as diagnostic data]
- [Source: _bmad-output/planning-artifacts/prd.md FR5: Detect and handle telemetry stream interruptions with gap markers]
- [Source: crates/telemetry-engine/src/ibt_importer.rs#compute_laps -- existing lap detection algorithm]
- [Source: crates/storage/src/types.rs -- Session, NewSession, NewLap, SessionUpdate types]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### Change Log

### File List
