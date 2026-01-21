use crate::ui::UiContext;
use ckrv_ui::start_server;
use clap::Args;

#[derive(Args, Debug)]
pub struct UiArgs {
    /// Port to listen on (default: 3000)
    #[arg(long, default_value = "3000")]
    port: u16,
}

pub async fn execute(args: UiArgs, json: bool, ui: &UiContext) -> anyhow::Result<()> {
    if !json {
        ui.success("Web UI", &format!("Starting on port {}...", args.port));
        ui.markdown("**Press Ctrl+C to stop**");
        ui.markdown(&format!(
            "Visit **http://localhost:{}** in your browser",
            args.port
        ));
    } else {
        println!(r#"{{"status": "starting", "port": {}}}"#, args.port);
    }

    // This will block until the server stops
    start_server(args.port)
        .await
        .map_err(|e| anyhow::anyhow!("UI Server error: {}", e))?;

    Ok(())
}
