//! History service for persistent run storage.
//!
//! Handles reading and writing run history to YAML files in the spec directory.

use std::path::{Path, PathBuf};
use std::fs;

use anyhow::{Context, Result, anyhow};

use crate::models::history::{Run, RunHistory, BatchResult, HistoryBatchStatus};

/// Service for managing run history persistence.
pub struct HistoryService {
    specs_dir: PathBuf,
}

impl HistoryService {
    /// Create a new history service for the given project root.
    pub fn new(project_root: &Path) -> Self {
        Self {
            specs_dir: project_root.join(".specs"),
        }
    }
    
    /// Get the path to the runs.yaml file for a spec.
    fn runs_file_path(&self, spec_name: &str) -> PathBuf {
        self.specs_dir.join(spec_name).join("runs.yaml")
    }
    
    /// Load run history for a specification.
    /// Returns empty history if file doesn't exist or is corrupted.
    pub fn load_history(&self, spec_name: &str) -> Result<RunHistory> {
        let path = self.runs_file_path(spec_name);
        
        if !path.exists() {
            return Ok(RunHistory::new(spec_name));
        }
        
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read history file: {:?}", path))?;
        
        // Graceful degradation: return empty history on parse error
        match serde_yaml::from_str::<RunHistory>(&content) {
            Ok(history) => Ok(history),
            Err(e) => {
                eprintln!("[HistoryService] Warning: Failed to parse history file, returning empty: {}", e);
                Ok(RunHistory::new(spec_name))
            }
        }
    }
    
    /// Save run history with atomic write (temp file + rename).
    pub fn save_history(&self, history: &RunHistory) -> Result<()> {
        let path = self.runs_file_path(&history.spec_name);
        let spec_dir = path.parent().unwrap();
        
        // Ensure directory exists
        if !spec_dir.exists() {
            return Err(anyhow!("Spec directory does not exist: {:?}", spec_dir));
        }
        
        // Serialize to YAML
        let content = serde_yaml::to_string(history)
            .context("Failed to serialize history to YAML")?;
        
        // Atomic write: create temp file in same directory, then rename
        let temp_path = spec_dir.join(format!(".runs.yaml.{}.tmp", uuid::Uuid::new_v4()));
        
        fs::write(&temp_path, content.as_bytes())
            .context("Failed to write history to temp file")?;
        
        fs::rename(&temp_path, &path)
            .context("Failed to rename temp file to history file")?;
        
        Ok(())
    }
    
    /// Create a new run and persist it.
    pub fn create_run(&self, spec_name: &str, run_id: &str, batches: Vec<(String, String)>, dry_run: bool) -> Result<Run> {
        let mut history = self.load_history(spec_name)?;
        
        // Check for running runs (concurrent detection)
        if let Some(running) = history.has_running_run() {
            return Err(anyhow!(
                "Another run is already in progress: {} (started at {})",
                running.id,
                running.started_at
            ));
        }
        
        // Create batch results
        let batch_results: Vec<BatchResult> = batches
            .into_iter()
            .map(|(id, name)| BatchResult::new(&id, &name))
            .collect();
        
        let run = Run::new(run_id, spec_name, batch_results, dry_run);
        history.add_run(run.clone());
        
        self.save_history(&history)?;
        
        Ok(run)
    }
    
    /// Update an existing run.
    pub fn update_run<F>(&self, spec_name: &str, run_id: &str, update_fn: F) -> Result<Run>
    where
        F: FnOnce(&mut Run),
    {
        let mut history = self.load_history(spec_name)?;
        
        let run = history.find_run_mut(run_id)
            .ok_or_else(|| anyhow!("Run not found: {}", run_id))?;
        
        update_fn(run);
        let updated_run = run.clone();
        
        self.save_history(&history)?;
        
        Ok(updated_run)
    }
    
    /// Update a batch status within a run.
    pub fn update_batch_status(
        &self,
        spec_name: &str,
        run_id: &str,
        batch_id: &str,
        status: HistoryBatchStatus,
        branch: Option<&str>,
        error: Option<&str>,
    ) -> Result<()> {
        self.update_run(spec_name, run_id, |run| {
            run.update_batch(batch_id, status, branch, error);
        })?;
        Ok(())
    }
    
    /// Mark a run as completed.
    pub fn complete_run(&self, spec_name: &str, run_id: &str) -> Result<Run> {
        self.update_run(spec_name, run_id, |run| {
            run.complete();
        })
    }
    
    /// Mark a run as failed.
    pub fn fail_run(&self, spec_name: &str, run_id: &str, error: &str) -> Result<Run> {
        self.update_run(spec_name, run_id, |run| {
            run.fail(error);
        })
    }
    
    /// Mark a run as aborted.
    pub fn abort_run(&self, spec_name: &str, run_id: &str) -> Result<Run> {
        self.update_run(spec_name, run_id, |run| {
            run.abort();
        })
    }
    
    /// Get a single run by ID.
    pub fn get_run(&self, spec_name: &str, run_id: &str) -> Result<Option<Run>> {
        let history = self.load_history(spec_name)?;
        Ok(history.find_run(run_id).cloned())
    }
    
    /// Check if spec directory exists.
    pub fn spec_exists(&self, spec_name: &str) -> bool {
        self.specs_dir.join(spec_name).exists()
    }
    
    /// Delete a run from history.
    pub fn delete_run(&self, spec_name: &str, run_id: &str) -> Result<()> {
        let mut history = self.load_history(spec_name)?;
        
        // Check if run is currently running
        if let Some(run) = history.find_run(run_id) {
            if run.is_running() {
                return Err(anyhow!("Cannot delete a running execution. Stop it first."));
            }
        }
        
        history.runs.retain(|r| r.id != run_id);
        self.save_history(&history)?;
        
        Ok(())
    }
}
