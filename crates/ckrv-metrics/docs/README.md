---
last_commit: c1bb442
last_updated: 2026-01-21
related_files:
  - src/lib.rs
  - src/collector.rs
  - src/report.rs
---

# ckrv-metrics

Cost and time metrics aggregation for Chakravarti.

## Overview

This crate collects, stores, and reports metrics for job execution including token usage, costs, and timing data.

## Key Types

| Type | Purpose |
|------|---------|
| `MetricsCollector` | Collects metrics during execution |
| `Metrics` | Aggregated metrics data |
| `CostEstimate` | Token cost calculation |
| `StepTimer` | Step execution timing |
| `Stopwatch` | General timing utility |

## Usage

```rust
use ckrv_metrics::{MetricsCollector, DefaultMetricsCollector, Stopwatch};

let mut collector = DefaultMetricsCollector::new();

// Time a step
let timer = Stopwatch::start();
// ... execute step ...
let elapsed = timer.elapsed();

// Record cost
collector.record_tokens(1500, 500, "claude-3-sonnet");
```

## Module Structure

```
src/
├── collector.rs   # Metrics collection
├── cost.rs        # Cost calculation
├── report.rs      # Report generation
├── time.rs        # Timing utilities
└── error.rs       # Error types
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `chrono` | Time handling |
| `serde` | Serialization |
