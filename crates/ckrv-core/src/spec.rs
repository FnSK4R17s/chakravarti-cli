//! Specification type.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::CoreError;

/// A specification defining a desired code change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spec {
    /// Unique identifier for the specification.
    pub id: String,

    /// Human-readable goal statement.
    pub goal: String,

    /// Constraints that must be respected.
    #[serde(default)]
    pub constraints: Vec<String>,

    /// Acceptance criteria to verify success.
    pub acceptance: Vec<String>,

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
    /// Validate the specification.
    ///
    /// # Errors
    ///
    /// Returns an error if the specification is invalid.
    pub fn validate(&self) -> Result<(), CoreError> {
        if self.id.is_empty() {
            return Err(CoreError::InvalidSpec("id is required".to_string()));
        }

        if !self.id.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(CoreError::InvalidSpec(
                "id must be alphanumeric with underscores".to_string(),
            ));
        }

        if self.goal.is_empty() {
            return Err(CoreError::InvalidSpec("goal is required".to_string()));
        }

        if self.acceptance.is_empty() {
            return Err(CoreError::InvalidSpec(
                "at least one acceptance criterion is required".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_spec() -> Spec {
        Spec {
            id: "add_rate_limiter".to_string(),
            goal: "Add rate limiting to API endpoints".to_string(),
            constraints: vec!["Must not break existing tests".to_string()],
            acceptance: vec!["Rate limiting returns 429 after threshold".to_string()],
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
    fn test_invalid_id_characters_fails_validation() {
        let mut spec = valid_spec();
        spec.id = "invalid-id-with-dashes".to_string();
        let result = spec.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_id_with_spaces_fails_validation() {
        let mut spec = valid_spec();
        spec.id = "invalid id".to_string();
        let result = spec.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_goal_fails_validation() {
        let mut spec = valid_spec();
        spec.goal = String::new();
        let result = spec.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_acceptance_fails_validation() {
        let mut spec = valid_spec();
        spec.acceptance = vec![];
        let result = spec.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_constraints_passes_validation() {
        let mut spec = valid_spec();
        spec.constraints = vec![];
        assert!(spec.validate().is_ok());
    }

    #[test]
    fn test_spec_serialization_roundtrip() {
        let spec = valid_spec();
        let json = serde_json::to_string(&spec).expect("serialize");
        let parsed: Spec = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(spec.id, parsed.id);
        assert_eq!(spec.goal, parsed.goal);
    }

    #[test]
    fn test_id_with_underscores_passes() {
        let mut spec = valid_spec();
        spec.id = "add_rate_limiter_v2".to_string();
        assert!(spec.validate().is_ok());
    }

    #[test]
    fn test_id_alphanumeric_passes() {
        let mut spec = valid_spec();
        spec.id = "feature123".to_string();
        assert!(spec.validate().is_ok());
    }
}
