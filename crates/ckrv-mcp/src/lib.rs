//! MCP Server Library.
//!
//! This crate provides an MCP (Model Context Protocol) server for Chakravarti CLI.
//! It exposes CLI commands as MCP tools over stdio transport, enabling AI assistants
//! to discover and invoke Chakravarti commands programmatically.

/// JSON Schema generation from clap command metadata.
pub mod schema;
/// Tool discovery from CLI introspection and execution via shell.
pub mod tools;
/// stdio transport loop for JSON-RPC 2.0 request/response.
pub mod transport;
/// MCP protocol types (requests, responses, errors, tool definitions).
pub mod types;

pub use tools::discover_tools;
pub use transport::run_stdio_transport;
pub use types::{MCPError, MCPRequest, MCPResponse, MCPTool, MCPToolAnnotations};
