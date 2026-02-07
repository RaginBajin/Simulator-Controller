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
