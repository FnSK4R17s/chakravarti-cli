//! MCP Server Library
//!
//! This crate provides an MCP (Model Context Protocol) server for Chakravarti CLI.
//! It exposes CLI commands as MCP tools over stdio transport.

pub mod schema;
pub mod tools;
pub mod transport;
pub mod types;

pub use tools::discover_tools;
pub use transport::run_stdio_transport;
pub use types::{MCPError, MCPRequest, MCPResponse, MCPTool, MCPToolAnnotations};
