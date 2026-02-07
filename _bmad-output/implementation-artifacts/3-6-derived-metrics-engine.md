# Story 3.6: Derived Metrics Engine

Status: ready-for-dev

## Story

As a user,
I want advanced metrics computed automatically,
so that I get deeper insights beyond raw telemetry.

## Acceptance Criteria

1. **Brake Application Count Per Lap**
   - For each lap, the system computes the number of distinct brake applications (transitions from brake=0 to brake>threshold)
   - Brake application threshold is configurable (default: 5% to filter noise)
   - A brake application is defined as: brake input crosses above threshold and remains above for at least 2 consecutive samples (debounce)
   - Each brake application records: start_distance (lap distance at onset), peak_pressure (maximum brake value), duration_ms
   - Per-lap brake count is stored in lap summary metadata

2. **Trail Braking Phase Identification and Quantification**
   - For each brake application, the system identifies the trail braking phase: the portion of braking that occurs after the turn-in point (when lateral G exceeds a threshold)
   - Trail braking phase is quantified as: `trail_brake_duration_ms`, `trail_brake_distance_m`, `trail_brake_pressure_avg` (average brake input during trail phase)
   - Turn-in detection uses lateral G threshold (default: 0.3G, configurable) combined with steering angle increase
   - Each corner's trail braking metrics are stored as part of the corner segmentation data (builds on FR8)

3. **Tire Degradation Curves**
   - The system computes tire degradation metrics per stint (contiguous laps between pit stops or session boundaries)
   - Degradation is measured as: temperature delta from stint start to stint end for each tire position (LF, RF, LR, RR)
   - Pressure change per lap is computed as a linear regression slope across the stint
   - Degradation severity is classified: `none` (<2C delta), `mild` (2-5C), `moderate` (5-10C), `severe` (>10C)
   - Degradation data is stored per stint in session metadata

4. **Corner Segmentation Integration (FR8)**
   - The derived metrics engine integrates with corner segmentation to produce per-corner derived metrics
   - Each corner zone has: `brake_application_count`, `avg_brake_pressure`, `trail_brake_avg_distance`, `min_speed`, `apex_speed`, `exit_speed`, `time_in_corner_ms`
   - Corner zones are defined by braking onset (brake>threshold) to full throttle application (throttle>90%) for each corner
   - Corner boundaries are determined by analyzing the best lap's braking/turning pattern and applied consistently across all laps

5. **Derived Metrics Storage**
   - All derived metrics are stored alongside session data in SQLite (not Parquet - these are computed summaries, not raw time-series)
   - Derived metrics are recomputable from raw telemetry (deterministic - NFR14)
   - A `derived_metrics` table stores JSON-serialized metric results keyed by session_id and metric_type
   - Metric computation is idempotent: re-running on the same session produces identical results

6. **Derived Metrics Pipeline**
   - The derived metrics engine runs as a post-processing step after telemetry capture completes and raw data is persisted
   - Processing time is <10ms per lap (NFR2) - the engine must be efficient
   - The pipeline processes laps sequentially but can skip invalid/incomplete laps (flagged by Story 3.3)
   - On completion, the engine emits a `metrics:computed` event with `{ sessionId, metricsComputed: ["brake_count", "trail_braking", "tire_degradation", "corner_segmentation"], type, timestamp, version }`

## Tasks / Subtasks

- [ ] Task 1: Create derived metrics module structure (AC: #5, #6)
  - [ ] 1.1 Create `crates/telemetry-engine/src/derived_metrics/mod.rs` - module root with pipeline orchestration
  - [ ] 1.2 Create `crates/telemetry-engine/src/derived_metrics/types.rs` - metric result types
  - [ ] 1.3 Define `DerivedMetrics` struct containing all computed metrics for a session
  - [ ] 1.4 Define `MetricType` enum: `BrakeCount`, `TrailBraking`, `TireDegradation`, `CornerSegmentation`
  - [ ] 1.5 Update `crates/telemetry-engine/src/lib.rs` to declare `derived_metrics` module

- [ ] Task 2: Implement brake application counter (AC: #1)
  - [ ] 2.1 Create `crates/telemetry-engine/src/derived_metrics/brake_counter.rs`
  - [ ] 2.2 Implement `count_brake_applications(lap_telemetry: &[TelemetrySample], threshold: f64) -> Vec<BrakeApplication>`
  - [ ] 2.3 `BrakeApplication` struct: `start_distance: f64`, `peak_pressure: f64`, `duration_ms: u64`, `release_distance: f64`
  - [ ] 2.4 Implement noise debounce: require 2 consecutive samples above threshold
  - [ ] 2.5 Compute per-lap summary: `brake_count`, `avg_peak_pressure`, `total_braking_distance`

- [ ] Task 3: Implement trail braking analyzer (AC: #2)
  - [ ] 3.1 Create `crates/telemetry-engine/src/derived_metrics/trail_braking.rs`
  - [ ] 3.2 Implement `analyze_trail_braking(brake_application: &BrakeApplication, lap_telemetry: &[TelemetrySample], lat_g_threshold: f64) -> Option<TrailBrakingPhase>`
  - [ ] 3.3 `TrailBrakingPhase` struct: `trail_duration_ms: u64`, `trail_distance_m: f64`, `trail_pressure_avg: f64`, `turn_in_distance: f64`
  - [ ] 3.4 Turn-in detection: find first sample where `|lat_g| > lat_g_threshold` AND `|steering| > steering_threshold` within the braking zone
  - [ ] 3.5 Trail braking = portion of brake application after turn-in point

- [ ] Task 4: Implement tire degradation calculator (AC: #3)
  - [ ] 4.1 Create `crates/telemetry-engine/src/derived_metrics/tire_degradation.rs`
  - [ ] 4.2 Implement `compute_tire_degradation(stint_laps: &[LapTelemetry]) -> TireDegradation`
  - [ ] 4.3 `TireDegradation` struct per tire position: `temp_delta_c: f64`, `pressure_slope_per_lap: f64`, `severity: DegradationSeverity`
  - [ ] 4.4 `DegradationSeverity` enum: `None`, `Mild`, `Moderate`, `Severe` (thresholds: <2C, 2-5C, 5-10C, >10C)
  - [ ] 4.5 Stint detection: group consecutive laps (gap > configurable threshold indicates pit stop)
  - [ ] 4.6 Compute linear regression slope for pressure change across stint laps

- [ ] Task 5: Implement corner segmentation engine (AC: #4)
  - [ ] 5.1 Create `crates/telemetry-engine/src/derived_metrics/corner_segmenter.rs` (replaces placeholder `corner_segmenter.rs` from architecture)
  - [ ] 5.2 Implement `segment_corners(best_lap: &[TelemetrySample]) -> Vec<CornerZone>`
  - [ ] 5.3 `CornerZone` struct: `corner_id: u32`, `name: String` (T1, T2...), `start_distance: f64`, `end_distance: f64`, `brake_onset_distance: f64`, `apex_distance: f64`
  - [ ] 5.4 Corner detection algorithm: identify braking zones (brake > threshold) followed by turning (lat_g > threshold)
  - [ ] 5.5 Apply consistent corner boundaries from best lap to all laps in the session
  - [ ] 5.6 Compute per-corner metrics: `brake_count`, `avg_brake_pressure`, `trail_brake_avg_distance`, `min_speed`, `apex_speed`, `exit_speed`, `time_in_corner_ms`

- [ ] Task 6: Implement derived metrics pipeline orchestrator (AC: #6)
  - [ ] 6.1 Create `compute_derived_metrics(session_id: &str, db: &Database, data_dir: &Path) -> Result<DerivedMetrics, TelemetryError>`
  - [ ] 6.2 Pipeline steps: load raw telemetry -> segment corners -> compute brake metrics -> analyze trail braking -> compute tire degradation
  - [ ] 6.3 Skip invalid/incomplete laps (check `completion_status` from Story 3.3)
  - [ ] 6.4 Measure and log processing time per lap to verify NFR2 compliance (<10ms/lap)
  - [ ] 6.5 Emit `metrics:computed` Tauri event on completion

- [ ] Task 7: Add SQLite storage for derived metrics (AC: #5)
  - [ ] 7.1 Create migration `crates/storage/migrations/NNN_add_derived_metrics.sql`
  - [ ] 7.2 Add `derived_metrics` table: `id INTEGER PRIMARY KEY`, `session_id TEXT NOT NULL REFERENCES sessions(id)`, `metric_type TEXT NOT NULL`, `data TEXT NOT NULL` (JSON), `computed_at TEXT NOT NULL`
  - [ ] 7.3 Add `corner_zones` table: `id INTEGER PRIMARY KEY`, `session_id TEXT NOT NULL REFERENCES sessions(id)`, `corner_id INTEGER NOT NULL`, `name TEXT NOT NULL`, `start_distance REAL NOT NULL`, `end_distance REAL NOT NULL`, `brake_onset_distance REAL`, `apex_distance REAL`
  - [ ] 7.4 Add query functions: `insert_derived_metrics`, `get_derived_metrics`, `insert_corner_zones`, `get_corner_zones`
  - [ ] 7.5 Add UNIQUE constraint on `(session_id, metric_type)` for idempotency

- [ ] Task 8: Write unit and integration tests (AC: all)
  - [ ] 8.1 Test: brake counter detects correct number of applications from synthetic telemetry
  - [ ] 8.2 Test: brake counter debounce filters single-sample spikes
  - [ ] 8.3 Test: trail braking identifies correct phase from known telemetry pattern
  - [ ] 8.4 Test: tire degradation computes correct severity classification
  - [ ] 8.5 Test: corner segmenter produces deterministic zones for identical input (NFR14/NFR16)
  - [ ] 8.6 Test: full pipeline processes a synthetic session and produces all metric types
  - [ ] 8.7 Test: re-running pipeline produces identical results (idempotency)
  - [ ] 8.8 Test: pipeline skips invalid laps without errors
  - [ ] 8.9 Run `cargo test` and `cargo build` - all pass

## Dev Notes

### Architecture Compliance

**Crate Boundaries:**
- All derived metrics computation lives in `crates/telemetry-engine/src/derived_metrics/` - telemetry-engine owns preprocessing and analysis
- Storage of computed results uses `crates/storage/` SQLite APIs - storage remains a leaf crate
- Tauri event emission in `src-tauri/src/events.rs` - app shell orchestrates events
- The architecture doc lists `corner_segmenter.rs` and `preprocessor.rs` as planned files in telemetry-engine; this story implements the corner segmenter

**Data Flow:**
```
Raw Parquet telemetry (from Story 1.4)
  -> Load telemetry data (storage crate reader)
  -> Segment corners (corner_segmenter)
  -> Compute brake metrics (brake_counter)
  -> Analyze trail braking (trail_braking)
  -> Compute tire degradation (tire_degradation)
  -> Store results in SQLite (derived_metrics table)
  -> Emit metrics:computed event
```

**Determinism Requirement (NFR14, NFR16):**
All computations must be deterministic: identical input telemetry must produce identical derived metrics and identical corner boundaries. This means:
- No floating-point non-determinism (use consistent rounding)
- No random or time-dependent values in computation
- Corner boundaries from best lap are applied consistently to all laps
- Re-processing same session produces identical results (verified by tests)

### Existing Code to Build On

From Story 1.4 (done):
- `crates/storage/src/parquet/reader.rs` - Read telemetry data from Parquet files
- `crates/storage/src/parquet/schema.rs` - Canonical 35-channel telemetry schema

From Story 1.8 (done):
- `crates/telemetry-engine/src/ibt_importer.rs` - Channel mapping patterns, RecordBatch handling

From Story 3.3 (backlog - should be implemented first):
- Lap boundary detection and lap summary computation
- `completion_status` metadata for incomplete laps

From Architecture doc:
- `crates/telemetry-engine/src/corner_segmenter.rs` - planned file, implemented in this story
- `crates/telemetry-engine/src/preprocessor.rs` - planned file, pipeline orchestrator

### Corner Segmentation Algorithm

The corner segmentation algorithm identifies corners by detecting braking-turning patterns:

1. **Find braking zones:** Identify segments where brake > threshold (default 5%)
2. **Find turning zones:** Identify segments where |lat_g| > threshold (default 0.3G)
3. **Match brake-to-turn pairs:** For each braking zone, find the next turning zone within a reasonable distance window
4. **Define corner boundaries:** Corner starts at brake onset, ends when throttle > 90% (full power)
5. **Name corners sequentially:** T1, T2, T3... based on lap distance order
6. **Use best lap as template:** Segment the best lap first, then apply boundaries to all laps

This algorithm is consistent with FR8 (segment based on lap distance and braking/turning patterns) and NFR16 (deterministic - same session always produces same corners).

### Performance Budget

Per NFR2, pre-processing must complete in <10ms per lap. For a 60-minute session at 60Hz:
- ~60 laps x 60 seconds x 60Hz = ~216,000 samples per session
- ~3,600 samples per lap
- Budget: 10ms per lap = ~2.8 microseconds per sample

This budget is achievable for simple arithmetic operations (threshold comparisons, running averages) on pre-loaded data. The key optimization: load telemetry once, iterate once, compute all metrics in a single pass where possible.

### File Structure for This Story

```
crates/telemetry-engine/
  src/
    lib.rs                              # UPDATED - declare derived_metrics module
    derived_metrics/
      mod.rs                            # NEW - Pipeline orchestrator
      types.rs                          # NEW - DerivedMetrics, BrakeApplication, TrailBrakingPhase, etc.
      brake_counter.rs                  # NEW - Brake application counter
      trail_braking.rs                  # NEW - Trail braking analyzer
      tire_degradation.rs               # NEW - Tire degradation calculator
      corner_segmenter.rs               # NEW - Corner zone segmentation (per architecture)

crates/storage/
  migrations/
    NNN_add_derived_metrics.sql         # NEW - derived_metrics + corner_zones tables
  src/
    sqlite/queries/
      derived_metrics.rs                # NEW - Derived metrics CRUD
      corner_zones.rs                   # NEW - Corner zone CRUD

src-tauri/
  src/
    events.rs                           # UPDATED - MetricsComputed event struct
```

### Naming Conventions (Enforced)

| Zone | Convention | Example |
|------|-----------|---------|
| Rust functions | `snake_case` | `count_brake_applications`, `analyze_trail_braking`, `segment_corners` |
| Rust types | `PascalCase` | `BrakeApplication`, `TrailBrakingPhase`, `CornerZone`, `TireDegradation` |
| DB tables | `snake_case` plural | `derived_metrics`, `corner_zones` |
| DB columns | `snake_case` | `metric_type`, `corner_id`, `start_distance`, `brake_onset_distance` |
| Tauri events | `kebab-case` with namespace | `metrics:computed` |
| IPC JSON fields | `camelCase` | `metricsComputed`, `sessionId`, `cornerId` |

### Cross-Story Dependencies

- **Story 3.2** (backlog): Telemetry channel capture - raw telemetry data this story processes
- **Story 3.3** (backlog): Session lifecycle - lap boundaries and completion_status used to skip invalid laps
- **Story 1.4** (done): Parquet storage - raw telemetry reader
- **Story 1.3** (done): SQLite database - session and lap queries
- **Story 4.3** (backlog): AI coaching prompt engineering - consumes derived metrics for coaching context
- **Story 4.4** (backlog): Per-corner analysis - uses corner segmentation and per-corner metrics

### NFR Compliance

| NFR | Target | How This Story Addresses It |
|-----|--------|-----------------------------|
| NFR2 | <10ms per lap preprocessing | Performance budget tracked; single-pass algorithms |
| NFR14 | Deterministic derived metrics | Same input -> same output; verified by idempotency tests |
| NFR16 | Deterministic corner segmentation | Best-lap template applied consistently; reproduced on re-process |

## References

- [Source: _bmad-output/planning-artifacts/epics-and-stories.md#Story 3.6: Derived Metrics Engine]
- [Source: _bmad-output/planning-artifacts/architecture.md#Complete Project Directory Structure - corner_segmenter.rs, preprocessor.rs]
- [Source: _bmad-output/planning-artifacts/architecture.md#Requirements to Structure Mapping - FR1-FR10]
- [Source: _bmad-output/planning-artifacts/prd.md#FR8] (segment telemetry into corner zones based on braking/turning patterns)
- [Source: _bmad-output/planning-artifacts/prd.md#FR9] (compute derived metrics: brake count, trail braking, tire degradation)
- [Source: _bmad-output/planning-artifacts/prd.md#NFR2] (pre-processing time <10ms per lap)
- [Source: _bmad-output/planning-artifacts/prd.md#NFR14] (derived metric reproducibility - identical output for identical input)
- [Source: _bmad-output/planning-artifacts/prd.md#NFR16] (corner segmentation determinism - same session -> same boundaries)
- [Source: _bmad-output/planning-artifacts/ux-design-specification.md#Journey 1 - corner-zoom navigation]
- [Source: _bmad-output/planning-artifacts/ux-design-specification.md#Journey 3 - tire degradation, stint analysis]

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List
