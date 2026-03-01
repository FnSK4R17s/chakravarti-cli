//! Log entry models for persistent execution logs.
//!
//! These models define the structure for log entries that are persisted
//! to disk and streamed to the UI via WebSocket.

// ============================================================
// Imports
// ============================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

// ============================================================
// Types
// ============================================================

/// A single log entry persisted to disk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Unique identifier for this log entry
    pub id: Uuid,

    /// ID of the execution run this log belongs to
    pub execution_id: String,

    /// When this log was generated (UTC)
    pub timestamp: DateTime<Utc>,

    /// Log level/type
    pub level: LogLevel,

    /// The log message content
    pub message: String,

    /// Optional source identifier (batch name, component, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    /// Optional batch ID for batch-attributed logs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_id: Option<String>,

    /// Optional batch name for batch-attributed logs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_name: Option<String>,
}

impl LogEntry {
    /// Create a new log entry with auto-generated ID and timestamp
    pub fn new(
        execution_id: impl Into<String>,
        level: LogLevel,
        message: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            execution_id: execution_id.into(),
            timestamp: Utc::now(),
            level,
            message: message.into(),
            source: None,
            batch_id: None,
            batch_name: None,
        }
    }

    /// Create a new log entry with a source identifier
    pub fn with_source(
        execution_id: impl Into<String>,
        level: LogLevel,
        message: impl Into<String>,
        source: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            execution_id: execution_id.into(),
            timestamp: Utc::now(),
            level,
            message: message.into(),
            source: Some(source.into()),
            batch_id: None,
            batch_name: None,
        }
    }

    /// Create a new log entry with batch attribution
    pub fn with_batch(
        execution_id: impl Into<String>,
        level: LogLevel,
        message: impl Into<String>,
        batch_id: impl Into<String>,
        batch_name: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            execution_id: execution_id.into(),
            timestamp: Utc::now(),
            level,
            message: message.into(),
            source: None,
            batch_id: Some(batch_id.into()),
            batch_name: Some(batch_name.into()),
        }
    }
}

/// Log level/type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LogLevel {
    /// Informational messages
    Info,
    /// Warning messages
    Warning,
    /// Error messages
    Error,
    /// Generic log output
    Log,
    /// Execution started
    Start,
    /// Batch started
    BatchStart,
    /// Batch completed successfully
    BatchComplete,
    /// Batch failed
    BatchError,
    /// Execution completed successfully
    Success,
    /// Status update
    Status,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Info => write!(f, "info"),
            LogLevel::Warning => write!(f, "warning"),
            LogLevel::Error => write!(f, "error"),
            LogLevel::Log => write!(f, "log"),
            LogLevel::Start => write!(f, "start"),
            LogLevel::BatchStart => write!(f, "batch_start"),
            LogLevel::BatchComplete => write!(f, "batch_complete"),
            LogLevel::BatchError => write!(f, "batch_error"),
            LogLevel::Success => write!(f, "success"),
            LogLevel::Status => write!(f, "status"),
        }
    }
}

/// Metadata about a log file for an execution
#[derive(Debug, Clone)]
pub struct ExecutionLogFile {
    /// The execution ID this log file belongs to
    pub execution_id: String,

    /// Path to the log file on disk
    pub path: PathBuf,

    /// Number of log lines in the file
    pub line_count: usize,

    /// When the log file was created
    pub created_at: DateTime<Utc>,

    /// When the log file was last modified
    pub last_modified: DateTime<Utc>,

    /// Total size in bytes
    pub size_bytes: u64,
}

// ============================================================
// Request/Response Types
// ============================================================

/// Request to fetch historical logs (T005)
#[derive(Debug, Deserialize)]
pub struct LogHistoryRequest {
    /// Start from this line offset (0-indexed)
    #[serde(default)]
    pub offset: usize,

    /// Maximum number of lines to return
    #[serde(default = "default_limit")]
    pub limit: usize,

    /// If provided, only return logs after this timestamp
    #[serde(default)]
    pub since: Option<DateTime<Utc>>,
}

fn default_limit() -> usize {
    100
}

/// Response containing historical logs (T005)
#[derive(Debug, Serialize)]
pub struct LogHistoryResponse {
    /// The execution ID
    pub execution_id: String,

    /// The log entries
    pub logs: Vec<LogEntry>,

    /// Total number of logs in the file
    pub total_count: usize,

    /// Offset used for this request
    pub offset: usize,

    /// Whether there are more logs after this batch
    pub has_more: bool,
}

/// Response for tail logs endpoint
#[derive(Debug, Serialize)]
pub struct LogTailResponse {
    /// The execution ID
    pub execution_id: String,

    /// The log entries (most recent N)
    pub logs: Vec<LogEntry>,

    /// Total number of logs in the file
    pub total_count: usize,
}

/// Response for delete logs endpoint
#[derive(Debug, Serialize)]
pub struct LogDeleteResponse {
    /// Whether the operation succeeded
    pub success: bool,

    /// The execution ID
    pub execution_id: String,

    /// Number of lines deleted
    pub deleted_lines: usize,
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_entry_new() {
        let entry = LogEntry::new("run-123", LogLevel::Info, "Test message");
        assert_eq!(entry.execution_id, "run-123");
        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.message, "Test message");
        assert!(entry.source.is_none());
    }

    #[test]
    fn test_log_entry_with_source() {
        let entry = LogEntry::with_source("run-123", LogLevel::BatchStart, "Starting", "batch-1");
        assert_eq!(entry.source, Some("batch-1".to_string()));
    }

    #[test]
    fn test_log_level_serialization() {
        let level = LogLevel::BatchStart;
        let json = serde_json::to_string(&level).unwrap();
        assert_eq!(json, "\"batch_start\"");
    }
}
