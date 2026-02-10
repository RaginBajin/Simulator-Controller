//! Telemetry ring buffer for live capture with snapshot capability.
//!
//! Story 3.3b: Manual Debrief Trigger - snapshot() method for point-in-time copy.

use std::collections::VecDeque;
use std::sync::Mutex;

/// Telemetry ring buffer that stores samples in-memory during capture.
///
/// Provides:
/// - `push()` to add samples
/// - `drain()` to remove and return all samples (for periodic flush)
/// - `snapshot()` to copy samples without removing them (for manual debrief)
pub struct TelemetryRingBuffer<T> {
    buffer: Mutex<VecDeque<T>>,
    capacity: usize,
}

impl<T: Clone> TelemetryRingBuffer<T> {
    /// Create a new ring buffer with the specified capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Mutex::new(VecDeque::with_capacity(capacity)),
            capacity,
        }
    }

    /// Push a sample into the buffer. If at capacity, oldest sample is removed.
    pub fn push(&self, item: T) {
        let mut buf = self.buffer.lock().unwrap_or_else(|e| e.into_inner());
        if buf.len() >= self.capacity {
            buf.pop_front();
        }
        buf.push_back(item);
    }

    /// Drain all samples from the buffer, returning them.
    /// This removes the samples from the buffer (used for periodic flush).
    pub fn drain(&self) -> Vec<T> {
        let mut buf = self.buffer.lock().unwrap_or_else(|e| e.into_inner());
        buf.drain(..).collect()
    }

    /// Snapshot the current buffer contents without draining.
    /// Returns a point-in-time copy of all samples.
    /// The buffer continues to hold the samples (capture continues).
    pub fn snapshot(&self) -> Vec<T> {
        let buf = self.buffer.lock().unwrap_or_else(|e| e.into_inner());
        buf.iter().cloned().collect()
    }

    /// Return the current number of samples in the buffer.
    pub fn len(&self) -> usize {
        let buf = self.buffer.lock().unwrap_or_else(|e| e.into_inner());
        buf.len()
    }

    /// Check if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_snapshot_returns_copy_without_draining() {
        let rb = TelemetryRingBuffer::new(10);
        rb.push(1);
        rb.push(2);
        rb.push(3);

        let snapshot = rb.snapshot();
        assert_eq!(snapshot, vec![1, 2, 3]);

        // Verify buffer still contains items (not drained)
        assert_eq!(rb.len(), 3);
        let snapshot2 = rb.snapshot();
        assert_eq!(snapshot2, vec![1, 2, 3]);
    }

    #[test]
    fn test_ring_buffer_drain_removes_items() {
        let rb = TelemetryRingBuffer::new(10);
        rb.push(1);
        rb.push(2);
        rb.push(3);

        let drained = rb.drain();
        assert_eq!(drained, vec![1, 2, 3]);

        // Verify buffer is empty after drain
        assert_eq!(rb.len(), 0);
        assert!(rb.is_empty());
    }

    #[test]
    fn test_ring_buffer_capacity_enforced() {
        let rb = TelemetryRingBuffer::new(3);
        rb.push(1);
        rb.push(2);
        rb.push(3);
        rb.push(4); // Should evict 1

        let snapshot = rb.snapshot();
        assert_eq!(snapshot, vec![2, 3, 4]);
        assert_eq!(rb.len(), 3);
    }

    #[test]
    fn test_ring_buffer_snapshot_empty() {
        let rb: TelemetryRingBuffer<i32> = TelemetryRingBuffer::new(10);
        let snapshot = rb.snapshot();
        assert_eq!(snapshot, Vec::<i32>::new());
        assert!(rb.is_empty());
    }

    #[test]
    fn test_ring_buffer_push_and_snapshot() {
        let rb = TelemetryRingBuffer::new(5);
        rb.push("a");
        rb.push("b");

        let snapshot = rb.snapshot();
        assert_eq!(snapshot, vec!["a", "b"]);

        rb.push("c");
        let snapshot2 = rb.snapshot();
        assert_eq!(snapshot2, vec!["a", "b", "c"]);
    }
}
