//! SKILL.md Generator for Chakravarti CLI
//!
//! This binary generates an Agent Skills compatible SKILL.md file from the CLI's
//! clap command definitions. The generated file can be validated with `agentskills validate`.
//!
//! Usage:
//! ```bash
//! cargo run -p ckrv-cli --bin skill_gen > .agent/skills/chakravarti-cli/SKILL.md
//! ```
#![allow(clippy::format_push_string)]

// ============================================================
// IMPORTS
// ============================================================

use chrono::Utc;
use ckrv_cli::{extract_command_metadata, CommandMetadata, OptionMetadata};

// ============================================================
// IMPLEMENTATION
// ============================================================

fn main() {
    let metadata = extract_command_metadata();
    let skill_md = generate_skill_md(&metadata);
    print!("{skill_md}");
}

/// Generate the complete SKILL.md content from command metadata.
fn generate_skill_md(metadata: &CommandMetadata) -> String {
    let mut output = String::new();

    // Generate frontmatter
    output.push_str(&generate_frontmatter());
    output.push('\n');

    // Title and overview
    output.push_str("# Chakravarti CLI\n\n");
    output.push_str(&metadata.description);
    output.push_str("\n\n");

    // Commands
    output.push_str("## Commands\n\n");

    // Get visible commands sorted by display order (we use name as fallback)
    let mut visible_commands: Vec<&CommandMetadata> = metadata
        .subcommands
        .iter()
        .filter(|cmd| !cmd.hidden)
        .collect();

    // Sort alphabetically for deterministic output
    visible_commands.sort_by(|a, b| a.name.cmp(&b.name));

    for cmd in visible_commands {
        output.push_str(&generate_command_section(cmd, "ckrv", 3));
        output.push('\n');
    }

    // Global Options
    output.push_str(&generate_global_options_section());

    output
}

/// Generate YAML frontmatter with name, description, and version metadata.
fn generate_frontmatter() -> String {
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ");

    format!(
        r#"---
name: chakravarti-cli
description: Spec-driven agent orchestration. Create specs, plan tasks, run jobs, and review changes.
license: MIT
compatibility: Claude Code, Cursor, any CLI-capable agent
metadata:
  version: "{}"
  auto-generated: true
  generated-at: "{now}"
---
"#,
        env!("CARGO_PKG_VERSION")
    )
}

/// Generate the global options table section.
fn generate_global_options_section() -> String {
    r"## Global Options

These options apply to all commands:

| Flag | Description |
|------|-------------|
| `--json` | Output format: JSON instead of human-readable |
| `--quiet, -q` | Suppress non-essential output |
| `--verbose, -v` | Enable verbose logging |
| `--help, -h` | Print help |
| `--version, -V` | Print version |
"
    .to_string()
}

/// Generate a command section with its subcommands recursively.
fn generate_command_section(
    cmd: &CommandMetadata,
    parent_path: &str,
    heading_level: usize,
) -> String {
    let mut output = String::new();
    let heading = "#".repeat(heading_level);
    let full_path = format!("{parent_path} {}", cmd.name);

    // Command heading
    output.push_str(&format!("{heading} {full_path}\n\n"));

    // Description - prefer long_description if available
    if let Some(ref long_desc) = cmd.long_description {
        output.push_str(long_desc);
        output.push_str("\n\n");
    } else if !cmd.description.is_empty() {
        output.push_str(&cmd.description);
        output.push_str("\n\n");
    }

    // Usage
    output.push_str("```bash\n");
    output.push_str(&full_path);

    // Add argument placeholders
    for arg in &cmd.arguments {
        if arg.required {
            output.push_str(&format!(" <{}>", arg.id.to_uppercase()));
        } else {
            output.push_str(&format!(" [{}]", arg.id.to_uppercase()));
        }
    }

    if !cmd.options.is_empty() {
        output.push_str(" [OPTIONS]");
    }

    output.push_str("\n```\n\n");

    // Arguments table
    if !cmd.arguments.is_empty() {
        output.push_str("**Arguments**:\n\n");
        output.push_str("| Name | Required | Description |\n");
        output.push_str("|------|----------|-------------|\n");

        for arg in &cmd.arguments {
            let required = if arg.required { "Yes" } else { "No" };
            let help = if arg.help.is_empty() { "-" } else { &arg.help };
            output.push_str(&format!("| `{}` | {} | {} |\n", arg.id, required, help));
        }
        output.push('\n');
    }

    // Options table
    if !cmd.options.is_empty() {
        output.push_str("**Options**:\n\n");
        output.push_str("| Flag | Description |\n");
        output.push_str("|------|-------------|\n");

        // Sort options alphabetically for determinism
        let mut sorted_options = cmd.options.clone();
        sorted_options.sort_by(|a, b| a.id.cmp(&b.id));

        for opt in &sorted_options {
            let flag = format_option_flag(opt);
            let help = if opt.help.is_empty() { "-" } else { &opt.help };
            output.push_str(&format!("| {} | {} |\n", flag, help));
        }
        output.push('\n');
    }

    // Examples/Notes from after_help
    if let Some(ref after_help) = cmd.after_help {
        output.push_str("**Examples**:\n\n");
        // Strip "Examples:" prefix if present
        let cleaned = after_help.strip_prefix("Examples:\n").unwrap_or(after_help);
        output.push_str(cleaned);
        output.push_str("\n\n");
    }

    // Subcommands
    if !cmd.subcommands.is_empty() {
        let mut visible_subcmds: Vec<&CommandMetadata> =
            cmd.subcommands.iter().filter(|c| !c.hidden).collect();

        // Sort alphabetically
        visible_subcmds.sort_by(|a, b| a.name.cmp(&b.name));

        for subcmd in visible_subcmds {
            output.push_str(&generate_command_section(
                subcmd,
                &full_path,
                heading_level + 1,
            ));
        }
    }

    output
}

/// Format an option flag for display in markdown tables.
fn format_option_flag(opt: &OptionMetadata) -> String {
    let mut flag = String::new();

    if let Some(long) = &opt.long {
        flag.push_str(&format!("`--{long}`"));
    }

    if let Some(short) = opt.short {
        if !flag.is_empty() {
            flag.push_str(", ");
        }
        flag.push_str(&format!("`-{short}`"));
    }

    if opt.takes_value && opt.value_type != "FLAG" {
        flag.push_str(&format!(" <{}>", opt.value_type));
    }

    flag
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frontmatter_contains_required_fields() {
        let frontmatter = generate_frontmatter();

        assert!(frontmatter.contains("name: chakravarti-cli"));
        assert!(frontmatter.contains("description:"));
        assert!(frontmatter.contains("auto-generated: true"));
    }

    #[test]
    fn test_skill_md_excludes_hidden_commands() {
        let metadata = extract_command_metadata();
        let skill_md = generate_skill_md(&metadata);

        // Hidden commands should not appear
        assert!(
            !skill_md.contains("### ckrv task"),
            "task should not appear"
        );
        assert!(
            !skill_md.contains("### ckrv status"),
            "status should not appear"
        );
        assert!(
            !skill_md.contains("### ckrv report"),
            "report should not appear"
        );
    }

    #[test]
    fn test_skill_md_includes_visible_commands() {
        let metadata = extract_command_metadata();
        let skill_md = generate_skill_md(&metadata);

        // Visible commands should appear (spec/plan/run are nested under "code")
        assert!(skill_md.contains("### ckrv init"), "init should appear");
        assert!(skill_md.contains("### ckrv code"), "code should appear");
        assert!(skill_md.contains("ckrv code spec"), "spec should appear");
        assert!(skill_md.contains("ckrv code plan"), "plan should appear");
        assert!(skill_md.contains("ckrv code run"), "run should appear");
    }

    #[test]
    fn test_skill_md_has_global_options() {
        let metadata = extract_command_metadata();
        let skill_md = generate_skill_md(&metadata);

        assert!(skill_md.contains("## Global Options"));
        assert!(skill_md.contains("--json"));
        assert!(skill_md.contains("--quiet"));
        assert!(skill_md.contains("--verbose"));
    }
}
