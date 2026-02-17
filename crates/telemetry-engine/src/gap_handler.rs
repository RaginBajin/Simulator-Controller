//! Gap detection and management for telemetry capture

use crate::capture_error::{GapMarker, GapReason};
use tracing::{info, warn};

/// Configuration for gap detection
#[derive(Debug, Clone)]
pub struct GapDetectionConfig {
    /// Threshold in milliseconds for detecting a gap (default: 100ms)
    pub threshold_ms: u64,
}

impl Default for GapDetectionConfig {
    fn default() -> Self {
        Self { threshold_ms: 100 }
    }
}

/// Manages gap detection and tracking for a telemetry capture session
#[derive(Debug)]
pub struct GapHandler {
    config: GapDetectionConfig,
    gaps: Vec<GapMarker>,
    last_sample_time: Option<i64>,
}

impl GapHandler {
    /// Creates a new gap handler with default configuration
    pub fn new() -> Self {
        Self::with_config(GapDetectionConfig::default())
    }

    /// Creates a new gap handler with custom configuration
    pub fn with_config(config: GapDetectionConfig) -> Self {
        Self {
            config,
            gaps: Vec::new(),
            last_sample_time: None,
        }
    }

    /// Checks for a gap between the last sample and the current sample time.
    /// Returns Some(GapMarker) if a gap is detected, None otherwise.
    ///
    /// # Arguments
    /// * `current_time` - Current sample timestamp in milliseconds
    /// * `lap_position` - Optional track position (meters from start/finish line)
    pub fn check_gap(&mut self, current_time: i64, lap_position: Option<f64>) -> Option<GapMarker> {
        if let Some(last_time) = self.last_sample_time {
            let delta = current_time - last_time;

            if delta > self.config.threshold_ms as i64 {
                let reason = self.classify_gap_reason(delta as u64);
                let gap = GapMarker::new(last_time, current_time, reason, lap_position);

                warn!(
                    gap_duration_ms = gap.duration_ms,
                    reason = ?gap.reason,
                    lap_position = ?gap.lap_position,
                    "Telemetry gap detected"
                );

                self.gaps.push(gap.clone());
                self.last_sample_time = Some(current_time);
                return Some(gap);
            }
        }

        self.last_sample_time = Some(current_time);
        None
    }

    /// Manually records a gap (e.g., from a known disconnection)
    pub fn record_gap(&mut self, gap: GapMarker) {
        warn!(
            gap_duration_ms = gap.duration_ms,
            reason = ?gap.reason,
            lap_position = ?gap.lap_position,
            "Gap marker recorded"
        );
        self.gaps.push(gap);
    }

    /// Returns all gaps detected so far
    pub fn gaps(&self) -> &[GapMarker] {
        &self.gaps
    }

    /// Returns the number of gaps detected
    pub fn gap_count(&self) -> usize {
        self.gaps.len()
    }

    /// Returns the total duration of all gaps in milliseconds
    pub fn total_gap_duration_ms(&self) -> u64 {
        self.gaps.iter().map(|g| g.duration_ms).sum()
    }

    /// Returns the number of significant gaps (>500ms per NFR7)
    pub fn significant_gap_count(&self) -> usize {
        self.gaps.iter().filter(|g| g.is_significant()).count()
    }

    /// Resets the gap handler for a new session
    pub fn reset(&mut self) {
        self.gaps.clear();
        self.last_sample_time = None;
        info!("Gap handler reset for new session");
    }

    /// Classifies the gap reason based on duration heuristics
    fn classify_gap_reason(&self, duration_ms: u64) -> GapReason {
        match duration_ms {
            100..=500 => GapReason::Stall,       // Brief stall
            501..=5000 => GapReason::Disconnect, // Connection issue
            _ => GapReason::Unknown,             // Very long gap or unknown cause
        }
    }
}

impl Default for GapHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gap_handler_creation() {
        let handler = GapHandler::new();
        assert_eq!(handler.gap_count(), 0);
        assert_eq!(handler.total_gap_duration_ms(), 0);
    }

    #[test]
    fn test_gap_handler_with_custom_config() {
        let config = GapDetectionConfig { threshold_ms: 200 };
        let handler = GapHandler::with_config(config);
        assert_eq!(handler.config.threshold_ms, 200);
    }

    #[test]
    fn test_no_gap_on_first_sample() {
        let mut handler = GapHandler::new();
        let gap = handler.check_gap(1000, None);
        assert!(gap.is_none());
        assert_eq!(handler.gap_count(), 0);
    }

    #[test]
    fn test_no_gap_within_threshold() {
        let mut handler = GapHandler::new();
        handler.check_gap(1000, None);
        let gap = handler.check_gap(1050, None); // 50ms delta < 100ms threshold
        assert!(gap.is_none());
        assert_eq!(handler.gap_count(), 0);
    }

    #[test]
    fn test_gap_detected_exceeds_threshold() {
        let mut handler = GapHandler::new();
        handler.check_gap(1000, None);
        let gap = handler.check_gap(1250, Some(1234.5)); // 250ms delta > 100ms threshold

        assert!(gap.is_some());
        let gap = gap.unwrap();
        assert_eq!(gap.start_time, 1000);
        assert_eq!(gap.end_time, 1250);
        assert_eq!(gap.duration_ms, 250);
        assert_eq!(gap.lap_position, Some(1234.5));
        assert_eq!(handler.gap_count(), 1);
    }

    #[test]
    fn test_gap_classification_stall() {
        let mut handler = GapHandler::new();
        handler.check_gap(1000, None);
        let gap = handler.check_gap(1250, None); // 250ms -> Stall

        assert_eq!(gap.unwrap().reason, GapReason::Stall);
    }

    #[test]
    fn test_gap_classification_disconnect() {
        let mut handler = GapHandler::new();
        handler.check_gap(1000, None);
        let gap = handler.check_gap(2000, None); // 1000ms -> Disconnect

        assert_eq!(gap.unwrap().reason, GapReason::Disconnect);
    }

    #[test]
    fn test_gap_classification_unknown() {
        let mut handler = GapHandler::new();
        handler.check_gap(1000, None);
        let gap = handler.check_gap(7000, None); // 6000ms -> Unknown

        assert_eq!(gap.unwrap().reason, GapReason::Unknown);
    }

    #[test]
    fn test_multiple_gaps_tracked() {
        let mut handler = GapHandler::new();
        handler.check_gap(1000, None);
        handler.check_gap(1300, None); // Gap 1: 300ms
        handler.check_gap(1350, None); // No gap
        handler.check_gap(1600, None); // Gap 2: 250ms

        assert_eq!(handler.gap_count(), 2);
        let gaps = handler.gaps();
        assert_eq!(gaps[0].duration_ms, 300);
        assert_eq!(gaps[1].duration_ms, 250);
    }

    #[test]
    fn test_total_gap_duration() {
        let mut handler = GapHandler::new();
        handler.check_gap(1000, None);
        handler.check_gap(1300, None); // 300ms
        handler.check_gap(1350, None);
        handler.check_gap(1600, None); // 250ms

        assert_eq!(handler.total_gap_duration_ms(), 550);
    }

    #[test]
    fn test_significant_gap_count() {
        let mut handler = GapHandler::new();
        handler.check_gap(1000, None);
        handler.check_gap(1400, None); // 400ms - NOT significant (<500ms)
        handler.check_gap(1450, None);
        handler.check_gap(2100, None); // 650ms - significant (>500ms)

        assert_eq!(handler.gap_count(), 2);
        assert_eq!(handler.significant_gap_count(), 1);
    }

    #[test]
    fn test_manual_gap_recording() {
        let mut handler = GapHandler::new();
        let manual_gap = GapMarker::new(5000, 8000, GapReason::Disconnect, Some(999.9));
        handler.record_gap(manual_gap);

        assert_eq!(handler.gap_count(), 1);
        assert_eq!(handler.gaps()[0].duration_ms, 3000);
        assert_eq!(handler.gaps()[0].reason, GapReason::Disconnect);
    }

    #[test]
    fn test_reset_clears_state() {
        let mut handler = GapHandler::new();
        handler.check_gap(1000, None);
        handler.check_gap(1300, None);
        assert_eq!(handler.gap_count(), 1);

        handler.reset();
        assert_eq!(handler.gap_count(), 0);
        assert_eq!(handler.total_gap_duration_ms(), 0);
        assert!(handler.last_sample_time.is_none());
    }

    #[test]
    fn test_custom_threshold() {
        let config = GapDetectionConfig { threshold_ms: 200 };
        let mut handler = GapHandler::with_config(config);

        handler.check_gap(1000, None);
        let gap1 = handler.check_gap(1150, None); // 150ms < 200ms threshold
        assert!(gap1.is_none());

        let gap2 = handler.check_gap(1400, None); // 250ms > 200ms threshold
        assert!(gap2.is_some());
    }
}
