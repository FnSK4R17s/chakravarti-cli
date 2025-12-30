//! Cloud login command - OAuth2 device flow authentication.

use clap::Args;

/// Arguments for the login command
#[derive(Debug, Args)]
pub struct LoginArgs {
    /// Skip opening browser automatically
    #[arg(long, default_value = "false")]
    pub no_browser: bool,
}

/// Execute the login command
pub async fn execute(args: LoginArgs, ui: &crate::ui::UiContext) -> anyhow::Result<()> {
    use crate::cloud::auth::DeviceAuthFlow;
    
    println!("Starting authentication with Chakravarti Cloud...");
    
    let auth_flow = DeviceAuthFlow::new()?;
    let device_code = auth_flow.request_device_code().await?;
    
    println!("Visit: {}", device_code.verification_uri);
    println!("Enter code: {}", device_code.user_code);
    
    if !args.no_browser {
        if let Err(e) = open::that(&device_code.verification_uri) {
            eprintln!("Could not open browser automatically: {}", e);
        }
    }
    
    println!("Waiting for authorization...");
    
    let tokens = auth_flow.poll_for_token(&device_code).await?;
    
    // Store tokens securely
    crate::cloud::credentials::store_tokens(&tokens)?;
    
    // Fetch and display user info
    let client = crate::cloud::client::CloudClient::new()?;
    let user = client.get_current_user().await?;
    
    ui.success("Login Successful", &format!("Authenticated as {}", user.email));
    println!("Subscription: {} ({} jobs remaining)", 
        user.subscription_tier, 
        user.job_quota_remaining
    );
    
    Ok(())
}
