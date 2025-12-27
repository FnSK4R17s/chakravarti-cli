# Data Model: Agent Orchestration

## Core Entities

### 1. Workflow

The static definition of a process.

```rust
#[derive(Debug, Deserialize, Serialize)]
pub struct Workflow {
    pub version: String,
    pub name: String,
    pub description: Option<String>,
    pub steps: Vec<WorkflowStep>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WorkflowStep {
    pub id: String,
    pub name: String,
    pub agent: Option<String>, // e.g. "claude", "gpt-4" (override)
    pub prompt: String,        // Handlebars template
    pub outputs: Vec<StepOutput>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StepOutput {
    pub name: String,
    pub r#type: OutputType, // "file" or "string"
    pub source: String,     // filename or json key
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputType {
    File,
    String,
}
```

### 2. Task

A runtime instance of a workflow execution.

```rust
#[derive(Debug, Deserialize, Serialize)]
pub struct Task {
    pub id: String, // e.g., "DOC-123"
    pub original_prompt: String,
    pub workflow_name: String,
    pub status: TaskStatus,
    pub worktree_path: PathBuf,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
}
```

### 3. Step Execution Result

The output of running a single step.

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct StepResult {
    pub step_id: String,
    pub status: StepStatus,
    pub outputs: HashMap<String, String>, // Key -> Content or FilePath
    pub stdout: String,
    pub stderr: String,
}
```

## State Persistence

File structure:
```text
.ckrv/
└── tasks/
    └── <TASK_ID>/
        ├── metadata.json       # Task struct
        ├── workflow_snapshot.yml # Copy of workflow used
        ├── steps/
        │   ├── 01_plan/
        │   │   ├── result.json # StepResult
        │   │   ├── stdout.log
        │   │   └── outputs/
        │   │       └── plan.md
        │   └── 02_implement/
        └── workspace/          # (Git worktree)
```
