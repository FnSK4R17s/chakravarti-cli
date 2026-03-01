//! Cost and time metrics aggregation for Chakravarti CLI.
//!
//! This crate collects, stores, and reports metrics for job execution,
//! including token usage tracking, cost estimation, and timing utilities.

/// Metrics collection trait and default implementation.
pub mod collector;
/// Cost estimation with per-model pricing.
pub mod cost;
/// Metrics error types.
pub mod error;
/// Metrics reporting, storage, and summary generation.
pub mod report;
/// Timing utilities and human-readable duration formatting.
pub mod time;

pub use collector::{DefaultMetricsCollector, MetricsCollector, StepTimer};
pub use cost::{CostEstimate, ModelPricing};
pub use error::MetricsError;
pub use report::{
    FileMetricsStorage, Metrics, MetricsStorage, MetricsSummary, StepMetrics, TokenUsageEntry,
};
pub use time::{format_duration, format_ms, Stopwatch};
