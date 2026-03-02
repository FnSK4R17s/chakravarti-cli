//! # Common Types
//!
//! Shared types used across multiple handlers.

// ============================================================
// IMPORTS
// ============================================================

use serde::{Deserialize, Serialize};

#[cfg(feature = "typescript")]
use ts_rs::TS;

// ============================================================
// TYPES
// ============================================================

/// Docker daemon status.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct DockerStatus {
    /// Whether Docker daemon is running
    pub available: bool,

    /// Docker version if available
    pub version: Option<String>,

    /// Error message if Docker is not available
    pub error: Option<String>,
}

impl Default for DockerStatus {
    fn default() -> Self {
        Self {
            available: false,
            version: None,
            error: Some("Docker status not checked".to_string()),
        }
    }
}

/// Generic success response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct SuccessResponse {
    /// Success message
    pub message: String,
}

impl SuccessResponse {
    /// Create a new success response.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Paginated list response wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
pub struct PaginatedResponse<T> {
    /// Items in this page
    pub items: Vec<T>,

    /// Total number of items
    pub total: usize,

    /// Current page (0-indexed)
    pub page: usize,

    /// Items per page
    pub per_page: usize,

    /// Whether there are more pages
    pub has_more: bool,
}

impl<T> PaginatedResponse<T> {
    /// Create a paginated response from a full list.
    #[must_use]
    pub fn from_vec(items: Vec<T>, page: usize, per_page: usize) -> Self {
        let total = items.len();
        let start = page * per_page;
        let end = (start + per_page).min(total);

        let page_items = if start < total {
            items.into_iter().skip(start).take(end - start).collect()
        } else {
            Vec::new()
        };

        Self {
            items: page_items,
            total,
            page,
            per_page,
            has_more: end < total,
        }
    }
}

// ============================================================
// TESTS
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docker_status_default() {
        let status = DockerStatus::default();
        assert!(!status.available);
        assert!(status.error.is_some());
    }

    #[test]
    fn test_paginated_response() {
        let items: Vec<i32> = (0..25).collect();
        let response = PaginatedResponse::from_vec(items, 1, 10);

        assert_eq!(response.items.len(), 10);
        assert_eq!(response.total, 25);
        assert_eq!(response.page, 1);
        assert!(response.has_more);
    }
}
