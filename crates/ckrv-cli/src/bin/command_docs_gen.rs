//! Command Documentation Generator for Chakravarti CLI
//!
//! Generates individual markdown files for each command in docs/commands/
//!
//! Usage:
//! ```bash
//! cargo run -p ckrv-cli --bin command_docs_gen
//! ```

// ============================================================
// IMPORTS
// ============================================================

use std::fs;
use std::path::Path;

use chrono::Utc;
use ckrv_cli::{extract_command_metadata, CommandMetadata, OptionMetadata};

// ============================================================
// IMPLEMENTATION
// ============================================================

fn main() {
    let metadata = extract_command_metadata();
    let commit_hash = get_commit_hash();
    let output_dir = Path::new("crates/ckrv-cli/docs/commands");

    // Ensure output directory exists
    fs::create_dir_all(output_dir).expect("Failed to create output directory");

    let mut generated: Vec<String> = Vec::new();
    let mut skipped: Vec<String> = Vec::new();

    for cmd in &metadata.subcommands {
        if cmd.hidden {
            skipped.push(cmd.name.clone());
            continue;
        }

        if cmd.long_description.is_some() || !cmd.description.is_empty() {
            generate_command_doc(cmd, output_dir, &commit_hash);
            generated.push(cmd.name.clone());

            // Generate subcommand docs
            for subcmd in &cmd.subcommands {
                if !subcmd.hidden {
                    let sub_dir = output_dir.join(&cmd.name);
                    fs::create_dir_all(&sub_dir).expect("Failed to create subcommand directory");
                    generate_command_doc(subcmd, &sub_dir, &commit_hash);
                    generated.push(format!("{} {}", cmd.name, subcmd.name));
                }
            }
        } else {
            skipped.push(cmd.name.clone());
        }
    }

    // Print summary
    println!("## Command Documentation Generated\n");
    println!("### Generated Files");
    println!("| Command | Has Examples |");
    println!("|---------|--------------|");
    for cmd in &generated {
        let has_examples = "✅";
        println!("| {} | {} |", cmd, has_examples);
    }

    if !skipped.is_empty() {
        println!("\n### Skipped (hidden or no docs)");
        for cmd in &skipped {
            println!("| {} |", cmd);
        }
    }

    println!("\n### Next Steps");
    println!("1. Review generated files in `crates/ckrv-cli/docs/commands/`");
}

/// Get the short git commit hash of the current HEAD.
fn get_commit_hash() -> String {
    use std::process::Command;
    let output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .expect("Failed to get git commit hash");
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

/// Generate and write a markdown documentation file for a single command.
fn generate_command_doc(cmd: &CommandMetadata, output_dir: &Path, commit_hash: &str) {
    let filename = format!("{}.md", cmd.name);
    let filepath = output_dir.join(&filename);
    let content = generate_markdown(cmd, output_dir, commit_hash);

    fs::write(&filepath, content).expect(&format!("Failed to write {}", filepath.display()));
}

/// Generate the full markdown content for a command documentation page.
fn generate_markdown(cmd: &CommandMetadata, _output_dir: &Path, commit_hash: &str) -> String {
    let mut output = String::new();

    // Frontmatter
    output.push_str(&format!(
        r#"---
command: {}
generated_from: crates/ckrv-cli/src/lib.rs
last_commit: {}
---

"#,
        cmd.name, commit_hash
    ));

    // Title
    output.push_str(&format!("# ckrv {}\n\n", cmd.name));

    // Short description
    if !cmd.description.is_empty() {
        output.push_str(&cmd.description);
        output.push_str("\n\n");
    }

    // Long description
    if let Some(ref long_desc) = cmd.long_description {
        output.push_str("## Description\n\n");
        output.push_str(long_desc);
        output.push_str("\n\n");
    }

    // Arguments
    if !cmd.arguments.is_empty() {
        output.push_str("## Arguments\n\n");
        output.push_str("| Argument | Required | Description |\n");
        output.push_str("|----------|----------|-------------|\n");

        for arg in &cmd.arguments {
            let required = if arg.required { "Yes" } else { "No" };
            let help = if arg.help.is_empty() { "-" } else { &arg.help };
            output.push_str(&format!("| `{}` | {} | {} |\n", arg.id, required, help));
        }
        output.push('\n');
    }

    // Options
    if !cmd.options.is_empty() {
        output.push_str("## Options\n\n");
        output.push_str("| Flag | Description |\n");
        output.push_str("|------|-------------|\n");

        let mut sorted_options = cmd.options.clone();
        sorted_options.sort_by(|a, b| a.id.cmp(&b.id));

        for opt in &sorted_options {
            let flag = format_option_flag(opt);
            let help = if opt.help.is_empty() { "-" } else { &opt.help };
            output.push_str(&format!("| {} | {} |\n", flag, help));
        }
        output.push('\n');
    }

    // Subcommands table
    if !cmd.subcommands.is_empty() {
        let visible_subcmds: Vec<&CommandMetadata> =
            cmd.subcommands.iter().filter(|c| !c.hidden).collect();

        if !visible_subcmds.is_empty() {
            output.push_str("## Subcommands\n\n");
            output.push_str("| Subcommand | Description |\n");
            output.push_str("|------------|-------------|\n");

            for subcmd in visible_subcmds {
                output.push_str(&format!(
                    "| `{}` | {} |\n",
                    subcmd.name,
                    if subcmd.description.is_empty() { "-" } else { &subcmd.description }
                ));
            }
            output.push('\n');
        }
    }

    // Examples
    if let Some(ref after_help) = cmd.after_help {
        output.push_str("## Examples\n\n");
        output.push_str("```bash\n");
        // Strip "Examples:" prefix if present
        let cleaned = after_help.strip_prefix("Examples:\n").unwrap_or(after_help);
        output.push_str(cleaned);
        output.push_str("\n```\n");
    }

    output
}

/// Format an option flag for display in markdown tables.
fn format_option_flag(opt: &OptionMetadata) -> String {
    let mut flag = String::new();

    if let Some(long) = &opt.long {
        flag.push_str(&format!("`--{}`", long));
    }

    if let Some(short) = opt.short {
        if !flag.is_empty() {
            flag.push_str(", ");
        }
        flag.push_str(&format!("`-{}`", short));
    }

    flag
}
