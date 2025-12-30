//! Cloud whoami command - Display current user identity.

use clap::Args;

/// Arguments for the whoami command
#[derive(Debug, Args)]
pub struct WhoamiArgs {
    /// Output as JSON
    #[arg(long, default_value = "false")]
    pub json: bool,
}

/// Execute the whoami command
pub async fn execute(args: WhoamiArgs, ui: &crate::ui::UiContext) -> anyhow::Result<()> {
    let client = crate::cloud::client::CloudClient::new();
    
    let client = match client {
        Ok(c) => c,
        Err(e) => {
            if e.to_string().contains("Not authenticated") {
                ui.error("Not Logged In", "Run 'ckrv cloud login' to authenticate.");
                return Ok(());
            }
            return Err(anyhow::anyhow!("{}", e));
        }
    };
    
    let user = match client.get_current_user().await {
        Ok(user) => user,
        Err(e) => {
            if e.to_string().contains("Not authenticated") {
                ui.error("Not Logged In", "Run 'ckrv cloud login' to authenticate.");
                return Ok(());
            }
            return Err(anyhow::anyhow!("{}", e));
        }
    };
    
    if args.json {
        println!("{}", serde_json::to_string_pretty(&user)?);
    } else {
        println!("Email: {}", user.email);
        if let Some(name) = &user.name {
            println!("Name: {}", name);
        }
        println!("Subscription: {}", user.subscription_tier);
        println!("Jobs remaining: {}", user.job_quota_remaining);
    }
    
    Ok(())
}
