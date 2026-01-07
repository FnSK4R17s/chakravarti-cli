//! Run history data models for persistent execution tracking.
//! 
//! This module defines the data structures for storing and retrieving
//! execution run history from YAML files in the spec directory.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Status of an execution run
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Aborted,
}

impl Default for RunStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// Status of a batch within a run
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HistoryBatchStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

impl Default for HistoryBatchStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// Summary statistics for a run (for quick display)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RunSummary {
    pub total_batches: u32,
    pub completed_batches: u32,
    pub failed_batches: u32,
    pub pending_batches: u32,
    pub tasks_completed: u32,
    pub branches_merged: u32,
}

/// Result of a single batch within a run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub status: HistoryBatchStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(default)]
    pub merged: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl BatchResult {
    /// Create a new pending batch result
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            status: HistoryBatchStatus::Pending,
            started_at: None,
            ended_at: None,
            branch: None,
            merged: false,
            error: None,
        }
    }
    
    /// Mark batch as running
    pub fn start(&mut self) {
        self.status = HistoryBatchStatus::Running;
        self.started_at = Some(Utc::now());
    }
    
    /// Mark batch as completed
    pub fn complete(&mut self, branch: Option<&str>) {
        self.status = HistoryBatchStatus::Completed;
        self.ended_at = Some(Utc::now());
        self.branch = branch.map(|s| s.to_string());
    }
    
    /// Mark batch as failed
    pub fn fail(&mut self, error: &str) {
        self.status = HistoryBatchStatus::Failed;
        self.ended_at = Some(Utc::now());
        self.error = Some(error.to_string());
    }
    
    /// Mark batch as merged
    pub fn mark_merged(&mut self) {
        self.merged = true;
    }
}

/// A single execution run for a specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Run {
    pub id: String,
    pub spec_name: String,
    pub started_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub status: RunStatus,
    #[serde(default)]
    pub dry_run: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elapsed_seconds: Option<u64>,
    pub batches: Vec<BatchResult>,
    #[serde(default)]
    pub summary: RunSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Run {
    /// Create a new run with the given batches
    pub fn new(id: &str, spec_name: &str, batches: Vec<BatchResult>, dry_run: bool) -> Self {
        let total_batches = batches.len() as u32;
        Self {
            id: id.to_string(),
            spec_name: spec_name.to_string(),
            started_at: Utc::now(),
            ended_at: None,
            status: RunStatus::Running,
            dry_run,
            elapsed_seconds: None,
            batches,
            summary: RunSummary {
                total_batches,
                pending_batches: total_batches,
                ..Default::default()
            },
            error: None,
        }
    }
    
    /// Generate a unique run ID
    pub fn generate_id() -> String {
        let now = Utc::now();
        let uuid_part = &uuid::Uuid::new_v4().to_string()[..6];
        format!("run-{}-{}", now.format("%Y-%m-%d"), uuid_part)
    }
    
    /// Update a batch status by ID
    pub fn update_batch(&mut self, batch_id: &str, status: HistoryBatchStatus, branch: Option<&str>, error: Option<&str>) {
        if let Some(batch) = self.batches.iter_mut().find(|b| b.id == batch_id) {
            match status {
                HistoryBatchStatus::Running => batch.start(),
                HistoryBatchStatus::Completed => batch.complete(branch),
                HistoryBatchStatus::Failed => {
                    if let Some(err) = error {
                        batch.fail(err);
                    }
                }
                HistoryBatchStatus::Pending => {}
            }
        }
        self.update_summary();
    }
    
    /// Update the run summary based on batch statuses
    pub fn update_summary(&mut self) {
        self.summary = RunSummary {
            total_batches: self.batches.len() as u32,
            completed_batches: self.batches.iter().filter(|b| b.status == HistoryBatchStatus::Completed).count() as u32,
            failed_batches: self.batches.iter().filter(|b| b.status == HistoryBatchStatus::Failed).count() as u32,
            pending_batches: self.batches.iter().filter(|b| b.status == HistoryBatchStatus::Pending).count() as u32,
            tasks_completed: 0, // Updated separately when tasks complete
            branches_merged: self.batches.iter().filter(|b| b.merged).count() as u32,
        };
    }
    
    /// Mark run as completed
    pub fn complete(&mut self) {
        self.status = RunStatus::Completed;
        self.ended_at = Some(Utc::now());
        if let Some(elapsed) = self.ended_at.map(|e| (e - self.started_at).num_seconds() as u64) {
            self.elapsed_seconds = Some(elapsed);
        }
        self.update_summary();
    }
    
    /// Mark run as failed
    pub fn fail(&mut self, error: &str) {
        self.status = RunStatus::Failed;
        self.ended_at = Some(Utc::now());
        if let Some(elapsed) = self.ended_at.map(|e| (e - self.started_at).num_seconds() as u64) {
            self.elapsed_seconds = Some(elapsed);
        }
        self.error = Some(error.to_string());
        self.update_summary();
    }
    
    /// Mark run as aborted
    pub fn abort(&mut self) {
        self.status = RunStatus::Aborted;
        self.ended_at = Some(Utc::now());
        if let Some(elapsed) = self.ended_at.map(|e| (e - self.started_at).num_seconds() as u64) {
            self.elapsed_seconds = Some(elapsed);
        }
        self.update_summary();
    }
    
    /// Check if run is in progress
    pub fn is_running(&self) -> bool {
        self.status == RunStatus::Running
    }
}

/// Collection of runs for a specification (root of runs.yaml)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RunHistory {
    #[serde(default = "default_version")]
    pub version: String,
    pub spec_name: String,
    #[serde(default)]
    pub runs: Vec<Run>,
}

fn default_version() -> String {
    "1.0".to_string()
}

impl RunHistory {
    /// Create a new empty run history
    pub fn new(spec_name: &str) -> Self {
        Self {
            version: "1.0".to_string(),
            spec_name: spec_name.to_string(),
            runs: Vec::new(),
        }
    }
    
    /// Add a new run (inserts at front for newest-first ordering)
    pub fn add_run(&mut self, run: Run) {
        self.runs.insert(0, run);
    }
    
    /// Find a run by ID
    pub fn find_run(&self, run_id: &str) -> Option<&Run> {
        self.runs.iter().find(|r| r.id == run_id)
    }
    
    /// Find a mutable run by ID
    pub fn find_run_mut(&mut self, run_id: &str) -> Option<&mut Run> {
        self.runs.iter_mut().find(|r| r.id == run_id)
    }
    
    /// Check if any run is currently in progress
    pub fn has_running_run(&self) -> Option<&Run> {
        self.runs.iter().find(|r| r.is_running())
    }
    
    /// Get runs with pagination
    pub fn get_runs(&self, limit: usize, offset: usize) -> &[Run] {
        let start = offset.min(self.runs.len());
        let end = (start + limit).min(self.runs.len());
        &self.runs[start..end]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_run_id_generation() {
        let id = Run::generate_id();
        assert!(id.starts_with("run-"));
        assert!(id.len() > 15); // run-YYYY-MM-DD-XXXXXX
    }
    
    #[test]
    fn test_batch_lifecycle() {
        let mut batch = BatchResult::new("batch-001", "Test Batch");
        assert_eq!(batch.status, HistoryBatchStatus::Pending);
        
        batch.start();
        assert_eq!(batch.status, HistoryBatchStatus::Running);
        assert!(batch.started_at.is_some());
        
        batch.complete(Some("feature-branch"));
        assert_eq!(batch.status, HistoryBatchStatus::Completed);
        assert!(batch.ended_at.is_some());
        assert_eq!(batch.branch, Some("feature-branch".to_string()));
    }
    
    #[test]
    fn test_run_summary_update() {
        let batches = vec![
            BatchResult::new("b1", "Batch 1"),
            BatchResult::new("b2", "Batch 2"),
        ];
        let mut run = Run::new("run-test", "test-spec", batches, false);
        
        run.update_batch("b1", HistoryBatchStatus::Completed, Some("branch-1"), None);
        assert_eq!(run.summary.completed_batches, 1);
        assert_eq!(run.summary.pending_batches, 1);
    }
}
