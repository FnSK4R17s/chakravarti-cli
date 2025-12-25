//! Chakravarti CLI - Spec-driven agent orchestration engine.
//!
//! This binary provides the `ckrv` command-line interface.

use clap::{Parser, Subcommand};

mod commands;
pub mod ui;

use ui::{UiContext, Renderable};
use ui::components::Banner;

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
    command: Option<Commands>,
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

    // Initialize UI Context
    let ui = UiContext::new(cli.json);

    match cli.command {
        Some(Commands::Init(args)) => commands::init::execute(args, cli.json, &ui),
        Some(Commands::Spec(args)) => commands::spec::execute(args, cli.json),
        Some(Commands::Run(args)) => commands::run::execute(args, cli.json, &ui),
        Some(Commands::Status(args)) => commands::status::execute(args, cli.json, &ui),
        Some(Commands::Diff(args)) => commands::diff::execute(args, cli.json),
        Some(Commands::Report(args)) => commands::report::execute(args, cli.json),
        Some(Commands::Promote(args)) => commands::promote::execute(args, cli.json),
        None => {
            // No command provided: Print Branding + Help
            let banner = Banner::new("CHAKRAVARTI")
                .subtitle("Spec-driven Agent Orchestration");
            
            ui.print(banner);
            
            // Print Help using clap's built-in mechanism
            use clap::CommandFactory;
            Cli::command().print_help()?;
            Ok(())
        }
    }
}
