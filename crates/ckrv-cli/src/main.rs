//! Chakravarti CLI - Spec-driven agent orchestration engine.
//!
//! This binary provides the `ckrv` command-line interface.

use clap::{CommandFactory, FromArgMatches};

use ckrv_cli::ui::components::Banner;
use ckrv_cli::ui::{Renderable, UiContext};
use ckrv_cli::{Cli, Commands};

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
        Some(Commands::Init(args)) => ckrv_cli::init::execute(args, cli.json, &ui).await,
        Some(Commands::Code(args)) => ckrv_cli::code::execute(args, cli.json, &ui).await,
        Some(Commands::Spec(args)) => ckrv_cli::spec::execute(args, cli.json, &ui).await,
        Some(Commands::Plan(args)) => ckrv_cli::plan::execute(args, cli.json, &ui).await,
        Some(Commands::Run(args)) => ckrv_cli::run::execute(args, cli.json, &ui).await,
        Some(Commands::Task(args)) => ckrv_cli::task::execute(args, cli.json, &ui).await,
        Some(Commands::Status(args)) => ckrv_cli::status::execute(args, cli.json, &ui).await,
        Some(Commands::Diff(args)) => ckrv_cli::diff::execute(args, cli.json, &ui).await,
        Some(Commands::Verify(args)) => ckrv_cli::verify::execute(args, cli.json, &ui).await,
        Some(Commands::Report(args)) => ckrv_cli::report::execute(args, cli.json).await,
        Some(Commands::Promote(args)) => ckrv_cli::promote::execute(args, cli.json, &ui).await,
        Some(Commands::Fix(args)) => ckrv_cli::fix::execute(args, cli.json, &ui).await,
        Some(Commands::Ui(args)) => ckrv_cli::ui_cmd::execute(args, cli.json, &ui).await,
        Some(Commands::Cloud(args)) => ckrv_cli::cloud_cmd::execute(args, &ui).await,
        Some(Commands::Logs(args)) => ckrv_cli::logs::execute(args, &ui).await,
        Some(Commands::Pull(args)) => ckrv_cli::pull::execute(args, &ui).await,
        Some(Commands::Test(args)) => ckrv_cli::test::execute(args, cli.json, &ui).await,
        Some(Commands::Qa(args)) => ckrv_cli::qa::execute(args, cli.json, &ui).await,
        Some(Commands::Term(args)) => ckrv_cli::term::execute(args, cli.json, &ui).await,
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
