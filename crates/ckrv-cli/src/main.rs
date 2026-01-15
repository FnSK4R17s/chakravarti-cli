//! Chakravarti CLI - Spec-driven agent orchestration engine.
//!
//! This binary provides the `ckrv` command-line interface.

use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};

mod cloud;
mod commands;
mod prompts;
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
        Some(Commands::Plan(args)) => commands::plan::execute(args, cli.json, &ui).await,
        Some(Commands::Run(args)) => commands::run::execute(args, cli.json, &ui).await,
        Some(Commands::Task(args)) => commands::task::execute(args, cli.json, &ui).await,
        Some(Commands::Status(args)) => commands::status::execute(args, cli.json, &ui).await,
        Some(Commands::Diff(args)) => commands::diff::execute(args, cli.json, &ui).await,
        Some(Commands::Verify(args)) => commands::verify::execute(args, cli.json, &ui).await,
        Some(Commands::Report(args)) => commands::report::execute(args, cli.json).await,
        Some(Commands::Promote(args)) => commands::promote::execute(args, cli.json, &ui).await,
        Some(Commands::Fix(args)) => commands::fix::execute(args, cli.json, &ui).await,
        Some(Commands::Ui(args)) => commands::ui::execute(args, cli.json, &ui).await,
        Some(Commands::Cloud(args)) => commands::cloud::execute(args, &ui).await,
        Some(Commands::Logs(args)) => commands::logs::execute(args, &ui).await,
        Some(Commands::Pull(args)) => commands::pull::execute(args, &ui).await,
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
