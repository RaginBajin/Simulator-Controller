-- Add telemetry file tracking columns to sessions
ALTER TABLE sessions ADD COLUMN telemetry_checksum TEXT;
ALTER TABLE sessions ADD COLUMN telemetry_path TEXT;
