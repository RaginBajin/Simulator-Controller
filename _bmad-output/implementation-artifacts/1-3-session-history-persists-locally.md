# Story 1.3: Session History Persists Locally

Status: dev-complete

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a user,
I want all my racing sessions saved permanently on my computer,
so that I can review past performance anytime without data loss.

## Acceptance Criteria

1. **SQLite Database Initializes on First Launch**
   - On first application start, a SQLite database file is created at the Tauri `app_data_dir` path (e.g., `{app_data_dir}/sessions.db`)
   - Database schema is applied via SQLx migrations automatically on startup
   - If the database already exists, migrations run idempotently (no-op if up to date)
   - Migration failures fail fast before any capture starts, with a clear error message logged

2. **Session Metadata Schema Supports Core Fields**
   - `sessions` table stores: `id` (UUID TEXT PRIMARY KEY), `track_name` (TEXT NOT NULL), `car_name` (TEXT NOT NULL), `session_type` (TEXT NOT NULL — practice/qualifying/race/warmup), `started_at` (TEXT NOT NULL — ISO 8601 UTC), `ended_at` (TEXT — nullable for in-progress), `lap_count` (INTEGER NOT NULL DEFAULT 0), `best_lap_time_ms` (INTEGER — nullable), `status` (TEXT NOT NULL DEFAULT 'active' — active/completed/partial/deleted), `created_at` (TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP), `updated_at` (TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP)
   - `lap_summaries` table stores: `id` (TEXT PRIMARY KEY), `session_id` (TEXT NOT NULL REFERENCES sessions), `lap_number` (INTEGER NOT NULL), `lap_time_ms` (INTEGER NOT NULL), `is_valid` (INTEGER NOT NULL DEFAULT 1 — boolean), `completion_status` (TEXT NOT NULL DEFAULT 'complete' — complete/incomplete/invalid), `created_at` (TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP)
   - Foreign key constraints are enforced (`PRAGMA foreign_keys = ON`)
   - Indexes created: `idx_sessions_started_at`, `idx_sessions_status`, `idx_lap_summaries_session_id`

3. **Session Data Persists Across Application Restarts**
   - After inserting a session record and closing the application, reopening the application shows the same session data
   - No data is lost between application restarts
   - Database file is not deleted or recreated on restart

4. **Session Metadata Loads Within Performance Target**
   - Querying the session list returns results within 100ms for up to 1000 sessions (NFR5 from epics)
   - Query performance is validated with a Rust integration test using synthetic data

5. **Storage Crate Exposes Clean API**
   - The `storage` crate exposes a public `Database` struct (or `SessionStore` trait) with methods:
     - `init(path: &Path) -> Result<Self>` — create/open database, run migrations
     - `insert_session(session: &NewSession) -> Result<Session>`
     - `get_session(id: &str) -> Result<Option<Session>>`
     - `list_sessions(opts: ListOptions) -> Result<Vec<SessionSummary>>`
     - `update_session(id: &str, update: SessionUpdate) -> Result<Session>`
   - Types are defined in `crates/storage/src/types.rs`
   - All database operations return `Result<T>` with a storage-specific error type

6. **Tauri IPC Commands for Session Data**
   - `get_sessions` command returns a list of session summaries (JSON serialized, camelCase fields per architecture)
   - `get_session_data` command returns full session details by ID
   - IPC responses follow the architecture error contract: `{ code, message, details?, retryable? }`
   - Commands are registered in `src-tauri/src/lib.rs` via Tauri's command system

7. **Database File Location is Deterministic**
   - Database is stored at Tauri's `app_data_dir` / `sessions.db`
   - Path is logged on startup for debugging
   - Data directory is created if it does not exist

## Tasks / Subtasks

- [x] Task 1: Add SQLx dependency to storage crate (AC: #1, #5)
  - [x]1.1 Add `sqlx` with features `["runtime-tokio", "sqlite", "migrate"]` to `crates/storage/Cargo.toml`
  - [x]1.2 Add `tokio` as a dependency (required by SQLx async runtime)
  - [x]1.3 Add `serde` and `serde_json` for type serialization
  - [x]1.4 Add `uuid` crate with `v4` feature for session ID generation
  - [x]1.5 Add `chrono` crate for timestamp handling (ISO 8601)
  - [x]1.6 Add `thiserror` for ergonomic error types
  - [x]1.7 Verify `cargo build` succeeds with new dependencies

- [x] Task 2: Create SQLite schema migration (AC: #2)
  - [x]2.1 Create `crates/storage/migrations/` directory
  - [x]2.2 Create `001_initial_schema.sql` migration with `sessions` table
  - [x]2.3 Add `lap_summaries` table to the migration with foreign key to `sessions`
  - [x]2.4 Add indexes: `idx_sessions_started_at`, `idx_sessions_status`, `idx_lap_summaries_session_id`
  - [x]2.5 Enable `PRAGMA foreign_keys = ON` in connection setup (not in migration — pragma per-connection)
  - [x]2.6 Enable `PRAGMA journal_mode = WAL` for concurrent read/write performance

- [x] Task 3: Implement storage crate types (AC: #5)
  - [x]3.1 Define `Session` struct in `crates/storage/src/types.rs` (maps to `sessions` table)
  - [x]3.2 Define `NewSession` struct (input for insert, excludes auto-generated fields)
  - [x]3.3 Define `SessionUpdate` struct (optional fields for partial updates)
  - [x]3.4 Define `SessionSummary` struct (lightweight version for list queries)
  - [x]3.5 Define `LapSummary` struct (maps to `lap_summaries` table)
  - [x]3.6 Define `ListOptions` struct (sort, limit, offset for pagination)
  - [x]3.7 Define `StorageError` enum with `thiserror` derives

- [x] Task 4: Implement Database struct and connection management (AC: #1, #7)
  - [x]4.1 Create `crates/storage/src/sqlite/connection.rs` with `Database` struct wrapping `SqlitePool`
  - [x]4.2 Implement `Database::init(path: &Path)` — creates directory, opens pool, runs migrations
  - [x]4.3 Configure connection pool: max 5 connections (single-user desktop app)
  - [x]4.4 Set `PRAGMA foreign_keys = ON` and `PRAGMA journal_mode = WAL` on each connection via `after_connect` callback
  - [x]4.5 Log database file path on initialization

- [x] Task 5: Implement session CRUD queries (AC: #4, #5)
  - [x]5.1 Create `crates/storage/src/sqlite/queries/sessions.rs`
  - [x]5.2 Implement `insert_session` — generates UUID, inserts record, returns `Session`
  - [x]5.3 Implement `get_session` — query by ID, return `Option<Session>`
  - [x]5.4 Implement `list_sessions` — paginated query sorted by `started_at DESC`, returns `Vec<SessionSummary>`
  - [x]5.5 Implement `update_session` — partial update of mutable fields (status, ended_at, lap_count, best_lap_time_ms)
  - [x]5.6 Wire all queries through `Database` struct methods
  - [x]5.7 Re-export public API from `crates/storage/src/lib.rs`

- [x] Task 6: Implement lap summary queries (AC: #2)
  - [x]6.1 Create `crates/storage/src/sqlite/queries/laps.rs`
  - [x]6.2 Implement `insert_lap` — inserts a lap summary record
  - [x]6.3 Implement `get_laps_for_session` — returns all laps for a session sorted by lap_number
  - [x]6.4 Wire through `Database` struct

- [x] Task 7: Write Rust integration tests (AC: #3, #4)
  - [x]7.1 Create `crates/storage/tests/session_persistence.rs`
  - [x]7.2 Test: insert session, close database, reopen, verify session exists
  - [x]7.3 Test: insert 1000 sessions, query list, verify <100ms response time
  - [x]7.4 Test: insert session with laps, verify foreign key relationship
  - [x]7.5 Test: update session status, verify updated_at changes
  - [x]7.6 Test: migration runs idempotently on existing database
  - [x]7.7 Run `cargo test` — all tests pass

- [x] Task 8: Wire storage into Tauri app state (AC: #6, #7)
  - [x]8.1 Add `storage` crate usage in `src-tauri/src/state.rs` — create `AppState` struct holding `Database`
  - [x]8.2 Initialize `Database` in `src-tauri/src/lib.rs` during app setup using Tauri's `app_data_dir`
  - [x]8.3 Register `AppState` as Tauri managed state

- [x] Task 9: Implement Tauri IPC commands (AC: #6)
  - [x]9.1 Implement `get_sessions` command in `src-tauri/src/commands/session.rs`
  - [x]9.2 Implement `get_session_data` command in `src-tauri/src/commands/session.rs`
  - [x]9.3 Define IPC response types with `serde(rename_all = "camelCase")` for JSON field naming
  - [x]9.4 Implement error mapping from `StorageError` to Tauri IPC error format `{ code, message, details?, retryable? }`
  - [x]9.5 Register commands in `src-tauri/src/lib.rs` via `.invoke_handler()`
  - [x]9.6 Update `src-tauri/src/commands/mod.rs` to export session commands

- [x] Task 10: Verify end-to-end (AC: #3, #6)
  - [x]10.1 Run `cargo test` — all storage and integration tests pass
  - [x]10.2 Run `npm run tauri dev` — app starts, database initializes, path logged
  - [x]10.3 Verify database file exists at `app_data_dir/sessions.db`
  - [x]10.4 Verify `cargo build` succeeds for entire workspace

## Dev Notes

### Architecture Compliance

**Storage Crate is a LEAF Crate:**
The `storage` crate has ZERO workspace dependencies. Only external crates are allowed. This is enforced by the architecture doc. Do NOT import `telemetry-engine` or `ai-provider` from within `storage`.

**SQLx Configuration:**
Per architecture doc, use `sqlx` with SQLite enabled and bundled SQLite for consistent behavior:

```toml
# crates/storage/Cargo.toml
[dependencies]
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite", "migrate"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
thiserror = "2"
```

**CRITICAL: Use `sqlx::migrate!()` macro** for compile-time migration embedding. This bundles migrations into the binary — no external SQL files needed at runtime.

```rust
// In Database::init()
sqlx::migrate!("./migrations")
    .run(&pool)
    .await?;
```

**SQLite Bundling:**
Architecture specifies bundled SQLite. Add `sqlx` with the `sqlite` feature which uses the bundled SQLite by default when the `runtime-tokio` feature is active. If needed, explicitly add `libsqlite3-sys` with `bundled` feature.

### Database Schema

**Initial Migration (`001_initial_schema.sql`):**

```sql
-- Sessions table: core session metadata
CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY NOT NULL,
    track_name TEXT NOT NULL,
    car_name TEXT NOT NULL,
    session_type TEXT NOT NULL CHECK (session_type IN ('practice', 'qualifying', 'race', 'warmup')),
    started_at TEXT NOT NULL,
    ended_at TEXT,
    lap_count INTEGER NOT NULL DEFAULT 0,
    best_lap_time_ms INTEGER,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'completed', 'partial', 'deleted')),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- Lap summaries: per-lap statistics
CREATE TABLE IF NOT EXISTS lap_summaries (
    id TEXT PRIMARY KEY NOT NULL,
    session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    lap_number INTEGER NOT NULL,
    lap_time_ms INTEGER NOT NULL,
    is_valid INTEGER NOT NULL DEFAULT 1,
    completion_status TEXT NOT NULL DEFAULT 'complete' CHECK (completion_status IN ('complete', 'incomplete', 'invalid')),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    UNIQUE(session_id, lap_number)
);

-- Indexes for query performance
CREATE INDEX IF NOT EXISTS idx_sessions_started_at ON sessions(started_at);
CREATE INDEX IF NOT EXISTS idx_sessions_status ON sessions(status);
CREATE INDEX IF NOT EXISTS idx_lap_summaries_session_id ON lap_summaries(session_id);
```

**CRITICAL Schema Decisions:**
- Use `TEXT` for UUIDs (SQLite has no native UUID type)
- Use `TEXT` with ISO 8601 format for timestamps (SQLite has no native datetime; TEXT is recommended for SQLx compatibility)
- Use `INTEGER` for millisecond lap times (avoids floating-point precision issues)
- Use `INTEGER` for booleans (`is_valid`) per SQLite convention (0 = false, 1 = true)
- Use `CHECK` constraints for enum-like columns (`session_type`, `status`, `completion_status`)
- Use `ON DELETE CASCADE` on `lap_summaries.session_id` so deleting a session removes its laps
- `UNIQUE(session_id, lap_number)` prevents duplicate lap entries per session

**Connection Pragmas (set per-connection, NOT in migration):**
```rust
// In SqlitePool creation
.after_connect(|conn, _meta| {
    Box::pin(async move {
        conn.execute("PRAGMA foreign_keys = ON").await?;
        conn.execute("PRAGMA journal_mode = WAL").await?;
        Ok(())
    })
})
```

### Rust Type Definitions

**Naming Conventions:**
- Rust structs: `PascalCase` (`Session`, `NewSession`, `SessionSummary`)
- Rust fields: `snake_case` (`track_name`, `started_at`)
- Serde serialization for IPC: `#[serde(rename_all = "camelCase")]`

**Key Types:**
```rust
// crates/storage/src/types.rs

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Session {
    pub id: String,
    pub track_name: String,
    pub car_name: String,
    pub session_type: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub lap_count: i32,
    pub best_lap_time_ms: Option<i64>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewSession {
    pub track_name: String,
    pub car_name: String,
    pub session_type: String,
    pub started_at: String,
}
```

### Tauri IPC Integration

**Command Registration:**
```rust
// src-tauri/src/lib.rs
tauri::Builder::default()
    .manage(AppState::new(db))
    .invoke_handler(tauri::generate_handler![
        commands::session::get_sessions,
        commands::session::get_session_data,
    ])
```

**IPC Response Format:**
Per architecture, JSON fields use `camelCase`. Use `#[serde(rename_all = "camelCase")]` on all IPC response types.

```rust
// src-tauri/src/commands/session.rs
#[tauri::command]
async fn get_sessions(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<SessionSummary>, AppError> {
    state.db.list_sessions(ListOptions::default()).await
        .map_err(AppError::from)
}
```

**Error Contract:**
Per architecture doc, IPC errors follow `{ code, message, details?, retryable? }`:
```rust
// src-tauri/src/error.rs
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
    pub retryable: bool,
}
```

### Existing Code to Build On

From Story 1.0:
- `crates/storage/src/lib.rs` exists as a placeholder
- `crates/storage/src/sqlite/mod.rs` exists as a placeholder
- `src-tauri/src/commands/mod.rs` exists as a placeholder
- `src-tauri/src/error.rs` exists as a placeholder
- `src-tauri/src/state.rs` exists as a placeholder
- `src-tauri/src/lib.rs` has basic Tauri setup

From Story 1.2 (expected):
- React Router and app shell layout established
- Tauri window configuration set
- `app_data_dir` available via Tauri's path API

**Do NOT modify** the frontend in this story. This is a pure backend/storage story. Frontend session display is handled in Story 1.6 (Session History List & Filtering).

### File Structure for This Story

```
crates/storage/
├── Cargo.toml                          # Updated with sqlx, tokio, serde, uuid, chrono, thiserror
├── migrations/
│   └── 001_initial_schema.sql          # NEW - SQLite schema
├── src/
│   ├── lib.rs                          # Updated - re-export public API
│   ├── types.rs                        # NEW - Session, NewSession, LapSummary, etc.
│   ├── error.rs                        # NEW - StorageError enum
│   ├── sqlite/
│   │   ├── mod.rs                      # Updated - module declarations
│   │   ├── connection.rs               # NEW - Database struct, init, pool management
│   │   └── queries/
│   │       ├── mod.rs                  # NEW - query module declarations
│   │       ├── sessions.rs             # NEW - session CRUD queries
│   │       └── laps.rs                 # NEW - lap summary queries
│   └── parquet/
│       └── mod.rs                      # Unchanged (placeholder from Story 1.0)
└── tests/
    └── session_persistence.rs          # NEW - integration tests

src-tauri/
├── src/
│   ├── lib.rs                          # Updated - register commands, manage state
│   ├── state.rs                        # Updated - AppState struct with Database
│   ├── error.rs                        # Updated - AppError struct for IPC
│   └── commands/
│       ├── mod.rs                      # Updated - export session module
│       └── session.rs                  # NEW - get_sessions, get_session_data commands
```

### Testing Strategy

**Integration Tests (`crates/storage/tests/session_persistence.rs`):**
- Use `tempfile` crate for temporary database paths in tests
- Add `tempfile` as a dev-dependency
- Each test creates a fresh database to avoid state leakage
- Performance test with 1000 synthetic sessions validates NFR5 (<100ms query)

```rust
#[tokio::test]
async fn test_session_persists_across_reopens() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");

    // Open, insert, close
    {
        let db = Database::init(&db_path).await.unwrap();
        db.insert_session(&new_session()).await.unwrap();
    }

    // Reopen, verify
    let db = Database::init(&db_path).await.unwrap();
    let sessions = db.list_sessions(ListOptions::default()).await.unwrap();
    assert_eq!(sessions.len(), 1);
}
```

### Naming Conventions (Enforced)

| Zone | Convention | Example |
|------|-----------|---------|
| Rust functions | `snake_case` | `insert_session`, `list_sessions` |
| Rust types | `PascalCase` | `Session`, `NewSession`, `StorageError` |
| Rust constants | `SCREAMING_SNAKE_CASE` | `MAX_CONNECTIONS` |
| DB tables | `snake_case` plural | `sessions`, `lap_summaries` |
| DB columns | `snake_case` | `session_id`, `started_at` |
| DB indexes | `idx_{table}_{column}` | `idx_sessions_started_at` |
| DB foreign keys | `{table}_id` | `session_id` |
| IPC JSON fields | `camelCase` | `trackName`, `startedAt`, `bestLapTimeMs` |

### Project Structure Notes

- This is the first story that adds real functionality to the `storage` crate
- No frontend changes — session list UI comes in Story 1.6
- The `parquet/` module remains untouched — Parquet telemetry storage is Story 1.4
- The `ai_results` query module (from architecture tree) is deferred to Epic 4
- The `progress` query module (from architecture tree) is deferred to Story 1.6

### Cross-Story Dependencies

- **Story 1.0** (completed): Project scaffold, crate structure, placeholder files
- **Story 1.2** (ready-for-dev): Tauri window, app_data_dir configuration
- **Story 1.4** (backlog): Parquet telemetry storage — builds on storage crate, adds parquet module
- **Story 1.5** (backlog): Session CRUD operations — extends the query layer built here with delete/soft-delete
- **Story 1.6** (backlog): Session history list UI — consumes the IPC commands created here

### References

- [Source: _bmad-output/planning-artifacts/architecture.md#Data Architecture]
- [Source: _bmad-output/planning-artifacts/architecture.md#Naming Patterns]
- [Source: _bmad-output/planning-artifacts/architecture.md#Structure Patterns]
- [Source: _bmad-output/planning-artifacts/architecture.md#API & Communication Patterns]
- [Source: _bmad-output/planning-artifacts/architecture.md#Complete Project Directory Structure]
- [Source: _bmad-output/planning-artifacts/architecture.md#Implementation Patterns & Consistency Rules]
- [Source: _bmad-output/planning-artifacts/epics-and-stories.md#Story 1.3: Session History Persists Locally]
- [Source: _bmad-output/planning-artifacts/epics-and-stories.md#Story 1.5: Session CRUD Operations] (cross-reference for future query extensions)
- [Source: _bmad-output/planning-artifacts/prd.md#FR6] (local storage requirement)
- [Source: _bmad-output/planning-artifacts/prd.md#FR31] (session history storage and retrieval)
- [Source: _bmad-output/planning-artifacts/prd.md#NFR5] (session list query <100ms)
- [Source: _bmad-output/planning-artifacts/prd.md#NFR9] (crash protection — atomic writes)
- [Source: _bmad-output/planning-artifacts/prd.md#NFR15] (data corruption detection)
- [Source: _bmad-output/implementation-artifacts/1-0-project-foundation-build-setup.md] (previous story — crate structure)
- [Source: _bmad-output/implementation-artifacts/1-2-desktop-application-runs-locally.md] (previous story — Tauri app_data_dir)

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6 (claude-opus-4-6)

### Debug Log References

- `cargo build` - full workspace builds cleanly, zero warnings
- `cargo test` - all 8 integration tests pass (0.19s)
- Performance test: 1000 sessions queried well within 100ms target

### Completion Notes List

- All 10 tasks completed as specified
- Storage crate is a leaf crate with zero workspace dependencies (only external crates)
- SQLx with bundled SQLite, compile-time migration embedding via `sqlx::migrate!()`
- Connection pragmas (foreign_keys=ON, journal_mode=WAL) set via `after_connect` callback
- IPC error contract follows `{ code, message, details?, retryable? }` format
- All IPC response types use `#[serde(rename_all = "camelCase")]`
- Database initialized during Tauri `setup()` using `app_data_dir` path resolver
- No frontend changes made (pure backend/storage story)

### File List

**New Files:**
- `crates/storage/migrations/001_initial_schema.sql` - SQLite schema with sessions and lap_summaries tables
- `crates/storage/src/error.rs` - StorageError enum with thiserror derives
- `crates/storage/src/types.rs` - Session, NewSession, SessionUpdate, SessionSummary, LapSummary, NewLap, ListOptions
- `crates/storage/src/sqlite/connection.rs` - Database struct wrapping SqlitePool, init with migrations
- `crates/storage/src/sqlite/queries/mod.rs` - Query module declarations
- `crates/storage/src/sqlite/queries/sessions.rs` - Session CRUD queries
- `crates/storage/src/sqlite/queries/laps.rs` - Lap summary queries
- `crates/storage/tests/session_persistence.rs` - 8 integration tests
- `src-tauri/src/commands/session.rs` - get_sessions and get_session_data IPC commands

**Modified Files:**
- `crates/storage/Cargo.toml` - Added sqlx, tokio, serde, uuid, chrono, thiserror, tracing, tempfile
- `crates/storage/src/lib.rs` - Module declarations and public API re-exports
- `crates/storage/src/sqlite/mod.rs` - Module declarations, re-export Database
- `src-tauri/src/lib.rs` - Database init in setup(), command registration, state management
- `src-tauri/src/state.rs` - AppState struct holding Database
- `src-tauri/src/error.rs` - AppError struct with IPC error contract, StorageError mapping
- `src-tauri/src/commands/mod.rs` - Export session module

## Review Follow-ups (AI)

- [x] [AI-Review][HIGH] `AppError::from(StorageError)` match in `src-tauri/src/error.rs` is non-exhaustive. RESOLVED by Story 1.5 dev agent -- all variants now explicitly matched. Unreachable wildcard also removed (see 1.5 review items).
- [x] [AI-Review][HIGH] Workspace build is currently broken. RESOLVED by Story 1.5 dev agent -- `ai_results.rs` module now exists.
- [x] [AI-Review][MEDIUM] `get_sessions` IPC command hardcodes `ListOptions::default()`. RESOLVED by Story 1.6 -- command now accepts optional `limit`, `offset`, `track`, `car`, `date_start`, `date_end` parameters with defaults.
- [x] [AI-Review][MEDIUM] No `tracing` subscriber is initialized in the Tauri application. RESOLVED: Added `tracing-subscriber` with `env-filter` feature to `src-tauri/Cargo.toml`. Initialized `tracing_subscriber::fmt()` with `EnvFilter` (defaults to `info`, overridable via `RUST_LOG` env var) at the start of `run()`.
- [x] [AI-Review][MEDIUM] Database init failure panics with `.expect()` and no logging. RESOLVED: Replaced both `.expect()` calls with `.map_err()` that logs via `error!()` before propagating the error through `?` operator. Errors now flow through Tauri's setup Result type instead of panicking.
- [ ] [AI-Review][MEDIUM] `update_session` performs 3 SQL round-trips (SELECT + UPDATE + SELECT). The function fetches the existing session, applies the update, then fetches again to return the updated record. A single UPDATE with COALESCE for optional fields plus one SELECT (or RETURNING if supported) would halve the query count. Acceptable for a desktop app but worth noting for when concurrent capture operations need low-latency updates. [crates/storage/src/sqlite/queries/sessions.rs:54-84]
- [ ] [AI-Review][LOW] `SessionSummary` struct is nearly identical to `Session` (same 9 fields, just missing `created_at`/`updated_at` and columns added by later stories). Consider whether a separate struct is justified or if `Session` with `#[serde(skip_serializing_if)]` on heavy fields would reduce type proliferation. Minor -- separate types do provide API clarity.
- [ ] [AI-Review][LOW] Queries use `SELECT *` for `Session` retrieval (`get_session`, post-insert fetch). If the schema evolves and columns don't match the `FromRow` struct, this will cause runtime errors. Using explicit column lists is safer for long-term schema stability. [crates/storage/src/sqlite/queries/sessions.rs:30, crates/storage/src/sqlite/queries/laps.rs:23,33]
