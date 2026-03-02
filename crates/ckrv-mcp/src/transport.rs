//! stdio Transport for MCP JSON-RPC.
//!
//! This module implements the stdio transport layer for the MCP server,
//! handling JSON-RPC 2.0 request/response over stdin/stdout.

// ============================================================
// IMPORTS
// ============================================================

use crate::tools::{discover_tools, execute_tool};
use crate::types::{MCPError, MCPRequest, MCPResponse};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

// ============================================================
// CONSTANTS
// ============================================================

/// MCP Server version.
const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// MCP Protocol version.
const PROTOCOL_VERSION: &str = "2024-11-05";

// ============================================================
// TRANSPORT LOOP
// ============================================================

/// Run the stdio transport loop
///
/// Reads JSON-RPC requests from stdin and writes responses to stdout.
pub fn run_stdio_transport() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let Ok(line) = line else {
            break;
        };

        // Skip empty lines
        if line.trim().is_empty() {
            continue;
        }

        // Handle the request
        if let Some(response) = handle_request_line(&line) {
            // Serialize and write response
            if let Ok(json_str) = serde_json::to_string(&response) {
                if writeln!(stdout, "{json_str}").is_err() {
                    break;
                }
                if stdout.flush().is_err() {
                    break;
                }
            }
        }
    }
}

// ============================================================
// REQUEST HANDLERS
// ============================================================

/// Handle a single request line.
fn handle_request_line(line: &str) -> Option<MCPResponse> {
    // Parse JSON
    let request: MCPRequest = match serde_json::from_str(line) {
        Ok(r) => r,
        Err(e) => {
            return Some(MCPResponse::error(
                Value::Null,
                MCPError::parse_error(Some(e.to_string())),
            ));
        }
    };

    // Get request ID (use Null for notifications)
    let id = request.id.clone().unwrap_or(Value::Null);

    // Handle the method
    match request.method.as_str() {
        "initialize" => Some(handle_initialize(id)),
        "initialized" => None, // Notification, no response
        "tools/list" => Some(handle_tools_list(id)),
        "tools/call" => Some(handle_tools_call(id, &request.params)),
        "ping" => Some(MCPResponse::success(id, json!({}))),
        _ => Some(MCPResponse::error(
            id,
            MCPError::method_not_found(&request.method),
        )),
    }
}

/// Handle the initialize method
fn handle_initialize(id: Value) -> MCPResponse {
    MCPResponse::success(
        id,
        json!({
            "protocolVersion": PROTOCOL_VERSION,
            "serverInfo": {
                "name": "chakravarti-mcp",
                "version": SERVER_VERSION
            },
            "capabilities": {
                "tools": {}
            }
        }),
    )
}

/// Handle the tools/list method
fn handle_tools_list(id: Value) -> MCPResponse {
    let tools = discover_tools();
    MCPResponse::success(id, json!({ "tools": tools }))
}

/// Handle the tools/call method
fn handle_tools_call(id: Value, params: &Value) -> MCPResponse {
    // Extract tool name
    let Some(name) = params.get("name").and_then(|n| n.as_str()) else {
        return MCPResponse::error(id, MCPError::invalid_params("missing 'name' parameter"));
    };

    // Extract arguments (default to empty object)
    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));

    // Execute the tool
    match execute_tool(name, &arguments) {
        Ok((output, is_error)) => MCPResponse::success(
            id,
            json!({
                "content": [{
                    "type": "text",
                    "text": output
                }],
                "isError": is_error
            }),
        ),
        Err(e) => MCPResponse::error(id, MCPError::tool_execution_failed(&e)),
    }
}

// ============================================================
// TESTS
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_initialize() {
        let response = handle_initialize(json!(1));

        assert!(response.result.is_some());
        assert!(response.error.is_none());

        let result = response.result.unwrap();
        assert_eq!(result["protocolVersion"], PROTOCOL_VERSION);
        assert_eq!(result["serverInfo"]["name"], "chakravarti-mcp");
    }

    #[test]
    fn test_handle_tools_list() {
        let response = handle_tools_list(json!(2));

        assert!(response.result.is_some());
        assert!(response.error.is_none());

        let result = response.result.unwrap();
        assert!(result["tools"].is_array());

        // Should have tools
        let tools = result["tools"].as_array().unwrap();
        assert!(!tools.is_empty());
    }

    #[test]
    fn test_handle_unknown_method() {
        let response = handle_request_line(r#"{"jsonrpc":"2.0","id":1,"method":"unknown/method"}"#);

        assert!(response.is_some());
        let resp = response.unwrap();
        assert!(resp.error.is_some());
        assert_eq!(resp.error.as_ref().unwrap().code, -32601);
    }

    #[test]
    fn test_handle_parse_error() {
        let response = handle_request_line("not valid json");

        assert!(response.is_some());
        let resp = response.unwrap();
        assert!(resp.error.is_some());
        assert_eq!(resp.error.as_ref().unwrap().code, -32700);
    }

    #[test]
    fn test_handle_tools_call_missing_name() {
        let response = handle_tools_call(json!(3), &json!({}));

        assert!(response.error.is_some());
        assert_eq!(response.error.as_ref().unwrap().code, -32602);
    }
}
