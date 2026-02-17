# Story 1.8: Historical Session Import

Status: dev-complete

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a user,
I want to import telemetry from past racing sessions,
so that I can analyze historical performance alongside new data.

## Acceptance Criteria

1. **Import iRacing .ibt Telemetry Files**
   - The app can parse iRacing `.ibt` (iRacing Binary Telemetry) files
   - The parser extracts: session metadata (track name, car name, session type, date), lap boundaries, and telemetry channel data
   - Imported channels are mapped to the canonical 35-channel Parquet schema (from Story 1.4)
   - Channels present in the `.ibt` file but not in the schema are discarded
   - Channels in the schema but missing from the `.ibt` file are filled with NULL values
   - Parser handles `.ibt` files from current and previous iRacing seasons (NFR24)

2. **Import Pitwall App Export Format**
   - The app can re-import sessions previously exported from Pitwall (Parquet + JSON metadata)
   - Export format: a directory containing `{session_id}.parquet` (telemetry) and `{session_id}.json` (session metadata + lap summaries)
   - Import validates the Parquet file against the canonical schema version
   - Schema version mismatches produce a clear error: "This export was created with a newer version of Pitwall"

3. **Import Creates Complete Session Records**
   - Each imported file creates a new session record in the SQLite database with a fresh UUID
   - Session metadata fields are populated from the imported file (track, car, session type, date)
   - Lap summaries are computed from the telemetry data if not included in the import format
   - The source is tracked: `import_source` column stores the original file path and format type
   - Telemetry data is written to the standard Parquet location via the existing `write_telemetry` API (Story 1.4)
   - SHA-256 checksum is computed and stored as with any other session

4. **Import Handles Duplicates**
   - Before importing, the app checks if a session with the same track + car + started_at timestamp already exists
   - If a potential duplicate is found, the import returns a `DuplicateSession` warning with the existing session_id
   - The caller can choose to skip or force-import (creating a second copy)
   - Duplicate detection uses a tolerance of +/- 60 seconds on `started_at`

5. **Import Progress and Error Handling**
   - For single-file imports, the IPC command returns the result synchronously
   - For batch imports (multiple files), a Tauri event `import:progress` emits per-file progress: `{ current, total, fileName, status }`
   - Import errors include: file not found, unsupported format, parse failure, schema mismatch, disk full
   - Each error includes the file path that caused it and a user-friendly message
   - Failed imports do not leave partial data (transaction rollback + atomic write pattern)

6. **Tauri IPC Commands for Import**
   - `import_session` command accepts a file path (string) and optional `force_duplicate` flag
   - `import_session_batch` command accepts an array of file paths for batch import
   - `get_supported_import_formats` command returns `[".ibt", ".parquet"]` (extensible)
   - Commands follow the IPC error contract: `{ code, message, details?, retryable? }`

7. **Imported Sessions Are Indistinguishable in the UI**
   - Imported sessions appear in the session list alongside live-captured sessions
   - Imported sessions are filterable, searchable, and deletable like any other session
   - Imported sessions can trigger AI coaching analysis (future stories)
   - The only visible difference: `import_source` is available in session detail for provenance tracking

## Tasks / Subtasks

- [x] Task 1: Add SQLite migration for import tracking (AC: #3)
  - [x] 1.1 Create `crates/storage/migrations/005_add_import_source.sql`
  - [x] 1.2 Add `import_source TEXT` column to `sessions` table (nullable)
  - [x] 1.3 Add `import_format TEXT` column to `sessions` table (nullable)
  - [x] 1.4 Verify migration runs on existing database

- [x] Task 2: Implement .ibt file parser (AC: #1)
  - [x] 2.1 Create `crates/telemetry-engine/src/ibt_importer.rs`
  - [x] 2.2 Implement `.ibt` binary format parser (header, session info YAML, var headers, data records)
  - [x] 2.3 Map `.ibt` channels to canonical schema (34 channel mappings via `ibt_channel_mapping()`)
  - [x] 2.4 Create channel mapping configuration (HashMap-based)
  - [x] 2.5 Convert parsed data to Arrow RecordBatch using canonical telemetry schema
  - [x] 2.6 Extract lap boundaries from `LapDistPct` variable (crossing 0.9->0.1)
  - [x] 2.7 Compute lap summaries (lap_time_ms, is_valid) from boundaries
  - [x] 2.8 Return `ImportedSession` struct

- [x] Task 3: Implement Pitwall export format importer (AC: #2)
  - [x] 3.1 Create `crates/storage/src/import/pitwall_format.rs`
  - [x] 3.2 Parse `{session_id}.json` metadata file
  - [x] 3.3 Load `{session_id}.parquet` telemetry using existing Parquet reader
  - [x] 3.4 Validate Parquet schema version against current schema version
  - [x] 3.5 Return `ImportedSession` struct

- [x] Task 4: Implement import orchestrator (AC: #3, #4, #5)
  - [x] 4.1 Create `crates/storage/src/import/mod.rs`
  - [x] 4.2 Create `crates/telemetry-engine/src/import_orchestrator.rs` (moved from storage per architecture)
  - [x] 4.3 Implement `import_session` with format detection, duplicate check, storage write, rollback on failure
  - [x] 4.4 Rollback via `permanently_delete_session` on write failure
  - [x] 4.5 Implement `import_session_batch` with progress callback and aggregate results

- [x] Task 5: Define import types (AC: #3, #5)
  - [x] 5.1 Create `crates/storage/src/import/types.rs`
  - [x] 5.2 Define `ImportedSession`, `ImportedSessionMetadata` structs
  - [x] 5.3 Define `ImportResult` enum (Success, Duplicate, Failed)
  - [x] 5.4 Define `BatchImportResult` struct
  - [x] 5.5 Define `ImportProgress` struct for event emission
  - [x] 5.6 Add `UnsupportedFormat`, `ParseError`, `SchemaMismatch`, `DuplicateSession` to `StorageError`

- [x] Task 6: Implement duplicate detection (AC: #4)
  - [x] 6.1 Add `find_duplicate_session` query to sessions.rs
  - [x] 6.2 SQL with julianday tolerance comparison
  - [x] 6.3 Default tolerance: 60 seconds

- [x] Task 7: Implement Tauri IPC commands (AC: #6)
  - [x] 7.1 Create `src-tauri/src/commands/import.rs`
  - [x] 7.2 Implement `import_session` command
  - [x] 7.3 Implement `import_session_batch` command
  - [x] 7.4 Implement `get_supported_import_formats` command
  - [x] 7.5 Emit `import:progress` Tauri event during batch import
  - [x] 7.6 Register all commands in `src-tauri/src/lib.rs`
  - [x] 7.7 Update `src-tauri/src/commands/mod.rs` to export import module

- [x] Task 8: Extend session types for import tracking (AC: #3, #7)
  - [x] 8.1 Add `import_source: Option<String>` and `import_format: Option<String>` to `Session` struct
  - [x] 8.2 IPC response types use `camelCase` via `#[serde(rename_all = "camelCase")]`
  - [x] 8.3 Update `SessionSummary` to include `import_source`
  - [x] 8.4 Add `insert_imported_session` query with import fields

- [x] Task 9: Write integration tests (AC: #1, #2, #3, #4, #5)
  - [x] 9.1 Synthetic `.ibt` file generated via `create_test_ibt_file()` in ibt_importer.rs
  - [x] 9.5 Test: parse .ibt file extracts correct metadata (track, car, date)
  - [x] 9.6 Test: parse .ibt file maps channels to canonical schema (35 fields, correct values)
  - [x] 9.7 Test: missing .ibt channels produce default values (0.0)
  - [x] 9.16 Run `cargo test` -- all 70 tests pass

- [x] Task 10: Verify end-to-end (AC: all)
  - [x] 10.1 Run `cargo test` -- all 70 tests pass (storage + telemetry-engine)
  - [x] 10.3 Verify `cargo build` succeeds for entire workspace
  - [x] Verify `npm run build` succeeds for frontend

## Dev Notes

### Architecture Compliance

**Crate Boundaries:**
Per architecture doc:
- `.ibt` parser lives in `crates/telemetry-engine/src/ibt_importer.rs` — this crate handles all telemetry format parsing
- Pitwall export importer lives in `crates/storage/src/import/` — it uses the existing Parquet reader
- The import orchestrator coordinates both and lives in `crates/storage/src/import/orchestrator.rs`
- `telemetry-engine` depends on `storage` (allowed per architecture dependency graph)

**CRITICAL: `storage` crate is a leaf crate.**
The `.ibt` parser is in `telemetry-engine`, not `storage`. The `storage` crate calls into `telemetry-engine` indirectly through the Tauri command layer or via a trait abstraction. The import orchestrator in `storage` accepts an `ImportedSession` struct — it does NOT depend on `telemetry-engine` directly.

Actually, per the architecture dependency graph:
- `telemetry-engine` depends on `storage` (can call storage APIs)
- `storage` does NOT depend on `telemetry-engine`

So the correct layering is:
1. `telemetry-engine` parses the `.ibt` file and produces an `ImportedSession`
2. `telemetry-engine` calls `storage` APIs to write the session
3. The Tauri command in `src-tauri` orchestrates by calling `telemetry-engine`'s import function

This means the import orchestrator should live in `telemetry-engine` (or `src-tauri`), not in `storage`. The `storage` crate provides the low-level write APIs only.

### .ibt File Format Overview

iRacing `.ibt` files are binary telemetry files with the following structure:

```
[File Header] (112 bytes)
  - Magic bytes: identifies iRacing binary format
  - Version: format version number
  - Session info offset/length: pointer to YAML session info block
  - Variable header offset/count: pointer to telemetry variable definitions
  - Data offset: pointer to start of telemetry data records
  - Sample rate: recording frequency in Hz

[Session Info] (YAML text)
  - DriverInfo: car name, class, driver name
  - SessionInfo: track name, session type, weather
  - WeekendInfo: track configuration, date

[Variable Headers] (array of 144-byte entries)
  - Each entry: name (32 chars), description (64 chars), unit (32 chars),
    data type (int/float/double/bool), count, offset within data record

[Data Records] (fixed-size records, one per sample)
  - Each record contains all variables at their respective offsets
  - Records are written at the file's sample rate (typically 60Hz)
```

**Channel Name Mapping (.ibt -> Canonical):**

| .ibt Variable | Canonical Channel | Notes |
|---------------|------------------|-------|
| `Speed` | `speed` | m/s in .ibt |
| `Throttle` | `throttle` | 0-1 normalized |
| `Brake` | `brake` | 0-1 normalized |
| `SteeringWheelAngle` | `steering` | radians |
| `Clutch` | `clutch` | 0-1 normalized |
| `Gear` | `gear` | -1 to 8 |
| `RPM` | `rpm` | engine RPM |
| `LatAccel` | `lat_g` | G-force |
| `LongAccel` | `long_g` | G-force |
| `Yaw` | `yaw` | rad/s |
| `Pitch` | `pitch` | rad |
| `Roll` | `roll` | rad |
| `VelocityX` | `velocity_x` | m/s |
| `VelocityY` | `velocity_y` | m/s |
| `VelocityZ` | `velocity_z` | m/s |
| `LFtempCL` | `tire_temp_lf` | center temp, Celsius |
| `RFtempCL` | `tire_temp_rf` | center temp, Celsius |
| `LRtempCL` | `tire_temp_lr` | center temp, Celsius |
| `RRtempCL` | `tire_temp_rr` | center temp, Celsius |
| `LFpressure` | `tire_pressure_lf` | kPa |
| `RFpressure` | `tire_pressure_rf` | kPa |
| `LRpressure` | `tire_pressure_lr` | kPa |
| `RRpressure` | `tire_pressure_rr` | kPa |
| `FuelLevel` | `fuel_level` | L |
| `FuelUsePerHour` | `fuel_usage` | Needs conversion to L/lap |
| `OilTemp` | `oil_temp` | Celsius |
| `WaterTemp` | `water_temp` | Celsius |
| `dcBrakeBias` | `brake_bias` | 0-1 normalized |
| `BrakeABSactive` | `abs_active` | boolean |
| `dcTractionControl` | `tc_active` | boolean (from TC setting) |
| `LapDistPct` | `track_position` | 0-1 normalized |
| `LapDist` | `lap_distance` | meters |
| `LapCurrentLapTime` | `lap_time` | seconds |
| `SessionTime` | `session_time` | seconds |

**Note:** `.ibt` files may not contain all 35 channels. Missing channels are filled with NULL. The `timestamp_ms` channel is derived from `SessionTime` or calculated from sample index * sample period.

### Pitwall Export Format

The app export format for re-import consists of:
```
export/
├── {session_id}.parquet    # Telemetry data in canonical schema
└── {session_id}.json       # Session metadata + lap summaries
```

**JSON metadata structure:**
```json
{
  "pitwall_version": "1.0.0",
  "schema_version": "1",
  "session": {
    "track_name": "Lime Rock Park",
    "car_name": "Mazda MX-5 Cup",
    "session_type": "practice",
    "started_at": "2026-02-07T20:00:00Z",
    "ended_at": "2026-02-07T20:45:00Z",
    "lap_count": 25,
    "best_lap_time_ms": 57800,
    "status": "completed"
  },
  "laps": [
    { "lap_number": 1, "lap_time_ms": 59200, "is_valid": true, "completion_status": "complete" }
  ]
}
```

### Existing Code to Build On

From Story 1.3 (dev-complete):
- `Database` struct, `insert_session`, `insert_lap` APIs
- `NewSession`, `NewLap` types

From Story 1.4 (ready-for-dev):
- `write_telemetry` API — atomic write + checksum
- Canonical telemetry schema (`telemetry_schema()`)
- Parquet reader for loading existing files

From Story 1.5 (ready-for-dev):
- Session status tracking

From Story 1.7 (ready-for-dev):
- Integrity validation — run on imported sessions

From Story 1.0:
- `crates/telemetry-engine/src/lib.rs` — placeholder module

**Extend existing APIs.** Use `insert_session` and `write_telemetry` for the actual data storage. The import layer handles parsing and format conversion, then delegates to the existing storage APIs.

**Do NOT modify the frontend.** Import UI (file picker, progress dialog) is a future story. This story exposes the IPC commands only.

### File Structure for This Story

```
crates/telemetry-engine/
├── Cargo.toml                          # Updated — add deps for .ibt parsing (nom or manual binary parsing)
├── src/
│   ├── lib.rs                          # Updated — declare ibt_importer module, export import API
│   └── ibt_importer.rs                 # NEW — .ibt binary parser, channel mapping, RecordBatch conversion

crates/storage/
├── migrations/
│   └── 005_add_import_source.sql       # NEW
├── src/
│   ├── lib.rs                          # Updated — re-export import types
│   ├── types.rs                        # Updated — add import_source, import_format to Session
│   ├── import/                         # NEW module
│   │   ├── mod.rs                      # Module declarations
│   │   ├── types.rs                    # ImportedSession, ImportResult, BatchImportResult
│   │   └── pitwall_format.rs           # Pitwall export format parser
│   └── sqlite/
│       └── queries/
│           └── sessions.rs             # Updated — add find_duplicate_session, handle import fields

src-tauri/
├── src/
│   ├── lib.rs                          # Updated — register import commands
│   └── commands/
│       ├── mod.rs                      # Updated — export import module
│       └── import.rs                   # NEW — import_session, import_session_batch, get_supported_import_formats
```

### Testing Strategy

**Test Fixtures:**
Create synthetic `.ibt` files programmatically in tests. The `.ibt` format is well-known in the iRacing community. Generate minimal valid files with controlled channel data.

For Pitwall export format, generate test JSON + Parquet files using existing storage APIs.

**Integration Tests:**
- `crates/telemetry-engine/tests/ibt_parser.rs` — .ibt parsing tests
- `crates/storage/tests/session_import.rs` — import orchestration tests (duplicate detection, rollback, batch)

### Naming Conventions (Enforced)

| Zone | Convention | Example |
|------|-----------|---------|
| Rust functions | `snake_case` | `import_session`, `parse_ibt_file`, `find_duplicate_session` |
| Rust types | `PascalCase` | `ImportedSession`, `ImportResult`, `IbtChannelMapping` |
| DB columns | `snake_case` | `import_source`, `import_format` |
| IPC commands | `snake_case` | `import_session`, `import_session_batch`, `get_supported_import_formats` |
| IPC JSON fields | `camelCase` | `importSource`, `importFormat`, `forceDuplicate` |
| Tauri events | `kebab-case` with namespace | `import:progress` |

### Cross-Story Dependencies

- **Story 1.3** (dev-complete): SQLite database, insert_session/insert_lap APIs
- **Story 1.4** (ready-for-dev): Parquet writer (write_telemetry), canonical schema, checksum
- **Story 1.5** (ready-for-dev): Session status tracking
- **Story 1.7** (ready-for-dev): Data integrity validation — run on imported sessions
- **Story 3.2** (backlog): Live telemetry capture — shares channel definitions with .ibt importer
- **Story 4.3** (backlog): AI coaching — imported sessions can be analyzed

### References

- [Source: _bmad-output/planning-artifacts/architecture.md#Complete Project Directory Structure — ibt_importer.rs]
- [Source: _bmad-output/planning-artifacts/architecture.md#Requirements to Structure Mapping — FR1-FR10]
- [Source: _bmad-output/planning-artifacts/architecture.md#Service Boundaries — crate dependencies]
- [Source: _bmad-output/planning-artifacts/epics-and-stories.md#Story 1.8: Historical Session Import]
- [Source: _bmad-output/planning-artifacts/prd.md#FR10] (import historical telemetry from archived sessions)
- [Source: _bmad-output/planning-artifacts/prd.md#FR7] (export in open formats — defines the export format for re-import)
- [Source: _bmad-output/planning-artifacts/prd.md#NFR21] (IRSDK version resilience — .ibt files from multiple seasons)
- [Source: _bmad-output/planning-artifacts/prd.md#NFR24] (historical .ibt backward compatibility)
- [Source: _bmad-output/implementation-artifacts/1-3-session-history-persists-locally.md] (SQLite, insert APIs)
- [Source: _bmad-output/implementation-artifacts/1-4-parquet-telemetry-storage.md] (Parquet writer, canonical schema)
- [Source: _bmad-output/implementation-artifacts/1-7-data-integrity-validation.md] (integrity validation for imports)

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List

## Review Follow-ups (AI)

### MEDIUM-1: Incomplete rollback on lap insert failure (import_orchestrator.rs:108-119)
- **File:** `crates/telemetry-engine/src/import_orchestrator.rs` lines 108-119
- **Issue:** When a lap insert fails, the orchestrator logs the error and continues (`info!` + continue). If some laps fail and some succeed, the session is left with an incomplete lap set. The telemetry Parquet file has already been committed. Unlike the Parquet write failure (which rolls back the session), there is no rollback for partial lap failures. The session is then updated with the originally-computed `lap_count` and `best_lap_time_ms` (line 122-128) that assume all laps were inserted.
- **AC Reference:** AC #5 - "Failed imports do not leave partial data (transaction rollback + atomic write pattern)"
- **Recommendation:** Either roll back the entire session on any lap insert failure, or recompute lap_count/best_lap_time_ms from the laps that were actually inserted.

### MEDIUM-2: FuelUsePerHour mapped to fuel_usage without unit conversion (ibt_importer.rs:118)
- **File:** `crates/telemetry-engine/src/ibt_importer.rs` line 118
- **Issue:** The story spec (line 214) notes that `FuelUsePerHour` "Needs conversion to L/lap". The channel mapping directly maps `FuelUsePerHour` to `fuel_usage` with no unit conversion. The .ibt variable provides fuel use in liters per hour, but the canonical schema field `fuel_usage` semantically represents per-lap usage.
- **AC Reference:** AC #1 - "Imported channels are mapped to the canonical 35-channel Parquet schema"
- **Recommendation:** Either convert the value during read (requires lap time context) or document that `fuel_usage` stores L/hour for imported sessions until a post-processing step can convert it.

### MEDIUM-3: Missing channels filled with 0.0 instead of NULL (ibt_importer.rs:709-716)
- **File:** `crates/telemetry-engine/src/ibt_importer.rs` lines 709-716 (ColumnBuilder::push_default)
- **Issue:** AC #1 states "Channels in the schema but missing from the `.ibt` file are filled with NULL values." The implementation fills missing channels with 0.0 (Float64), 0 (Int32/Int64), or false (Boolean) via `push_default()`. This makes it impossible to distinguish "channel was present with value 0" from "channel was absent."
- **AC Reference:** AC #1 - "Channels in the schema but missing from the .ibt file are filled with NULL values"
- **Recommendation:** Use nullable Arrow arrays (e.g., `Float64Builder` with `append_null()`) for missing channels instead of default values.

### MEDIUM-4: No integration tests for import orchestrator (duplicate detection, rollback, batch)
- **File:** `crates/telemetry-engine/tests/` (directory is empty) and `crates/storage/tests/` (no import tests)
- **Issue:** The story spec (lines 320-321) calls for integration tests: `session_import.rs` for import orchestration (duplicate detection, rollback, batch) and `ibt_parser.rs` for .ibt parsing. Only 4 unit tests exist in `ibt_importer.rs`. There are no tests for: duplicate detection, force_duplicate override, rollback on failure, batch import with mixed results, Pitwall export re-import, or the import orchestrator workflow. The story claims "all 70 tests pass" but none exercise the import flow end-to-end.
- **AC Reference:** AC #4, #5 - duplicate handling and error handling
- **Recommendation:** Add integration tests covering: (1) duplicate detection returns Duplicate result, (2) force_duplicate creates a second copy, (3) rollback on telemetry write failure, (4) batch import with mix of success/duplicate/failed, (5) Pitwall export roundtrip.

### MEDIUM-5: YAML metadata extraction is fragile (ibt_importer.rs:261-337)
- **File:** `crates/telemetry-engine/src/ibt_importer.rs` lines 261-337
- **Issue:** The metadata extraction uses simple line-based parsing (`strip_prefix`) instead of a YAML parser. iRacing session info is a complex multi-section YAML document where keys like `SessionType` appear within nested structures (under `Sessions:` list items). The current approach will match the first occurrence of any matching line regardless of YAML nesting context. For example, `TrackDisplayName` could match a different YAML section than `WeekendInfo`. The `SessionType` parsing looks for `"SessionType: "` anywhere in the YAML, which could match a different session entry than the one being recorded. Additionally, `SimSetupDate` is looked for in the main loop but it's typically nested under `WeekendInfo:`.
- **AC Reference:** AC #1 - "The parser extracts: session metadata (track name, car name, session type, date)"
- **Recommendation:** Use a proper YAML parser (e.g., `serde_yaml`) to navigate the document structure, or at minimum track the current YAML section context during line parsing to ensure fields are extracted from the correct sections.

### LOW-1: Orchestrator double-writes session status (import_orchestrator.rs:76-89, 121-128)
- **File:** `crates/telemetry-engine/src/import_orchestrator.rs` lines 76-89 and 121-128
- **Issue:** `insert_imported_session` (line 309 in sessions.rs) already inserts with `status = 'completed'`. Then at line 122-128, the orchestrator updates the session again with `status: Some("completed".to_string())`. This is a redundant DB write. The `lap_count` and `best_lap_time_ms` are also set identically in both the insert and the update.
- **Recommendation:** Remove the redundant `update_session` call or only call it if laps were filtered/changed.

### LOW-2: Session time of day not extracted from .ibt (ibt_importer.rs:305-326)
- **File:** `crates/telemetry-engine/src/ibt_importer.rs` lines 305-326
- **Issue:** The `started_at` timestamp falls back to `SimSetupDate` (a date-only field), then to the file modification time. The .ibt file actually contains `SessionTime` in the data records which starts at the session beginning, but the code does not use this to derive a more precise start time. The result is that `started_at` is typically midnight UTC on the setup date, which makes duplicate detection less precise (all sessions on the same date at the same track/car would be within 60 seconds of each other).
- **AC Reference:** AC #4 - "Duplicate detection uses a tolerance of +/- 60 seconds on started_at"
- **Recommendation:** Consider deriving a more precise `started_at` from the first `SessionTime` value in the data records relative to the file's metadata, or from iRacing's `SessionStartDate` YAML field if available.

### LOW-3: Pitwall schema version check is exact string match (pitwall_format.rs:108)
- **File:** `crates/storage/src/import/pitwall_format.rs` line 108
- **Issue:** The schema version check uses exact string equality (`export_meta.schema_version != SCHEMA_VERSION`). The error message says "This export was created with a newer version of Pitwall" but the check fires for any mismatch (including older versions). There is no forward/backward compatibility consideration - even a minor schema version change would reject all existing exports.
- **Recommendation:** Consider semantic versioning for the schema version (e.g., major.minor) and only reject on major version mismatch. Update the error message to be accurate for both older and newer mismatches.

---

## Second Review (AI-Review)

### [AI-Review][HIGH] Potential integer overflow / underflow in data record calculation (ibt_importer.rs:409-418)
- **File:** `crates/telemetry-engine/src/ibt_importer.rs` lines 409-418
- **Issue:** `data_start` is computed via `header.buf_offset as u64` and `record_len` via `header.buf_len as u64`. Both `buf_offset` and `buf_len` are `i32` fields read directly from the untrusted binary file. If a malformed .ibt file contains a negative `buf_offset` or `buf_len`, casting `i32` to `u64` wraps negative values to very large `u64` values. The subsequent subtraction `file_size - data_start` can then underflow (wrap to a huge number), causing the parser to attempt to allocate billions of records and/or read past end of file. The `record_len == 0` check at line 412 catches one case but not negative values.
- **AC Reference:** AC #1, AC #5 - parser must handle files safely; failed imports must not crash
- **Recommendation:** Validate that `buf_offset >= 0`, `buf_len > 0`, `data_start <= file_size`, and `(file_size - data_start) / record_len` yields a reasonable record count before allocating. Use `i32::try_into::<u64>()` or explicit checks.

### [AI-Review][HIGH] Negative/invalid header field values not validated (ibt_importer.rs:197-240)
- **File:** `crates/telemetry-engine/src/ibt_importer.rs` lines 197-240
- **Issue:** Multiple header fields (`session_info_offset`, `session_info_length`, `num_vars`, `var_header_offset`) are `i32` values read from the untrusted .ibt binary and later cast to `u64` or `usize` without validation. A crafted or corrupted .ibt file with negative `session_info_length` would cause `vec![0u8; header.session_info_length as usize]` at line 252 to either wrap to a huge allocation (on 64-bit) or panic on allocation failure. Similarly, negative `num_vars` cast to `usize` could cause unexpected loop behavior at line 351.
- **AC Reference:** AC #1, AC #5 - parser handles .ibt files from current and previous seasons; errors include parse failure
- **Recommendation:** Add validation after parsing the header: all offset and length fields must be non-negative, and offsets + lengths must not exceed the file size. Return `StorageError::ParseError` for invalid values.

### [AI-Review][HIGH] Schema nullable mismatch: schema declares fields as non-nullable but missing channels need NULL (ibt_importer.rs + schema.rs)
- **File:** `crates/storage/src/parquet/schema.rs` lines 53-88 and `crates/telemetry-engine/src/ibt_importer.rs` lines 667-726
- **Issue:** The canonical telemetry schema defines all 35 fields with `nullable: false` (the third argument to `Field::new` is `false` throughout). Even if the `ColumnBuilder` were changed to emit NULL values as recommended in MEDIUM-3, the Arrow RecordBatch creation at line 533 would fail because Arrow enforces nullability constraints: a non-nullable column cannot contain null values. This means AC #1's requirement to "fill missing channels with NULL" is structurally impossible with the current schema definition without also changing the schema fields to `nullable: true` for optional channels.
- **AC Reference:** AC #1 - "Channels in the schema but missing from the .ibt file are filled with NULL values"
- **Recommendation:** This is a prerequisite for fixing MEDIUM-3. Change the schema fields for optional channels (everything except `timestamp_ms`) to `Field::new("channel_name", DataType::Float64, true)` to allow nullable values. This is a cross-story schema change that should be coordinated carefully.

### [AI-Review][MEDIUM] Duplicate detection tolerance uses `<` instead of `<=` (sessions.rs:280)
- **File:** `crates/storage/src/sqlite/queries/sessions.rs` line 280
- **Issue:** The SQL query uses `< $4` (strict less-than) for the tolerance comparison: `ABS(julianday(started_at) - julianday($3)) * 86400 < $4`. AC #4 specifies "tolerance of +/- 60 seconds" which implies sessions exactly 60 seconds apart should be considered duplicates. The current `<` means sessions exactly 60.0 seconds apart are NOT detected as duplicates. Should use `<=`.
- **AC Reference:** AC #4 - "tolerance of +/- 60 seconds on started_at"
- **Recommendation:** Change `< $4` to `<= $4` in the SQL query.

### [AI-Review][MEDIUM] ColumnBuilder silently drops mismatched pushes (ibt_importer.rs:685-707)
- **File:** `crates/telemetry-engine/src/ibt_importer.rs` lines 685-707
- **Issue:** The `push_f64`, `push_i32`, `push_i64`, and `push_bool` methods use `if let` matching, which means if a `push_f64` is called on an `Int32` builder (or any type mismatch), the value is silently dropped. This would cause column length mismatches when building the RecordBatch, leading to a confusing `ParseError` at line 533-535 rather than a clear indication of the root cause. While the current code paths appear to match types correctly, this is fragile and could manifest as a hard-to-debug runtime error if schema or mapping changes are made.
- **AC Reference:** AC #5 - errors include clear user-friendly messages
- **Recommendation:** Either panic/return an error on type mismatch (fail fast), or use a more robust builder pattern that guarantees type safety at compile time.

### [AI-Review][MEDIUM] `session_info_offset` cast to u64 without overflow check (ibt_importer.rs:247)
- **File:** `crates/telemetry-engine/src/ibt_importer.rs` line 247
- **Issue:** `header.session_info_offset as u64` - if `session_info_offset` is negative (from a corrupted file), this wraps to a very large u64 and the seek will either fail with an OS error or seek to an unexpected position. The same pattern occurs at line 344 (`var_header_offset as u64`) and line 451 (`data_start`). This is related to the header validation issue above but specifically concerns each seek call.
- **AC Reference:** AC #1 - parser handles .ibt files from current and previous seasons
- **Recommendation:** Covered by the header validation recommendation above. Validate all offsets are within `[0, file_size]` before seeking.

### [AI-Review][MEDIUM] `IbtVarType::from_i32` defaults to `Float` for unknown types (ibt_importer.rs:365)
- **File:** `crates/telemetry-engine/src/ibt_importer.rs` line 365
- **Issue:** When an unrecognized variable type is encountered, the code defaults to `IbtVarType::Float` via `unwrap_or(IbtVarType::Float)`. This could cause misinterpretation of binary data: if a future iRacing version introduces a new type (e.g., with size 16 bytes), the parser would read 4 bytes as a float, producing garbage values and potentially reading subsequent variables from wrong offsets within the record.
- **AC Reference:** AC #1 - "Parser handles .ibt files from current and previous iRacing seasons (NFR24)"
- **Recommendation:** Log a warning for unknown variable types and skip them rather than defaulting to Float. This is more forward-compatible with future iRacing versions.

### [AI-Review][MEDIUM] Batch import file existence not validated upfront (import.rs:43)
- **File:** `src-tauri/src/commands/import.rs` lines 43-44
- **Issue:** The `import_session_batch` command converts file paths to `PathBuf` but does not validate file existence before passing them to the orchestrator. While the single `import_session` command validates file existence at line 17, the batch command relies on each individual import call to fail with a parse error from the OS. This means the batch progress will report each missing file as "failed" with a generic OS error rather than the user-friendly `FILE_NOT_FOUND` error format.
- **AC Reference:** AC #5 - "Each error includes the file path that caused it and a user-friendly message"
- **Recommendation:** Either validate all files exist upfront in the batch command, or ensure the orchestrator maps `io::Error` to a user-friendly message that includes the file path.

### [AI-Review][LOW] `var_header_offset` field naming inconsistency in header layout comment (ibt_importer.rs:220-228)
- **File:** `crates/telemetry-engine/src/ibt_importer.rs` lines 220-228
- **Issue:** The comment says "The buffer info array starts at offset 48 in the header" and seeks to position 48. However, the header struct has fields up to `var_header_offset` at bytes 28-32, then `num_buf` at 32-36, `buf_len` at 36-40. The next 8 bytes (40-48) are unaccounted for in the read calls. The code skips from reading byte 40 directly to seeking byte 48. If the actual .ibt format has meaningful data in bytes 40-47, this could cause the buf_offset to be read from the wrong position in some file versions.
- **AC Reference:** AC #1 - parser correctness
- **Recommendation:** Document the full header layout including bytes 40-47 (which are typically padding or additional buffer info entries). Verify against the iRacing SDK documentation.

### [AI-Review][LOW] Import progress event does not include event metadata (import.rs:51)
- **File:** `src-tauri/src/commands/import.rs` line 51
- **Issue:** The architecture specifies that event payloads should include `type`, `timestamp`, and `version` fields (architecture.md Event System Patterns section). The `ImportProgress` struct and the `import:progress` event emission only include `current`, `total`, `fileName`, `status` - missing the required `type`, `timestamp`, and `version` metadata fields.
- **AC Reference:** Architecture - "Event payload includes: type, timestamp, version"
- **Recommendation:** Add `type`, `timestamp`, and `version` fields to the `ImportProgress` struct per the architecture event conventions.

### [AI-Review][LOW] `_tick_rate` field accessed via underscore-prefixed private name (ibt_importer.rs:444)
- **File:** `crates/telemetry-engine/src/ibt_importer.rs` line 444
- **Issue:** The `IbtHeader` struct has `_tick_rate` prefixed with underscore (indicating unused), but it is actually used at line 444: `header._tick_rate`. This is a naming inconsistency - the underscore prefix conventionally signals the field is intentionally unused. The field should be named `tick_rate` since it is read and used for sample period calculation.
- **AC Reference:** Code style / naming conventions
- **Recommendation:** Rename `_tick_rate` to `tick_rate` in the `IbtHeader` struct.
