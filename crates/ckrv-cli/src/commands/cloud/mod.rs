//! Cloud subcommand group for Chakravarti CLI.
//!
//! Provides commands for interacting with Chakravarti Cloud:
//! - `ckrv cloud login` - Authenticate with the cloud service
//! - `ckrv cloud logout` - Clear stored credentials
//! - `ckrv cloud whoami` - Display current user identity
//! - `ckrv cloud credentials` - Manage git credentials for private repos

use clap::{Args, Subcommand};

pub mod credentials;
pub mod login;
pub mod logout;
pub mod whoami;

/// Cloud service commands for remote job execution
#[derive(Debug, Args)]
pub struct CloudArgs {
    #[command(subcommand)]
    pub command: CloudCommand,
}

/// Available cloud subcommands
#[derive(Debug, Subcommand)]
pub enum CloudCommand {
    /// Authenticate with Chakravarti Cloud
    Login(login::LoginArgs),
    /// Clear stored cloud credentials
    Logout(logout::LogoutArgs),
    /// Display current authenticated user
    Whoami(whoami::WhoamiArgs),
    /// Manage git credentials for private repositories
    Credentials(credentials::CredentialsArgs),
}

/// Execute the cloud subcommand
pub async fn execute(args: CloudArgs, ui: &crate::ui::UiContext) -> anyhow::Result<()> {
    match args.command {
        CloudCommand::Login(args) => login::execute(args, ui).await,
        CloudCommand::Logout(args) => logout::execute(args, ui).await,
        CloudCommand::Whoami(args) => whoami::execute(args, ui).await,
        CloudCommand::Credentials(args) => credentials::execute(args, ui).await,
    }
}
