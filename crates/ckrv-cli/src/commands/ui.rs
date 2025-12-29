use clap::Args;
use ckrv_ui::start_server;
use crate::ui::UiContext;

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

        // Attempt to open the browser
        let url = format!("http://localhost:{}", args.port);
        if let Err(e) = open::that(&url) {
            ui.markdown(&format!("*Failed to open browser automatically: {}*", e));
            ui.markdown(&format!("Please visit **{}** manually", url));
        } else {
             ui.success("Browser", "Opened automatically");
        }
    } else {
        println!(r#"{{"status": "starting", "port": {}}}"#, args.port);
    }

    // This will block until the server stops
    start_server(args.port).await.map_err(|e| anyhow::anyhow!("UI Server error: {}", e))?;

    Ok(())
}
