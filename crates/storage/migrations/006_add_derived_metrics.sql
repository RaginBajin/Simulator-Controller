-- Add derived metrics storage tables
-- Story 3.6 Task 7: SQLite storage for derived metrics

-- Derived metrics summary table (stores JSON-serialized metric results)
CREATE TABLE IF NOT EXISTS derived_metrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    metric_type TEXT NOT NULL,  -- 'brake_count', 'trail_braking', 'tire_degradation', 'corner_segmentation'
    data TEXT NOT NULL,          -- JSON-serialized metric data
    computed_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(session_id, metric_type)  -- Idempotency: one entry per metric type per session
);

CREATE INDEX IF NOT EXISTS idx_derived_metrics_session ON derived_metrics(session_id);
CREATE INDEX IF NOT EXISTS idx_derived_metrics_type ON derived_metrics(metric_type);

-- Corner zones table (stores corner boundaries identified from best lap)
CREATE TABLE IF NOT EXISTS corner_zones (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    corner_id INTEGER NOT NULL,            -- Sequential corner ID (1, 2, 3...)
    name TEXT NOT NULL,                    -- Corner name (T1, T2, T3...)
    start_distance REAL NOT NULL,          -- Lap distance where corner starts (brake onset) meters
    end_distance REAL NOT NULL,            -- Lap distance where corner ends (full throttle) meters
    brake_onset_distance REAL,             -- Lap distance where braking begins meters (nullable)
    apex_distance REAL,                    -- Lap distance at apex (min speed point) meters (nullable)
    UNIQUE(session_id, corner_id)          -- One entry per corner per session
);

CREATE INDEX IF NOT EXISTS idx_corner_zones_session ON corner_zones(session_id);
CREATE INDEX IF NOT EXISTS idx_corner_zones_corner_id ON corner_zones(session_id, corner_id);
