//! Timing utilities.

use std::time::{Duration, Instant};

/// A stopwatch for measuring elapsed time.
#[derive(Debug)]
pub struct Stopwatch {
    start: Instant,
    splits: Vec<(String, Duration)>,
}

impl Default for Stopwatch {
    fn default() -> Self {
        Self::new()
    }
}

impl Stopwatch {
    /// Create and start a new stopwatch.
    #[must_use]
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            splits: Vec::new(),
        }
    }

    /// Get elapsed time since start.
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Get elapsed time in milliseconds.
    #[must_use]
    pub fn elapsed_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }

    /// Record a split time with a label.
    pub fn split(&mut self, label: impl Into<String>) {
        self.splits.push((label.into(), self.elapsed()));
    }

    /// Get all recorded splits.
    #[must_use]
    pub fn splits(&self) -> &[(String, Duration)] {
        &self.splits
    }

    /// Reset the stopwatch.
    pub fn reset(&mut self) {
        self.start = Instant::now();
        self.splits.clear();
    }
}

/// Format a duration for human display.
#[must_use]
pub fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    let ms = duration.as_millis() % 1000;

    if secs >= 60 {
        let mins = secs / 60;
        let secs = secs % 60;
        format!("{mins}m {secs}s")
    } else if secs > 0 {
        format!("{secs}.{ms:03}s")
    } else {
        format!("{ms}ms")
    }
}

/// Format milliseconds for human display.
#[must_use]
pub fn format_ms(ms: u64) -> String {
    format_duration(Duration::from_millis(ms))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_stopwatch_new() {
        let sw = Stopwatch::new();
        assert!(sw.elapsed() >= Duration::ZERO);
    }

    #[test]
    fn test_stopwatch_elapsed() {
        let sw = Stopwatch::new();
        thread::sleep(Duration::from_millis(10));
        assert!(sw.elapsed_ms() >= 10);
    }

    #[test]
    fn test_stopwatch_split() {
        let mut sw = Stopwatch::new();
        thread::sleep(Duration::from_millis(5));
        sw.split("first");
        thread::sleep(Duration::from_millis(5));
        sw.split("second");

        assert_eq!(sw.splits().len(), 2);
        assert!(sw.splits()[1].1 > sw.splits()[0].1);
    }

    #[test]
    fn test_format_duration_ms() {
        let d = Duration::from_millis(500);
        assert_eq!(format_duration(d), "500ms");
    }

    #[test]
    fn test_format_duration_seconds() {
        let d = Duration::from_millis(2345);
        assert_eq!(format_duration(d), "2.345s");
    }

    #[test]
    fn test_format_duration_minutes() {
        let d = Duration::from_secs(125);
        assert_eq!(format_duration(d), "2m 5s");
    }

    #[test]
    fn test_format_ms() {
        assert_eq!(format_ms(100), "100ms");
        assert_eq!(format_ms(5000), "5.000s");
    }
}
