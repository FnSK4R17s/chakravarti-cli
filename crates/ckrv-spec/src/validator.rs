//! Spec validation.

use ckrv_core::Spec;

/// Result of spec validation.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the spec is valid.
    pub valid: bool,
    /// Validation errors found.
    pub errors: Vec<ValidationError>,
    /// Warnings (non-blocking issues).
    pub warnings: Vec<String>,
}

/// A validation error.
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Field that failed validation.
    pub field: String,
    /// Error message.
    pub message: String,
}

/// Validate a specification (new format with overview).
#[must_use]
pub fn validate(spec: &Spec) -> ValidationResult {
    let mut errors = Vec::new();
    let warnings = Vec::new();

    // Check required fields
    if spec.id.is_empty() {
        errors.push(ValidationError {
            field: "id".to_string(),
            message: "id is required".to_string(),
        });
    } else if !spec.id.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        errors.push(ValidationError {
            field: "id".to_string(),
            message: "id must be alphanumeric with underscores or hyphens only".to_string(),
        });
    }

    // Overview is required
    if spec.overview.as_ref().map_or(true, |o| o.is_empty()) {
        errors.push(ValidationError {
            field: "overview".to_string(),
            message: "overview is required".to_string(),
        });
    }

    ValidationResult {
        valid: errors.is_empty(),
        errors,
        warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_spec() -> Spec {
        Spec {
            id: "001-new-feature".to_string(),
            branch: Some("feature/001-new-feature".to_string()),
            created: Some("2026-01-14".to_string()),
            status: Some("draft".to_string()),
            overview: Some("This is the spec overview".to_string()),
            constraints: vec![],
            verify: None,
            source_path: None,
        }
    }

    #[test]
    fn test_valid_spec_passes() {
        let spec = valid_spec();
        let result = validate(&spec);

        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_empty_id_fails() {
        let mut spec = valid_spec();
        spec.id = String::new();

        let result = validate(&spec);

        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.field == "id"));
    }

    #[test]
    fn test_id_with_dashes_passes() {
        let mut spec = valid_spec();
        spec.id = "001-my-feature".to_string();

        let result = validate(&spec);

        assert!(result.valid);
    }

    #[test]
    fn test_id_with_spaces_fails() {
        let mut spec = valid_spec();
        spec.id = "has spaces".to_string();

        let result = validate(&spec);

        assert!(!result.valid);
    }

    #[test]
    fn test_empty_overview_fails() {
        let mut spec = valid_spec();
        spec.overview = None;

        let result = validate(&spec);

        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.field == "overview"));
    }

    #[test]
    fn test_underscore_in_id_allowed() {
        let mut spec = valid_spec();
        spec.id = "my_feature_v2".to_string();

        let result = validate(&spec);

        assert!(result.valid);
    }
}
