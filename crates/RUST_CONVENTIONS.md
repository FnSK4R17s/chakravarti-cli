# Rust Code Conventions

> Making Rust code self-documenting for humans and LLMs alike.

Rust already has excellent documentation tooling (`///`, `//!`, rustdoc). This document establishes patterns that maximize LLM comprehension while leveraging Rust's strengths.

---

## Core Principle

**Every module should be self-contained context.** An LLM reading a single file should understand:
- What this module does (purpose)
- How to use it (interface) 
- Why it exists (context)
- What invariants it maintains (contracts)

Rust's type system already communicates much of this—our job is to fill in the **why** and **how**.

---

## Crate-Specific Requirements

### ckrv-cli: Command Documentation (Required)

Every CLI command in `ckrv-cli` **must** have `long_about` and `after_help` attributes. These are used to:
1. Generate the `SKILL.md` file for AI agents
2. Power the MCP server tool descriptions
3. Provide `ckrv <cmd> --help` output

#### Required Pattern

```rust
/// Short description shown in `ckrv --help` command list.
#[command(
    display_order = 1,
    long_about = "Detailed multi-paragraph description.\n\n\
                  Explain what this command does in depth, including:\n\
                  - What files/artifacts are created or modified\n\
                  - Prerequisites and requirements\n\
                  - How it fits into the overall workflow\n\n\
                  Note any important caveats or limitations.",
    after_help = "Examples:\n\
                  # Most common usage\n\
                  ckrv <cmd>\n\n\
                  # With common options\n\
                  ckrv <cmd> --option value\n\n\
                  # Advanced usage\n\
                  ckrv <cmd> --advanced --flags"
)]
CommandName(commands::cmd::CmdArgs),
```

#### Format Guidelines

**For `long_about`:**
- Paragraph 1: Expanded version of the short description
- Paragraph 2: What gets created/modified (files, state)
- Paragraph 3: Prerequisites, requirements, or important notes
- Use `\n\n` between paragraphs
- Use `\n` for line breaks within paragraphs
- End with a period

**For `after_help`:**
- Start with `Examples:\n`
- Use `# comment` format for descriptions
- Show 2-3 practical examples from simple to complex
- Include common option combinations
- Mirror real-world usage patterns

#### Example: `init` Command

```rust
/// Initialize a repository for Chakravarti orchestration.
#[command(
    display_order = 1,
    long_about = "Initialize a repository for Chakravarti orchestration.\n\n\
                  Creates the .ckrv configuration directory with default settings.\n\
                  This is required before running any other ckrv commands.\n\n\
                  If the repository is already initialized, this command will\n\
                  report the current status without making changes.",
    after_help = "Examples:\n\
                  # Initialize current directory\n\
                  ckrv init\n\n\
                  # Initialize with verbose output\n\
                  ckrv init -v\n\n\
                  # Check if already initialized\n\
                  ckrv init --check"
)]
Init(commands::init::InitArgs),
```

#### Subcommand Pattern

For commands with subcommands (e.g., `spec`, `test`, `qa`), document both the parent and each subcommand:

```rust
/// Manage feature specifications.
#[command(
    display_order = 2,
    long_about = "Manage feature specifications.\n\n\
                  Specs are YAML files that describe what to build.\n\
                  Use subcommands to create, edit, and validate specs.",
    after_help = "Subcommands:\n\
                  spec new    Create a new spec from description\n\
                  spec tasks  Generate implementation tasks\n\
                  spec show   Display spec contents"
)]
Spec(commands::spec::SpecCommand),

// In commands/spec.rs:
#[derive(Subcommand)]
pub enum SpecSubcommand {
    /// Create a new feature specification.
    #[command(
        long_about = "Create a new feature specification from a description.\n\n\
                      Generates a structured spec.md file with:\n\
                      - Feature overview and goals\n\
                      - Acceptance criteria\n\
                      - Technical requirements\n\n\
                      The AI will ask clarifying questions if the description\n\
                      is ambiguous.",
        after_help = "Examples:\n\
                      # Create from inline description\n\
                      ckrv spec new \"Add user authentication\"\n\n\
                      # Create interactively\n\
                      ckrv spec new\n\n\
                      # Create in specific directory\n\
                      ckrv spec new \"Feature\" --dir specs/my-feature"
    )]
    New(NewArgs),
}
```

#### Priority Order for Documentation

| Priority | Commands | Reason |
|----------|----------|--------|
| HIGH | init, spec new, plan, run | Core workflow commands |
| MEDIUM | verify, promote, test, qa | Quality and delivery |
| LOW | ui, logs, pull, cloud | Auxiliary features |

---

## Module Structure

Every `.rs` file follows this structure:

```rust
//! # Module Name
//!
//! Brief description of what this module does (1-2 sentences).
//!
//! ## Overview
//!
//! Longer explanation of the module's purpose, when you'd use it,
//! and how it fits into the larger system. This should be 1-2 paragraphs.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────┐     ┌─────────────┐
//! │   Spec      │────▶│   Planner   │
//! └─────────────┘     └─────────────┘
//!                           │
//!                           ▼
//!                     ┌─────────────┐
//!                     │   Tasks     │
//!                     └─────────────┘
//! ```
//!
//! ## Example
//!
//! ```rust
//! use ckrv_core::executor::Executor;
//!
//! let executor = Executor::new(config)?;
//! let result = executor.run(tasks).await?;
//! ```
//!
//! ## See Also
//!
//! - [`crate::planner`] - Generates tasks from specs
//! - [`crate::worktree`] - Manages Git worktrees

// ============================================================
// IMPORTS
// ============================================================

// Group 1: Standard library
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

// Group 2: External crates (alphabetical)
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{debug, error, info, instrument, warn};

// Group 3: Workspace crates
use ckrv_types::{Spec, Task, ExecutionResult};

// Group 4: Internal modules
use crate::config::ExecutorConfig;
use crate::worktree::WorktreeManager;

// ============================================================
// CONSTANTS
// ============================================================

/// Maximum concurrent tasks per execution batch.
/// 
/// This limit prevents overwhelming the system with too many
/// parallel Git operations and agent processes.
const MAX_PARALLEL_TASKS: usize = 10;

/// Default timeout for task execution in seconds.
/// 
/// Individual tasks can override this via `task.timeout`.
const DEFAULT_TASK_TIMEOUT_SECS: u64 = 300;

// ============================================================
// TYPES
// ============================================================

/// Status of a task during execution.
///
/// Tasks progress through these states linearly, except `Failed`
/// which can occur from any active state.
///
/// ```text
/// Pending ──▶ Running ──▶ Completed
///    │           │
///    └───────────┴──────▶ Failed
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Task is queued, waiting for dependencies or capacity.
    Pending,
    
    /// Task is actively executing in a worktree.
    Running {
        /// When execution started (Unix timestamp)
        started_at: u64,
        /// PID of the agent process
        pid: u32,
    },
    
    /// Task completed successfully.
    Completed {
        /// When execution finished
        finished_at: u64,
        /// Summary of changes made
        summary: String,
    },
    
    /// Task failed with an error.
    Failed {
        /// When the failure occurred
        failed_at: u64,
        /// Error message
        error: String,
        /// Whether this task can be retried
        retriable: bool,
    },
}

// ============================================================
// TRAITS
// ============================================================

/// Executes tasks in isolated environments.
///
/// This trait abstracts over different execution backends:
/// - [`LocalExecutor`] - Runs tasks on the local machine
/// - [`CloudExecutor`] - Runs tasks on remote infrastructure
/// - [`MockExecutor`] - For testing
///
/// # Implementing
///
/// Implementations must ensure:
/// 1. Tasks run in isolated Git worktrees
/// 2. Failures in one task don't affect others
/// 3. Resources are cleaned up even on panic
///
/// # Example
///
/// ```rust
/// struct MyExecutor;
///
/// #[async_trait]
/// impl Executor for MyExecutor {
///     async fn execute(&self, task: &Task) -> Result<ExecutionResult> {
///         // Your implementation
///     }
/// }
/// ```
#[async_trait::async_trait]
pub trait Executor: Send + Sync {
    /// Execute a single task.
    ///
    /// # Arguments
    ///
    /// * `task` - The task to execute, must have a valid spec reference
    ///
    /// # Returns
    ///
    /// * `Ok(ExecutionResult)` - Task completed (check `result.success`)
    /// * `Err(_)` - Infrastructure failure (not task failure)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Worktree creation fails
    /// - Agent process cannot be spawned
    /// - Timeout exceeded with no response
    async fn execute(&self, task: &Task) -> Result<ExecutionResult>;
    
    /// Execute multiple tasks with dependency ordering.
    ///
    /// Tasks are executed in topological order based on their
    /// `dependencies` field. Independent tasks run in parallel
    /// up to [`MAX_PARALLEL_TASKS`].
    async fn execute_batch(&self, tasks: &[Task]) -> Result<Vec<ExecutionResult>> {
        // Default implementation with parallel execution
        // ...
    }
}

// ============================================================
// PRIMARY TYPES
// ============================================================

/// Orchestrates parallel task execution across Git worktrees.
///
/// The `TaskRunner` is the main entry point for executing specs.
/// It handles:
/// - Dependency resolution and ordering
/// - Worktree allocation and cleanup  
/// - Progress tracking and reporting
/// - Error recovery and retries
///
/// # Lifecycle
///
/// ```text
/// new() ──▶ run() ──▶ [executing] ──▶ [cleanup] ──▶ Result
///              │
///              └──▶ cancel() ──▶ [cleanup] ──▶ Cancelled
/// ```
///
/// # Thread Safety
///
/// `TaskRunner` is `Send + Sync` and can be shared across threads.
/// Internal state is protected by appropriate synchronization.
///
/// # Example
///
/// ```rust
/// let runner = TaskRunner::new(config, executor)?;
///
/// // Subscribe to progress updates
/// let mut progress_rx = runner.subscribe();
/// tokio::spawn(async move {
///     while let Some(update) = progress_rx.recv().await {
///         println!("Progress: {:?}", update);
///     }
/// });
///
/// // Run tasks
/// let results = runner.run(tasks).await?;
/// ```
#[derive(Debug)]
pub struct TaskRunner<E: Executor> {
    /// Configuration for this runner instance
    config: ExecutorConfig,
    
    /// The executor backend (local, cloud, mock)
    executor: Arc<E>,
    
    /// Manages Git worktree lifecycle
    worktree_manager: WorktreeManager,
    
    /// Channel for broadcasting progress updates
    progress_tx: broadcast::Sender<ProgressUpdate>,
    
    /// Tracks running tasks for cancellation
    running_tasks: Arc<Mutex<HashMap<TaskId, CancellationToken>>>,
}

impl<E: Executor> TaskRunner<E> {
    /// Creates a new task runner with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Execution configuration (parallelism, timeouts, etc.)
    /// * `executor` - The execution backend to use
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Working directory doesn't exist
    /// - Git repository is not initialized
    /// - Insufficient permissions
    ///
    /// # Example
    ///
    /// ```rust
    /// let config = ExecutorConfig::default();
    /// let executor = LocalExecutor::new()?;
    /// let runner = TaskRunner::new(config, executor)?;
    /// ```
    pub fn new(config: ExecutorConfig, executor: E) -> Result<Self> {
        // Validate working directory exists
        let work_dir = &config.work_dir;
        anyhow::ensure!(
            work_dir.exists(),
            "Working directory does not exist: {}",
            work_dir.display()
        );
        
        // Initialize worktree manager
        let worktree_manager = WorktreeManager::new(work_dir)
            .context("Failed to initialize worktree manager")?;
        
        let (progress_tx, _) = broadcast::channel(100);
        
        Ok(Self {
            config,
            executor: Arc::new(executor),
            worktree_manager,
            progress_tx,
            running_tasks: Arc::new(Mutex::new(HashMap::new())),
        })
    }
    
    /// Executes a batch of tasks, respecting dependencies.
    ///
    /// Tasks are executed in waves based on their dependency graph.
    /// Within each wave, independent tasks run in parallel.
    ///
    /// # Arguments
    ///
    /// * `tasks` - Tasks to execute (will be reordered by dependencies)
    ///
    /// # Returns
    ///
    /// Results for all tasks, in the same order as input.
    ///
    /// # Cancellation
    ///
    /// Call [`cancel()`](Self::cancel) to abort execution. Already-running
    /// tasks will complete, but no new tasks will start.
    ///
    /// # Example
    ///
    /// ```rust
    /// let tasks = planner.generate_tasks(&spec)?;
    /// let results = runner.run(tasks).await?;
    ///
    /// for (task, result) in tasks.iter().zip(results.iter()) {
    ///     match result {
    ///         Ok(r) if r.success => println!("✓ {}", task.title),
    ///         Ok(r) => println!("✗ {}: {}", task.title, r.error),
    ///         Err(e) => println!("! {}: {}", task.title, e),
    ///     }
    /// }
    /// ```
    #[instrument(skip(self, tasks), fields(task_count = tasks.len()))]
    pub async fn run(&self, tasks: Vec<Task>) -> Result<Vec<ExecutionResult>> {
        info!("Starting execution of {} tasks", tasks.len());
        
        // Build dependency graph
        let graph = self.build_dependency_graph(&tasks)?;
        
        // Execute in topological order
        let mut results = Vec::with_capacity(tasks.len());
        
        for wave in graph.topological_waves() {
            debug!("Executing wave with {} tasks", wave.len());
            
            let wave_results = self.execute_wave(wave).await?;
            results.extend(wave_results);
        }
        
        Ok(results)
    }
    
    // ... private methods ...
}
```

---

## Documentation Requirements

### Module-Level Docs (Required)

Every `lib.rs` or standalone module must start with `//!` documentation:

```rust
//! # Module Name
//!
//! One-line description.
//!
//! ## Overview
//!
//! What this module does, when you'd use it.
//!
//! ## Example
//!
//! ```rust
//! // Working example
//! ```
```

### Public Items (Required)

Every `pub` item needs `///` documentation:

```rust
/// Brief one-line description.
///
/// Longer explanation if needed. Explain **why** this exists,
/// not just what it does (the code shows what).
///
/// # Arguments / Fields
///
/// * `name` - Description of parameter/field
///
/// # Returns
///
/// What the function returns and when.
///
/// # Errors
///
/// Document each error case:
/// - When `IoError` is returned
/// - When `ValidationError` is returned
///
/// # Panics
///
/// Document if/when this function can panic.
///
/// # Safety (for unsafe)
///
/// Document invariants that must be upheld.
///
/// # Example
///
/// ```rust
/// // Working example that compiles
/// ```
pub fn my_function(name: &str) -> Result<Output> {
    // ...
}
```

### Private Items (Recommended for Complex Logic)

Use `//` comments to explain **why**, not **what**:

```rust
// ❌ Bad: describes what (obvious from code)
// Iterate through tasks
for task in tasks {

// ✅ Good: explains why
// Process in reverse order so dependent tasks see their dependencies' results
for task in tasks.iter().rev() {

// ✅ Good: explains business logic
// Cap at 10 to avoid overwhelming the Git worktree manager
let batch_size = tasks.len().min(MAX_PARALLEL_TASKS);
```

---

## Error Handling Patterns

### Use `anyhow` for Applications, `thiserror` for Libraries

```rust
// In library code (ckrv-core): specific error types
use thiserror::Error;

/// Errors that can occur during task execution.
#[derive(Debug, Error)]
pub enum ExecutionError {
    /// Task exceeded its timeout limit.
    #[error("Task {task_id} timed out after {timeout_secs}s")]
    Timeout {
        task_id: String,
        timeout_secs: u64,
    },
    
    /// Git worktree operation failed.
    #[error("Worktree error for {path}: {source}")]
    Worktree {
        path: PathBuf,
        #[source]
        source: git2::Error,
    },
    
    /// Agent process crashed or was killed.
    #[error("Agent crashed with exit code {code}: {stderr}")]
    AgentCrash {
        code: i32,
        stderr: String,
    },
}

// In application code (ckrv-cli): use anyhow for convenience
use anyhow::{Context, Result};

fn main() -> Result<()> {
    let config = load_config()
        .context("Failed to load configuration")?;
        
    let runner = TaskRunner::new(config)
        .context("Failed to initialize task runner")?;
        
    // ...
}
```

### Context Chain Pattern

Always add context when propagating errors:

```rust
// ❌ Bad: loses context
let file = File::open(path)?;

// ✅ Good: adds context
let file = File::open(path)
    .with_context(|| format!("Failed to open spec file: {}", path.display()))?;

// ✅ Good: chain of context
let spec = load_spec(path)
    .context("Failed to load spec")?;
let tasks = generate_tasks(&spec)
    .context("Failed to generate tasks from spec")?;
let results = execute_tasks(tasks)
    .context("Failed to execute tasks")?;
```

---

## Struct and Enum Patterns

### Builder Pattern for Complex Structs

```rust
/// Configuration for the task executor.
///
/// Use [`ExecutorConfigBuilder`] to construct:
///
/// ```rust
/// let config = ExecutorConfig::builder()
///     .work_dir("/tmp/ckrv")
///     .parallelism(4)
///     .timeout(Duration::from_secs(300))
///     .build()?;
/// ```
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    work_dir: PathBuf,
    parallelism: usize,
    timeout: Duration,
    // ... more fields
}

impl ExecutorConfig {
    /// Creates a builder for constructing configuration.
    pub fn builder() -> ExecutorConfigBuilder {
        ExecutorConfigBuilder::default()
    }
}

/// Builder for [`ExecutorConfig`].
#[derive(Debug, Default)]
pub struct ExecutorConfigBuilder {
    work_dir: Option<PathBuf>,
    parallelism: Option<usize>,
    timeout: Option<Duration>,
}

impl ExecutorConfigBuilder {
    /// Sets the working directory for worktrees.
    ///
    /// # Default
    ///
    /// Current directory if not specified.
    pub fn work_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.work_dir = Some(path.into());
        self
    }
    
    /// Sets maximum parallel task count.
    ///
    /// # Default
    ///
    /// Number of CPU cores.
    ///
    /// # Panics
    ///
    /// Panics if `n` is 0.
    pub fn parallelism(mut self, n: usize) -> Self {
        assert!(n > 0, "Parallelism must be at least 1");
        self.parallelism = Some(n);
        self
    }
    
    /// Builds the configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if required fields are missing or invalid.
    pub fn build(self) -> Result<ExecutorConfig> {
        Ok(ExecutorConfig {
            work_dir: self.work_dir.unwrap_or_else(|| PathBuf::from(".")),
            parallelism: self.parallelism.unwrap_or_else(num_cpus::get),
            timeout: self.timeout.unwrap_or(Duration::from_secs(300)),
        })
    }
}
```

### Enum Documentation

```rust
/// The model provider for agent execution.
///
/// Each variant configures provider-specific settings.
/// Use [`Provider::default()`] for OpenRouter with automatic routing.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Provider {
    /// OpenRouter meta-provider (recommended).
    ///
    /// Automatically routes to the best available model.
    /// Supports fallbacks and load balancing.
    OpenRouter {
        /// API key (defaults to `OPENROUTER_API_KEY` env var)
        api_key: Option<String>,
        /// Model identifier (e.g., "anthropic/claude-3-opus")
        model: String,
    },
    
    /// Direct Anthropic API access.
    ///
    /// Use when you need specific Claude features not
    /// available through OpenRouter.
    Anthropic {
        /// API key (defaults to `ANTHROPIC_API_KEY` env var)
        api_key: Option<String>,
        /// Model identifier (e.g., "claude-3-opus-20240229")
        model: String,
    },
    
    /// Local model via Ollama.
    ///
    /// Useful for development and testing without API costs.
    /// Performance varies significantly by model.
    Ollama {
        /// Ollama server URL
        #[serde(default = "default_ollama_url")]
        url: String,
        /// Model name (e.g., "codellama:34b")
        model: String,
    },
}

fn default_ollama_url() -> String {
    "http://localhost:11434".to_string()
}

impl Default for Provider {
    /// Returns OpenRouter with Claude 3 Opus.
    fn default() -> Self {
        Self::OpenRouter {
            api_key: None,
            model: "anthropic/claude-3-opus".to_string(),
        }
    }
}
```

---

## Async Patterns

### Instrument Async Functions

```rust
use tracing::{instrument, info, debug, error};

impl TaskRunner {
    /// Executes a single task in its worktree.
    #[instrument(
        skip(self),
        fields(
            task_id = %task.id,
            task_title = %task.title,
        )
    )]
    async fn execute_task(&self, task: &Task) -> Result<ExecutionResult> {
        info!("Starting task execution");
        
        let worktree = self.worktree_manager
            .create(&task.id)
            .await
            .context("Failed to create worktree")?;
            
        debug!(worktree_path = %worktree.path().display(), "Worktree created");
        
        let result = self.executor
            .execute(task)
            .await;
            
        match &result {
            Ok(r) if r.success => info!("Task completed successfully"),
            Ok(r) => warn!(error = %r.error, "Task failed"),
            Err(e) => error!(error = %e, "Task execution error"),
        }
        
        result
    }
}
```

### Cancellation Pattern

```rust
use tokio_util::sync::CancellationToken;

/// Handle for controlling a running execution.
///
/// Drop this handle to cancel the execution.
#[derive(Debug)]
pub struct ExecutionHandle {
    cancel_token: CancellationToken,
    result_rx: oneshot::Receiver<Result<Vec<ExecutionResult>>>,
}

impl ExecutionHandle {
    /// Cancels the execution.
    ///
    /// Running tasks will complete, but no new tasks will start.
    /// Returns immediately; use [`wait()`](Self::wait) to get results.
    pub fn cancel(&self) {
        self.cancel_token.cancel();
    }
    
    /// Waits for execution to complete and returns results.
    pub async fn wait(self) -> Result<Vec<ExecutionResult>> {
        self.result_rx.await?
    }
}

impl Drop for ExecutionHandle {
    fn drop(&mut self) {
        // Cancel on drop if not already cancelled
        self.cancel_token.cancel();
    }
}
```

---

## Testing Patterns

### Doc Tests (Required for Public API)

```rust
/// Parses a spec from YAML content.
///
/// # Example
///
/// ```rust
/// use ckrv_core::spec::parse_spec;
///
/// let yaml = r#"
/// name: example
/// tasks:
///   - title: Hello
///     prompt: Say hello
/// "#;
///
/// let spec = parse_spec(yaml)?;
/// assert_eq!(spec.name, "example");
/// assert_eq!(spec.tasks.len(), 1);
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// # Errors
///
/// Returns an error if the YAML is malformed or missing required fields.
pub fn parse_spec(yaml: &str) -> Result<Spec> {
    // ...
}
```

### Integration Test Structure

```rust
// tests/executor_tests.rs

//! Integration tests for the executor module.
//!
//! These tests require a Git repository and may create temporary files.

use ckrv_core::{Executor, ExecutorConfig, Task};
use tempfile::TempDir;

/// Test fixture that sets up a temporary Git repository.
struct TestFixture {
    temp_dir: TempDir,
    executor: Box<dyn Executor>,
}

impl TestFixture {
    async fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        // Initialize git repo, create executor...
        Self { temp_dir, executor }
    }
}

#[tokio::test]
async fn test_single_task_execution() {
    // Arrange
    let fixture = TestFixture::new().await;
    let task = Task::builder()
        .id("test_1")
        .title("Test task")
        .prompt("Create a file named hello.txt")
        .build();
    
    // Act
    let result = fixture.executor.execute(&task).await;
    
    // Assert
    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.success);
    assert!(fixture.temp_dir.path().join("hello.txt").exists());
}
```

---

## Import Organization

```rust
// 1. Standard library (std::)
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

// 2. External crates (alphabetical)
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, mpsc, Mutex};
use tracing::{debug, error, info, instrument, warn};

// 3. Workspace crates (ckrv-*)
use ckrv_types::{Spec, Task, ExecutionResult};

// 4. Crate-level imports (crate::)
use crate::config::ExecutorConfig;
use crate::worktree::WorktreeManager;

// 5. Module-level imports (super::, self::)
use super::common::SharedState;
```

---

## File Size Guidelines

| Lines | Status | Action |
|-------|--------|--------|
| < 300 | ✅ Good | Maintain |
| 300-500 | ⚠️ Monitor | Consider splitting on next change |
| 500-800 | 🟠 Warning | Plan to split |
| > 800 | 🔴 Critical | Must split |

### Splitting Strategy

```rust
// Before: executor.rs (900 lines)

// After:
// executor/
// ├── mod.rs          // Re-exports, module docs (~50 lines)
// ├── runner.rs       // TaskRunner struct (~250 lines)
// ├── worker.rs       // Worker pool (~200 lines)
// ├── progress.rs     // Progress tracking (~150 lines)
// └── tests.rs        // Unit tests (~200 lines)
```

---

## Attributes Cheatsheet

```rust
// Mark functions that return values that should be used
#[must_use = "this returns a new String, it doesn't modify the original"]
pub fn to_uppercase(&self) -> String { ... }

// Deprecation with migration path
#[deprecated(since = "0.5.0", note = "Use `execute_batch` instead")]
pub fn execute_all(&self, tasks: &[Task]) -> Result<()> { ... }

// Conditional compilation
#[cfg(feature = "cloud")]
pub mod cloud_executor;

// Derive common traits (order: Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskId(String);

// Serde customization
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]  // For JSON APIs
pub struct ApiResponse { ... }
```

---

## Changelog

| Date | Change | Author |
|------|--------|--------|
| 2026-02-03 | Initial version | Claude + Shikhar |

---

*Run `cargo doc --open` to generate and view documentation.*
