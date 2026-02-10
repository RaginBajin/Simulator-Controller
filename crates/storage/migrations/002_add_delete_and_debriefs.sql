-- Add soft delete columns to sessions
ALTER TABLE sessions ADD COLUMN deleted_at TEXT;
ALTER TABLE sessions ADD COLUMN previous_status TEXT;

-- AI debrief storage
CREATE TABLE IF NOT EXISTS ai_debriefs (
    id TEXT PRIMARY KEY NOT NULL,
    session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    coaching_text TEXT,
    insights_json TEXT,
    recommendations_json TEXT,
    provider_name TEXT,
    model_name TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_sessions_deleted_at ON sessions(deleted_at);
CREATE INDEX IF NOT EXISTS idx_ai_debriefs_session_id ON ai_debriefs(session_id);
