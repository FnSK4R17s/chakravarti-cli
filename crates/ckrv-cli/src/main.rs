//! Chakravarti CLI - Spec-driven agent orchestration engine.
//!
//! This binary provides the `ckrv` command-line interface.

use clap::{Parser, Subcommand};

mod commands;

/// Chakravarti CLI - Spec-driven agent orchestration engine
#[derive(Parser)]
#[command(name = "ckrv")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Output format: JSON instead of human-readable
    #[arg(long, global = true)]
    json: bool,

    /// Suppress non-essential output
    #[arg(short, long, global = true)]
    quiet: bool,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize Chakravarti in the current repository
    Init(commands::init::InitArgs),

    /// Create or manage feature specifications
    Spec(commands::spec::SpecArgs),

    /// Run a job based on a specification
    Run(commands::run::RunArgs),

    /// Check the status of a job
    Status(commands::status::StatusArgs),

    /// View the diff produced by a job
    Diff(commands::diff::DiffArgs),

    /// View the metrics report for a job
    Report(commands::report::ReportArgs),

    /// Promote a successful job's changes to a branch
    Promote(commands::promote::PromoteArgs),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize tracing based on verbosity
    let filter = if cli.verbose {
        "debug"
    } else if cli.quiet {
        "error"
    } else {
        "info"
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .init();

    match cli.command {
        Commands::Init(args) => commands::init::execute(args, cli.json),
        Commands::Spec(args) => commands::spec::execute(args, cli.json),
        Commands::Run(args) => commands::run::execute(args, cli.json),
        Commands::Status(args) => commands::status::execute(args, cli.json),
        Commands::Diff(args) => commands::diff::execute(args, cli.json),
        Commands::Report(args) => commands::report::execute(args, cli.json),
        Commands::Promote(args) => commands::promote::execute(args, cli.json),
    }
}
