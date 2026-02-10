-- Migration 006: Add gap markers table and session gap tracking columns
-- Story 3.4: Capture Error Handling & Recovery

-- Create gap_markers table to store telemetry gaps for each session
CREATE TABLE IF NOT EXISTS gap_markers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL,
    start_time INTEGER NOT NULL,
    end_time INTEGER NOT NULL,
    duration_ms INTEGER NOT NULL,
    reason TEXT NOT NULL CHECK(reason IN ('disconnect', 'stall', 'reset', 'unknown')),
    lap_position REAL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
);

-- Index for efficient gap marker queries by session
CREATE INDEX IF NOT EXISTS idx_gap_markers_session_id ON gap_markers(session_id);

-- Index for finding significant gaps (>500ms per NFR7)
CREATE INDEX IF NOT EXISTS idx_gap_markers_duration ON gap_markers(duration_ms);

-- Add gap summary columns to sessions table
ALTER TABLE sessions ADD COLUMN gap_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE sessions ADD COLUMN total_gap_duration_ms INTEGER NOT NULL DEFAULT 0;

-- Add disconnected_at timestamp for partial sessions
ALTER TABLE sessions ADD COLUMN disconnected_at TEXT;
