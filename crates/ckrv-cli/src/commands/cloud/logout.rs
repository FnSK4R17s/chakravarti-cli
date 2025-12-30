//! Cloud logout command - Clear stored credentials.

use clap::Args;

/// Arguments for the logout command
#[derive(Debug, Args)]
pub struct LogoutArgs {
    /// Force logout without confirmation
    #[arg(long, short, default_value = "false")]
    pub force: bool,
}

/// Execute the logout command
pub async fn execute(args: LogoutArgs, ui: &crate::ui::UiContext) -> anyhow::Result<()> {
    if !args.force {
        println!("This will clear your stored cloud credentials.");
    }
    
    crate::cloud::credentials::clear_tokens()?;
    
    ui.success("Logout", "Logged out from Chakravarti Cloud");
    
    Ok(())
}
