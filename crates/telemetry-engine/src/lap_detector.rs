//! Lap boundary detection for live telemetry streaming.
//!
//! Detects lap boundaries using iRacing `LapDistPct` crossing logic
//! (value drops from >0.9 to <0.1) with cross-reference to iRacing's `Lap` variable.

use storage::types::NewLap;

/// Lap detection state tracker.
#[derive(Debug)]
pub struct LapDetector {
    /// Previous lap distance percentage (0.0 to 1.0).
    prev_lap_dist_pct: Option<f64>,
    /// Previous lap number from iRacing.
    prev_lap_number: Option<i32>,
    /// Current lap number being tracked.
    current_lap: i32,
}

impl Default for LapDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl LapDetector {
    /// Creates a new lap detector.
    pub fn new() -> Self {
        Self {
            prev_lap_dist_pct: None,
            prev_lap_number: None,
            current_lap: 0,
        }
    }

    /// Processes a new telemetry sample and detects lap boundaries.
    ///
    /// Returns `Some(lap_summary)` if a lap boundary was crossed, `None` otherwise.
    ///
    /// # Arguments
    /// * `lap_dist_pct` - Current lap distance percentage (0.0 to 1.0)
    /// * `lap_number` - Current lap number from iRacing
    /// * `_session_time` - Session time in seconds (reserved for future use)
    /// * `lap_last_lap_time` - Last completed lap time from iRacing (seconds, -1.0 if invalid)
    /// * `session_id` - Session ID for the lap summary
    pub fn process_sample(
        &mut self,
        lap_dist_pct: f64,
        lap_number: i32,
        _session_time: f64,
        lap_last_lap_time: f64,
        session_id: &str,
    ) -> Option<NewLap> {
        // Initialize if first sample
        if self.prev_lap_dist_pct.is_none() {
            self.prev_lap_dist_pct = Some(lap_dist_pct);
            self.prev_lap_number = Some(lap_number);
            self.current_lap = lap_number;
            return None;
        }

        let prev_dist = self.prev_lap_dist_pct.unwrap();
        let prev_lap = self.prev_lap_number.unwrap();

        // Detect lap boundary: distance crossing from >0.9 to <0.1
        let crossed_start_line = prev_dist > 0.9 && lap_dist_pct < 0.1;

        // Cross-reference with iRacing lap counter
        let lap_counter_incremented = lap_number > prev_lap;

        // Update state
        self.prev_lap_dist_pct = Some(lap_dist_pct);
        self.prev_lap_number = Some(lap_number);

        // If boundary detected, create lap summary
        if crossed_start_line || lap_counter_incremented {
            let completed_lap_number = self.current_lap;
            self.current_lap = lap_number;

            // Use iRacing's official lap time if available (>0), otherwise compute from session time
            let lap_time_ms = if lap_last_lap_time > 0.0 {
                (lap_last_lap_time * 1000.0) as i64
            } else {
                // Fallback: cannot compute without previous lap start time
                // In production, we'd track lap start times, but for now mark as -1
                -1
            };

            // Lap is valid if iRacing provided a positive lap time
            let is_valid = lap_last_lap_time > 0.0;

            let lap_summary = NewLap {
                session_id: session_id.to_string(),
                lap_number: completed_lap_number,
                lap_time_ms,
                is_valid,
                completion_status: if is_valid {
                    "complete".to_string()
                } else {
                    "incomplete_unknown".to_string()
                },
            };

            tracing::info!(
                lap_number = completed_lap_number,
                lap_time_ms = lap_time_ms,
                is_valid = is_valid,
                "Lap boundary detected"
            );

            return Some(lap_summary);
        }

        None
    }

    /// Detects incomplete laps based on sudden position jumps.
    ///
    /// Returns completion status if a large jump (>0.3) in lap distance is detected,
    /// indicating a spin, reset, or other interruption.
    ///
    /// Detects both forward jumps (teleport/reset) and backward jumps (track reset),
    /// excluding normal lap boundary crossings (>0.9 to <0.1).
    pub fn detect_incomplete_lap(&self, lap_dist_pct: f64) -> Option<String> {
        if let Some(prev_dist) = self.prev_lap_dist_pct {
            // Detect forward jump >0.3 (not crossing start/finish line)
            let forward_jump = lap_dist_pct > prev_dist + 0.3;

            // Detect backward jump >0.3 (excluding normal lap crossing)
            // Normal crossing: prev > 0.9 and curr < 0.1 (diff ~0.9-1.0)
            // Abnormal: prev < 0.9 and backward jump > 0.3
            let backward_jump = lap_dist_pct < prev_dist - 0.3 && prev_dist < 0.9;

            if forward_jump || backward_jump {
                return Some("incomplete_reset".to_string());
            }
        }

        None
    }

    /// Resets the lap detector state (for new session).
    pub fn reset(&mut self) {
        self.prev_lap_dist_pct = None;
        self.prev_lap_number = None;
        self.current_lap = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SESSION_ID: &str = "test-session-123";

    #[test]
    fn test_initial_state_no_boundary() {
        let mut detector = LapDetector::new();

        // First sample - no boundary
        let result = detector.process_sample(0.5, 1, 10.0, -1.0, TEST_SESSION_ID);
        assert!(result.is_none());
    }

    #[test]
    fn test_lap_boundary_via_distance_crossing() {
        let mut detector = LapDetector::new();

        // Initialize
        detector.process_sample(0.95, 1, 10.0, -1.0, TEST_SESSION_ID);

        // Cross start/finish line
        let result = detector.process_sample(0.05, 2, 85.0, 75.5, TEST_SESSION_ID);
        assert!(result.is_some());

        let lap = result.unwrap();
        assert_eq!(lap.lap_number, 1);
        assert_eq!(lap.lap_time_ms, 75500); // 75.5s * 1000
        assert!(lap.is_valid);
        assert_eq!(lap.completion_status, "complete");
    }

    #[test]
    fn test_lap_boundary_via_lap_counter() {
        let mut detector = LapDetector::new();

        // Initialize
        detector.process_sample(0.5, 1, 10.0, -1.0, TEST_SESSION_ID);

        // Lap counter increments (even without distance crossing)
        let result = detector.process_sample(0.6, 2, 85.0, 75.0, TEST_SESSION_ID);
        assert!(result.is_some());

        let lap = result.unwrap();
        assert_eq!(lap.lap_number, 1);
        assert_eq!(lap.lap_time_ms, 75000);
        assert!(lap.is_valid);
    }

    #[test]
    fn test_invalid_lap_no_official_time() {
        let mut detector = LapDetector::new();

        // Initialize
        detector.process_sample(0.95, 1, 10.0, -1.0, TEST_SESSION_ID);

        // Cross boundary but no official time (-1.0)
        let result = detector.process_sample(0.05, 2, 85.0, -1.0, TEST_SESSION_ID);
        assert!(result.is_some());

        let lap = result.unwrap();
        assert_eq!(lap.lap_number, 1);
        assert_eq!(lap.lap_time_ms, -1);
        assert!(!lap.is_valid);
        assert_eq!(lap.completion_status, "incomplete_unknown");
    }

    #[test]
    fn test_multiple_laps() {
        let mut detector = LapDetector::new();

        // Lap 1
        detector.process_sample(0.95, 1, 10.0, -1.0, TEST_SESSION_ID);
        let lap1 = detector.process_sample(0.05, 2, 85.0, 75.0, TEST_SESSION_ID);
        assert!(lap1.is_some());
        assert_eq!(lap1.unwrap().lap_number, 1);

        // Continue lap 2
        detector.process_sample(0.5, 2, 120.0, -1.0, TEST_SESSION_ID);
        detector.process_sample(0.95, 2, 155.0, -1.0, TEST_SESSION_ID);

        // Complete lap 2
        let lap2 = detector.process_sample(0.05, 3, 160.0, 70.0, TEST_SESSION_ID);
        assert!(lap2.is_some());
        assert_eq!(lap2.unwrap().lap_number, 2);
    }

    #[test]
    fn test_detect_incomplete_lap_forward_jump() {
        let mut detector = LapDetector::new();

        // Initialize at 0.3
        detector.process_sample(0.3, 1, 10.0, -1.0, TEST_SESSION_ID);

        // Large forward jump (>0.3) - indicates reset/teleport
        let incomplete = detector.detect_incomplete_lap(0.7);
        assert!(incomplete.is_some());
        assert_eq!(incomplete.unwrap(), "incomplete_reset");
    }

    #[test]
    fn test_detect_incomplete_lap_no_jump() {
        let mut detector = LapDetector::new();

        // Initialize
        detector.process_sample(0.3, 1, 10.0, -1.0, TEST_SESSION_ID);

        // Normal progression (no jump)
        let incomplete = detector.detect_incomplete_lap(0.4);
        assert!(incomplete.is_none());
    }

    #[test]
    fn test_detect_incomplete_lap_backward_jump() {
        let mut detector = LapDetector::new();

        // Initialize at 0.8 (mid-lap)
        detector.process_sample(0.8, 1, 10.0, -1.0, TEST_SESSION_ID);

        // Large backward jump (>0.3) - indicates track reset
        let incomplete = detector.detect_incomplete_lap(0.2);
        assert!(incomplete.is_some());
        assert_eq!(incomplete.unwrap(), "incomplete_reset");
    }

    #[test]
    fn test_detect_incomplete_lap_normal_crossing_not_detected() {
        let mut detector = LapDetector::new();

        // Initialize at 0.95 (near finish line)
        detector.process_sample(0.95, 1, 10.0, -1.0, TEST_SESSION_ID);

        // Normal lap crossing (backward from 0.95 to 0.05) - should NOT be detected as incomplete
        let incomplete = detector.detect_incomplete_lap(0.05);
        assert!(incomplete.is_none()); // This is a normal lap boundary, not an incomplete lap
    }

    #[test]
    fn test_reset_clears_state() {
        let mut detector = LapDetector::new();

        // Initialize with data
        detector.process_sample(0.5, 1, 10.0, -1.0, TEST_SESSION_ID);

        // Reset
        detector.reset();

        // Should behave like initial state
        assert!(detector.prev_lap_dist_pct.is_none());
        assert!(detector.prev_lap_number.is_none());
        assert_eq!(detector.current_lap, 0);
    }

    #[test]
    fn test_lap_time_precision() {
        let mut detector = LapDetector::new();

        detector.process_sample(0.95, 1, 10.0, -1.0, TEST_SESSION_ID);

        // Test precision: 75.123 seconds -> 75123 ms
        let result = detector.process_sample(0.05, 2, 85.123, 75.123, TEST_SESSION_ID);
        assert!(result.is_some());

        let lap = result.unwrap();
        assert_eq!(lap.lap_time_ms, 75123);
    }
}
