-- Add import tracking columns to sessions
ALTER TABLE sessions ADD COLUMN import_source TEXT;
ALTER TABLE sessions ADD COLUMN import_format TEXT;
