# Story 3.4: Capture Error Handling & Recovery

Status: ready-for-dev

## Story

As a sim racer experiencing telemetry issues,
I want the app to gracefully handle data gaps and provide clear error messages,
so that I understand what went wrong and can take action if needed.

## Acceptance Criteria

1. **Telemetry Gap Detection and Marking**
   - When a data gap occurs during active capture (IRSDK stalls for >100ms), the system detects the gap within 1 second (NFR8)
   - The gap is marked with a `GapMarker` struct containing: `start_time`, `end_time`, `duration_ms`, `reason` (disconnect/reset/stall), and `lap_position` (track distance at gap start)
   - Capture resumes automatically after the gap without user action
   - Gap duration and count are recorded in session metadata for the debrief
   - Any gap >500ms is explicitly flagged with timestamp, duration, and lap position in session data (NFR7)

2. **IRSDK Reconnection Logic**
   - When IRSDK connection fails during an active session, the system attempts reconnection every 1 second
   - Reconnection attempts continue for 30 seconds maximum
   - During reconnection, the system tray tooltip displays "Reconnecting..." with attempt count
   - A Tauri event `capture:reconnecting` is emitted with `{ attemptNumber, maxAttempts, elapsedMs }`
   - If reconnection succeeds within 30 seconds, capture resumes and a gap marker is inserted for the disconnection period
   - On successful reconnection, a Tauri event `capture:reconnected` is emitted with gap duration

3. **Connection Failure After Timeout**
   - If IRSDK connection cannot be re-established after 30 seconds, the system stops capture attempts
   - The system emits `capture:error` event with `{ code: "IRSDK_CONNECTION_LOST", message: "Telemetry connection lost", details: "No response from iRacing after 30 seconds", retryable: false }`
   - All captured data up to the disconnection point is preserved as a partial session
   - The session status is set to `"partial"` with a `disconnected_at` timestamp
   - The system tray icon changes to red with tooltip "Capture error - Telemetry connection lost"

4. **Session Completion Triggers Debrief Pipeline**
   - When a session end is detected (normal completion or timeout after disconnect), the system emits `session:completed` event with `{ sessionId, lapCount, duration, status, gapCount }`
   - The `session_completed` event triggers the debrief generation pipeline (Epic 4 dependency - event emitted, downstream handled by ai-provider)
   - Partial sessions (disconnected) also emit `session:completed` so they can be analyzed
   - The event follows the architecture event contract: includes `type`, `timestamp`, `version` fields

5. **Error Classification and Reporting**
   - Capture errors are classified into categories: `IrsdkStall` (brief data gap), `IrsdkDisconnect` (connection lost), `IrsdkTimeout` (reconnection failed), `StorageError` (write failure), `ResourceExhausted` (memory/CPU limit)
   - Each error includes: error code, user-friendly message, technical details (for logging), and `retryable` flag
   - Errors follow the IPC error contract: `{ code, message, details?, retryable? }`
   - All errors are logged via the `tracing` crate with structured fields (session_id, lap_number, track_position)

6. **Graceful Degradation Chain**
   - The system follows the degradation chain: Full capture -> Gap marking -> Partial session -> Capture stopped
   - At each degradation level, the system preserves as much data as possible
   - Gap markers allow downstream analysis to correctly handle discontinuities without misinterpreting data continuity
   - Partial sessions are stored and queryable like complete sessions, with a clear status indicator

## Tasks / Subtasks

- [ ] Task 1: Define capture error types and gap marker model (AC: #1, #5)
  - [ ] 1.1 Create `crates/telemetry-engine/src/capture_error.rs` with `CaptureError` enum: `IrsdkStall`, `IrsdkDisconnect`, `IrsdkTimeout`, `StorageError`, `ResourceExhausted`
  - [ ] 1.2 Define `GapMarker` struct with fields: `start_time: i64`, `end_time: i64`, `duration_ms: u64`, `reason: GapReason`, `lap_position: Option<f64>`
  - [ ] 1.3 Define `GapReason` enum: `Disconnect`, `Stall`, `Reset`, `Unknown`
  - [ ] 1.4 Implement `From<CaptureError>` for the IPC error contract struct `{ code, message, details, retryable }`
  - [ ] 1.5 Add `CaptureError` variants to `StorageError` or create a `TelemetryError` enum in `telemetry-engine`

- [ ] Task 2: Implement gap detection logic (AC: #1)
  - [ ] 2.1 Create `crates/telemetry-engine/src/gap_handler.rs`
  - [ ] 2.2 Implement gap detection: compare timestamps between consecutive samples; if delta > 100ms (configurable threshold), create a `GapMarker`
  - [ ] 2.3 Maintain a `Vec<GapMarker>` in the capture session state
  - [ ] 2.4 Log each detected gap via `tracing::warn!` with structured fields: `gap_duration_ms`, `reason`, `lap_position`
  - [ ] 2.5 Expose gap summary (count, total duration) for session metadata

- [ ] Task 3: Implement IRSDK reconnection state machine (AC: #2, #3)
  - [ ] 3.1 Add reconnection state to the IRSDK connection manager: `Connected`, `Reconnecting { attempt: u32, started_at: Instant }`, `Disconnected`
  - [ ] 3.2 Implement reconnection loop: poll IRSDK shared memory every 1 second, up to 30 attempts
  - [ ] 3.3 On successful reconnection: transition to `Connected`, insert gap marker for the disconnection period, emit `capture:reconnected` event
  - [ ] 3.4 On timeout (30s): transition to `Disconnected`, emit `capture:error` event, preserve partial session data
  - [ ] 3.5 Ensure reconnection logic runs on a background thread and does not block the capture pipeline

- [ ] Task 4: Implement session completion event emission (AC: #4)
  - [ ] 4.1 Define `SessionCompleted` event payload struct with: `session_id`, `lap_count`, `duration_ms`, `status` (completed/partial), `gap_count`, `type`, `timestamp`, `version`
  - [ ] 4.2 Emit `session:completed` Tauri event on normal session end detection
  - [ ] 4.3 Emit `session:completed` Tauri event on connection timeout (partial session)
  - [ ] 4.4 Ensure event payload follows architecture event conventions (includes `type`, `timestamp`, `version`)

- [ ] Task 5: Add SQLite migration for gap markers (AC: #1, #6)
  - [ ] 5.1 Create migration `crates/storage/migrations/NNN_add_gap_markers.sql`
  - [ ] 5.2 Add `gap_markers` table: `id INTEGER PRIMARY KEY`, `session_id TEXT NOT NULL REFERENCES sessions(id)`, `start_time INTEGER NOT NULL`, `end_time INTEGER NOT NULL`, `duration_ms INTEGER NOT NULL`, `reason TEXT NOT NULL`, `lap_position REAL`
  - [ ] 5.3 Add `gap_count INTEGER DEFAULT 0` and `total_gap_duration_ms INTEGER DEFAULT 0` columns to `sessions` table
  - [ ] 5.4 Add query functions: `insert_gap_marker`, `get_gap_markers_for_session`

- [ ] Task 6: Implement Tauri event emission for capture status (AC: #2, #3, #4)
  - [ ] 6.1 Define event payload structs in `src-tauri/src/events.rs`: `CaptureReconnecting`, `CaptureReconnected`, `CaptureError`, `SessionCompleted`
  - [ ] 6.2 All event structs include `type`, `timestamp`, `version` per architecture convention
  - [ ] 6.3 Wire event emission into the capture error handler and reconnection state machine

- [ ] Task 7: Update session status for partial sessions (AC: #3, #6)
  - [ ] 7.1 Add `"partial"` as a valid session status alongside existing statuses
  - [ ] 7.2 Add `disconnected_at` column to sessions table (nullable timestamp)
  - [ ] 7.3 Update session record on disconnect timeout: set status to partial, record disconnected_at
  - [ ] 7.4 Ensure partial sessions appear in session list and are filterable

- [ ] Task 8: Write unit and integration tests (AC: all)
  - [ ] 8.1 Test: gap detection triggers when sample delta exceeds threshold
  - [ ] 8.2 Test: gap marker contains correct start_time, end_time, duration, reason
  - [ ] 8.3 Test: reconnection state machine transitions through Connected -> Reconnecting -> Connected on success
  - [ ] 8.4 Test: reconnection state machine transitions to Disconnected after 30s timeout
  - [ ] 8.5 Test: session_completed event is emitted for both normal and partial sessions
  - [ ] 8.6 Test: error classification produces correct error codes and retryable flags
  - [ ] 8.7 Test: gap markers are persisted to SQLite and retrievable
  - [ ] 8.8 Run `cargo test` and `cargo build` - all pass

## Dev Notes

### Architecture Compliance

**Crate Boundaries:**
Per architecture doc:
- Gap detection and reconnection logic live in `crates/telemetry-engine/src/` - this crate owns all capture pipeline logic
- Gap marker storage (SQLite migration, queries) lives in `crates/storage/` - leaf crate handles persistence
- Tauri event emission and IPC commands live in `src-tauri/src/` - orchestrates events to frontend
- `telemetry-engine` depends on `storage` (allowed per dependency graph)
- `storage` does NOT depend on `telemetry-engine`

**Event Contract:**
All events follow architecture conventions:
- Event names: `domain:action` in kebab-case (e.g., `capture:reconnecting`, `capture:error`, `session:completed`)
- Event payloads include: `type`, `timestamp`, `version` fields
- Streaming payloads include `sequenceId` for ordering where applicable

**Error Contract:**
All errors follow the IPC error format: `{ code, message, details?, retryable? }`

### Existing Code to Build On

From Story 3.1 (backlog - IRSDK connection):
- `crates/telemetry-engine/src/irsdk/mod.rs` - IRSDK connection module (currently placeholder)
- Connection detection and polling logic

From Story 3.2 (backlog - telemetry channel capture):
- 60Hz capture loop with ring buffers
- Sample timestamp tracking (needed for gap detection)

From Story 3.3 (backlog - session lifecycle):
- Session state machine: idle -> recording -> processing -> completed
- Session start/end detection
- Lap boundary detection

From Epic 1 (done):
- `crates/storage/src/sqlite/queries/sessions.rs` - session CRUD
- `crates/storage/src/types.rs` - `Session`, `NewSession` types
- `crates/storage/src/error.rs` - `StorageError` enum

**Dependencies:** Stories 3.1, 3.2, and 3.3 should be implemented first as they provide the IRSDK connection, capture loop, and session lifecycle that this story adds error handling around. This story adds resilience to the capture pipeline built in those stories.

### File Structure for This Story

```
crates/telemetry-engine/
  src/
    capture_error.rs           # NEW - CaptureError enum, GapMarker, GapReason
    gap_handler.rs             # NEW - Gap detection logic, gap marker management
    irsdk/
      mod.rs                   # UPDATED - Add reconnection state machine
      reconnection.rs          # NEW - Reconnection loop logic

crates/storage/
  migrations/
    NNN_add_gap_markers.sql    # NEW - gap_markers table, session columns
  src/
    sqlite/queries/
      sessions.rs              # UPDATED - partial session support, gap columns
      gap_markers.rs           # NEW - gap marker CRUD queries
    types.rs                   # UPDATED - GapMarker storage type, session status "partial"

src-tauri/
  src/
    events.rs                  # UPDATED - Capture event payload structs
```

### Naming Conventions (Enforced)

| Zone | Convention | Example |
|------|-----------|---------|
| Rust functions | `snake_case` | `detect_gap`, `attempt_reconnection`, `emit_session_completed` |
| Rust types | `PascalCase` | `GapMarker`, `CaptureError`, `GapReason`, `ReconnectionState` |
| DB columns | `snake_case` | `gap_count`, `total_gap_duration_ms`, `disconnected_at` |
| DB tables | `snake_case` plural | `gap_markers` |
| Tauri events | `kebab-case` with namespace | `capture:reconnecting`, `capture:error`, `session:completed` |
| IPC JSON fields | `camelCase` | `gapCount`, `attemptNumber`, `sessionId` |

### Cross-Story Dependencies

- **Story 3.1** (backlog): IRSDK connection - provides the connection layer that this story adds reconnection to
- **Story 3.2** (backlog): Telemetry channel capture - provides the capture loop that this story adds gap detection to
- **Story 3.3** (backlog): Session lifecycle management - provides session state machine that this story extends with partial status
- **Story 1.3** (done): SQLite database - provides Database struct and session APIs
- **Story 4.3** (backlog): AI coaching prompt engineering - consumes `session:completed` event to trigger debrief
- **Story 2.2** (ready-for-dev): System tray - displays reconnection status and error states

### NFR Compliance

| NFR | Target | How This Story Addresses It |
|-----|--------|-----------------------------|
| NFR1 | <2% CPU, <200MB RSS | Gap detection is lightweight timestamp comparison, no additional CPU load |
| NFR7 | 99%+ sample completeness | Gap markers enable accurate completeness measurement |
| NFR8 | <1s gap detection | Gap detection within 100ms threshold comparison |
| NFR9 | Zero data corruption on crash | Partial sessions use same atomic write patterns |
| NFR10 | Graceful degradation on AI failure | Session completed events emitted regardless of AI availability |
| NFR11 | Recovery from unexpected iRacing shutdown | All data preserved up to last captured sample |

## References

- [Source: _bmad-output/planning-artifacts/epics-and-stories.md#Story 3.4: Capture Error Handling & Recovery]
- [Source: _bmad-output/planning-artifacts/architecture.md#API & Communication Patterns - Event System]
- [Source: _bmad-output/planning-artifacts/architecture.md#Failure Modes & Preventive Controls]
- [Source: _bmad-output/planning-artifacts/prd.md#FR5] (detect and handle telemetry stream interruptions with gap markers)
- [Source: _bmad-output/planning-artifacts/prd.md#FR4] (preserve telemetry from incomplete laps)
- [Source: _bmad-output/planning-artifacts/prd.md#NFR7] (telemetry data completeness - 99%+ samples, gaps flagged)
- [Source: _bmad-output/planning-artifacts/prd.md#NFR8] (telemetry stream interruption detection - <1 second)
- [Source: _bmad-output/planning-artifacts/prd.md#NFR9] (data protection on crash)
- [Source: _bmad-output/planning-artifacts/prd.md#NFR11] (recovery from unexpected iRacing shutdown)
- [Source: _bmad-output/planning-artifacts/ux-design-specification.md#Journey 3 - The Bad Session]
- [Source: _bmad-output/planning-artifacts/ux-design-specification.md#System Tray Icon State System]

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List
