//! Chakravarti CLI library - exports CLI types for SKILL.md generation and MCP server.
//!
//! This module provides public access to the CLI command structure for external tools
//! that need to introspect the command definitions (e.g., `skill_gen` and `ckrv-mcp`).

use clap::{CommandFactory, Parser, Subcommand};
use serde::Serialize;

// Internal modules - cloud is for API client, not re-exported
mod cloud;
mod commands;
mod prompts;
mod services;

// Public modules
pub mod ui;

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
pub use commands::test;
pub use commands::verify;

// Re-export commands with name conflicts (cloud command vs cloud module, ui command vs ui module)
pub use commands::cloud as cloud_cmd;
pub use commands::ui as ui_cmd;

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
    #[command(display_order = 1)]
    Init(commands::init::InitArgs),

    /// Create or manage feature specifications
    #[command(display_order = 2)]
    Spec(commands::spec::SpecArgs),

    /// Generate execution plan from tasks (in Docker)
    #[command(display_order = 3)]
    Plan(commands::plan::PlanArgs),

    /// Run a job based on a specification
    #[command(display_order = 4)]
    Run(commands::run::RunArgs),

    /// View changes between current branch and base
    #[command(display_order = 4)]
    Diff(commands::diff::DiffArgs),

    /// Run tests, lint, and quality checks
    #[command(display_order = 5)]
    Verify(commands::verify::VerifyArgs),

    /// Create a pull request for the current branch
    #[command(display_order = 6)]
    Promote(commands::promote::PromoteArgs),

    /// Fix verification errors with AI
    #[command(display_order = 7)]
    Fix(commands::fix::FixArgs),

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
    #[command(display_order = 8)]
    Ui(commands::ui::UiArgs),

    /// Cloud execution commands
    #[command(display_order = 9)]
    Cloud(commands::cloud::CloudArgs),

    /// Stream or view logs from a cloud job
    #[command(display_order = 10)]
    Logs(commands::logs::LogsArgs),

    /// Pull results from a completed cloud job
    #[command(display_order = 11)]
    Pull(commands::pull::PullArgs),

    /// Run tests in sandbox, plan and write new tests
    #[command(display_order = 12)]
    Test(commands::test::TestArgs),

    /// QA code review and bug analysis
    #[command(display_order = 13)]
    Qa(commands::qa::QaArgs),
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
