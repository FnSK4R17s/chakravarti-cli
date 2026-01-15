# Data Model: AI-Powered Spec Workflow

**Feature**: 001-spec-setup
**Date**: 2026-01-13

## Entities

### SpecDirectory

The top-level container for all specification artifacts.

```
.specs/<spec-id>/
├── spec.yaml            # Main specification (YAML - renders on UI)
├── research.md          # Research findings and decisions
├── design.md            # Technical design (not plan.yaml!)
├── data-model.md        # Entity definitions
├── tasks.yaml           # Implementation tasks
├── plan.yaml            # Execution plan (existing - for task orchestration)
├── checklists/          # Validation checklists
│   └── requirements.md
└── contracts/           # API contracts (optional)
    └── api.yaml
```

**Attributes**:
- `id`: String - Unique identifier (e.g., "001-feature-name")
- `path`: PathBuf - Absolute path to spec directory
- `status`: SpecStatus - Current workflow phase
- `created_at`: DateTime - Creation timestamp
- `updated_at`: DateTime - Last modification

### SpecStatus

```rust
enum SpecStatus {
    Draft,          // Initial creation, spec.md exists
    NeedsClarify,   // Has [NEEDS CLARIFICATION] markers
    Ready,          // Clarified, ready for planning
    Planned,        // Has plan.md
    TasksGenerated, // Has tasks.yaml
    InProgress,     // Tasks being executed
    Complete,       // All tasks done
}
```

### Specification (spec.md structure)

```rust
struct Specification {
    id: String,
    branch: String,
    created: String,
    status: SpecStatus,
    
    // Content sections
    overview: Option<String>,
    user_stories: Vec<UserStory>,
    edge_cases: Vec<String>,
    requirements: Vec<Requirement>,
    success_criteria: Vec<Criterion>,
    assumptions: Vec<String>,
    
    // Clarifications
    clarifications: Vec<Clarification>,
}
```

### UserStory

```rust
struct UserStory {
    id: String,              // e.g., "US1"
    title: String,
    priority: Priority,      // P1, P2, P3
    description: String,
    why_priority: String,
    independent_test: String,
    acceptance_scenarios: Vec<AcceptanceScenario>,
}

struct AcceptanceScenario {
    given: String,
    when: String,
    then: String,
}

enum Priority {
    P1,  // Must have (MVP)
    P2,  // Should have
    P3,  // Nice to have
}
```

### Requirement

```rust
struct Requirement {
    id: String,           // e.g., "FR-001"
    description: String,
    category: RequirementCategory,
}

enum RequirementCategory {
    Functional,    // FR-xxx
    NonFunctional, // NFR-xxx
    Security,      // SEC-xxx
}
```

### Criterion

```rust
struct Criterion {
    id: String,           // e.g., "SC-001"
    metric: String,
    target: Option<String>,
    measurement: Option<String>,
}
```

### Clarification

```rust
struct Clarification {
    id: String,
    topic: String,
    question: String,
    context: Option<String>,  // Relevant spec section
    options: Vec<ClarificationOption>,
    resolved: Option<String>,
}

struct ClarificationOption {
    label: String,    // A, B, C, Custom
    answer: String,
    implications: String,
}
```

### Task (existing, extended)

```rust
struct Task {
    id: String,              // T001, T002, etc.
    phase: String,           // Setup, Foundation, US1, Polish
    title: String,
    description: String,
    file: Option<String>,    // Primary file target
    user_story: Option<String>,
    parallel: bool,
    complexity: u8,          // 1-5
    model_tier: ModelTier,
    estimated_tokens: u32,
    risk: Risk,
    context_required: Vec<String>,
    status: TaskStatus,
}

enum ModelTier {
    Light,     // Complexity 1-2
    Standard,  // Complexity 2-3
    Heavy,     // Complexity 3-4
    Reasoning, // Complexity 5
}

enum Risk {
    Low,
    Medium,
    High,
    Critical,
}

enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}
```

## Relationships

```
SpecDirectory
    │
    ├── 1:1 ──── Specification (spec.md)
    │               │
    │               ├── 1:N ──── UserStory
    │               │               └── 1:N ──── AcceptanceScenario
    │               │
    │               ├── 1:N ──── Requirement
    │               │
    │               ├── 1:N ──── Criterion
    │               │
    │               └── 1:N ──── Clarification
    │                               └── 1:N ──── ClarificationOption
    │
    ├── 0:1 ──── Research (research.md)
    │
    ├── 0:1 ──── Design (design.md) ← renamed from plan.md
    │
    ├── 0:1 ──── ExecutionPlan (plan.yaml) ← existing, for task orchestration
    │
    └── 0:1 ──── TaskFile (tasks.yaml)
                    └── 1:N ──── Task
```

## State Transitions

```
                    ┌─────────────────┐
                    │     Draft       │
                    └────────┬────────┘
                             │ spec new
                             ▼
            ┌────────────────┴────────────────┐
            │                                 │
            ▼                                 ▼
   ┌────────────────┐               ┌────────────────┐
   │ NeedsClarify   │               │     Ready      │
   └────────┬───────┘               └────────┬───────┘
            │ spec clarify                   │
            └────────────┬───────────────────┘
                         ▼
                ┌────────────────┐
                │    Planned     │ spec plan
                └────────┬───────┘
                         │ spec tasks
                         ▼
                ┌────────────────┐
                │ TasksGenerated │
                └────────┬───────┘
                         │ task run
                         ▼
                ┌────────────────┐
                │   InProgress   │
                └────────┬───────┘
                         │ all tasks complete
                         ▼
                ┌────────────────┐
                │    Complete    │
                └────────────────┘
```

## Validation Rules

### Specification
- `id` must match directory name
- `user_stories` must have at least 1 entry
- All user stories must have at least 1 acceptance scenario
- `requirements` must have at least 3 entries
- `success_criteria` must have at least 1 measurable metric

### Clarification
- Must not have unresolved clarifications before planning
- Maximum 3 clarifications per spec

### Task
- `id` must be unique within task file
- `complexity` must be 1-5
- `model_tier` must match complexity range
- Completed tasks cannot be re-run without force flag
