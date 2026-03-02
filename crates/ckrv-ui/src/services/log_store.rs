//! Log storage service for persisting execution logs to disk.
//!
//! Logs are stored as JSONL (JSON Lines) files, one line per log entry.
//! This format supports append-only writes and streaming reads.

// ============================================================
// Imports
// ============================================================

use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};

use crate::models::log::{ExecutionLogFile, LogEntry};

// ============================================================
// LogStore
// ============================================================

/// Service for reading and writing execution logs to disk.
///
/// Logs are stored in `.ckrv/logs/{execution_id}/log.jsonl` format.
pub struct LogStore {
    /// Base path for all log files (typically `.ckrv/logs`)
    base_path: PathBuf,
}

impl LogStore {
    /// Create a new LogStore with the given base path.
    ///
    /// The base path should be the project root; logs will be stored in `.ckrv/logs/`.
    pub fn new(project_root: &Path) -> Self {
        Self {
            base_path: project_root.join(".ckrv").join("logs"),
        }
    }

    /// Get the path to the log file for an execution.
    fn log_file_path(&self, execution_id: &str) -> PathBuf {
        self.base_path.join(execution_id).join("log.jsonl")
    }

    /// Ensure the directory for an execution's logs exists.
    fn ensure_log_dir(&self, execution_id: &str) -> Result<PathBuf> {
        let dir = self.base_path.join(execution_id);
        fs::create_dir_all(&dir)
            .with_context(|| format!("Failed to create log directory: {}", dir.display()))?;
        Ok(dir)
    }

    /// Append a log entry to disk (T006).
    ///
    /// Creates the execution's log directory if it doesn't exist.
    /// Appends the entry as a single JSONL line.
    pub fn append(&self, execution_id: &str, entry: &LogEntry) -> Result<()> {
        self.ensure_log_dir(execution_id)?;
        let path = self.log_file_path(execution_id);

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .with_context(|| format!("Failed to open log file: {}", path.display()))?;

        let mut writer = BufWriter::new(file);
        let json = serde_json::to_string(entry).with_context(|| "Failed to serialize log entry")?;

        writeln!(writer, "{}", json).with_context(|| "Failed to write log entry")?;

        writer
            .flush()
            .with_context(|| "Failed to flush log entry")?;

        Ok(())
    }

    /// Read all log entries for an execution (T007).
    ///
    /// Returns an empty Vec if no logs exist.
    pub fn read_all(&self, execution_id: &str) -> Result<Vec<LogEntry>> {
        let path = self.log_file_path(execution_id);

        if !path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&path)
            .with_context(|| format!("Failed to open log file: {}", path.display()))?;

        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line.with_context(|| "Failed to read log line")?;
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<LogEntry>(&line) {
                Ok(entry) => entries.push(entry),
                Err(e) => {
                    // Log corruption - skip the line but don't fail
                    eprintln!("Warning: Failed to parse log line: {}", e);
                }
            }
        }

        Ok(entries)
    }

    /// Read a range of log entries with offset and limit (T028 - for pagination).
    ///
    /// Returns entries starting from `offset`, up to `limit` entries.
    pub fn read_range(
        &self,
        execution_id: &str,
        offset: usize,
        limit: usize,
    ) -> Result<(Vec<LogEntry>, usize)> {
        let path = self.log_file_path(execution_id);

        if !path.exists() {
            return Ok((Vec::new(), 0));
        }

        let file = File::open(&path)
            .with_context(|| format!("Failed to open log file: {}", path.display()))?;

        let reader = BufReader::new(file);
        let mut entries = Vec::new();
        let mut total_count = 0;
        let mut current_index = 0;

        for line in reader.lines() {
            let line = line.with_context(|| "Failed to read log line")?;
            if line.trim().is_empty() {
                continue;
            }

            total_count += 1;

            // Skip entries before offset
            if current_index < offset {
                current_index += 1;
                continue;
            }

            // Stop after limit entries
            if entries.len() >= limit {
                current_index += 1;
                continue; // Continue counting total
            }

            match serde_json::from_str::<LogEntry>(&line) {
                Ok(entry) => entries.push(entry),
                Err(e) => {
                    eprintln!("Warning: Failed to parse log line: {}", e);
                }
            }

            current_index += 1;
        }

        Ok((entries, total_count))
    }

    /// Read logs since a given timestamp (T037 - for reconnection).
    ///
    /// Returns all log entries with timestamp > since.
    pub fn read_since(&self, execution_id: &str, since: DateTime<Utc>) -> Result<Vec<LogEntry>> {
        let all_entries = self.read_all(execution_id)?;
        Ok(all_entries
            .into_iter()
            .filter(|entry| entry.timestamp > since)
            .collect())
    }

    /// Read the last N log entries (for tail endpoint).
    pub fn read_tail(&self, execution_id: &str, count: usize) -> Result<(Vec<LogEntry>, usize)> {
        let all_entries = self.read_all(execution_id)?;
        let total_count = all_entries.len();

        let tail = if all_entries.len() > count {
            all_entries.into_iter().skip(total_count - count).collect()
        } else {
            all_entries
        };

        Ok((tail, total_count))
    }

    /// Delete all logs for an execution (T050).
    ///
    /// Returns the number of lines deleted.
    pub fn delete(&self, execution_id: &str) -> Result<usize> {
        let path = self.log_file_path(execution_id);

        if !path.exists() {
            return Ok(0);
        }

        // Count lines before deleting
        let (_, line_count) = self.read_range(execution_id, 0, usize::MAX)?;

        // Delete the entire execution directory
        let dir = self.base_path.join(execution_id);
        fs::remove_dir_all(&dir)
            .with_context(|| format!("Failed to delete log directory: {}", dir.display()))?;

        Ok(line_count)
    }

    /// Get metadata about a log file.
    pub fn get_file_info(&self, execution_id: &str) -> Result<Option<ExecutionLogFile>> {
        let path = self.log_file_path(execution_id);

        if !path.exists() {
            return Ok(None);
        }

        let metadata = fs::metadata(&path)
            .with_context(|| format!("Failed to get metadata for: {}", path.display()))?;

        let (_, line_count) = self.read_range(execution_id, 0, usize::MAX)?;

        // Get timestamps from file metadata
        let created_at = metadata
            .created()
            .ok()
            .map(DateTime::<Utc>::from)
            .unwrap_or_else(Utc::now);

        let last_modified = metadata
            .modified()
            .ok()
            .map(DateTime::<Utc>::from)
            .unwrap_or_else(Utc::now);

        Ok(Some(ExecutionLogFile {
            execution_id: execution_id.to_string(),
            path,
            line_count,
            created_at,
            last_modified,
            size_bytes: metadata.len(),
        }))
    }

    /// Check if logs exist for an execution.
    pub fn exists(&self, execution_id: &str) -> bool {
        self.log_file_path(execution_id).exists()
    }

    /// List all execution IDs that have logs.
    pub fn list_executions(&self) -> Result<Vec<String>> {
        if !self.base_path.exists() {
            return Ok(Vec::new());
        }

        let mut executions = Vec::new();

        for entry in fs::read_dir(&self.base_path)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    // Skip .gitkeep
                    if name != ".gitkeep" {
                        executions.push(name.to_string());
                    }
                }
            }
        }

        Ok(executions)
    }
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::log::LogLevel;
    use tempfile::TempDir;

    fn create_test_store() -> (LogStore, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let store = LogStore::new(temp_dir.path());
        (store, temp_dir)
    }

    #[test]
    fn test_append_and_read_all() {
        let (store, _temp) = create_test_store();

        let entry1 = LogEntry::new("run-123", LogLevel::Info, "First message");
        let entry2 = LogEntry::new("run-123", LogLevel::Success, "Second message");

        store.append("run-123", &entry1).unwrap();
        store.append("run-123", &entry2).unwrap();

        let entries = store.read_all("run-123").unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].message, "First message");
        assert_eq!(entries[1].message, "Second message");
    }

    #[test]
    fn test_read_range() {
        let (store, _temp) = create_test_store();

        for i in 0..10 {
            let entry = LogEntry::new("run-123", LogLevel::Log, format!("Message {}", i));
            store.append("run-123", &entry).unwrap();
        }

        let (entries, total) = store.read_range("run-123", 3, 4).unwrap();
        assert_eq!(total, 10);
        assert_eq!(entries.len(), 4);
        assert_eq!(entries[0].message, "Message 3");
        assert_eq!(entries[3].message, "Message 6");
    }

    #[test]
    fn test_read_tail() {
        let (store, _temp) = create_test_store();

        for i in 0..10 {
            let entry = LogEntry::new("run-123", LogLevel::Log, format!("Message {}", i));
            store.append("run-123", &entry).unwrap();
        }

        let (entries, total) = store.read_tail("run-123", 3).unwrap();
        assert_eq!(total, 10);
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].message, "Message 7");
        assert_eq!(entries[2].message, "Message 9");
    }

    #[test]
    fn test_delete() {
        let (store, _temp) = create_test_store();

        let entry = LogEntry::new("run-123", LogLevel::Info, "Test");
        store.append("run-123", &entry).unwrap();

        assert!(store.exists("run-123"));

        let deleted = store.delete("run-123").unwrap();
        assert_eq!(deleted, 1);
        assert!(!store.exists("run-123"));
    }

    #[test]
    fn test_read_nonexistent() {
        let (store, _temp) = create_test_store();
        let entries = store.read_all("nonexistent").unwrap();
        assert!(entries.is_empty());
    }
}
