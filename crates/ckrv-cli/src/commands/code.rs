//! Code command - unified entry point for the Code workflow.
//!
//! Groups spec, tasks, plan, run, and diff under a single `ckrv code`
//! namespace that mirrors the Code page tabs in the Web UI.

// ============================================================
// IMPORTS
// ============================================================

use std::path::PathBuf;

use clap::{Args, Subcommand};

use crate::ui::UiContext;

// ============================================================
// TYPES
// ============================================================

/// Arguments for the code command.
#[derive(Args)]
pub struct CodeArgs {
    /// Code workflow subcommand to execute.
    #[command(subcommand)]
    pub command: CodeCommand,
}

/// Code workflow subcommands (mirrors Code page tabs in UI).
#[derive(Subcommand)]
pub enum CodeCommand {
    /// Create or manage feature specifications
    #[command(
        long_about = "Create or manage feature specifications.\n\n\
                      Specifications are the source of truth for AI-driven development. \
                      They define what needs to be built, the requirements, and acceptance criteria.\n\n\
                      Subcommands: new, list, validate, clarify, design, init, tasks",
        after_help = "Examples:\n\
                      # Create a new specification\n\
                      ckrv code spec new \"Add user authentication\"\n\n\
                      # List all specifications\n\
                      ckrv code spec list\n\n\
                      # Validate a specification\n\
                      ckrv code spec validate my-feature"
    )]
    Spec(super::spec::SpecArgs),

    /// Generate implementation tasks from a specification
    #[command(
        long_about = "Generate implementation tasks from a specification.\n\n\
                      Analyzes the specification and produces a structured task breakdown \
                      that can be used for planning and execution.\n\n\
                      This is a convenience alias for `ckrv code spec tasks`.",
        after_help = "Examples:\n\
                      # Generate tasks for auto-detected spec\n\
                      ckrv code tasks\n\n\
                      # Generate tasks for a specific spec\n\
                      ckrv code tasks path/to/spec\n\n\
                      # Force regeneration\n\
                      ckrv code tasks --force"
    )]
    Tasks {
        /// Path to the spec file (optional - auto-detects from current branch if not provided)
        spec: Option<PathBuf>,

        /// Force regeneration of tasks even if they exist
        #[arg(short, long)]
        force: bool,
    },

    /// Generate execution plan from tasks (in Docker)
    #[command(
        long_about = "Generate execution plan from tasks using AI.\n\n\
                      Analyzes the specification and tasks file to create a detailed \
                      implementation plan. Runs in a Docker container for isolation.\n\n\
                      The plan breaks down work into atomic steps that AI agents can execute.",
        after_help = "Examples:\n\
                      # Generate plan for auto-detected spec\n\
                      ckrv code plan\n\n\
                      # Generate plan for a specific spec\n\
                      ckrv code plan my-feature\n\n\
                      # Force regeneration\n\
                      ckrv code plan --force"
    )]
    Plan(super::plan::PlanArgs),

    /// Run a job based on a specification
    #[command(
        long_about = "Run a job based on a specification.\n\n\
                      Executes the implementation plan using AI agents in isolated Docker sandboxes. \
                      Each task is executed in sequence with full logging and progress tracking.\n\n\
                      Results are committed to a feature branch for review.",
        after_help = "Examples:\n\
                      # Run all tasks for auto-detected spec\n\
                      ckrv code run\n\n\
                      # Run with specific agent\n\
                      ckrv code run my-feature --agent claude\n\n\
                      # Run with cost optimization\n\
                      ckrv code run --optimize cost"
    )]
    Run(super::run::RunArgs),

    /// View changes between current branch and base
    #[command(
        long_about = "View changes between current branch and base.\n\n\
                      Shows a summary of modified, added, and deleted files compared to the \
                      base branch. Helps verify what will be included in a pull request.\n\n\
                      Output can be formatted as JSON for programmatic use.",
        after_help = "Examples:\n\
                      # Show diff summary\n\
                      ckrv code diff\n\n\
                      # Show diff against specific branch\n\
                      ckrv code diff --base main\n\n\
                      # Output as JSON\n\
                      ckrv code diff --json"
    )]
    Diff(super::diff::DiffArgs),
}

// ============================================================
// IMPLEMENTATION
// ============================================================

/// Execute the code command by delegating to existing handlers.
pub async fn execute(args: CodeArgs, json: bool, ui: &UiContext) -> anyhow::Result<()> {
    match args.command {
        CodeCommand::Spec(spec_args) => super::spec::execute(spec_args, json, ui).await,
        CodeCommand::Tasks { spec, force } => {
            // Thin alias: delegate to spec tasks
            let spec_args = super::spec::SpecArgs {
                command: super::spec::SpecCommand::Tasks { spec, force },
            };
            super::spec::execute(spec_args, json, ui).await
        }
        CodeCommand::Plan(plan_args) => super::plan::execute(plan_args, json, ui).await,
        CodeCommand::Run(run_args) => super::run::execute(run_args, json, ui).await,
        CodeCommand::Diff(diff_args) => super::diff::execute(diff_args, json, ui).await,
    }
}
