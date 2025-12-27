//! Chakravarti CLI - Spec-driven agent orchestration engine.
//!
//! This binary provides the `ckrv` command-line interface.

use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};

mod commands;
pub mod ui;

use ui::components::Banner;
use ui::{Renderable, UiContext};

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
    #[command(display_order = 1)]
    Init(commands::init::InitArgs),

    /// Create or manage feature specifications
    #[command(display_order = 2)]
    Spec(commands::spec::SpecArgs),

    /// Run a job based on a specification
    #[command(display_order = 3)]
    Run(commands::run::RunArgs),

    /// View the diff produced by a job
    #[command(display_order = 4)]
    Diff(commands::diff::DiffArgs),

    /// Promote a successful job's changes to a branch
    #[command(display_order = 5)]
    Promote(commands::promote::PromoteArgs),

    /// Execute a workflow-based agent task
    #[command(hide = true)]
    Task(commands::task::TaskArgs),

    /// Check the status of a job
    #[command(hide = true)]
    Status(commands::status::StatusArgs),

    /// View the metrics report for a job
    #[command(hide = true)]
    Report(commands::report::ReportArgs),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // We want the banner to show up on --help and --version too.
    let temp_ui = UiContext::new(false);
    let banner_struct = Banner::new("CHAKRAVARTI").subtitle("Spec-driven Agent Orchestration");
    let banner_str = banner_struct.render(&temp_ui.theme);

    // Build the clap command manually to inject the banner
    let command = Cli::command();
    let command = command.before_help(banner_str);

    // Parse matches
    let matches = command.get_matches();
    // Convert back to Cli struct
    let cli = Cli::from_arg_matches(&matches)?;

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
        Some(Commands::Init(args)) => commands::init::execute(args, cli.json, &ui).await,
        Some(Commands::Spec(args)) => commands::spec::execute(args, cli.json, &ui).await,
        Some(Commands::Run(args)) => commands::run::execute(args, cli.json, &ui).await,
        Some(Commands::Task(args)) => commands::task::execute(args, cli.json, &ui).await,
        Some(Commands::Status(args)) => commands::status::execute(args, cli.json, &ui).await,
        Some(Commands::Diff(args)) => commands::diff::execute(args, cli.json).await,
        Some(Commands::Report(args)) => commands::report::execute(args, cli.json).await,
        Some(Commands::Promote(args)) => commands::promote::execute(args, cli.json).await,
        None => {
            use clap::CommandFactory;
            let mut cmd = Cli::command();
            let banner = Banner::new("CHAKRAVARTI").subtitle("Spec-driven Agent Orchestration");
            cmd = cmd.before_help(banner.render(&ui.theme));
            cmd.print_help()?;
            Ok(())
        }
    }
}
