//! Specification type.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::CoreError;

/// A specification defining a desired code change.
/// Uses the new spec format with overview, user_stories, and requirements.functional.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spec {
    /// Unique identifier for the specification.
    pub id: String,

    /// Branch name for this spec.
    #[serde(default)]
    pub branch: Option<String>,

    /// Creation date.
    #[serde(default)]
    pub created: Option<String>,

    /// Status (draft, ready, etc).
    #[serde(default)]
    pub status: Option<String>,

    /// Overview/description of the spec.
    #[serde(default)]
    pub overview: Option<String>,

    /// Constraints that must be respected (optional).
    #[serde(default)]
    pub constraints: Vec<String>,

    /// Verification configuration (run in Docker sandbox).
    #[serde(default)]
    pub verify: Option<VerifyConfig>,

    /// Path to the source file.
    #[serde(skip)]
    pub source_path: Option<PathBuf>,
}

/// Configuration for verification in Docker sandbox.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyConfig {
    /// Docker image to use for verification.
    #[serde(default)]
    pub image: Option<String>,

    /// Commands to run for verification.
    #[serde(default)]
    pub commands: Vec<String>,
}

impl Spec {
    /// Get the overview/description.
    pub fn description(&self) -> &str {
        self.overview.as_deref().unwrap_or("")
    }

    /// Validate the specification.
    ///
    /// # Errors
    ///
    /// Returns an error if the specification is invalid.
    pub fn validate(&self) -> Result<(), CoreError> {
        if self.id.is_empty() {
            return Err(CoreError::InvalidSpec("id is required".to_string()));
        }

        // Allow dashes in ID (e.g., "001-make-hello-world")
        if !self.id.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err(CoreError::InvalidSpec(
                "id must be alphanumeric with underscores or dashes".to_string(),
            ));
        }

        // Overview is required
        if self.overview.as_ref().map_or(true, |o| o.is_empty()) {
            return Err(CoreError::InvalidSpec("overview is required".to_string()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_spec() -> Spec {
        Spec {
            id: "001-add-feature".to_string(),
            branch: Some("feature/001-add-feature".to_string()),
            created: Some("2026-01-14".to_string()),
            status: Some("draft".to_string()),
            overview: Some("Add a new feature to the application".to_string()),
            constraints: vec![],
            verify: None,
            source_path: None,
        }
    }

    #[test]
    fn test_valid_spec_passes_validation() {
        let spec = valid_spec();
        assert!(spec.validate().is_ok());
    }

    #[test]
    fn test_empty_id_fails_validation() {
        let mut spec = valid_spec();
        spec.id = String::new();
        let result = spec.validate();
        assert!(result.is_err());
        assert!(matches!(result, Err(CoreError::InvalidSpec(_))));
    }

    #[test]
    fn test_id_with_dashes_passes_validation() {
        let mut spec = valid_spec();
        spec.id = "001-make-hello-world".to_string();
        let result = spec.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_id_with_spaces_fails_validation() {
        let mut spec = valid_spec();
        spec.id = "invalid id".to_string();
        let result = spec.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_overview_fails_validation() {
        let mut spec = valid_spec();
        spec.overview = None;
        let result = spec.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_description_returns_overview() {
        let spec = valid_spec();
        assert_eq!(spec.description(), "Add a new feature to the application");
    }

    #[test]
    fn test_spec_serialization_roundtrip() {
        let spec = valid_spec();
        let json = serde_json::to_string(&spec).expect("serialize");
        let parsed: Spec = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(spec.id, parsed.id);
        assert_eq!(spec.overview, parsed.overview);
    }
}
