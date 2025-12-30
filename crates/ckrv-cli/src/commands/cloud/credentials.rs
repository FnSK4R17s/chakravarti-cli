//! Cloud credentials command - Manage git credentials for private repos.

use clap::{Args, Subcommand};

/// Arguments for the credentials command
#[derive(Debug, Args)]
pub struct CredentialsArgs {
    #[command(subcommand)]
    pub command: CredentialsCommand,
}

/// Available credentials subcommands
#[derive(Debug, Subcommand)]
pub enum CredentialsCommand {
    /// Add a new git credential
    Add(AddArgs),
    /// List stored credentials
    List(ListArgs),
    /// Remove a credential
    Remove(RemoveArgs),
}

/// Arguments for adding a credential
#[derive(Debug, Args)]
pub struct AddArgs {
    /// Name for this credential (e.g., "github-work")
    #[arg(long)]
    pub name: String,
    
    /// Git provider (github, gitlab, bitbucket, generic)
    #[arg(long, default_value = "github")]
    pub provider: String,
    
    /// Credential type (pat, deploy_key)
    #[arg(long, default_value = "pat")]
    pub credential_type: String,
}

/// Arguments for listing credentials
#[derive(Debug, Args)]
pub struct ListArgs {
    /// Output as JSON
    #[arg(long, default_value = "false")]
    pub json: bool,
}

/// Arguments for removing a credential
#[derive(Debug, Args)]
pub struct RemoveArgs {
    /// Name of the credential to remove
    pub name: String,
}

/// Execute the credentials command
pub async fn execute(args: CredentialsArgs, ui: &crate::ui::UiContext) -> anyhow::Result<()> {
    match args.command {
        CredentialsCommand::Add(add_args) => execute_add(add_args, ui).await,
        CredentialsCommand::List(list_args) => execute_list(list_args, ui).await,
        CredentialsCommand::Remove(remove_args) => execute_remove(remove_args, ui).await,
    }
}

async fn execute_add(args: AddArgs, ui: &crate::ui::UiContext) -> anyhow::Result<()> {
    use std::io::{self, Write};
    
    println!("Adding credential '{}' for {}", args.name, args.provider);
    
    // Prompt for token securely
    print!("Enter token: ");
    io::stdout().flush()?;
    
    let token = rpassword::read_password()?;
    
    if token.is_empty() {
        anyhow::bail!("Token cannot be empty");
    }
    
    let client = crate::cloud::client::CloudClient::new().map_err(|e| anyhow::anyhow!("{}", e))?;
    client.add_credential(&args.name, &args.provider, &args.credential_type, &token).await
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    
    ui.success("Credential Added", &format!("Credential '{}' saved", args.name));
    
    Ok(())
}

async fn execute_list(args: ListArgs, ui: &crate::ui::UiContext) -> anyhow::Result<()> {
    let client = crate::cloud::client::CloudClient::new().map_err(|e| anyhow::anyhow!("{}", e))?;
    let credentials = client.list_credentials().await.map_err(|e| anyhow::anyhow!("{}", e))?;
    
    if credentials.is_empty() {
        println!("No credentials stored. Use 'ckrv cloud credentials add' to add one.");
        return Ok(());
    }
    
    if args.json {
        println!("{}", serde_json::to_string_pretty(&credentials)?);
    } else {
        println!("Stored credentials:");
        for cred in credentials {
            println!("  {} ({}) - {}", cred.name, cred.provider, cred.credential_type);
        }
    }
    
    let _ = ui; // Suppress unused warning
    Ok(())
}

async fn execute_remove(args: RemoveArgs, ui: &crate::ui::UiContext) -> anyhow::Result<()> {
    let client = crate::cloud::client::CloudClient::new().map_err(|e| anyhow::anyhow!("{}", e))?;
    client.remove_credential(&args.name).await.map_err(|e| anyhow::anyhow!("{}", e))?;
    
    ui.success("Credential Removed", &format!("Credential '{}' removed", args.name));
    
    Ok(())
}
