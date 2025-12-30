//! Pull command - Download cloud job results.

use clap::Args;
use std::process::Command;

/// Arguments for the pull command
#[derive(Debug, Args)]
pub struct PullArgs {
    /// Job ID to pull results from
    pub job_id: String,
    
    /// Apply diff to current worktree (default: true)
    #[arg(long, default_value = "true")]
    pub apply: bool,
    
    /// Output diff to file instead of applying
    #[arg(long)]
    pub output: Option<String>,
}

/// Execute the pull command
pub async fn execute(args: PullArgs, ui: &crate::ui::UiContext) -> anyhow::Result<()> {
    use crate::cloud::client::CloudClient;
    
    let client = CloudClient::new().map_err(|e| anyhow::anyhow!("{}", e))?;
    
    // Check job status first
    let job = client.get_job(&args.job_id).await.map_err(|e| anyhow::anyhow!("{}", e))?;
    
    if job.status != "succeeded" {
        ui.error("Cannot Pull", &format!("Job status is '{}', cannot pull results", job.status));
        if let Some(ref error) = job.error_message {
            eprintln!("Error: {}", error);
        }
        return Ok(());
    }
    
    println!("Fetching diff for job {}...", args.job_id);
    
    // Get the diff
    let diff = client.get_job_diff(&args.job_id).await.map_err(|e| anyhow::anyhow!("{}", e))?;
    
    if let Some(output_path) = args.output {
        // Write to file
        std::fs::write(&output_path, &diff)?;
        ui.success("Diff Saved", &format!("Diff written to: {}", output_path));
    } else if args.apply {
        // Apply diff to current worktree
        println!("Applying diff to worktree...");
        
        let mut child = Command::new("git")
            .arg("apply")
            .arg("--verbose")
            .stdin(std::process::Stdio::piped())
            .spawn()?;
        
        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            stdin.write_all(diff.as_bytes())?;
        }
        
        let status = child.wait()?;
        
        if status.success() {
            ui.success("Diff Applied", "Changes applied successfully!");
            println!("Review with: git diff");
            println!("Commit with: git add . && git commit -m 'Apply cloud job results'");
        } else {
            ui.error("Apply Failed", "The changes may conflict with local modifications.");
            println!("You can save the diff and apply manually:");
            println!("  ckrv pull {} --output changes.patch", args.job_id);
            println!("  git apply changes.patch");
        }
    } else {
        // Just print the diff
        println!("{}", diff);
    }
    
    Ok(())
}
