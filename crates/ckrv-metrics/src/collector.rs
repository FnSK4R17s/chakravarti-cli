//! Metrics collection.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::cost::CostEstimate;
use crate::report::{Metrics, StepMetrics, TokenUsageEntry};

/// Trait for collecting metrics.
pub trait MetricsCollector: Send + Sync {
    /// Record timing for a step.
    fn record_timing(&self, step_id: &str, duration: Duration);

    /// Record token usage.
    fn record_tokens(&self, model: &str, input: u64, output: u64);

    /// Start a new job.
    fn start_job(&self, job_id: &str, spec_id: &str);

    /// Finish the job.
    fn finish_job(&self, success: bool) -> Metrics;

    /// Get current metrics snapshot.
    fn snapshot(&self) -> Metrics;
}

/// Default metrics collector implementation.
#[derive(Debug)]
pub struct DefaultMetricsCollector {
    inner: Arc<Mutex<CollectorState>>,
}

#[derive(Debug, Default)]
struct CollectorState {
    job_id: String,
    spec_id: String,
    start_time: Option<Instant>,
    step_metrics: Vec<StepMetrics>,
    token_usage: Vec<TokenUsageEntry>,
    cost: CostEstimate,
}

impl Default for DefaultMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultMetricsCollector {
    /// Create a new collector.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(CollectorState::default())),
        }
    }
}

impl MetricsCollector for DefaultMetricsCollector {
    fn record_timing(&self, step_id: &str, duration: Duration) {
        if let Ok(mut state) = self.inner.lock() {
            state.step_metrics.push(StepMetrics {
                step_id: step_id.to_string(),
                duration_ms: duration.as_millis() as u64,
                model: None,
                tokens: None,
            });
        }
    }

    fn record_tokens(&self, model: &str, input: u64, output: u64) {
        if let Ok(mut state) = self.inner.lock() {
            // Add token usage
            state.token_usage.push(TokenUsageEntry {
                model: model.to_string(),
                input_tokens: input,
                output_tokens: output,
            });

            // Calculate and add cost
            let cost = CostEstimate::from_tokens(model, input, output);
            state.cost.add(model, cost);
        }
    }

    fn start_job(&self, job_id: &str, spec_id: &str) {
        if let Ok(mut state) = self.inner.lock() {
            state.job_id = job_id.to_string();
            state.spec_id = spec_id.to_string();
            state.start_time = Some(Instant::now());
            state.step_metrics.clear();
            state.token_usage.clear();
            state.cost = CostEstimate::default();
        }
    }

    fn finish_job(&self, success: bool) -> Metrics {
        if let Ok(state) = self.inner.lock() {
            let total_time_ms = state
                .start_time
                .map(|t| t.elapsed().as_millis() as u64)
                .unwrap_or(0);

            Metrics {
                job_id: state.job_id.clone(),
                spec_id: state.spec_id.clone(),
                total_time_ms,
                token_usage: state.token_usage.clone(),
                cost: state.cost.clone(),
                step_metrics: state.step_metrics.clone(),
                retry_count: 0,
                success,
            }
        } else {
            Metrics::default()
        }
    }

    fn snapshot(&self) -> Metrics {
        self.finish_job(false)
    }
}

/// Timer for measuring step duration.
pub struct StepTimer<'a, C: MetricsCollector> {
    collector: &'a C,
    step_id: String,
    start: Instant,
}

impl<'a, C: MetricsCollector> StepTimer<'a, C> {
    /// Create and start a timer.
    pub fn start(collector: &'a C, step_id: impl Into<String>) -> Self {
        Self {
            collector,
            step_id: step_id.into(),
            start: Instant::now(),
        }
    }

    /// Stop the timer and record the duration.
    pub fn stop(self) {
        self.collector
            .record_timing(&self.step_id, self.start.elapsed());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collector_start_job() {
        let collector = DefaultMetricsCollector::new();
        collector.start_job("job-123", "spec-abc");

        let snapshot = collector.snapshot();
        assert_eq!(snapshot.job_id, "job-123");
        assert_eq!(snapshot.spec_id, "spec-abc");
    }

    #[test]
    fn test_collector_record_timing() {
        let collector = DefaultMetricsCollector::new();
        collector.start_job("job-123", "spec-abc");
        collector.record_timing("step-1", Duration::from_millis(100));

        let snapshot = collector.snapshot();
        assert_eq!(snapshot.step_metrics.len(), 1);
        assert_eq!(snapshot.step_metrics[0].step_id, "step-1");
        assert_eq!(snapshot.step_metrics[0].duration_ms, 100);
    }

    #[test]
    fn test_collector_record_tokens() {
        let collector = DefaultMetricsCollector::new();
        collector.start_job("job-123", "spec-abc");
        collector.record_tokens("gpt-4o-mini", 1000, 500);

        let snapshot = collector.snapshot();
        assert_eq!(snapshot.token_usage.len(), 1);
        assert_eq!(snapshot.total_tokens(), 1500);
        assert!(snapshot.cost.total_usd > 0.0);
    }

    #[test]
    fn test_collector_finish_job() {
        let collector = DefaultMetricsCollector::new();
        collector.start_job("job-123", "spec-abc");
        collector.record_tokens("gpt-4o", 100, 50);

        let metrics = collector.finish_job(true);
        assert!(metrics.success);
    }

    #[test]
    fn test_step_timer() {
        let collector = DefaultMetricsCollector::new();
        collector.start_job("job-123", "spec-abc");

        let timer = StepTimer::start(&collector, "step-1");
        std::thread::sleep(Duration::from_millis(10));
        timer.stop();

        let snapshot = collector.snapshot();
        assert_eq!(snapshot.step_metrics.len(), 1);
        assert!(snapshot.step_metrics[0].duration_ms >= 10);
    }
}
