//! # UI Command
//!
//! Launches the Chakravarti web-based user interface.
//!
//! ## Overview
//!
//! This command starts a local web server that serves the interactive
//! dashboard for managing specs, tasks, and agent execution. The UI
//! provides real-time updates via WebSocket connections.
//!
//! ## Usage
//!
//! ```bash
//! ckrv ui           # Start on default port 3000
//! ckrv ui --port 8080  # Start on custom port
//! ```

use crate::ui::UiContext;
use ckrv_ui::start_server;
use clap::Args;

/// Arguments for the UI command.
#[derive(Args, Debug)]
pub struct UiArgs {
    /// Port to listen on (default: 3000)
    #[arg(long, default_value = "3000")]
    port: u16,
}

/// Execute the UI command, starting the web dashboard server.
pub async fn execute(args: UiArgs, json: bool, ui: &UiContext) -> anyhow::Result<()> {
    if json {
        println!(r#"{{"status": "starting", "port": {}}}"#, args.port);
    } else {
        ui.success("Web UI", &format!("Starting on port {}...", args.port));
        ui.markdown("**Press Ctrl+C to stop**");
        ui.markdown(&format!(
            "Visit **http://localhost:{}** in your browser",
            args.port
        ));
    }

    // This will block until the server stops
    start_server(args.port)
        .await
        .map_err(|e| anyhow::anyhow!("UI Server error: {}", e))?;

    Ok(())
}
