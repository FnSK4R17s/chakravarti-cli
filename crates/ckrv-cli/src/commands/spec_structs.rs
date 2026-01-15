//! Data structures for spec generation and management
//!
//! These structures define the rich spec.yaml format and related data types.

use serde::{Deserialize, Serialize};

/// Status of a specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SpecStatus {
    Draft,
    NeedsClarify,
    Ready,
    HasTasks,
    InProgress,
    Complete,
}

impl Default for SpecStatus {
    fn default() -> Self {
        Self::Draft
    }
}

impl std::fmt::Display for SpecStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpecStatus::Draft => write!(f, "draft"),
            SpecStatus::NeedsClarify => write!(f, "needs_clarify"),
            SpecStatus::Ready => write!(f, "ready"),
            SpecStatus::HasTasks => write!(f, "has_tasks"),
            SpecStatus::InProgress => write!(f, "in_progress"),
            SpecStatus::Complete => write!(f, "complete"),
        }
    }
}

/// Priority level for user stories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Priority {
    P1,
    P2,
    P3,
    P4,
    P5,
}

impl Default for Priority {
    fn default() -> Self {
        Self::P1
    }
}

/// An acceptance scenario in Given/When/Then format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceScenario {
    pub given: String,
    pub when: String,
    pub then: String,
}

/// A user story with acceptance scenarios
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStory {
    pub id: String,
    pub title: String,
    pub priority: Priority,
    pub description: String,
    #[serde(default)]
    pub why_priority: Option<String>,
    #[serde(default)]
    pub independent_test: Option<String>,
    #[serde(default)]
    pub acceptance_scenarios: Vec<AcceptanceScenario>,
}

/// Category of a requirement
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RequirementCategory {
    Functional,
    NonFunctional,
    Security,
    Performance,
}

impl Default for RequirementCategory {
    fn default() -> Self {
        Self::Functional
    }
}

/// A functional or non-functional requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirement {
    pub id: String,
    pub description: String,
    #[serde(default)]
    pub category: RequirementCategory,
}

/// Requirements container for spec
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Requirements {
    #[serde(default)]
    pub functional: Vec<Requirement>,
    #[serde(default)]
    pub non_functional: Vec<Requirement>,
    #[serde(default)]
    pub security: Vec<Requirement>,
}

/// A success criterion with measurable target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Criterion {
    pub id: String,
    pub metric: String,
    #[serde(default)]
    pub measurement: Option<String>,
}

/// A clarification option
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarificationOption {
    pub label: String,
    pub answer: String,
    #[serde(default)]
    pub implications: Option<String>,
}

/// A clarification item that needs user input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clarification {
    pub topic: String,
    pub question: String,
    #[serde(default)]
    pub options: Vec<ClarificationOption>,
    #[serde(default)]
    pub resolved: Option<String>,
}

/// The complete specification structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecOutput {
    pub id: String,
    #[serde(default)]
    pub branch: Option<String>,
    #[serde(default)]
    pub created: Option<String>,
    #[serde(default)]
    pub status: SpecStatus,
    #[serde(default)]
    pub overview: Option<String>,
    #[serde(default)]
    pub user_stories: Vec<UserStory>,
    #[serde(default)]
    pub requirements: Requirements,
    #[serde(default)]
    pub success_criteria: Vec<Criterion>,
    #[serde(default)]
    pub edge_cases: Vec<String>,
    #[serde(default)]
    pub assumptions: Vec<String>,
    #[serde(default)]
    pub clarifications: Vec<Clarification>,
}

impl SpecOutput {
    /// Create a new spec with just an ID
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            branch: None,
            created: None,
            status: SpecStatus::Draft,
            overview: None,
            user_stories: Vec::new(),
            requirements: Requirements::default(),
            success_criteria: Vec::new(),
            edge_cases: Vec::new(),
            assumptions: Vec::new(),
            clarifications: Vec::new(),
        }
    }

    /// Check if spec has unresolved clarifications
    pub fn has_unresolved_clarifications(&self) -> bool {
        self.clarifications.iter().any(|c| c.resolved.is_none())
    }

    /// Get count of user stories
    pub fn user_story_count(&self) -> usize {
        self.user_stories.len()
    }

    /// Get count of requirements
    pub fn requirement_count(&self) -> usize {
        self.requirements.functional.len()
            + self.requirements.non_functional.len()
            + self.requirements.security.len()
    }
}

// ============================================================================
// Task structures (existing, kept for compatibility)
// ============================================================================

#[derive(Debug, Serialize, Deserialize, tabled::Tabled)]
pub struct Task {
    #[tabled(rename = "ID")]
    pub id: String,

    #[tabled(rename = "Phase")]
    pub phase: String,

    #[tabled(rename = "Title")]
    pub title: String,

    #[tabled(skip)]
    pub description: String,

    #[tabled(skip)]
    pub file: Option<String>,

    #[tabled(skip)]
    pub user_story: Option<String>,

    #[tabled(skip)]
    pub parallel: bool,

    #[tabled(rename = "Status")]
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskFile {
    pub tasks: Vec<Task>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spec_output_new() {
        let spec = SpecOutput::new("001-test");
        assert_eq!(spec.id, "001-test");
        assert_eq!(spec.status, SpecStatus::Draft);
        assert!(spec.user_stories.is_empty());
    }

    #[test]
    fn test_spec_status_display() {
        assert_eq!(SpecStatus::Draft.to_string(), "draft");
        assert_eq!(SpecStatus::NeedsClarify.to_string(), "needs_clarify");
        assert_eq!(SpecStatus::Ready.to_string(), "ready");
    }

    #[test]
    fn test_spec_yaml_serialization() {
        let spec = SpecOutput::new("001-test");
        let yaml = serde_yaml::to_string(&spec).unwrap();
        assert!(yaml.contains("id: 001-test"));
        assert!(yaml.contains("status: draft"));
    }

    #[test]
    fn test_spec_yaml_deserialization() {
        let yaml = r#"
id: 001-test
status: ready
user_stories:
  - id: US1
    title: Test Story
    priority: P1
    description: A test story
"#;
        let spec: SpecOutput = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(spec.id, "001-test");
        assert_eq!(spec.status, SpecStatus::Ready);
        assert_eq!(spec.user_stories.len(), 1);
    }
}
