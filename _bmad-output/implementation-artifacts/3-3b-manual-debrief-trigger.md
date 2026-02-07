# Story 3.3b: Manual Debrief Trigger

Status: ready-for-dev

## Story

As a user,
I want to manually trigger debrief analysis anytime,
so that I can analyze partial sessions or re-analyze completed sessions.

## Acceptance Criteria

1. **System Tray Menu "Generate Debrief" Option**
   - The system tray context menu includes a "Generate Debrief" menu item
   - The menu item is visible when a session is active (recording) or completed
   - The menu item is disabled (grayed out) when no session data exists (idle, no previous sessions)

2. **Immediate Debrief Generation on Manual Trigger**
   - When the user selects "Generate Debrief" from the system tray menu, debrief analysis starts immediately
   - For active sessions: current telemetry is flushed from the ring buffer to Parquet, then debrief pipeline begins on captured data so far (capture continues in background)
   - For completed sessions: debrief pipeline runs on the most recent completed session
   - The debrief generation follows the normal pipeline (pre-processing -> AI analysis -> store results)

3. **Manual Trigger Works for Incomplete Sessions**
   - If the session is still recording, the manual trigger generates a debrief from data captured up to that point
   - The session is NOT stopped -- capture continues after the debrief snapshot
   - The debrief is tagged as `manual` trigger type (vs `automatic` for normal end-of-session)
   - Multiple manual triggers during a single session create multiple debriefs (each with progressively more data)

4. **Re-analysis of Completed Sessions**
   - Users can trigger re-analysis of any previously completed session
   - Re-analysis generates a new debrief (does not overwrite the existing one)
   - This supports use cases like: changed AI provider, updated coaching prompts, wanting a fresh analysis

5. **Tauri Events for Manual Trigger**
   - System emits `session:debrief-requested` Tauri event with payload `{ type: "debrief-requested", timestamp: <ISO8601>, version: 1, sessionId, triggerType: "manual" }`
   - Normal debrief pipeline events follow (`ai:analysis-complete` or `ai:analysis-error`)

## Tasks / Subtasks

- [ ] Task 1: Add "Generate Debrief" to system tray menu (AC: #1)
  - [ ] 1.1 Modify `src-tauri/src/tray.rs` to add "Generate Debrief" menu item to the tray context menu
  - [ ] 1.2 Set menu item to disabled by default (no session data)
  - [ ] 1.3 Enable menu item when `SessionManager` state is `Recording` or when completed sessions exist
  - [ ] 1.4 Update menu item state dynamically when session state changes

- [ ] Task 2: Implement manual debrief trigger command (AC: #2, #3, #5)
  - [ ] 2.1 Create `trigger_debrief` Tauri command in `src-tauri/src/commands/capture.rs`
  - [ ] 2.2 Command accepts optional `session_id` parameter: if provided, trigger for that session; if absent, use current/most recent session
  - [ ] 2.3 For active sessions: snapshot ring buffer (drain a copy without stopping capture), flush snapshot to temp Parquet, trigger debrief pipeline
  - [ ] 2.4 For completed sessions: trigger debrief pipeline directly on existing Parquet data
  - [ ] 2.5 Emit `session:debrief-requested` Tauri event with `triggerType: "manual"`
  - [ ] 2.6 Return `{ sessionId, debriefId, status: "started" }` response to frontend

- [ ] Task 3: Implement ring buffer snapshot for active sessions (AC: #3)
  - [ ] 3.1 Add `snapshot()` method to `TelemetryRingBuffer` (from Story 3.2) that returns a copy of current buffer contents without draining
  - [ ] 3.2 Snapshot is a point-in-time copy -- capture continues writing to the live buffer
  - [ ] 3.3 Convert snapshot to RecordBatch and write to a temporary Parquet file for debrief analysis
  - [ ] 3.4 Tag the snapshot Parquet with metadata indicating it's a partial capture

- [ ] Task 4: Support multiple debriefs per session (AC: #3, #4)
  - [ ] 4.1 Verify `storage::sqlite::queries::ai_results` supports multiple debriefs for the same `session_id`
  - [ ] 4.2 Add `trigger_type` field to `NewAiDebrief` struct: `"automatic"` | `"manual"`
  - [ ] 4.3 Add `data_range` field to debrief metadata: `{ from_time_ms, to_time_ms }` to indicate what portion of the session was analyzed
  - [ ] 4.4 If storage schema needs modification, add a migration for the new fields

- [ ] Task 5: Wire tray menu click to command (AC: #1, #2)
  - [ ] 5.1 Handle tray menu "Generate Debrief" click event in `tray.rs`
  - [ ] 5.2 On click: invoke the `trigger_debrief` command internally (or emit an internal event that the command handler processes)
  - [ ] 5.3 Show tray tooltip update: "Generating debrief..." while processing

- [ ] Task 6: Unit tests and build verification (AC: all)
  - [ ] 6.1 Write unit test for ring buffer `snapshot()` -- verify it returns a copy without draining
  - [ ] 6.2 Write unit test for `trigger_debrief` command with active session
  - [ ] 6.3 Write unit test for `trigger_debrief` command with completed session
  - [ ] 6.4 Write unit test for multiple debriefs per session (verify they don't overwrite)
  - [ ] 6.5 Run `cargo build` -- must pass
  - [ ] 6.6 Run `cargo test` -- all new and existing tests pass
  - [ ] 6.7 Run `npm run build` -- frontend builds clean

## Dev Notes

### What's Already Built

From Story 2.1/2.2 (Desktop Foundation):
- `src-tauri/src/tray.rs` -- basic tray icon setup (may have initial menu items)
- Tray icon registered in `lib.rs` setup

From Story 3.2 (Telemetry Capture):
- `TelemetryRingBuffer` with `push()` and `drain()` methods
- `CaptureEngine` with start/stop capture

From Story 3.3 (Session Lifecycle):
- `SessionManager` with session state machine
- Session CRUD via storage APIs
- Lap detection and session events

From Epic 1 (Storage):
- `storage::sqlite::queries::ai_results` -- `create_debrief()`, `get_debrief_for_session()`
- `storage::types::NewAiDebrief`, `AiDebrief`

### Architecture Patterns to Follow

- **IPC Events:** `session:debrief-requested` with `{ type, timestamp, version, sessionId, triggerType }`
- **Error Contract:** `{ code, message, details?, retryable? }` for IPC errors
- **UX Philosophy:** Manual debrief is a power-user feature accessed via system tray menu -- not a prominent UI button. This preserves the automatic-first UX philosophy (per PRD FR1b: "System tray menu option for power users; no prominent UI button")
- **Event-Based Decoupling:** The manual trigger emits the same `session_completed` (or similar) internal event that automatic session end emits, so the debrief pipeline doesn't need to know whether the trigger was manual or automatic

### System Tray Menu Structure

The tray context menu should look like:
```
Pitwall
---------
Generate Debrief    [enabled when session data exists]
---------
Settings
Quit
```

### Ring Buffer Snapshot vs Drain

- `drain()` removes items from the buffer (used for periodic flush and session end)
- `snapshot()` copies items without removing them (used for manual debrief mid-session)
- Implementation: `snapshot()` locks the buffer, clones the VecDeque contents, unlocks
- This is safe because the clone happens under the mutex; capture thread is briefly blocked

### Key Files to Create/Modify

| File | Action | Purpose |
|------|--------|---------|
| `src-tauri/src/tray.rs` | Modify | Add "Generate Debrief" menu item |
| `src-tauri/src/commands/capture.rs` | Modify | Add `trigger_debrief` command |
| `crates/telemetry-engine/src/ring_buffer.rs` | Modify | Add `snapshot()` method |
| `crates/storage/src/types.rs` | Modify | Add `trigger_type` to `NewAiDebrief` (if needed) |

### Relationship to Other Stories

- **Depends on Story 3.2** (Telemetry Capture) for ring buffer
- **Depends on Story 3.3** (Session Lifecycle) for session state machine
- **Depends on Story 2.2** (System Tray) for tray menu infrastructure
- **Story 4.3 (Coaching Prompt)** consumes the debrief trigger event to start AI analysis
- This story enables the PRD's power-user workflow: manual analysis during practice sessions

## References

- [Source: _bmad-output/planning-artifacts/epics-and-stories.md#Story 3.3b: Manual Debrief Trigger]
- [Source: _bmad-output/planning-artifacts/prd.md FR1b: User can manually trigger debrief analysis]
- [Source: _bmad-output/planning-artifacts/ux-design-specification.md#Effortless Interactions -- "Power users can manually trigger debrief via system tray menu option"]
- [Source: _bmad-output/planning-artifacts/architecture.md#API & Communication Patterns]
- [Source: crates/storage/src/types.rs -- AiDebrief, NewAiDebrief types]

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### Change Log

### File List
