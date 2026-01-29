---
last_commit: 5160ff1
last_updated: 2026-01-29
related_files:
  - src/lib.rs
  - src/collector.rs
  - src/report.rs
  - src/cost.rs
---

# ckrv-metrics

Cost and time metrics aggregation for Chakravarti.

## Overview

This crate collects, stores, and reports metrics for job execution including token usage, costs, and timing data. Used by `ckrv run`, `ckrv status`, and `ckrv report` commands.

## Key Types

| Type | Module | Purpose |
|------|--------|---------|
| `MetricsCollector` | collector.rs | Trait for collecting metrics |
| `DefaultMetricsCollector` | collector.rs | Thread-safe implementation |
| `StepTimer` | collector.rs | RAII timer for steps |
| `Metrics` | report.rs | Aggregated job metrics |
| `MetricsSummary` | report.rs | Display-friendly summary |
| `StepMetrics` | report.rs | Per-step timing |
| `TokenUsageEntry` | report.rs | Model token counts |
| `MetricsStorage` | report.rs | Trait for persistence |
| `FileMetricsStorage` | report.rs | JSON file storage |
| `CostEstimate` | cost.rs | Token cost calculation |
| `ModelPricing` | cost.rs | Per-model pricing |
| `MetricsError` | error.rs | Error types |
| `Stopwatch` | time.rs | General timing utility |
| `format_duration` | time.rs | Format Duration for display |
| `format_ms` | time.rs | Format milliseconds |

## Module Structure

```
src/
├── lib.rs         # Public exports
├── collector.rs   # MetricsCollector trait + impl (6KB)
├── cost.rs        # Cost calculation (5KB)
├── report.rs      # Metrics + storage (8KB)
├── time.rs        # Timing utilities (3KB)
└── error.rs       # Error types
```

## Usage

### Collecting Metrics

```rust
use ckrv_metrics::{DefaultMetricsCollector, MetricsCollector, StepTimer};

let collector = DefaultMetricsCollector::new();

// Start tracking a job
collector.start_job("job-123", "spec-abc");

// Record token usage (model, input, output)
collector.record_tokens("claude-3-5-sonnet", 1500, 500);

// Time a step with RAII timer
let timer = StepTimer::start(&collector, "execute");
// ... do work ...
timer.stop(); // Records duration automatically

// Finish and get final metrics
let metrics = collector.finish_job(true);
println!("Total tokens: {}", metrics.total_tokens());
println!("Cost: ${:.4}", metrics.cost.total_usd);
```

### Storing Metrics

```rust
use ckrv_metrics::{FileMetricsStorage, MetricsStorage, Metrics};

let storage = FileMetricsStorage::new(".chakravarti");

// Save metrics
let metrics = Metrics::new("job-123", "spec-abc");
storage.save(&metrics)?;

// Load metrics
if storage.exists("job-123") {
    let loaded = storage.load("job-123")?;
    println!("Duration: {}s", loaded.total_time_ms as f64 / 1000.0);
}
```

### Timing Utilities

```rust
use ckrv_metrics::{Stopwatch, format_duration, format_ms};
use std::time::Duration;

let watch = Stopwatch::start();
// ... do work ...
let elapsed = watch.elapsed();

println!("Duration: {}", format_duration(elapsed)); // "1m 23s"
println!("Milliseconds: {}", format_ms(1234));      // "1.234s"
```

## Traits

### MetricsCollector

```rust
pub trait MetricsCollector: Send + Sync {
    fn record_timing(&self, step_id: &str, duration: Duration);
    fn record_tokens(&self, model: &str, input: u64, output: u64);
    fn start_job(&self, job_id: &str, spec_id: &str);
    fn finish_job(&self, success: bool) -> Metrics;
    fn snapshot(&self) -> Metrics;
}
```

### MetricsStorage

```rust
pub trait MetricsStorage: Send + Sync {
    fn save(&self, metrics: &Metrics) -> Result<(), MetricsError>;
    fn load(&self, job_id: &str) -> Result<Metrics, MetricsError>;
    fn exists(&self, job_id: &str) -> bool;
}
```

## Storage Format

Metrics are stored as JSON in `.chakravarti/runs/{job_id}/metrics.json`:

```json
{
  "job_id": "job-123",
  "spec_id": "spec-abc",
  "total_time_ms": 45000,
  "token_usage": [
    { "model": "claude-3-5-sonnet", "input_tokens": 1500, "output_tokens": 500 }
  ],
  "cost": { "total_usd": 0.015 },
  "success": true
}
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `chrono` | Time handling |
| `serde` | Serialization |
| `serde_json` | JSON persistence |
