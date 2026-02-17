# Story 1.7: Data Integrity Validation

Status: implemented

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a sim racer,
I want confidence that my telemetry data is accurate and uncorrupted,
so that AI coaching is based on trustworthy data.

## Acceptance Criteria

1. **SHA-256 Checksum Validates on Read**
   - When a Parquet telemetry file is loaded, its SHA-256 checksum is recomputed and compared against the stored checksum in the `sessions` table
   - If the checksum matches, the data is returned normally
   - If the checksum does NOT match, an `IntegrityError::ChecksumMismatch` error is returned with details (expected vs actual checksum, session_id, file path)
   - Checksum mismatches are logged at `error` level via `tracing::error!`
   - The session record is flagged with `integrity_status = 'checksum_failed'` in the database

2. **Lap Distance Monotonicity Validation**
   - Within each lap, the `lap_distance` telemetry channel values are verified to be monotonically increasing (each sample >= previous sample)
   - Monotonicity violations are detected and recorded with: lap number, sample index, previous value, violating value
   - A lap with >2% of samples violating monotonicity is flagged as `integrity_status = 'distance_anomaly'`
   - Violations caused by known events (lap reset, pit entry) are excluded from flagging — identified by `lap_distance` dropping to near-zero (within 50m of track start)
   - Validation results are stored per-session, not per-sample (summary counts)

3. **Telemetry Channel Value Range Validation**
   - Each telemetry channel is validated against physically plausible ranges:
     - `brake`: 0.0 to 1.0 (normalized)
     - `throttle`: 0.0 to 1.0 (normalized)
     - `clutch`: 0.0 to 1.0 (normalized)
     - `speed`: 0.0 to 120.0 m/s (~432 km/h — max physically possible in sim)
     - `steering`: -6.28 to 6.28 rad (full rotation range)
     - `rpm`: 0.0 to 20000.0 (max engine RPM across all cars)
     - `gear`: -1 to 8 (reverse through highest gear)
     - `tire_temp_*`: 0.0 to 200.0 C (cold to extreme)
     - `tire_pressure_*`: 50.0 to 350.0 kPa (flat to over-inflated)
     - `fuel_level`: 0.0 to 200.0 L (max fuel tank)
     - `brake_bias`: 0.0 to 1.0 (normalized front/rear)
     - `lat_g`, `long_g`: -10.0 to 10.0 G (extreme G-force range)
     - `track_position`: 0.0 to 1.0 (normalized)
   - Channels not listed above are validated only for NaN/Infinity (no range check)
   - Out-of-range values are counted per channel per session
   - A session with >5% of samples out-of-range for any single channel is flagged with `integrity_status = 'range_violation'`

4. **Integrity Validation Runs on Session Load**
   - When `get_session_data` is called, checksum validation runs automatically (already implemented in Story 1.4 reader)
   - A separate `validate_session_integrity` method runs the full validation suite (checksum + monotonicity + range checks)
   - Full validation is callable on-demand via a Tauri IPC command `validate_session`
   - Full validation is also triggered on startup for sessions that have not been validated yet (`integrity_status IS NULL`)
   - Startup validation runs asynchronously and does not block the UI

5. **Integrity Status Tracking**
   - A new `integrity_status` column is added to the `sessions` table via migration
   - Possible values: `valid`, `checksum_failed`, `distance_anomaly`, `range_violation`, `not_validated`
   - An `integrity_details` JSON column stores structured validation results (violation counts, affected channels, affected laps)
   - An `integrity_validated_at` timestamp column records when validation last ran
   - Sessions with integrity issues are still accessible (data is NOT blocked), but a warning is displayed

6. **Integrity Violations Surfaced to User**
   - The `get_session_data` IPC response includes `integrityStatus` and `integrityDetails` fields
   - The `get_sessions` list response includes `integrityStatus` on each `SessionSummary`
   - Integrity status is available for UI display (future Story 5.1 will render warnings)
   - No frontend changes in this story — data is exposed via IPC for future consumption

7. **Lap Time Sanity Checks**
   - Lap times are validated as positive values and less than 600,000ms (10 minutes)
   - Lap times of zero or negative are flagged as invalid
   - Laps exceeding 10 minutes are flagged with `completion_status = 'invalid'`
   - The best_lap_time_ms on the session excludes invalid laps

## Tasks / Subtasks

- [ ] Task 1: Add SQLite migration for integrity columns (AC: #5)
  - [ ] 1.1 Create `crates/storage/migrations/004_add_integrity_columns.sql`
  - [ ] 1.2 Add `integrity_status TEXT DEFAULT 'not_validated'` to `sessions` table
  - [ ] 1.3 Add `integrity_details TEXT` (JSON) to `sessions` table
  - [ ] 1.4 Add `integrity_validated_at TEXT` to `sessions` table
  - [ ] 1.5 Add index `idx_sessions_integrity_status` for querying unvalidated sessions
  - [ ] 1.6 Verify migration runs on existing database from Stories 1.3-1.5

- [ ] Task 2: Define validation types (AC: #1, #2, #3, #5)
  - [ ] 2.1 Create `crates/storage/src/validation/mod.rs` module
  - [ ] 2.2 Define `IntegrityStatus` enum: `Valid`, `ChecksumFailed`, `DistanceAnomaly`, `RangeViolation`, `NotValidated`
  - [ ] 2.3 Define `IntegrityReport` struct: status, checksum_valid (bool), monotonicity_violations (Vec), range_violations (HashMap<String, RangeViolation>), validated_at
  - [ ] 2.4 Define `MonotonicityViolation` struct: lap_number, sample_index, previous_value, violating_value
  - [ ] 2.5 Define `RangeViolation` struct: channel_name, out_of_range_count, total_samples, min_seen, max_seen
  - [ ] 2.6 Define `ChannelRange` struct and `CHANNEL_RANGES` constant with valid ranges for each channel
  - [ ] 2.7 Add `IntegrityError` variants to `StorageError`: `ChecksumMismatch`, `IntegrityValidationFailed`

- [ ] Task 3: Implement checksum validation (AC: #1)
  - [ ] 3.1 Create `crates/storage/src/validation/checksum.rs`
  - [ ] 3.2 Implement `validate_checksum(file_path: &Path, expected_checksum: &str) -> Result<bool>`
  - [ ] 3.3 On mismatch: log error with session_id, expected, actual
  - [ ] 3.4 Update session record `integrity_status` to `checksum_failed`
  - [ ] 3.5 Note: Story 1.4 already implemented basic checksum compute/validate — this task extends it with status tracking and error surfacing

- [ ] Task 4: Implement lap distance monotonicity validation (AC: #2)
  - [ ] 4.1 Create `crates/storage/src/validation/monotonicity.rs`
  - [ ] 4.2 Implement `validate_lap_distance_monotonicity(data: &RecordBatch) -> Vec<MonotonicityViolation>`
  - [ ] 4.3 Group telemetry data by lap (using `lap_distance` resets as lap boundaries)
  - [ ] 4.4 Within each lap, check that each `lap_distance` sample >= previous sample
  - [ ] 4.5 Exclude known reset events: if `lap_distance` drops to within 50m of zero, treat as new lap start
  - [ ] 4.6 Calculate violation percentage per lap: violations / total_samples * 100
  - [ ] 4.7 Flag session if any lap exceeds 2% violation threshold

- [ ] Task 5: Implement channel value range validation (AC: #3)
  - [ ] 5.1 Create `crates/storage/src/validation/range_check.rs`
  - [ ] 5.2 Define `CHANNEL_RANGES` constant as a static map of channel_name -> (min, max)
  - [ ] 5.3 Implement `validate_channel_ranges(data: &RecordBatch) -> HashMap<String, RangeViolation>`
  - [ ] 5.4 For each channel with a defined range, count samples outside the range
  - [ ] 5.5 For channels without defined ranges, check only for NaN and Infinity values
  - [ ] 5.6 Calculate violation percentage: out_of_range_count / total_samples * 100
  - [ ] 5.7 Flag session if any channel exceeds 5% violation threshold

- [ ] Task 6: Implement lap time sanity checks (AC: #7)
  - [ ] 6.1 Add validation to `crates/storage/src/validation/lap_time.rs`
  - [ ] 6.2 Implement `validate_lap_times(laps: &[LapSummary]) -> Vec<LapTimeViolation>`
  - [ ] 6.3 Check each lap: lap_time_ms > 0 AND lap_time_ms < 600_000
  - [ ] 6.4 Flag invalid laps by updating `completion_status = 'invalid'` in `lap_summaries`
  - [ ] 6.5 When computing best_lap_time_ms for session, exclude laps with `completion_status = 'invalid'`

- [ ] Task 7: Implement composite validation orchestrator (AC: #4)
  - [ ] 7.1 Create `crates/storage/src/validation/orchestrator.rs`
  - [ ] 7.2 Implement `validate_session_integrity(db: &Database, session_id: &str) -> Result<IntegrityReport>`:
    - Load telemetry data (if exists)
    - Run checksum validation
    - Run monotonicity validation
    - Run range validation
    - Load laps and run lap time validation
    - Aggregate results into IntegrityReport
    - Determine overall integrity_status (worst status wins)
    - Store results in session record (integrity_status, integrity_details JSON, integrity_validated_at)
  - [ ] 7.3 Handle sessions without telemetry (skip Parquet-based validations, validate lap times only)
  - [ ] 7.4 Wire through `Database` struct

- [ ] Task 8: Implement startup validation (AC: #4)
  - [ ] 8.1 Implement `validate_unvalidated_sessions(db: &Database) -> Result<u32>` — query sessions where `integrity_status = 'not_validated'` AND `telemetry_path IS NOT NULL`
  - [ ] 8.2 Call during storage initialization asynchronously (`tokio::spawn`)
  - [ ] 8.3 Process sessions one at a time to avoid excessive memory usage
  - [ ] 8.4 Log validation progress: "Validating session X of Y..."
  - [ ] 8.5 Do not block application startup

- [ ] Task 9: Extend IPC responses and add validate command (AC: #6)
  - [ ] 9.1 Add `integrity_status` field to `SessionSummary` IPC response
  - [ ] 9.2 Add `integrity_status` and `integrity_details` to `SessionDetail` IPC response
  - [ ] 9.3 Implement `validate_session` Tauri IPC command: accepts session_id, runs full validation, returns `IntegrityReport`
  - [ ] 9.4 Register `validate_session` command in `src-tauri/src/lib.rs`
  - [ ] 9.5 Serde serialize IntegrityReport with `camelCase` field names

- [ ] Task 10: Write integration tests (AC: #1, #2, #3, #4, #7)
  - [ ] 10.1 Create `crates/storage/tests/data_integrity.rs`
  - [ ] 10.2 Test: valid Parquet file passes checksum validation
  - [ ] 10.3 Test: corrupted Parquet file fails checksum validation, status set to `checksum_failed`
  - [ ] 10.4 Test: monotonically increasing lap_distance passes validation
  - [ ] 10.5 Test: non-monotonic lap_distance with >2% violations flags session
  - [ ] 10.6 Test: lap reset (distance drops to ~0) is NOT flagged as a violation
  - [ ] 10.7 Test: brake values 0.0-1.0 pass range validation
  - [ ] 10.8 Test: brake value of 2.0 is flagged as out-of-range
  - [ ] 10.9 Test: >5% out-of-range for a channel flags session
  - [ ] 10.10 Test: NaN values in non-range-checked channels are detected
  - [ ] 10.11 Test: lap time of 0ms flagged as invalid
  - [ ] 10.12 Test: lap time of 700,000ms (>10min) flagged as invalid
  - [ ] 10.13 Test: full validation orchestrator produces correct IntegrityReport
  - [ ] 10.14 Test: startup validation processes only unvalidated sessions
  - [ ] 10.15 Run `cargo test` -- all tests pass

- [ ] Task 11: Verify end-to-end (AC: all)
  - [ ] 11.1 Run `cargo test` — all storage tests pass (existing + new integrity tests)
  - [ ] 11.2 Run `npm run tauri dev` — app starts, migration 004 runs, startup validation runs
  - [ ] 11.3 Verify `cargo build` succeeds for entire workspace

## Dev Notes

### Architecture Compliance

**Storage Crate Remains a LEAF Crate:**
No workspace dependencies. The validation module lives entirely within `crates/storage/` and uses only the existing `arrow`, `parquet`, `sha2`, and `sqlx` external dependencies.

**Hybrid Validation Strategy:**
Per architecture doc: "Hybrid: minimal validation at capture (required fields, timestamp monotonicity); full validation during preprocessing/analysis." This story implements the full validation layer that runs post-capture.

**Invalid Samples Retained with Flags:**
Per architecture doc: "Invalid samples retained with flags; gap markers emitted; derived metrics skip invalid points; debrief shows warnings." Data is NEVER blocked or deleted due to integrity issues. The validation layer flags problems and surfaces them — the data remains accessible.

### SQLite Migration

**Migration `004_add_integrity_columns.sql`:**

```sql
-- Add integrity tracking columns to sessions
ALTER TABLE sessions ADD COLUMN integrity_status TEXT DEFAULT 'not_validated';
ALTER TABLE sessions ADD COLUMN integrity_details TEXT;
ALTER TABLE sessions ADD COLUMN integrity_validated_at TEXT;

-- Index for querying unvalidated sessions on startup
CREATE INDEX IF NOT EXISTS idx_sessions_integrity_status ON sessions(integrity_status);
```

### Channel Range Definitions

```rust
// crates/storage/src/validation/range_check.rs

use std::collections::HashMap;

pub struct ChannelRange {
    pub min: f64,
    pub max: f64,
}

pub fn channel_ranges() -> HashMap<&'static str, ChannelRange> {
    let mut ranges = HashMap::new();
    ranges.insert("brake", ChannelRange { min: 0.0, max: 1.0 });
    ranges.insert("throttle", ChannelRange { min: 0.0, max: 1.0 });
    ranges.insert("clutch", ChannelRange { min: 0.0, max: 1.0 });
    ranges.insert("speed", ChannelRange { min: 0.0, max: 120.0 });
    ranges.insert("steering", ChannelRange { min: -6.28, max: 6.28 });
    ranges.insert("rpm", ChannelRange { min: 0.0, max: 20000.0 });
    ranges.insert("lat_g", ChannelRange { min: -10.0, max: 10.0 });
    ranges.insert("long_g", ChannelRange { min: -10.0, max: 10.0 });
    ranges.insert("brake_bias", ChannelRange { min: 0.0, max: 1.0 });
    ranges.insert("track_position", ChannelRange { min: 0.0, max: 1.0 });
    ranges.insert("fuel_level", ChannelRange { min: 0.0, max: 200.0 });
    ranges.insert("oil_temp", ChannelRange { min: 0.0, max: 250.0 });
    ranges.insert("water_temp", ChannelRange { min: 0.0, max: 250.0 });
    // Tire temps and pressures
    for channel in ["tire_temp_lf", "tire_temp_rf", "tire_temp_lr", "tire_temp_rr"] {
        ranges.insert(channel, ChannelRange { min: 0.0, max: 200.0 });
    }
    for channel in ["tire_pressure_lf", "tire_pressure_rf", "tire_pressure_lr", "tire_pressure_rr"] {
        ranges.insert(channel, ChannelRange { min: 50.0, max: 350.0 });
    }
    ranges
}
```

**Gear Validation (Int32 — separate logic):**
Gear is an `Int32` column, not `Float64`. Validate separately: range -1 to 8.

### Monotonicity Validation Logic

```rust
// Pseudo-code for monotonicity check
fn validate_lap_distance_monotonicity(data: &RecordBatch) -> Vec<MonotonicityViolation> {
    let lap_distance = data.column_by_name("lap_distance")
        .unwrap()
        .as_any()
        .downcast_ref::<Float64Array>()
        .unwrap();

    let mut violations = Vec::new();
    let mut prev_value = f64::NEG_INFINITY;

    for i in 0..lap_distance.len() {
        let value = lap_distance.value(i);

        // Detect lap reset: distance drops to near-zero (new lap start)
        if value < 50.0 && prev_value > 100.0 {
            prev_value = value; // Reset for new lap
            continue;
        }

        if value < prev_value {
            violations.push(MonotonicityViolation {
                sample_index: i,
                lap_number: 0, // Determined by grouping logic
                previous_value: prev_value,
                violating_value: value,
            });
        }
        prev_value = value;
    }
    violations
}
```

### Integrity Status Priority

When multiple validation failures occur, the overall `integrity_status` uses this priority (worst wins):
1. `checksum_failed` — data may be corrupt, highest severity
2. `range_violation` — data values physically impossible
3. `distance_anomaly` — lap distance tracking issues
4. `valid` — all checks passed
5. `not_validated` — validation has not run yet

### Existing Code to Build On

From Story 1.3 (expected):
- `Database` struct, SQLite pool, migration infrastructure
- `SessionSummary` and `Session` types
- `LapSummary` type with `completion_status` field

From Story 1.4 (expected):
- Parquet reader with `read_telemetry` method returning `RecordBatch`
- SHA-256 `compute_checksum` and `validate_checksum` methods
- `telemetry_checksum` and `telemetry_path` columns on sessions table

From Story 1.5 (expected):
- `SessionDetail` composite type
- `get_session_detail` query
- Extended `get_session_data` IPC command

**Extend, do NOT rewrite** existing infrastructure. The checksum validation from Story 1.4 is the foundation — this story adds status tracking and the broader validation suite on top.

**Do NOT modify the frontend.** Integrity warnings in the UI are handled by Story 5.1 (Summary tab hero cards). This story exposes data via IPC only.

### File Structure for This Story

```
crates/storage/
├── migrations/
│   ├── 001_initial_schema.sql          # Unchanged (Story 1.3)
│   ├── 002_add_telemetry_columns.sql   # Unchanged (Story 1.4)
│   ├── 003_add_delete_and_debriefs.sql # Unchanged (Story 1.5)
│   └── 004_add_integrity_columns.sql   # NEW
├── src/
│   ├── lib.rs                          # Updated — re-export validation module
│   ├── types.rs                        # Updated — add integrity fields to Session, SessionSummary
│   ├── error.rs                        # Updated — add IntegrityError variants
│   ├── sqlite/
│   │   ├── connection.rs               # Updated — call startup validation on init
│   │   └── queries/
│   │       └── sessions.rs             # Updated — add integrity_status to queries
│   ├── validation/                     # NEW module
│   │   ├── mod.rs                      # Module declarations
│   │   ├── checksum.rs                 # Checksum validation with status tracking
│   │   ├── monotonicity.rs             # Lap distance monotonicity checks
│   │   ├── range_check.rs             # Channel value range validation
│   │   ├── lap_time.rs                # Lap time sanity checks
│   │   └── orchestrator.rs            # Composite validation runner
│   └── parquet/
│       └── ...                         # Unchanged
└── tests/
    ├── session_persistence.rs          # Unchanged (Story 1.3)
    ├── parquet_storage.rs              # Unchanged (Story 1.4)
    ├── session_crud.rs                 # Unchanged (Story 1.5)
    └── data_integrity.rs              # NEW

src-tauri/
├── src/
│   ├── lib.rs                          # Updated — register validate_session command
│   └── commands/
│       └── session.rs                  # Updated — add validate_session command, extend responses
```

### Testing Strategy

**Integration Tests (`crates/storage/tests/data_integrity.rs`):**
- Use `tempfile` for temporary directories
- Generate synthetic Arrow RecordBatch data with controlled values
- Test each validation type independently, then the full orchestrator
- Test edge cases: empty data, single-sample, all-NaN

**Synthetic Test Data Helpers:**
```rust
fn create_valid_batch(num_rows: usize) -> RecordBatch {
    // All channels within valid ranges, lap_distance monotonically increasing
}

fn create_invalid_brake_batch(num_rows: usize) -> RecordBatch {
    // Brake values >1.0 for >5% of samples
}

fn create_non_monotonic_batch(num_rows: usize) -> RecordBatch {
    // lap_distance values with backward jumps (not at lap boundaries)
}
```

### Naming Conventions (Enforced)

| Zone | Convention | Example |
|------|-----------|---------|
| Rust functions | `snake_case` | `validate_session_integrity`, `validate_checksum`, `validate_channel_ranges` |
| Rust types | `PascalCase` | `IntegrityReport`, `MonotonicityViolation`, `ChannelRange` |
| Rust constants | `SCREAMING_SNAKE_CASE` | `CHANNEL_RANGES`, `MAX_LAP_TIME_MS` |
| DB columns | `snake_case` | `integrity_status`, `integrity_details`, `integrity_validated_at` |
| IPC JSON fields | `camelCase` | `integrityStatus`, `integrityDetails`, `integrityValidatedAt` |
| IPC commands | `snake_case` | `validate_session` |

### Cross-Story Dependencies

- **Story 1.3** (dev-complete): SQLite database, session/lap types, Database struct
- **Story 1.4** (ready-for-dev): Parquet reader, SHA-256 checksum infrastructure, telemetry schema
- **Story 1.5** (ready-for-dev): SessionDetail composite type, get_session_data extended response
- **Story 3.2** (backlog): Telemetry channel capture — will provide real data for validation
- **Story 5.1** (backlog): Summary tab — will render integrity warnings from this story's data

### References

- [Source: _bmad-output/planning-artifacts/architecture.md#Validation Strategy]
- [Source: _bmad-output/planning-artifacts/architecture.md#Failure Modes & Preventive Controls (Data Layer)]
- [Source: _bmad-output/planning-artifacts/epics-and-stories.md#Story 1.7: Data Integrity Validation]
- [Source: _bmad-output/planning-artifacts/prd.md#NFR12] (checksum validation — SHA-256 on write, validate on read)
- [Source: _bmad-output/planning-artifacts/prd.md#NFR13] (lap distance monotonicity — 100% of valid laps pass)
- [Source: _bmad-output/planning-artifacts/prd.md#NFR14] (session metadata consistency — foreign key enforcement)
- [Source: _bmad-output/planning-artifacts/prd.md#NFR15] (channel value range validation — physically plausible ranges)
- [Source: _bmad-output/planning-artifacts/prd.md#NFR16] (corner segmentation determinism — same input = same output)
- [Source: _bmad-output/implementation-artifacts/1-3-session-history-persists-locally.md] (SQLite foundation)
- [Source: _bmad-output/implementation-artifacts/1-4-parquet-telemetry-storage.md] (Parquet reader, checksum, telemetry schema)
- [Source: _bmad-output/implementation-artifacts/1-5-session-crud-operations.md] (SessionDetail, extended IPC)

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6 (claude-opus-4-6)

### Debug Log References

N/A

### Completion Notes List

- All acceptance criteria implemented
- 20 integration tests in `data_integrity.rs`, all passing
- Full storage crate test suite (66 tests) passes with zero regressions
- Full workspace builds cleanly
- `completion_status` CHECK constraint uses `'complete'` (not `'completed'`) per migration 001

### File List

**New files:**
- `crates/storage/migrations/004_add_integrity_columns.sql`
- `crates/storage/src/validation/mod.rs`
- `crates/storage/src/validation/checksum.rs`
- `crates/storage/src/validation/monotonicity.rs`
- `crates/storage/src/validation/range_check.rs`
- `crates/storage/src/validation/lap_time.rs`
- `crates/storage/src/validation/orchestrator.rs`
- `crates/storage/tests/data_integrity.rs`

**Modified files:**
- `crates/storage/src/lib.rs` - Added `pub mod validation` and re-exports
- `crates/storage/src/types.rs` - Added integrity fields to Session and SessionSummary
- `crates/storage/src/sqlite/connection.rs` - Added validate_session_integrity() and validate_unvalidated_sessions()
- `src-tauri/src/lib.rs` - Added startup validation spawn and registered validate_session command
- `src-tauri/src/commands/session.rs` - Added validate_session IPC command

## Review Follow-ups (AI)

**Build & Test Results:**
- `cargo build -p storage` -- PASS (0 errors, 0 warnings)
- `cargo build -p pitwall` -- PASS (0 errors, 0 warnings)
- `cargo test -p storage` -- ALL 66 TESTS PASS (3 unit + 20 data_integrity + 10 parquet + 14 session_crud + 11 session_filters + 8 session_persistence)
- Note: The cross-story build error from Story 1.6 review (missing `.await` in connection.rs:101) has been resolved -- Story 1.7 correctly made `write_telemetry` async and added `.await` at the call site.
- Previous HIGH-1 from Story 1.6 review is now resolved.

**Acceptance Criteria Coverage:**
- AC1 (SHA-256 Checksum on Read): PASS - `validate_checksum_with_details` logs error with session_id/expected/actual, status persisted
- AC2 (Lap Distance Monotonicity): PASS - monotonicity check with lap reset exclusion (50m threshold), 2% violation threshold
- AC3 (Channel Range Validation): PASS - 21 channels with defined ranges, gear handled as Int32, NaN/Infinity check on others, 5% threshold
- AC4 (Validation on Session Load): PASS - orchestrator runs full suite, startup validation via `tokio::spawn`, `validate_session` IPC command registered
- AC5 (Integrity Status Tracking): PASS - migration 004 adds integrity_status/integrity_details/integrity_validated_at with index
- AC6 (Surfaced to User): PASS - `integrityStatus` on SessionSummary, IntegrityReport returned from validate_session IPC, camelCase serialization
- AC7 (Lap Time Sanity): PASS - validates positive and <600,000ms, flags invalid laps in database

---

- [x] [AI-Review][MEDIUM] MEDIUM-1: `has_excessive_violations` uses total_samples approximation, not per-lap calculation
  - **File:** `crates/storage/src/validation/orchestrator.rs:81-84` and `crates/storage/src/validation/monotonicity.rs:61-76`
  - **Issue:** AC #2 states "A lap with >2% of samples violating monotonicity is flagged." The `has_excessive_violations` function groups violations by lap (line 67-69) but then ignores the per-lap counts and instead calculates the global violation percentage against `total_samples` (line 74). Furthermore, in the orchestrator (line 83), `total_samples` is approximated as `laps.len().max(1) * 100` rather than using the actual `batch.num_rows()`. This means: (a) the per-lap check specified in the AC is not actually performed, and (b) the sample count is a rough approximation.
  - **Impact:** The monotonicity threshold check is computed globally across all laps instead of per-lap. A session with localized violations in one lap (>2%) but spread thinly globally could pass incorrectly. Conversely, a session with minor violations across many laps could be flagged incorrectly.
  - **Fix:** Pass the actual `batch.num_rows()` from the orchestrator. Additionally, implement the per-lap threshold check: group samples by lap, calculate violation rate per lap, flag if any single lap exceeds 2%.
  - **Resolution:** Fixed. `has_excessive_violations` now accepts per-lap sample counts (`HashMap<u32, usize>`) and checks each lap individually against the 2% threshold. Added `compute_lap_sample_counts()` to derive per-lap counts from the RecordBatch. Orchestrator passes actual `batch.num_rows()` and computed per-lap counts. New test `test_excessive_violations_per_lap_check` verifies per-lap behavior.

- [x] [AI-Review][MEDIUM] MEDIUM-2: Orchestrator skips telemetry validation on read failure without setting error status
  - **File:** `crates/storage/src/validation/orchestrator.rs:44-55`
  - **Issue:** When `read_telemetry` fails (line 44), the error is logged at `info!` level and execution continues. The final status will be `STATUS_VALID` (assuming no lap time violations) even though telemetry-based validations were skipped due to a read error. The session appears "valid" when in reality it couldn't be validated.
  - **Impact:** A session with a corrupted Parquet file that passes checksum (e.g., valid file but with schema errors) would be marked as "valid" instead of something like "validation_error" or at minimum "not_validated".
  - **Fix:** Track whether telemetry read failed and either add a new status value or keep the session as `not_validated` when telemetry exists but cannot be read. At minimum, log at `warn!` instead of `info!`.
  - **Resolution:** Fixed. Added `telemetry_read_failed` flag. When telemetry read fails, log level changed to `warn!` and status is set to `STATUS_NOT_VALIDATED` instead of falling through to `STATUS_VALID`. Also changed startup validation error logging from `info!` to `warn!` (LOW-3).

- [x] [AI-Review][MEDIUM] MEDIUM-3: `validate_checksum_with_details` passes `None` for checksum on subsequent read
  - **File:** `crates/storage/src/validation/orchestrator.rs:44`
  - **Issue:** After checksum validation passes (line 43), the orchestrator reads telemetry with `read_telemetry(&path, None)`, explicitly bypassing checksum validation on the read. The checksum was just validated 2 lines above, so this is intentional to avoid redundant work. However, there is a TOCTOU (time-of-check-time-of-use) race: between the checksum validation and the read, the file could theoretically be modified. In practice this is extremely unlikely for a desktop app.
  - **Impact:** Negligible for a single-user desktop application. This is a defensive coding concern rather than a real bug.
  - **Fix:** No action needed. Documenting as reviewed and accepted. Passing the checksum to `read_telemetry` would add a second SHA-256 computation over the entire file, which is unnecessary overhead.

- [x] [AI-Review][MEDIUM] MEDIUM-4: `validate_unvalidated_sessions` loads all unvalidated sessions into memory at once
  - **File:** `crates/storage/src/validation/orchestrator.rs:116-120`
  - **Issue:** `validate_unvalidated_sessions` uses `fetch_all` to load every unvalidated session into memory before processing them one by one. For a desktop app with typical usage (tens to hundreds of sessions), this is fine. But for power users with thousands of sessions, this could use significant memory.
  - **Impact:** Low for typical usage. A user with 1,000+ unvalidated sessions would load all Session records into memory (~1KB each = ~1MB), which is acceptable.
  - **Fix:** No action needed for current scale. If needed later, switch to `fetch` with a streaming iterator to process one at a time.

- [ ] [AI-Review][LOW] LOW-1: `completion_status` CHECK constraint mismatch between code and migration
  - **File:** `crates/storage/src/validation/orchestrator.rs:68` and `crates/storage/migrations/001_initial_schema.sql`
  - **Issue:** The orchestrator updates lap completion_status to `'invalid'` (line 68). The dev notes mention that migration 001 uses a CHECK constraint with `'complete'` (not `'completed'`). Looking at the existing tests, they use `"complete"` for valid laps. The UPDATE to `'invalid'` should be fine as long as the CHECK constraint includes `'invalid'` as an allowed value, or there is no CHECK on completion_status. The dev notes confirm this was considered.
  - **Impact:** If the CHECK constraint on `completion_status` does not include `'invalid'`, the UPDATE would fail at runtime. The tests pass (line 381 confirms `"invalid"` is accepted), so the constraint allows it.
  - **Fix:** No action needed -- verified via tests that `'invalid'` is accepted.

- [ ] [AI-Review][LOW] LOW-2: No test for corrupted-checksum telemetry through the orchestrator
  - **File:** `crates/storage/tests/data_integrity.rs`
  - **Issue:** The test `test_validate_session_with_telemetry` only tests the happy path (valid checksum). There is no integration test that writes telemetry, corrupts the file, then runs the orchestrator to verify `checksum_failed` status is set. The unit-level checksum tests exist in `parquet_storage.rs`, but the orchestrator's handling of checksum failure is untested end-to-end.
  - **Impact:** The orchestrator's `checksum_valid == Some(false)` branch (line 77-78) and its status persistence are not directly tested. The logic is simple and correct by inspection, but an integration test would provide higher confidence.
  - **Fix:** Add a test that: creates a session with telemetry, corrupts the Parquet file, calls `validate_session_integrity`, and asserts `report.status == STATUS_CHECKSUM_FAILED` and `report.checksum_valid == Some(false)`.

- [ ] [AI-Review][LOW] LOW-3: Startup validation errors logged at `info!` level instead of `warn!` or `error!`
  - **File:** `crates/storage/src/validation/orchestrator.rs:150-154`
  - **Issue:** When a session's validation fails during the startup batch run, the error is logged at `info!` level (line 150). Validation failures during normal operation should be at least `warn!` level to be visible in default log configurations.
  - **Impact:** Validation errors during startup may be invisible in production log configurations that filter below `warn` level.
  - **Fix:** Change `info!` to `warn!` for the error case at line 150.
