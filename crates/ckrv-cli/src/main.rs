//! Chakravarti CLI - Spec-driven agent orchestration engine.
//!
//! This binary provides the `ckrv` command-line interface.

use clap::{Parser, Subcommand, CommandFactory, FromArgMatches};

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
    // We want the banner to show up on --help and --version too.
    // To do this, we must build the command, inject the banner, and then parse.
    
    // We need a temporary UI context to render the banner string first
    // Note: We don't know if json/silent is requested yet, but for Help, visual is priority.
    // We'll optimistically assume standard terminal mode for the banner generation.
    let temp_ui = UiContext::new(false);
    let banner_struct = Banner::new("CHAKRAVARTI").subtitle("Spec-driven Agent Orchestration");
    // Render the banner to a string
    // We use a helper from Renderable or just call render directly if we made it public?
    // Renderable::render(banner, theme)
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
        Some(Commands::Init(args)) => commands::init::execute(args, cli.json, &ui),
        Some(Commands::Spec(args)) => commands::spec::execute(args, cli.json),
        Some(Commands::Run(args)) => commands::run::execute(args, cli.json, &ui),
        Some(Commands::Status(args)) => commands::status::execute(args, cli.json, &ui),
        Some(Commands::Diff(args)) => commands::diff::execute(args, cli.json),
        Some(Commands::Report(args)) => commands::report::execute(args, cli.json),
        Some(Commands::Promote(args)) => commands::promote::execute(args, cli.json),
        None => {
            // No command provided: Print Help (includes banner now)
            // We use the same command builder to ensure consistency?
            // Actually Cli::command().print_help() might NOT include the dynamic modifications?
            // Yes, calling Cli::command() creates a NEW command builder.
            // We need to print help using the 'command' variable we created earlier?
            // But we consumed it with get_matches().
            
            // Re-inject banner for this specific case or just rely on clap
            use clap::CommandFactory;
            let mut cmd = Cli::command();
            let banner = Banner::new("CHAKRAVARTI").subtitle("Spec-driven Agent Orchestration");
            cmd = cmd.before_help(banner.render(&ui.theme));
            cmd.print_help()?;
            Ok(())
        }
    }
}
