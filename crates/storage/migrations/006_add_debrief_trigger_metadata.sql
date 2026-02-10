-- Add trigger_type and data_range metadata to ai_debriefs for manual debrief support
ALTER TABLE ai_debriefs ADD COLUMN trigger_type TEXT;
ALTER TABLE ai_debriefs ADD COLUMN data_range_from_ms INTEGER;
ALTER TABLE ai_debriefs ADD COLUMN data_range_to_ms INTEGER;
