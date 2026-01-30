//! MCP Server Types
//!
//! JSON-RPC 2.0 types for the Model Context Protocol server.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC 2.0 request from MCP client
#[derive(Debug, Deserialize)]
pub struct MCPRequest {
    /// JSON-RPC version (must be "2.0")
    pub jsonrpc: String,

    /// Request ID (can be number or string, optional for notifications)
    #[serde(default)]
    pub id: Option<Value>,

    /// Method name (e.g., "initialize", "tools/list", "tools/call")
    pub method: String,

    /// Method parameters
    #[serde(default)]
    pub params: Value,
}

/// JSON-RPC 2.0 response to MCP client
#[derive(Debug, Serialize)]
pub struct MCPResponse {
    /// JSON-RPC version (always "2.0")
    pub jsonrpc: String,

    /// Request ID (echo back from request)
    pub id: Value,

    /// Result on success
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,

    /// Error on failure
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<MCPError>,
}

impl MCPResponse {
    /// Create a success response
    #[must_use]
    pub fn success(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Create an error response
    #[must_use]
    pub fn error(id: Value, error: MCPError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(error),
        }
    }
}

/// JSON-RPC error object
#[derive(Debug, Serialize)]
pub struct MCPError {
    /// Error code (standard JSON-RPC codes or custom)
    pub code: i32,

    /// Human-readable error message
    pub message: String,

    /// Additional error data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl MCPError {
    /// Parse error (-32700)
    #[must_use]
    pub fn parse_error(details: Option<String>) -> Self {
        Self {
            code: -32700,
            message: "Parse error".to_string(),
            data: details.map(Value::String),
        }
    }

    /// Invalid request (-32600)
    #[must_use]
    pub fn invalid_request(details: Option<String>) -> Self {
        Self {
            code: -32600,
            message: "Invalid Request".to_string(),
            data: details.map(Value::String),
        }
    }

    /// Method not found (-32601)
    #[must_use]
    pub fn method_not_found(method: &str) -> Self {
        Self {
            code: -32601,
            message: format!("Method not found: {method}"),
            data: None,
        }
    }

    /// Invalid params (-32602)
    #[must_use]
    pub fn invalid_params(details: &str) -> Self {
        Self {
            code: -32602,
            message: format!("Invalid params: {details}"),
            data: None,
        }
    }

    /// Internal error (-32603)
    #[must_use]
    pub fn internal_error(details: Option<String>) -> Self {
        Self {
            code: -32603,
            message: "Internal error".to_string(),
            data: details.map(Value::String),
        }
    }

    /// Tool execution failed (-32001)
    #[must_use]
    pub fn tool_execution_failed(details: &str) -> Self {
        Self {
            code: -32001,
            message: format!("Tool execution failed: {details}"),
            data: None,
        }
    }
}

/// MCP Tool definition
#[derive(Debug, Clone, Serialize)]
pub struct MCPTool {
    /// Tool name (e.g., "ckrv_spec_new")
    pub name: String,

    /// Human-readable description
    pub description: String,

    /// JSON Schema for input parameters
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,

    /// Optional annotations for Claude Desktop hints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<MCPToolAnnotations>,
}

/// Tool annotations for client hints
#[derive(Debug, Clone, Serialize)]
pub struct MCPToolAnnotations {
    /// Hint that this tool only reads data
    #[serde(rename = "readOnlyHint", skip_serializing_if = "Option::is_none")]
    pub read_only_hint: Option<bool>,

    /// Hint that this tool may modify state
    #[serde(rename = "destructiveHint", skip_serializing_if = "Option::is_none")]
    pub destructive_hint: Option<bool>,
}
