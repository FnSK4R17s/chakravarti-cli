//! Data structures for spec generation and management
//!
//! These structures define the rich spec.yaml format and related data types.

use serde::{Deserialize, Serialize};

/// Status of a specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum SpecStatus {
    #[default]
    Draft,
    NeedsClarify,
    Ready,
    HasTasks,
    InProgress,
    Complete,
}

impl std::fmt::Display for SpecStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "draft"),
            Self::NeedsClarify => write!(f, "needs_clarify"),
            Self::Ready => write!(f, "ready"),
            Self::HasTasks => write!(f, "has_tasks"),
            Self::InProgress => write!(f, "in_progress"),
            Self::Complete => write!(f, "complete"),
        }
    }
}

/// Priority level for user stories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum Priority {
    #[default]
    P1,
    P2,
    P3,
    P4,
    P5,
}

/// An acceptance scenario in Given/When/Then format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceScenario {
    /// Precondition for the scenario.
    pub given: String,
    /// Action being tested.
    pub when: String,
    /// Expected outcome.
    pub then: String,
}

/// A user story with acceptance scenarios.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStory {
    /// Unique identifier for the user story (e.g., "US1").
    pub id: String,
    /// Short title summarizing the user story.
    pub title: String,
    /// Priority level of the user story.
    pub priority: Priority,
    /// Full description of the user story.
    pub description: String,
    /// Explanation of why this priority was assigned.
    #[serde(default)]
    pub why_priority: Option<String>,
    /// How this story can be tested independently.
    #[serde(default)]
    pub independent_test: Option<String>,
    /// Acceptance scenarios in Given/When/Then format.
    #[serde(default)]
    pub acceptance_scenarios: Vec<AcceptanceScenario>,
}

/// Category of a requirement
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum RequirementCategory {
    #[default]
    Functional,
    NonFunctional,
    Security,
    Performance,
}

/// A functional or non-functional requirement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirement {
    /// Unique identifier for the requirement (e.g., "FR1").
    pub id: String,
    /// Description of the requirement.
    pub description: String,
    /// Category classification of the requirement.
    #[serde(default)]
    pub category: RequirementCategory,
}

/// Requirements container for spec.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Requirements {
    /// Functional requirements.
    #[serde(default)]
    pub functional: Vec<Requirement>,
    /// Non-functional requirements (performance, scalability, etc.).
    #[serde(default)]
    pub non_functional: Vec<Requirement>,
    /// Security requirements.
    #[serde(default)]
    pub security: Vec<Requirement>,
}

/// A success criterion with measurable target.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Criterion {
    /// Unique identifier for the criterion (e.g., "SC1").
    pub id: String,
    /// The metric being measured.
    pub metric: String,
    /// How the metric is measured, if specified.
    #[serde(default)]
    pub measurement: Option<String>,
}

/// A clarification option presented to the user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarificationOption {
    /// Display label for the option.
    pub label: String,
    /// The answer text if this option is chosen.
    pub answer: String,
    /// What choosing this option implies for the spec.
    #[serde(default)]
    pub implications: Option<String>,
}

/// A clarification item that needs user input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clarification {
    /// Topic area of the clarification.
    pub topic: String,
    /// Question to ask the user.
    pub question: String,
    /// Available options for the user to choose from.
    #[serde(default)]
    pub options: Vec<ClarificationOption>,
    /// The resolved answer, if clarification has been addressed.
    #[serde(default)]
    pub resolved: Option<String>,
}

/// The complete specification structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecOutput {
    /// Unique identifier for the spec (e.g., "001-auth").
    pub id: String,
    /// Git branch associated with this spec.
    #[serde(default)]
    pub branch: Option<String>,
    /// Creation timestamp.
    #[serde(default)]
    pub created: Option<String>,
    /// Current status of the specification.
    #[serde(default)]
    pub status: SpecStatus,
    /// High-level overview of the feature.
    #[serde(default)]
    pub overview: Option<String>,
    /// User stories describing the feature from a user perspective.
    #[serde(default)]
    pub user_stories: Vec<UserStory>,
    /// Functional and non-functional requirements.
    #[serde(default)]
    pub requirements: Requirements,
    /// Measurable success criteria for the feature.
    #[serde(default)]
    pub success_criteria: Vec<Criterion>,
    /// Edge cases to consider during implementation.
    #[serde(default)]
    pub edge_cases: Vec<String>,
    /// Assumptions made during specification.
    #[serde(default)]
    pub assumptions: Vec<String>,
    /// Clarifications that need user input.
    #[serde(default)]
    pub clarifications: Vec<Clarification>,
}

impl SpecOutput {
    /// Create a new spec with just an ID
    #[allow(dead_code)]
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

/// A single implementation task generated from a specification.
#[derive(Debug, Serialize, Deserialize, tabled::Tabled)]
#[allow(dead_code)]
pub struct Task {
    /// Unique task identifier (e.g., "T001").
    #[tabled(rename = "ID")]
    pub id: String,

    /// Implementation phase this task belongs to.
    #[tabled(rename = "Phase")]
    pub phase: String,

    /// Short title for the task.
    #[tabled(rename = "Title")]
    pub title: String,

    /// Detailed description of what the task involves.
    #[tabled(skip)]
    pub description: String,

    /// Primary file to modify, if applicable.
    #[tabled(skip)]
    pub file: Option<String>,

    /// User story this task implements.
    #[tabled(skip)]
    pub user_story: Option<String>,

    /// Whether this task can run in parallel with others.
    #[tabled(skip)]
    pub parallel: bool,

    /// Current status of the task (e.g., "pending", "done").
    #[tabled(rename = "Status")]
    pub status: String,
}

/// Container for a tasks.yaml file.
#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct TaskFile {
    /// List of implementation tasks.
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
