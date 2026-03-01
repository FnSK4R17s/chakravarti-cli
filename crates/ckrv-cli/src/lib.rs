//! # Chakravarti CLI Library
//!
//! Exports CLI types for SKILL.md generation and MCP server.
//!
//! ## Overview
//!
//! This module provides public access to the CLI command structure for external tools
//! that need to introspect the command definitions (e.g., `skill_gen` and `ckrv-mcp`).
//! It defines the complete CLI interface including all commands, arguments, and metadata
//! extraction for documentation generation.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                        lib.rs                               │
//! ├─────────────────────────────────────────────────────────────┤
//! │  Cli struct ──────▶ Commands enum ──────▶ Command handlers  │
//! │        │                   │                    │           │
//! │        └───────────────────┴────────────────────┘           │
//! │                          │                                  │
//! │                          ▼                                  │
//! │              extract_command_metadata()                     │
//! │                          │                                  │
//! │              ┌───────────┴───────────┐                      │
//! │              ▼                       ▼                      │
//! │         SKILL.md              MCP Server                    │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Example
//!
//! ```rust
//! use ckrv_cli::{Cli, Commands, extract_command_metadata};
//!
//! // Extract metadata for documentation
//! let metadata = extract_command_metadata();
//! println!("CLI has {} commands", metadata.subcommands.len());
//! ```
//!
//! ## See Also
//!
//! - [`commands`] - Individual command implementations
//! - [`crate::extract_command_metadata`] - Extract CLI structure for docs/MCP

// ============================================================
// IMPORTS
// ============================================================

// External crates (alphabetical)
use clap::{CommandFactory, Parser, Subcommand};
use serde::Serialize;

// ============================================================
// MODULES
// ============================================================

// Internal modules - cloud is for API client, not re-exported
mod cloud;
mod commands;
mod prompts;
mod services;

// Public modules
pub mod ui;

// ============================================================
// RE-EXPORTS
// ============================================================

// Re-export command modules for main.rs access
pub use commands::diff;
pub use commands::fix;
pub use commands::init;
pub use commands::logs;
pub use commands::plan;
pub use commands::promote;
pub use commands::pull;
pub use commands::qa;
pub use commands::report;
pub use commands::run;
pub use commands::spec;
pub use commands::status;
pub use commands::task;
pub use commands::term;
pub use commands::test;
pub use commands::usage;
pub use commands::verify;

// Re-export commands with name conflicts (cloud command vs cloud module, ui command vs ui module)
pub use commands::cloud as cloud_cmd;
pub use commands::ui as ui_cmd;

// ============================================================
// TYPES
// ============================================================

/// Chakravarti CLI - Spec-driven agent orchestration engine
#[derive(Parser)]
#[command(name = "ckrv")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Output format: JSON instead of human-readable
    #[arg(long, global = true)]
    pub json: bool,

    /// Suppress non-essential output
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Top-level CLI commands
#[derive(Subcommand)]
pub enum Commands {
    /// Initialize Chakravarti in the current repository
    #[command(
        display_order = 1,
        long_about = "Initialize Chakravarti in the current repository.\n\n\
                      Creates the `.chakravarti/` directory with default configuration files \
                      including `config.yaml` for project settings and initializes the specs directory.\n\n\
                      This is typically the first command to run when setting up a new project \
                      for AI-driven development with Chakravarti.",
        after_help = "Examples:\n\
                      # Initialize in current directory\n\
                      ckrv init\n\n\
                      # Initialize with verbose output\n\
                      ckrv init --verbose"
    )]
    Init(commands::init::InitArgs),

    /// Create or manage feature specifications
    #[command(
        display_order = 2,
        long_about = "Create or manage feature specifications.\n\n\
                      Specifications are the source of truth for AI-driven development. \
                      They define what needs to be built, the requirements, and acceptance criteria.\n\n\
                      Subcommands: new, list, validate, edit, show",
        after_help = "Examples:\n\
                      # Create a new specification\n\
                      ckrv spec new \"Add user authentication\"\n\n\
                      # List all specifications\n\
                      ckrv spec list\n\n\
                      # Validate a specification\n\
                      ckrv spec validate my-feature"
    )]
    Spec(commands::spec::SpecArgs),

    /// Generate execution plan from tasks (in Docker)
    #[command(
        display_order = 3,
        long_about = "Generate execution plan from tasks using AI.\n\n\
                      Analyzes the specification and tasks file to create a detailed \
                      implementation plan. Runs in a Docker container for isolation.\n\n\
                      The plan breaks down work into atomic steps that AI agents can execute.",
        after_help = "Examples:\n\
                      # Generate plan for a specification\n\
                      ckrv plan my-feature\n\n\
                      # Generate plan with GLM model\n\
                      ckrv plan my-feature --model glm-4.7\n\n\
                      # Skip confirmation prompt\n\
                      ckrv plan my-feature --yes"
    )]
    Plan(commands::plan::PlanArgs),

    /// Run a job based on a specification
    #[command(
        display_order = 4,
        long_about = "Run a job based on a specification.\n\n\
                      Executes the implementation plan using AI agents in isolated Docker sandboxes. \
                      Each task is executed in sequence with full logging and progress tracking.\n\n\
                      Results are committed to a feature branch for review.",
        after_help = "Examples:\n\
                      # Run all tasks for a specification\n\
                      ckrv run my-feature\n\n\
                      # Run with specific agent\n\
                      ckrv run my-feature --agent claude-3.5\n\n\
                      # Dry run (show what would be done)\n\
                      ckrv run my-feature --dry-run"
    )]
    Run(commands::run::RunArgs),

    /// View changes between current branch and base
    #[command(
        display_order = 4,
        long_about = "View changes between current branch and base.\n\n\
                      Shows a summary of modified, added, and deleted files compared to the \
                      base branch. Helps verify what will be included in a pull request.\n\n\
                      Output can be formatted as JSON for programmatic use.",
        after_help = "Examples:\n\
                      # Show diff summary\n\
                      ckrv diff\n\n\
                      # Show diff against specific branch\n\
                      ckrv diff --base main\n\n\
                      # Output as JSON\n\
                      ckrv diff --json"
    )]
    Diff(commands::diff::DiffArgs),

    /// Run tests, lint, and quality checks
    #[command(
        display_order = 5,
        long_about = "Run tests, lint, and quality checks.\n\n\
                      Validates the current code against project quality standards. \
                      Runs the test suite, linters, and any custom verification scripts.\n\n\
                      Failed verifications can be fixed with `ckrv fix`.",
        after_help = "Examples:\n\
                      # Run all verifications\n\
                      ckrv verify\n\n\
                      # Run only tests\n\
                      ckrv verify --tests-only\n\n\
                      # Run in JSON output mode\n\
                      ckrv verify --json"
    )]
    Verify(commands::verify::VerifyArgs),

    /// Create a pull request for the current branch
    #[command(
        display_order = 6,
        long_about = "Create a pull request for the current branch.\n\n\
                      Pushes the feature branch and creates a pull request on GitHub/GitLab. \
                      Auto-generates PR title and description from the specification.\n\n\
                      Requires remote repository access and appropriate permissions.",
        after_help = "Examples:\n\
                      # Create PR with auto-generated description\n\
                      ckrv promote\n\n\
                      # Create as draft PR\n\
                      ckrv promote --draft\n\n\
                      # Create PR with custom title\n\
                      ckrv promote --title \"feat: add user auth\""
    )]
    Promote(commands::promote::PromoteArgs),

    /// Fix verification errors with AI
    #[command(
        display_order = 7,
        long_about = "Fix verification errors with AI.\n\n\
                      Analyzes failed tests, lint errors, or build issues and uses AI to \
                      automatically generate fixes. Runs in an isolated Docker sandbox.\n\n\
                      Best used after `ckrv verify` identifies issues.",
        after_help = "Examples:\n\
                      # Fix all errors\n\
                      ckrv fix\n\n\
                      # Fix with specific agent\n\
                      ckrv fix --agent claude-3.5\n\n\
                      # Fix only test failures\n\
                      ckrv fix --tests-only"
    )]
    Fix(commands::fix::FixArgs),

    /// View aggregate usage metrics across all jobs
    #[command(
        display_order = 8,
        long_about = "View aggregate usage metrics across all jobs.\n\n\
                      Shows total token usage, cost estimates, and timing across all recorded \
                      job runs. Breaks down usage by model to help monitor resource consumption \
                      of agents such as Claude, Codex, and OpenRouter models.",
        after_help = "Examples:\n\
                      # Show usage summary\n\
                      ckrv usage\n\n\
                      # Show per-job breakdown\n\
                      ckrv usage --detailed\n\n\
                      # Show per-agent quota/usage summary\n\
                      ckrv usage --agents\n\n\
                      # Output as JSON\n\
                      ckrv usage --json"
    )]
    Usage(commands::usage::UsageArgs),

    /// Execute a workflow-based agent task
    #[command(hide = true)]
    Task(commands::task::TaskArgs),

    /// Check the status of a job
    #[command(hide = true)]
    Status(commands::status::StatusArgs),

    /// View the metrics report for a job
    #[command(hide = true)]
    Report(commands::report::ReportArgs),

    /// Start the Web UI dashboard
    #[command(
        display_order = 8,
        long_about = "Start the Web UI dashboard.\n\n\
                      Launches a local web server providing a visual interface for managing \
                      specifications, viewing execution progress, and reviewing AI agent output.\n\n\
                      Opens automatically in your default browser.",
        after_help = "Examples:\n\
                      # Start UI on default port\n\
                      ckrv ui\n\n\
                      # Start on custom port\n\
                      ckrv ui --port 8080\n\n\
                      # Don't open browser automatically\n\
                      ckrv ui --no-open"
    )]
    Ui(commands::ui::UiArgs),

    /// Cloud execution commands
    #[command(
        display_order = 9,
        long_about = "Cloud execution commands.\n\n\
                      Manage remote job execution via Chakravarti Cloud. Submit jobs, \
                      monitor progress, and retrieve results from cloud workers.\n\n\
                      Subcommands: login, submit, status, cancel",
        after_help = "Examples:\n\
                      # Login to cloud\n\
                      ckrv cloud login\n\n\
                      # Submit a job\n\
                      ckrv cloud submit my-feature\n\n\
                      # Check job status\n\
                      ckrv cloud status <job-id>"
    )]
    Cloud(commands::cloud::CloudArgs),

    /// Stream or view logs from a cloud job
    #[command(
        display_order = 10,
        long_about = "Stream or view logs from a cloud job.\n\n\
                      Shows real-time output from running jobs or historical logs from \
                      completed jobs. Supports filtering by task or agent.\n\n\
                      Use --follow for continuous streaming.",
        after_help = "Examples:\n\
                      # Stream logs from running job\n\
                      ckrv logs <job-id> --follow\n\n\
                      # View completed job logs\n\
                      ckrv logs <job-id>\n\n\
                      # Filter by task\n\
                      ckrv logs <job-id> --task 3"
    )]
    Logs(commands::logs::LogsArgs),

    /// Pull results from a completed cloud job
    #[command(
        display_order = 11,
        long_about = "Pull results from a completed cloud job.\n\n\
                      Downloads all changes made during cloud execution and applies them \
                      to the local repository. Creates or updates the feature branch.\n\n\
                      Jobs must be in a 'completed' state to pull.",
        after_help = "Examples:\n\
                      # Pull results to current directory\n\
                      ckrv pull <job-id>\n\n\
                      # Pull and create new branch\n\
                      ckrv pull <job-id> --branch feature/new"
    )]
    Pull(commands::pull::PullArgs),

    /// Run tests in sandbox, plan and write new tests
    #[command(
        display_order = 12,
        long_about = "Run tests in sandbox, plan and write new tests.\n\n\
                      Comprehensive test management with AI assistance. Can run existing tests, \
                      analyze coverage gaps, and generate new tests using AI agents.\n\n\
                      Subcommands: run, plan, write",
        after_help = "Examples:\n\
                      # Run all tests\n\
                      ckrv test run\n\n\
                      # Plan tests for uncovered code\n\
                      ckrv test plan\n\n\
                      # Write new tests with AI\n\
                      ckrv test write --agent claude-3.5"
    )]
    Test(commands::test::TestArgs),

    /// QA code review and bug analysis
    #[command(
        display_order = 13,
        long_about = "QA code review and bug analysis.\n\n\
                      AI-powered code review and quality assurance. Analyzes changes for \
                      potential bugs, security issues, and code quality improvements.\n\n\
                      Subcommands: review, bugs, report",
        after_help = "Examples:\n\
                      # Review current changes\n\
                      ckrv qa review\n\n\
                      # Analyze for bugs\n\
                      ckrv qa bugs\n\n\
                      # Generate QA report\n\
                      ckrv qa report"
    )]
    Qa(commands::qa::QaArgs),

    /// Spawn an interactive AI agent terminal
    #[command(
        display_order = 14,
        long_about = "Spawn an interactive AI agent terminal session.\n\n\
                      Quickly launch any configured agent (Claude, OpenRouter, Z.AI, Codex, Kilo Code) \
                      with the correct environment variables automatically configured.\n\n\
                      Without arguments, presents an interactive selection menu with options \
                      for common flags. Use -- to pass arguments directly for scripting.",
        after_help = "Examples:\n\
                      # Interactive selection with options prompt\n\
                      ckrv term\n\n\
                      # Launch specific agent (skips agent selection)\n\
                      ckrv term --agent my-openrouter-agent\n\n\
                      # Pass flags directly (scripting)\n\
                      ckrv term -- --dangerously-skip-permissions --continue\n\n\
                      # List available agents\n\
                      ckrv term --list"
    )]
    Term(commands::term::TermArgs),
}

// ============================================================================
// Command Metadata Types for SKILL.md and MCP generation
// ============================================================================

/// Metadata extracted from a clap Command for documentation/MCP generation
#[derive(Debug, Clone, Serialize)]
pub struct CommandMetadata {
    /// Full command path (e.g., ["ckrv", "spec", "new"])
    pub path: Vec<String>,

    /// Command name (e.g., "new")
    pub name: String,

    /// Short description from #[command(about = "...")]
    pub description: String,

    /// Long description from #[command(long_about = "...")]
    pub long_description: Option<String>,

    /// Examples/notes from #[command(after_help = "...")]
    pub after_help: Option<String>,

    /// Positional arguments
    pub arguments: Vec<ArgumentMetadata>,

    /// Optional flags and options (--flag, --option=value)
    pub options: Vec<OptionMetadata>,

    /// Whether this command is hidden from external interfaces
    pub hidden: bool,

    /// Nested subcommands (empty for leaf commands)
    pub subcommands: Vec<CommandMetadata>,
}

/// Metadata for a positional argument
#[derive(Debug, Clone, Serialize)]
pub struct ArgumentMetadata {
    /// Argument identifier (e.g., "description")
    pub id: String,

    /// Help text describing the argument
    pub help: String,

    /// Whether the argument is required
    pub required: bool,

    /// Type hint for documentation (e.g., "STRING", "PATH")
    pub type_hint: String,
}

/// Metadata for a flag or option
#[derive(Debug, Clone, Serialize)]
pub struct OptionMetadata {
    /// Option identifier (e.g., "force")
    pub id: String,

    /// Long flag name (e.g., "force" -> --force)
    pub long: Option<String>,

    /// Short flag character (e.g., 'f' -> -f)
    pub short: Option<char>,

    /// Help text describing the option
    pub help: String,

    /// Whether this is a flag (no value) or takes a value
    pub takes_value: bool,

    /// Type hint for value (e.g., "STRING", "NUMBER")
    pub value_type: String,

    /// Default value if any
    pub default: Option<String>,
}

/// Extract command metadata from the CLI for SKILL.md and MCP generation.
///
/// This function uses clap's introspection API to extract all command information
/// including subcommands, arguments, options, and help text.
#[must_use]
pub fn extract_command_metadata() -> CommandMetadata {
    let cmd = Cli::command();
    extract_metadata_recursive(&cmd, vec![])
}

/// Recursively extract metadata from a clap Command
fn extract_metadata_recursive(cmd: &clap::Command, parent_path: Vec<String>) -> CommandMetadata {
    let mut path = parent_path;
    path.push(cmd.get_name().to_string());

    // Extract positional arguments
    let arguments: Vec<ArgumentMetadata> = cmd
        .get_positionals()
        .map(|arg| ArgumentMetadata {
            id: arg.get_id().to_string(),
            help: arg.get_help().map(|h| h.to_string()).unwrap_or_default(),
            required: arg.is_required_set(),
            type_hint: infer_type_hint(arg),
        })
        .collect();

    // Extract options (--flag, --option=value)
    let options: Vec<OptionMetadata> = cmd
        .get_opts()
        .filter(|arg| {
            // Skip global args that are inherited (json, quiet, verbose)
            let id = arg.get_id().as_str();
            !matches!(id, "json" | "quiet" | "verbose" | "help" | "version")
        })
        .map(|arg| OptionMetadata {
            id: arg.get_id().to_string(),
            long: arg.get_long().map(|s| s.to_string()),
            short: arg.get_short(),
            help: arg.get_help().map(|h| h.to_string()).unwrap_or_default(),
            takes_value: arg
                .get_num_args()
                .map(|n| n.max_values() > 0)
                .unwrap_or(false),
            value_type: infer_type_hint(arg),
            default: arg
                .get_default_values()
                .first()
                .map(|v| v.to_string_lossy().to_string()),
        })
        .collect();

    // Recursively extract subcommands
    let subcommands: Vec<CommandMetadata> = cmd
        .get_subcommands()
        .map(|subcmd| extract_metadata_recursive(subcmd, path.clone()))
        .collect();

    CommandMetadata {
        path,
        name: cmd.get_name().to_string(),
        description: cmd.get_about().map(|s| s.to_string()).unwrap_or_default(),
        long_description: cmd.get_long_about().map(|s| s.to_string()),
        after_help: cmd.get_after_help().map(|s| s.to_string()),
        arguments,
        options,
        hidden: cmd.is_hide_set(),
        subcommands,
    }
}

/// Infer type hint from clap argument
fn infer_type_hint(arg: &clap::Arg) -> String {
    // Check for explicit value hint
    match arg.get_value_hint() {
        clap::ValueHint::FilePath | clap::ValueHint::DirPath | clap::ValueHint::AnyPath => {
            return "PATH".to_string();
        }
        clap::ValueHint::Url => return "URL".to_string(),
        clap::ValueHint::CommandName => return "COMMAND".to_string(),
        clap::ValueHint::Username => return "USER".to_string(),
        clap::ValueHint::Hostname => return "HOST".to_string(),
        _ => {}
    }

    // Check if it's a flag (boolean)
    if arg
        .get_num_args()
        .map(|n| n.max_values() == 0)
        .unwrap_or(false)
    {
        return "FLAG".to_string();
    }

    // Default to STRING
    "STRING".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_command_metadata_filters_hidden() {
        let metadata = extract_command_metadata();

        // Find the hidden commands in subcommands
        let hidden_names: Vec<&str> = metadata
            .subcommands
            .iter()
            .filter(|cmd| cmd.hidden)
            .map(|cmd| cmd.name.as_str())
            .collect();

        // Task, Status, Report should be hidden
        assert!(hidden_names.contains(&"task"), "task should be hidden");
        assert!(hidden_names.contains(&"status"), "status should be hidden");
        assert!(hidden_names.contains(&"report"), "report should be hidden");
    }

    #[test]
    fn test_extract_command_metadata_includes_visible() {
        let metadata = extract_command_metadata();

        // Find visible commands
        let visible_names: Vec<&str> = metadata
            .subcommands
            .iter()
            .filter(|cmd| !cmd.hidden)
            .map(|cmd| cmd.name.as_str())
            .collect();

        // Core commands should be visible
        assert!(visible_names.contains(&"init"), "init should be visible");
        assert!(visible_names.contains(&"spec"), "spec should be visible");
        assert!(visible_names.contains(&"plan"), "plan should be visible");
        assert!(visible_names.contains(&"run"), "run should be visible");
    }

    #[test]
    fn test_spec_subcommands_extracted() {
        let metadata = extract_command_metadata();

        // Find spec command
        let spec_cmd = metadata.subcommands.iter().find(|cmd| cmd.name == "spec");

        assert!(spec_cmd.is_some(), "spec command should exist");
        let spec = spec_cmd.expect("spec command should exist");

        // Spec should have subcommands
        assert!(!spec.subcommands.is_empty(), "spec should have subcommands");

        // Check for expected subcommands
        let subcmd_names: Vec<&str> = spec.subcommands.iter().map(|c| c.name.as_str()).collect();
        assert!(
            subcmd_names.contains(&"new"),
            "spec should have 'new' subcommand"
        );
        assert!(
            subcmd_names.contains(&"list"),
            "spec should have 'list' subcommand"
        );
    }
}
