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

/// Validate a specification.
#[must_use]
pub fn validate(spec: &Spec) -> ValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Check required fields
    if spec.id.is_empty() {
        errors.push(ValidationError {
            field: "id".to_string(),
            message: "id is required".to_string(),
        });
    } else if !spec.id.chars().all(|c| c.is_alphanumeric() || c == '_') {
        errors.push(ValidationError {
            field: "id".to_string(),
            message: "id must be alphanumeric with underscores only".to_string(),
        });
    }

    if spec.goal.is_empty() {
        errors.push(ValidationError {
            field: "goal".to_string(),
            message: "goal is required".to_string(),
        });
    }

    if spec.acceptance.is_empty() {
        errors.push(ValidationError {
            field: "acceptance".to_string(),
            message: "at least one acceptance criterion is required".to_string(),
        });
    }

    // Warnings
    if spec.constraints.is_empty() {
        warnings.push("No constraints defined (optional but recommended)".to_string());
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
            id: "valid_id".to_string(),
            goal: "A valid goal".to_string(),
            constraints: vec!["Constraint".to_string()],
            acceptance: vec!["Criterion".to_string()],
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
    fn test_invalid_id_format_fails() {
        let mut spec = valid_spec();
        spec.id = "invalid-id".to_string();

        let result = validate(&spec);

        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.field == "id"));
    }

    #[test]
    fn test_id_with_spaces_fails() {
        let mut spec = valid_spec();
        spec.id = "has spaces".to_string();

        let result = validate(&spec);

        assert!(!result.valid);
    }

    #[test]
    fn test_empty_goal_fails() {
        let mut spec = valid_spec();
        spec.goal = String::new();

        let result = validate(&spec);

        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.field == "goal"));
    }

    #[test]
    fn test_empty_acceptance_fails() {
        let mut spec = valid_spec();
        spec.acceptance = vec![];

        let result = validate(&spec);

        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.field == "acceptance"));
    }

    #[test]
    fn test_empty_constraints_gives_warning() {
        let mut spec = valid_spec();
        spec.constraints = vec![];

        let result = validate(&spec);

        assert!(result.valid); // Still valid
        assert!(!result.warnings.is_empty());
    }

    #[test]
    fn test_multiple_errors_reported() {
        let spec = Spec {
            id: String::new(),
            goal: String::new(),
            constraints: vec![],
            acceptance: vec![],
            source_path: None,
        };

        let result = validate(&spec);

        assert!(!result.valid);
        assert!(result.errors.len() >= 3); // id, goal, acceptance
    }

    #[test]
    fn test_underscore_in_id_allowed() {
        let mut spec = valid_spec();
        spec.id = "my_feature_v2".to_string();

        let result = validate(&spec);

        assert!(result.valid);
    }

    #[test]
    fn test_alphanumeric_id_allowed() {
        let mut spec = valid_spec();
        spec.id = "feature123".to_string();

        let result = validate(&spec);

        assert!(result.valid);
    }
}
