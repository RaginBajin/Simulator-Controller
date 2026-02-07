-- Add integrity tracking columns to sessions
ALTER TABLE sessions ADD COLUMN integrity_status TEXT DEFAULT 'not_validated';
ALTER TABLE sessions ADD COLUMN integrity_details TEXT;
ALTER TABLE sessions ADD COLUMN integrity_validated_at TEXT;

-- Index for querying unvalidated sessions on startup
CREATE INDEX IF NOT EXISTS idx_sessions_integrity_status ON sessions(integrity_status);
