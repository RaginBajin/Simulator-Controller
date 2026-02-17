//! Thread-safe ring buffer for telemetry samples
//!
//! Provides a circular buffer for capturing telemetry samples with concurrent
//! write (capture thread) and read (flush thread) access.

use crate::sample::TelemetrySample;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// Thread-safe ring buffer for telemetry samples
#[derive(Clone)]
pub struct TelemetryRingBuffer {
    inner: Arc<Mutex<RingBufferInner>>,
}

struct RingBufferInner {
    buffer: VecDeque<TelemetrySample>,
    capacity: usize,
}

impl TelemetryRingBuffer {
    /// Create a new ring buffer with the given capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(RingBufferInner {
                buffer: VecDeque::with_capacity(capacity),
                capacity,
            })),
        }
    }

    /// Push a sample into the ring buffer
    /// If buffer is at capacity, oldest sample is dropped
    pub fn push(&self, sample: TelemetrySample) {
        let mut inner = self.inner.lock().unwrap();

        if inner.buffer.len() >= inner.capacity {
            inner.buffer.pop_front();
        }

        inner.buffer.push_back(sample);
    }

    /// Drain all samples from the buffer and return them
    /// Clears the buffer after draining
    pub fn drain(&self) -> Vec<TelemetrySample> {
        let mut inner = self.inner.lock().unwrap();
        inner.buffer.drain(..).collect()
    }

    /// Get the current number of samples in the buffer
    pub fn len(&self) -> usize {
        let inner = self.inner.lock().unwrap();
        inner.buffer.len()
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the capacity of the buffer
    pub fn capacity(&self) -> usize {
        let inner = self.inner.lock().unwrap();
        inner.capacity
    }

    /// Calculate buffer utilization as a percentage (0.0 to 1.0)
    pub fn utilization(&self) -> f64 {
        let inner = self.inner.lock().unwrap();
        if inner.capacity == 0 {
            return 0.0;
        }
        inner.buffer.len() as f64 / inner.capacity as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_creation() {
        let buffer = TelemetryRingBuffer::new(100);
        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.capacity(), 100);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_ring_buffer_push() {
        let buffer = TelemetryRingBuffer::new(10);

        buffer.push(TelemetrySample::new(1000));
        assert_eq!(buffer.len(), 1);

        buffer.push(TelemetrySample::new(1016));
        assert_eq!(buffer.len(), 2);
    }

    #[test]
    fn test_ring_buffer_drain() {
        let buffer = TelemetryRingBuffer::new(10);

        buffer.push(TelemetrySample::new(1000));
        buffer.push(TelemetrySample::new(1016));
        buffer.push(TelemetrySample::new(1032));

        assert_eq!(buffer.len(), 3);

        let samples = buffer.drain();
        assert_eq!(samples.len(), 3);
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());

        // Verify timestamps
        assert_eq!(samples[0].timestamp_ms, 1000);
        assert_eq!(samples[1].timestamp_ms, 1016);
        assert_eq!(samples[2].timestamp_ms, 1032);
    }

    #[test]
    fn test_ring_buffer_capacity_overflow() {
        let buffer = TelemetryRingBuffer::new(3);

        buffer.push(TelemetrySample::new(1000));
        buffer.push(TelemetrySample::new(1016));
        buffer.push(TelemetrySample::new(1032));
        assert_eq!(buffer.len(), 3);

        // Push 4th sample - should drop oldest
        buffer.push(TelemetrySample::new(1048));
        assert_eq!(buffer.len(), 3); // Still at capacity

        let samples = buffer.drain();
        assert_eq!(samples.len(), 3);

        // First sample (1000) should be gone, we have 1016, 1032, 1048
        assert_eq!(samples[0].timestamp_ms, 1016);
        assert_eq!(samples[1].timestamp_ms, 1032);
        assert_eq!(samples[2].timestamp_ms, 1048);
    }

    #[test]
    fn test_ring_buffer_utilization() {
        let buffer = TelemetryRingBuffer::new(100);

        assert_eq!(buffer.utilization(), 0.0);

        for i in 0..50 {
            buffer.push(TelemetrySample::new(i * 16));
        }

        assert_eq!(buffer.utilization(), 0.5);

        for i in 50..100 {
            buffer.push(TelemetrySample::new(i * 16));
        }

        assert_eq!(buffer.utilization(), 1.0);
    }

    #[test]
    fn test_ring_buffer_concurrent_access() {
        use std::thread;

        let buffer = TelemetryRingBuffer::new(1000);
        let buffer_clone = buffer.clone();

        // Writer thread
        let writer = thread::spawn(move || {
            for i in 0..100 {
                buffer_clone.push(TelemetrySample::new(i));
            }
        });

        writer.join().unwrap();

        assert_eq!(buffer.len(), 100);

        // Reader thread
        let buffer_clone = buffer.clone();
        let reader = thread::spawn(move || buffer_clone.drain());

        let samples = reader.join().unwrap();
        assert_eq!(samples.len(), 100);
        assert_eq!(buffer.len(), 0);
    }
}
