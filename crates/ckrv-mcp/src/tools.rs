//! Tool Discovery and Execution
//!
//! This module discovers CLI commands using clap's introspection API and
//! executes them by shelling out to the `ckrv` binary.

use crate::schema::build_json_schema;
use crate::types::{MCPTool, MCPToolAnnotations};
use ckrv_cli::{extract_command_metadata, CommandMetadata};
use serde_json::Value;
use std::process::Command;

/// Discover all available tools from CLI command metadata
#[must_use]
pub fn discover_tools() -> Vec<MCPTool> {
    let metadata = extract_command_metadata();
    let mut tools = Vec::new();

    // Recursively extract tools from command tree
    extract_tools_recursive(&metadata, &[], &mut tools);

    tools
}

/// Recursively extract tools from command metadata
fn extract_tools_recursive(
    cmd: &CommandMetadata,
    parent_path: &[String],
    tools: &mut Vec<MCPTool>,
) {
    // Skip hidden commands
    if cmd.hidden {
        return;
    }

    // Build full path
    let mut full_path = parent_path.to_vec();
    if cmd.name != "ckrv" {
        full_path.push(cmd.name.clone());
    }

    // If this command has subcommands, recurse into them
    if !cmd.subcommands.is_empty() {
        for subcmd in &cmd.subcommands {
            extract_tools_recursive(subcmd, &full_path, tools);
        }
    } else if !full_path.is_empty() {
        // Leaf command - create a tool for it
        let tool = create_tool_from_command(cmd, &full_path);
        tools.push(tool);
    }
}

/// Create an MCP tool from a leaf command
fn create_tool_from_command(cmd: &CommandMetadata, path: &[String]) -> MCPTool {
    // Build tool name: ckrv_spec_new, ckrv_plan, etc.
    let name = format!("ckrv_{}", path.join("_"));

    // Build input schema from command arguments and options
    let input_schema = build_json_schema(cmd);

    // Determine annotations based on command nature
    let annotations = infer_annotations(&name, cmd);

    MCPTool {
        name,
        description: cmd.description.clone(),
        input_schema,
        annotations,
    }
}

/// Infer tool annotations based on command characteristics
fn infer_annotations(name: &str, _cmd: &CommandMetadata) -> Option<MCPToolAnnotations> {
    // Read-only commands
    let read_only_patterns = ["_list", "_validate", "_diff", "_status", "_report"];

    // Destructive commands
    let destructive_patterns = [
        "_init", "_new", "_plan", "_run", "_fix", "_promote", "_submit",
    ];

    let is_read_only = read_only_patterns.iter().any(|p| name.ends_with(p));
    let is_destructive = destructive_patterns.iter().any(|p| name.contains(p));

    if is_read_only || is_destructive {
        Some(MCPToolAnnotations {
            read_only_hint: if is_read_only { Some(true) } else { None },
            destructive_hint: if is_destructive { Some(true) } else { None },
        })
    } else {
        None
    }
}

/// Convert tool name back to CLI command parts
/// e.g., "ckrv_spec_new" -> ["spec", "new"]
#[must_use]
pub fn parse_tool_name(name: &str) -> Vec<String> {
    name.strip_prefix("ckrv_")
        .unwrap_or(name)
        .split('_')
        .map(String::from)
        .collect()
}

/// Build CLI arguments from JSON arguments object
#[must_use]
pub fn build_cli_args(arguments: &Value, cmd: &CommandMetadata) -> Vec<String> {
    let mut args = Vec::new();

    if let Value::Object(map) = arguments {
        // First, add positional arguments in order
        for arg_meta in &cmd.arguments {
            if let Some(value) = map.get(&arg_meta.id) {
                if let Some(s) = value.as_str() {
                    args.push(s.to_string());
                } else if let Some(n) = value.as_i64() {
                    args.push(n.to_string());
                } else if let Some(b) = value.as_bool() {
                    args.push(b.to_string());
                }
            }
        }

        // Then, add options
        for opt_meta in &cmd.options {
            if let Some(value) = map.get(&opt_meta.id) {
                if let Some(long) = &opt_meta.long {
                    if opt_meta.takes_value {
                        if let Some(s) = value.as_str() {
                            args.push(format!("--{long}"));
                            args.push(s.to_string());
                        } else if let Some(n) = value.as_i64() {
                            args.push(format!("--{long}"));
                            args.push(n.to_string());
                        }
                    } else if value.as_bool().unwrap_or(false) {
                        args.push(format!("--{long}"));
                    }
                }
            }
        }
    }

    args
}

/// Execute a tool by shelling out to ckrv
///
/// # Errors
/// Returns an error if the command fails to execute or produces invalid output
pub fn execute_tool(tool_name: &str, arguments: &Value) -> Result<(String, bool), String> {
    // Parse tool name to get command parts
    let parts = parse_tool_name(tool_name);

    // Find the command metadata to build proper arguments
    let metadata = extract_command_metadata();
    let cmd_meta = find_command_metadata(&metadata, &parts);

    // Build the command
    let mut cmd = Command::new("ckrv");
    cmd.arg("--json");

    // Add command parts
    for part in &parts {
        cmd.arg(part);
    }

    // Add arguments if we found the command metadata
    if let Some(meta) = cmd_meta {
        let cli_args = build_cli_args(arguments, meta);
        for arg in cli_args {
            cmd.arg(arg);
        }
    }

    // Execute
    match cmd.output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            if output.status.success() {
                Ok((stdout, false))
            } else {
                // Return stderr or stdout as error message
                let error_msg = if stderr.is_empty() { stdout } else { stderr };
                Ok((error_msg, true))
            }
        }
        Err(e) => Err(format!("Failed to execute ckrv: {e}")),
    }
}

/// Find command metadata for a given path
fn find_command_metadata<'a>(
    root: &'a CommandMetadata,
    path: &[String],
) -> Option<&'a CommandMetadata> {
    if path.is_empty() {
        return Some(root);
    }

    let mut current = root;
    for part in path {
        current = current.subcommands.iter().find(|c| &c.name == part)?;
    }
    Some(current)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tool_name() {
        assert_eq!(parse_tool_name("ckrv_spec_new"), vec!["spec", "new"]);
        assert_eq!(parse_tool_name("ckrv_init"), vec!["init"]);
        assert_eq!(parse_tool_name("ckrv_test_run"), vec!["test", "run"]);
    }

    #[test]
    fn test_discover_tools_excludes_hidden() {
        let tools = discover_tools();
        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();

        // Hidden commands (task, status, report) should not be tools
        // Note: spec has a "tasks" subcommand which IS visible, so we check exact names
        assert!(!names.contains(&"ckrv_task"), "ckrv_task should be hidden");
        assert!(
            !names.contains(&"ckrv_status"),
            "ckrv_status should be hidden"
        );
        assert!(
            !names.contains(&"ckrv_report"),
            "ckrv_report should be hidden"
        );

        // But ckrv_spec_tasks should be visible
        assert!(
            names.contains(&"ckrv_spec_tasks"),
            "ckrv_spec_tasks should be visible"
        );
    }

    #[test]
    fn test_discover_tools_includes_visible() {
        let tools = discover_tools();
        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();

        // Core commands should be tools
        assert!(names.contains(&"ckrv_init"));
    }

    #[test]
    fn test_infer_annotations_read_only() {
        let cmd = CommandMetadata {
            path: vec![],
            name: "list".to_string(),
            description: "List items".to_string(),
            arguments: vec![],
            options: vec![],
            hidden: false,
            subcommands: vec![],
        };

        let annotations = infer_annotations("ckrv_spec_list", &cmd);
        assert!(annotations.is_some());
        assert_eq!(annotations.as_ref().unwrap().read_only_hint, Some(true));
    }

    #[test]
    fn test_infer_annotations_destructive() {
        let cmd = CommandMetadata {
            path: vec![],
            name: "new".to_string(),
            description: "Create new".to_string(),
            arguments: vec![],
            options: vec![],
            hidden: false,
            subcommands: vec![],
        };

        let annotations = infer_annotations("ckrv_spec_new", &cmd);
        assert!(annotations.is_some());
        assert_eq!(annotations.as_ref().unwrap().destructive_hint, Some(true));
    }
}
