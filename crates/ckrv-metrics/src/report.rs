//! Metrics reporting and storage.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{CostEstimate, MetricsError};

/// Aggregated metrics for a job.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Metrics {
    /// Job ID.
    pub job_id: String,
    /// Spec ID.
    pub spec_id: String,
    /// Total wall-clock time in milliseconds.
    pub total_time_ms: u64,
    /// Token usage by model.
    pub token_usage: Vec<TokenUsageEntry>,
    /// Cost estimate.
    pub cost: CostEstimate,
    /// Per-step metrics.
    pub step_metrics: Vec<StepMetrics>,
    /// Number of retry attempts.
    pub retry_count: u32,
    /// Whether the job succeeded.
    pub success: bool,
}

impl Metrics {
    /// Create new metrics for a job.
    #[must_use]
    pub fn new(job_id: impl Into<String>, spec_id: impl Into<String>) -> Self {
        Self {
            job_id: job_id.into(),
            spec_id: spec_id.into(),
            ..Default::default()
        }
    }

    /// Add token usage for a model.
    pub fn add_token_usage(&mut self, model: impl Into<String>, input: u64, output: u64) {
        self.token_usage.push(TokenUsageEntry {
            model: model.into(),
            input_tokens: input,
            output_tokens: output,
        });
    }

    /// Add step metrics.
    pub fn add_step(&mut self, step_id: impl Into<String>, duration_ms: u64) {
        self.step_metrics.push(StepMetrics {
            step_id: step_id.into(),
            duration_ms,
            model: None,
            tokens: None,
        });
    }

    /// Get total token count.
    #[must_use]
    pub fn total_tokens(&self) -> u64 {
        self.token_usage
            .iter()
            .map(|t| t.input_tokens + t.output_tokens)
            .sum()
    }

    /// Get summary for display.
    #[must_use]
    pub fn summary(&self) -> MetricsSummary {
        MetricsSummary {
            job_id: self.job_id.clone(),
            spec_id: self.spec_id.clone(),
            duration_secs: self.total_time_ms as f64 / 1000.0,
            total_tokens: self.total_tokens(),
            estimated_cost_usd: self.cost.total_usd,
            steps: self.step_metrics.len(),
            retries: self.retry_count,
            success: self.success,
        }
    }
}

/// Summary of metrics for display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSummary {
    /// Job ID.
    pub job_id: String,
    /// Spec ID.
    pub spec_id: String,
    /// Duration in seconds.
    pub duration_secs: f64,
    /// Total tokens used.
    pub total_tokens: u64,
    /// Estimated cost in USD.
    pub estimated_cost_usd: f64,
    /// Number of steps executed.
    pub steps: usize,
    /// Number of retries.
    pub retries: u32,
    /// Whether job succeeded.
    pub success: bool,
}

/// Token usage entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsageEntry {
    /// Model name.
    pub model: String,
    /// Input tokens.
    pub input_tokens: u64,
    /// Output tokens.
    pub output_tokens: u64,
}

impl TokenUsageEntry {
    /// Get total tokens.
    #[must_use]
    pub fn total(&self) -> u64 {
        self.input_tokens + self.output_tokens
    }
}

/// Metrics for a single step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepMetrics {
    /// Step ID.
    pub step_id: String,
    /// Duration in milliseconds.
    pub duration_ms: u64,
    /// Model used (if applicable).
    pub model: Option<String>,
    /// Tokens used (if applicable).
    pub tokens: Option<TokenUsageEntry>,
}

/// Trait for metrics storage.
pub trait MetricsStorage: Send + Sync {
    /// Save metrics for a job.
    ///
    /// # Errors
    ///
    /// Returns an error if saving fails.
    fn save(&self, metrics: &Metrics) -> Result<(), MetricsError>;

    /// Load metrics for a job.
    ///
    /// # Errors
    ///
    /// Returns an error if loading fails.
    fn load(&self, job_id: &str) -> Result<Metrics, MetricsError>;

    /// Check if metrics exist for a job.
    fn exists(&self, job_id: &str) -> bool;
}

/// File-based metrics storage.
pub struct FileMetricsStorage {
    base_path: PathBuf,
}

impl FileMetricsStorage {
    /// Create a new file-based storage.
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }

    fn metrics_path(&self, job_id: &str) -> PathBuf {
        self.base_path
            .join("runs")
            .join(job_id)
            .join("metrics.json")
    }
}

impl MetricsStorage for FileMetricsStorage {
    fn save(&self, metrics: &Metrics) -> Result<(), MetricsError> {
        let path = self.metrics_path(&metrics.job_id);

        // Create directory
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| MetricsError::StorageError(e.to_string()))?;
        }

        // Write metrics
        let json = serde_json::to_string_pretty(metrics)
            .map_err(|e| MetricsError::SerializationError(e.to_string()))?;

        std::fs::write(&path, json).map_err(|e| MetricsError::StorageError(e.to_string()))?;

        Ok(())
    }

    fn load(&self, job_id: &str) -> Result<Metrics, MetricsError> {
        let path = self.metrics_path(job_id);

        if !path.exists() {
            return Err(MetricsError::NotFound(job_id.to_string()));
        }

        let content = std::fs::read_to_string(&path)
            .map_err(|e| MetricsError::StorageError(e.to_string()))?;

        serde_json::from_str(&content).map_err(|e| MetricsError::SerializationError(e.to_string()))
    }

    fn exists(&self, job_id: &str) -> bool {
        self.metrics_path(job_id).exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_metrics_new() {
        let metrics = Metrics::new("job-123", "spec-abc");
        assert_eq!(metrics.job_id, "job-123");
        assert_eq!(metrics.spec_id, "spec-abc");
    }

    #[test]
    fn test_metrics_add_token_usage() {
        let mut metrics = Metrics::new("job-123", "spec-abc");
        metrics.add_token_usage("gpt-4o-mini", 100, 500);

        assert_eq!(metrics.token_usage.len(), 1);
        assert_eq!(metrics.total_tokens(), 600);
    }

    #[test]
    fn test_metrics_summary() {
        let mut metrics = Metrics::new("job-123", "spec-abc");
        metrics.total_time_ms = 5000;
        metrics.add_token_usage("gpt-4o", 1000, 2000);
        metrics.success = true;

        let summary = metrics.summary();
        assert_eq!(summary.duration_secs, 5.0);
        assert_eq!(summary.total_tokens, 3000);
    }

    #[test]
    fn test_file_storage_save_load() {
        let dir = TempDir::new().expect("temp dir");
        let storage = FileMetricsStorage::new(dir.path().join(".chakravarti"));

        let mut metrics = Metrics::new("test-job", "test-spec");
        metrics.total_time_ms = 1234;
        metrics.add_token_usage("test-model", 100, 200);

        storage.save(&metrics).expect("save");
        assert!(storage.exists("test-job"));

        let loaded = storage.load("test-job").expect("load");
        assert_eq!(loaded.job_id, "test-job");
        assert_eq!(loaded.total_time_ms, 1234);
    }

    #[test]
    fn test_file_storage_not_found() {
        let dir = TempDir::new().expect("temp dir");
        let storage = FileMetricsStorage::new(dir.path().join(".chakravarti"));

        let result = storage.load("nonexistent");
        assert!(result.is_err());
    }
}
