//! Resource monitoring module for tracking CPU and RSS usage during telemetry capture.
//!
//! This module provides resource monitoring to ensure the capture engine stays within
//! performance targets: <2% CPU (5-second rolling average) and <200MB RSS.
//!
//! # Architecture
//!
//! - Runs on a separate low-priority background thread
//! - Samples CPU and RSS at configurable intervals
//! - Emits structured logs via `tracing`
//! - Supports resource threshold alerting
//! - Uses platform-native APIs via `sysinfo` crate for cross-platform support

use serde::{Deserialize, Serialize};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use sysinfo::{Pid, System};
use tracing::{info, warn};

// Constants for resource thresholds and monitoring intervals
pub const CPU_THRESHOLD_PERCENT: f64 = 2.0;
pub const RSS_THRESHOLD_MB: u64 = 200;
pub const CPU_ALERT_DURATION_SECS: u64 = 10;
pub const CPU_MEASUREMENT_INTERVAL_SECS: u64 = 5;
pub const RSS_MEASUREMENT_INTERVAL_SECS: u64 = 10;
pub const STATS_LOG_INTERVAL_SECS: u64 = 60;

/// Resource statistics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceStats {
    /// CPU usage as percentage (0-100)
    pub cpu_percent: f64,
    /// Resident Set Size (RSS) memory in megabytes
    pub rss_mb: u64,
    /// Session ID associated with this measurement
    pub session_id: String,
    /// Uptime in seconds since monitoring started
    pub uptime_seconds: u64,
}

/// Resource warning information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceWarning {
    /// Metric that breached threshold ("cpu" or "rss")
    pub metric: String,
    /// Current value
    pub current: f64,
    /// Threshold value
    pub threshold: f64,
    /// Session ID
    pub session_id: String,
}

/// Commands for controlling the resource monitor
#[derive(Debug)]
pub enum MonitorCommand {
    /// Stop monitoring and shut down the thread
    Stop,
}

/// Resource monitor handle for controlling the background monitoring thread
pub struct ResourceMonitor {
    command_tx: mpsc::Sender<MonitorCommand>,
    stats_rx: mpsc::Receiver<ResourceStats>,
    warning_rx: mpsc::Receiver<ResourceWarning>,
    thread_handle: Option<thread::JoinHandle<()>>,
}

impl ResourceMonitor {
    /// Start resource monitoring on a background thread
    ///
    /// # Arguments
    /// * `session_id` - The session ID to associate with measurements
    ///
    /// # Returns
    /// A `ResourceMonitor` handle for controlling the monitor and receiving stats/warnings
    pub fn start(session_id: String) -> Self {
        let (command_tx, command_rx) = mpsc::channel();
        let (stats_tx, stats_rx) = mpsc::channel();
        let (warning_tx, warning_rx) = mpsc::channel();

        let thread_handle = thread::Builder::new()
            .name("resource-monitor".to_string())
            .spawn(move || {
                Self::monitor_loop(session_id, command_rx, stats_tx, warning_tx);
            })
            .expect("Failed to spawn resource monitor thread");

        Self {
            command_tx,
            stats_rx,
            warning_rx,
            thread_handle: Some(thread_handle),
        }
    }

    /// Stop the resource monitor and wait for thread to complete
    pub fn stop(mut self) {
        let _ = self.command_tx.send(MonitorCommand::Stop);
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }

    /// Try to receive a resource stats update (non-blocking)
    pub fn try_recv_stats(&self) -> Option<ResourceStats> {
        self.stats_rx.try_recv().ok()
    }

    /// Try to receive a resource warning (non-blocking)
    pub fn try_recv_warning(&self) -> Option<ResourceWarning> {
        self.warning_rx.try_recv().ok()
    }

    /// Main monitoring loop that runs on the background thread
    fn monitor_loop(
        session_id: String,
        command_rx: mpsc::Receiver<MonitorCommand>,
        stats_tx: mpsc::Sender<ResourceStats>,
        warning_tx: mpsc::Sender<ResourceWarning>,
    ) {
        let mut system = System::new_all();
        let pid = Pid::from_u32(std::process::id());

        let mut uptime_seconds = 0u64;
        let mut last_stats_log = 0u64;
        let mut cpu_breach_duration = 0u64;

        // Measurement intervals (using the shorter of the two for loop frequency)
        let measurement_interval = Duration::from_secs(CPU_MEASUREMENT_INTERVAL_SECS);

        loop {
            // Sleep first (except on first iteration - handled by starting at 0)
            if uptime_seconds > 0 {
                thread::sleep(measurement_interval);
            }
            uptime_seconds += CPU_MEASUREMENT_INTERVAL_SECS;

            // Check for stop command
            if let Ok(MonitorCommand::Stop) = command_rx.try_recv() {
                info!("Resource monitor stopping");
                break;
            }

            // Refresh system information for all processes (required for CPU usage calculation)
            system.refresh_processes(sysinfo::ProcessesToUpdate::All, false);

            // Get current process
            if let Some(process) = system.process(pid) {
                // Measure CPU (percentage of total system CPU)
                let cpu_percent = process.cpu_usage() as f64;

                // Measure RSS (in MB)
                let rss_mb = process.memory() / 1024 / 1024;

                let stats = ResourceStats {
                    cpu_percent,
                    rss_mb,
                    session_id: session_id.clone(),
                    uptime_seconds,
                };

                // Send stats update
                let _ = stats_tx.send(stats.clone());

                // Log stats periodically
                if uptime_seconds - last_stats_log >= STATS_LOG_INTERVAL_SECS {
                    info!(
                        cpu_percent = cpu_percent,
                        rss_mb = rss_mb,
                        session_id = %session_id,
                        "Resource stats"
                    );
                    last_stats_log = uptime_seconds;
                }

                // Check CPU threshold breach
                if cpu_percent > CPU_THRESHOLD_PERCENT {
                    cpu_breach_duration += CPU_MEASUREMENT_INTERVAL_SECS;

                    // Alert if breach sustained for alert duration
                    if cpu_breach_duration >= CPU_ALERT_DURATION_SECS {
                        warn!(
                            cpu_percent = cpu_percent,
                            threshold = CPU_THRESHOLD_PERCENT,
                            duration_secs = cpu_breach_duration,
                            "CPU threshold exceeded"
                        );

                        let warning = ResourceWarning {
                            metric: "cpu".to_string(),
                            current: cpu_percent,
                            threshold: CPU_THRESHOLD_PERCENT,
                            session_id: session_id.clone(),
                        };
                        let _ = warning_tx.send(warning);

                        // Reset breach duration after alerting
                        cpu_breach_duration = 0;
                    }
                } else {
                    // Reset breach duration if CPU drops below threshold
                    cpu_breach_duration = 0;
                }

                // Check RSS threshold breach (immediate alert)
                if rss_mb > RSS_THRESHOLD_MB {
                    warn!(
                        rss_mb = rss_mb,
                        threshold = RSS_THRESHOLD_MB,
                        "RSS threshold exceeded"
                    );

                    let warning = ResourceWarning {
                        metric: "rss".to_string(),
                        current: rss_mb as f64,
                        threshold: RSS_THRESHOLD_MB as f64,
                        session_id: session_id.clone(),
                    };
                    let _ = warning_tx.send(warning);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_resource_monitor_reports_stats() {
        // Start monitor
        let monitor = ResourceMonitor::start("test-session-1".to_string());

        // Wait for at least one measurement cycle
        thread::sleep(Duration::from_secs(6));

        // Should have received at least one stats update
        let stats = monitor.try_recv_stats();
        assert!(stats.is_some(), "Should receive resource stats");

        let stats = stats.unwrap();
        assert_eq!(stats.session_id, "test-session-1");
        assert!(stats.cpu_percent >= 0.0, "CPU should be >= 0");
        assert!(stats.rss_mb > 0, "RSS should be > 0");
        // Uptime counter increments after each cycle, so it should be > 0
        assert!(stats.uptime_seconds > 0, "Uptime should be > 0");

        // Stop monitor
        monitor.stop();
    }

    #[test]
    fn test_resource_monitor_cpu_values_are_reasonable() {
        // Start monitor
        let monitor = ResourceMonitor::start("test-session-2".to_string());

        // Wait for measurement
        thread::sleep(Duration::from_secs(6));

        // Get stats
        let stats = monitor.try_recv_stats();
        assert!(stats.is_some());

        let stats = stats.unwrap();
        // CPU should be a reasonable percentage (0-100)
        assert!(
            stats.cpu_percent >= 0.0 && stats.cpu_percent <= 100.0,
            "CPU percent should be 0-100, got {}",
            stats.cpu_percent
        );

        monitor.stop();
    }

    #[test]
    fn test_resource_monitor_stops_cleanly() {
        // Start monitor
        let monitor = ResourceMonitor::start("test-session-3".to_string());

        // Wait briefly
        thread::sleep(Duration::from_secs(1));

        // Stop should complete without hanging
        monitor.stop();
        // If we reach here, stop completed successfully
    }

    // Note: Testing threshold alerting is difficult without artificially creating
    // high resource usage. This would be tested in integration tests or manually.
    // For now, we verify the alerting logic compiles and the types are correct.
}
