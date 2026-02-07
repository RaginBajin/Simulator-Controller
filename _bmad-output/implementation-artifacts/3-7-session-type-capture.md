# Story 3.7: Session Type Capture

Status: ready-for-dev

## Story

As a user,
I want session type automatically detected,
so that analysis is contextually relevant (practice vs race).

## Acceptance Criteria

1. **Session Type Detection from IRSDK**
   - When telemetry capture starts, the system reads the iRacing session type from IRSDK session info data
   - Supported session types: `practice`, `qualifying`, `race`, `warmup`, `testing`
   - Session type is detected from the IRSDK `SessionInfo` YAML data (specifically the `Sessions` array, current `SessionType` field)
   - Detection happens at session start and is stored as immutable metadata for the session

2. **Session Type Stored with Metadata**
   - Session type is stored in the `sessions` SQLite table as a `session_type TEXT` column
   - Session type is included in the `NewSession` struct used by `insert_session`
   - Session type is included in all session query responses (`Session`, `SessionSummary` types)
   - IPC responses include `sessionType` field (camelCase per convention)

3. **Session Type Visible in UI and Used for Filtering**
   - Session type is included in the `get_sessions` IPC response payload
   - The `get_sessions` command supports filtering by session type (e.g., `?sessionType=race`)
   - Session type is displayed in the SessionCard component (future UI story - this story provides the data)
   - Session type badge values map to display labels: `practice` -> "Practice", `qualifying` -> "Qualifying", `race` -> "Race", `warmup` -> "Warmup", `testing` -> "Testing"

4. **Session Type Contextual Behavior**
   - Different session types influence downstream analysis context:
     - `race`: Full competitive analysis, position tracking, incident detection priority
     - `qualifying`: Focus on single-lap performance, peak lap analysis
     - `practice`: Focus on consistency improvement, learning progression
     - `warmup`/`testing`: Baseline data collection, minimal coaching expectations
   - Session type is included in the AI coaching prompt context (consumed by Story 4.3)
   - Session type is available for session history filtering (consumed by Story 1.6)

5. **Fallback for Unknown Session Types**
   - If the IRSDK session type is unrecognized or unavailable, default to `"practice"`
   - Log a `tracing::warn!` for unrecognized session types with the raw value
   - The raw IRSDK session type string is stored in a `raw_session_type` column for diagnostic purposes

6. **Session Type Detection from .ibt Import**
   - When importing .ibt files (Story 1.8), session type is extracted from the session info YAML
   - The `.ibt` parser's existing metadata extraction (ibt_importer.rs) already reads `SessionType`; this story ensures the value is correctly mapped and stored
   - If the `.ibt` file's session type is unrecognized, fallback to `"practice"`

## Tasks / Subtasks

- [ ] Task 1: Add SQLite migration for session type columns (AC: #2, #5)
  - [ ] 1.1 Create migration `crates/storage/migrations/NNN_add_session_type.sql`
  - [ ] 1.2 Add `session_type TEXT NOT NULL DEFAULT 'practice'` to `sessions` table
  - [ ] 1.3 Add `raw_session_type TEXT` to `sessions` table (nullable, for diagnostics)
  - [ ] 1.4 Verify migration runs on existing database without breaking existing sessions

- [ ] Task 2: Update storage types and queries (AC: #2, #3)
  - [ ] 2.1 Add `session_type: String` to `NewSession` struct in `crates/storage/src/types.rs`
  - [ ] 2.2 Add `session_type: String` and `raw_session_type: Option<String>` to `Session` struct
  - [ ] 2.3 Add `session_type: String` to `SessionSummary` struct
  - [ ] 2.4 Update `insert_session` query to include `session_type` and `raw_session_type`
  - [ ] 2.5 Update `get_sessions` query to include `session_type` in results
  - [ ] 2.6 Add `session_type` filter parameter to `get_sessions` query (optional filter)
  - [ ] 2.7 Ensure IPC response types use `#[serde(rename_all = "camelCase")]` for `sessionType`

- [ ] Task 3: Implement session type mapping (AC: #1, #4, #5)
  - [ ] 3.1 Create `crates/telemetry-engine/src/session_type.rs`
  - [ ] 3.2 Define `SessionType` enum: `Practice`, `Qualifying`, `Race`, `Warmup`, `Testing`
  - [ ] 3.3 Implement `SessionType::from_irsdk(raw: &str) -> SessionType` with mapping logic
  - [ ] 3.4 iRacing session type strings to map: `"Practice"` -> Practice, `"Qualify"` -> Qualifying, `"Race"` -> Race, `"Warmup"` -> Warmup, `"Testing"` -> Testing, `"Lone Qualify"` -> Qualifying, `"Open Qualify"` -> Qualifying
  - [ ] 3.5 Default to `Practice` for unrecognized values; log warning with raw value
  - [ ] 3.6 Implement `SessionType::display_label(&self) -> &str` for UI labels
  - [ ] 3.7 Implement `Serialize`/`Deserialize` for SessionType (serialize as lowercase string)

- [ ] Task 4: Wire session type into capture pipeline (AC: #1)
  - [ ] 4.1 Read session type from IRSDK session info at capture start
  - [ ] 4.2 Parse the `Sessions` array in the IRSDK YAML to find the current session's `SessionType`
  - [ ] 4.3 Store session type in the capture session state alongside other metadata
  - [ ] 4.4 Include session type in the `NewSession` passed to `insert_session` when session is persisted

- [ ] Task 5: Wire session type into .ibt import (AC: #6)
  - [ ] 5.1 The existing `ibt_importer.rs` already extracts `SessionType` from YAML metadata
  - [ ] 5.2 Map the extracted session type through `SessionType::from_irsdk`
  - [ ] 5.3 Include mapped session type in the `ImportedSession` metadata
  - [ ] 5.4 Ensure the import orchestrator passes session type to `insert_session`/`insert_imported_session`

- [ ] Task 6: Include session type in Tauri IPC (AC: #3)
  - [ ] 6.1 Update `get_sessions` command response to include `sessionType` field
  - [ ] 6.2 Add optional `session_type` filter parameter to `get_sessions` command
  - [ ] 6.3 Include `sessionType` in `session:completed` and `session:state-changed` events
  - [ ] 6.4 Include `sessionType` in `get_session_data` response

- [ ] Task 7: Write unit and integration tests (AC: all)
  - [ ] 7.1 Test: `SessionType::from_irsdk("Practice")` returns `Practice`
  - [ ] 7.2 Test: `SessionType::from_irsdk("Race")` returns `Race`
  - [ ] 7.3 Test: `SessionType::from_irsdk("Qualify")` returns `Qualifying`
  - [ ] 7.4 Test: `SessionType::from_irsdk("Lone Qualify")` returns `Qualifying`
  - [ ] 7.5 Test: `SessionType::from_irsdk("UnknownType")` returns `Practice` (fallback)
  - [ ] 7.6 Test: session type persists to SQLite and is retrievable
  - [ ] 7.7 Test: session type filter in `get_sessions` returns only matching sessions
  - [ ] 7.8 Test: `SessionType` serializes as lowercase string (e.g., `"qualifying"`)
  - [ ] 7.9 Run `cargo test` and `cargo build` - all pass

## Dev Notes

### Architecture Compliance

**Crate Boundaries:**
- Session type mapping logic lives in `crates/telemetry-engine/src/session_type.rs` - telemetry-engine owns IRSDK data interpretation
- Storage schema and queries live in `crates/storage/` - leaf crate handles persistence
- IPC command updates live in `src-tauri/src/commands/session.rs` - app shell handles frontend communication
- `telemetry-engine` depends on `storage` (allowed per dependency graph)

**Data Flow:**
```
IRSDK Session Info (YAML)
  -> telemetry-engine parses SessionType field
  -> Maps to SessionType enum via from_irsdk()
  -> Stored in session metadata (SQLite via storage crate)
  -> Returned in IPC responses to frontend
  -> Used by AI coaching for contextual prompts (Epic 4)
```

### IRSDK Session Type Values

iRacing's IRSDK provides session type in the session info YAML block. The relevant YAML structure:

```yaml
SessionInfo:
  Sessions:
    - SessionNum: 0
      SessionType: Practice
      ...
    - SessionNum: 1
      SessionType: Qualify
      ...
    - SessionNum: 2
      SessionType: Race
      ...
```

Known iRacing session type strings:
- `"Practice"` - Solo or hosted practice
- `"Qualify"` - Standard qualifying
- `"Lone Qualify"` - Single-car qualifying
- `"Open Qualify"` - Open qualifying
- `"Race"` - Race session
- `"Warmup"` - Pre-race warmup
- `"Testing"` - Test session (less common)

The current session can be identified by cross-referencing `SessionNum` with the `SessionInfo.CurrentSessionNum` field or by matching the session state.

### Existing Code to Build On

From Story 1.8 (done):
- `crates/telemetry-engine/src/ibt_importer.rs` lines 261-337 - already extracts `SessionType` from .ibt YAML metadata via `extract_metadata_from_yaml`
- The current extraction stores it as a raw string; this story adds proper mapping

From Story 1.3 (done):
- `crates/storage/src/sqlite/queries/sessions.rs` - `insert_session`, `get_sessions` queries
- `crates/storage/src/types.rs` - `Session`, `NewSession`, `SessionSummary` types

From Story 3.1 (backlog):
- IRSDK connection provides access to session info YAML
- Session info is read at connection time and on session changes

From Story 1.6 (done):
- Session list filtering - this story adds session_type as a new filter dimension

### File Structure for This Story

```
crates/telemetry-engine/
  src/
    lib.rs                      # UPDATED - declare session_type module
    session_type.rs             # NEW - SessionType enum, from_irsdk mapping

crates/storage/
  migrations/
    NNN_add_session_type.sql    # NEW - session_type columns
  src/
    types.rs                    # UPDATED - add session_type to Session, NewSession, SessionSummary
    sqlite/queries/
      sessions.rs               # UPDATED - include session_type in queries, add filter

src-tauri/
  src/
    commands/
      session.rs                # UPDATED - include sessionType in responses, add filter param
```

### Naming Conventions (Enforced)

| Zone | Convention | Example |
|------|-----------|---------|
| Rust functions | `snake_case` | `from_irsdk`, `display_label`, `get_sessions_by_type` |
| Rust types | `PascalCase` | `SessionType` |
| DB columns | `snake_case` | `session_type`, `raw_session_type` |
| IPC JSON fields | `camelCase` | `sessionType`, `rawSessionType` |
| Enum variants | `PascalCase` | `Practice`, `Qualifying`, `Race`, `Warmup`, `Testing` |
| Serialized enum values | lowercase | `"practice"`, `"qualifying"`, `"race"` |

### Cross-Story Dependencies

- **Story 3.1** (backlog): IRSDK connection - provides access to session info YAML for type detection
- **Story 3.3** (backlog): Session lifecycle - session type is captured at session start
- **Story 1.3** (done): SQLite database - session insert/query APIs
- **Story 1.6** (done): Session history filtering - session type becomes a new filter dimension
- **Story 1.8** (done): Historical session import - .ibt parser already extracts session type string
- **Story 4.3** (backlog): AI coaching prompt - uses session type for contextual coaching
- **Story 5.1** (backlog): Summary tab - displays session type in debrief header

### NFR Compliance

This story has minimal NFR impact:
- Session type detection is a single read operation at session start (negligible performance impact)
- Session type column is indexed for filter queries (maintains NFR3 query performance)
- Deterministic mapping: same IRSDK input always produces same session type (NFR14)

## References

- [Source: _bmad-output/planning-artifacts/epics-and-stories.md#Story 3.7: Session Type Capture]
- [Source: _bmad-output/planning-artifacts/architecture.md#Naming Conventions]
- [Source: _bmad-output/planning-artifacts/architecture.md#Requirements to Structure Mapping - FR40]
- [Source: _bmad-output/planning-artifacts/prd.md#FR40] (detect iRacing session type: practice, qualifying, race, warmup)
- [Source: _bmad-output/planning-artifacts/ux-design-specification.md#Component Strategy - Session Card] (session type badge)
- [Source: _bmad-output/planning-artifacts/ux-design-specification.md#Adaptive Patterns] (coaching adapts to session type)
- [Source: _bmad-output/implementation-artifacts/1-8-historical-session-import.md] (ibt_importer already extracts SessionType)

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List
