-- safety:allow-drop
-- Add raw_session_type column for diagnostic purposes
ALTER TABLE sessions ADD COLUMN raw_session_type TEXT;

-- Note: SQLite does not support ALTER TABLE to modify CHECK constraints.
-- The existing CHECK constraint allows: 'practice', 'qualifying', 'race', 'warmup'
-- To add 'testing', we need to recreate the table with the new constraint.

-- Step 1: Create a new sessions table with the updated CHECK constraint
CREATE TABLE sessions_new (
    id TEXT PRIMARY KEY NOT NULL,
    track_name TEXT NOT NULL,
    car_name TEXT NOT NULL,
    session_type TEXT NOT NULL CHECK (session_type IN ('practice', 'qualifying', 'race', 'warmup', 'testing')),
    started_at TEXT NOT NULL,
    ended_at TEXT,
    lap_count INTEGER NOT NULL DEFAULT 0,
    best_lap_time_ms INTEGER,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'completed', 'partial', 'deleted')),
    telemetry_checksum TEXT,
    telemetry_path TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    deleted_at TEXT,
    previous_status TEXT,
    integrity_status TEXT DEFAULT 'not_validated' CHECK (integrity_status IN ('not_validated', 'valid', 'checksum_failed', 'range_violation', 'distance_anomaly')),
    integrity_details TEXT,
    integrity_validated_at TEXT,
    import_source TEXT,
    import_format TEXT,
    raw_session_type TEXT
);

-- Step 2: Copy all data from the old table to the new table
INSERT INTO sessions_new SELECT
    id,
    track_name,
    car_name,
    session_type,
    started_at,
    ended_at,
    lap_count,
    best_lap_time_ms,
    status,
    telemetry_checksum,
    telemetry_path,
    created_at,
    updated_at,
    deleted_at,
    previous_status,
    integrity_status,
    integrity_details,
    integrity_validated_at,
    import_source,
    import_format,
    NULL as raw_session_type
FROM sessions;

-- Step 3: Drop the old table
DROP TABLE sessions;

-- Step 4: Rename the new table to the original name
ALTER TABLE sessions_new RENAME TO sessions;

-- Step 5: Recreate indexes
CREATE INDEX IF NOT EXISTS idx_sessions_started_at ON sessions(started_at);
CREATE INDEX IF NOT EXISTS idx_sessions_status ON sessions(status);
CREATE INDEX IF NOT EXISTS idx_sessions_track_name ON sessions(track_name);
CREATE INDEX IF NOT EXISTS idx_sessions_car_name ON sessions(car_name);
CREATE INDEX IF NOT EXISTS idx_sessions_session_type ON sessions(session_type);
CREATE INDEX IF NOT EXISTS idx_sessions_deleted_at ON sessions(deleted_at);
CREATE INDEX IF NOT EXISTS idx_sessions_integrity_status ON sessions(integrity_status);
