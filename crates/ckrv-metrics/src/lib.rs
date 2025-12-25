//! Cost and time metrics aggregation for Chakravarti CLI.
//!
//! This crate collects, stores, and reports metrics for job execution.

pub mod collector;
pub mod cost;
pub mod error;
pub mod report;
pub mod time;

pub use collector::{DefaultMetricsCollector, MetricsCollector, StepTimer};
pub use cost::{CostEstimate, ModelPricing};
pub use error::MetricsError;
pub use report::{
    FileMetricsStorage, Metrics, MetricsStorage, MetricsSummary, StepMetrics, TokenUsageEntry,
};
pub use time::{format_duration, format_ms, Stopwatch};
