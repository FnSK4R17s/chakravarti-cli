//! Chakravarti MCP Server
//!
//! This binary provides an MCP (Model Context Protocol) server that exposes
//! Chakravarti CLI commands as tools for AI agents like Claude Desktop.
//!
//! Usage:
//! ```bash
//! # Run directly
//! ckrv-mcp
//!
//! # Test with MCP Inspector
//! npx @anthropic-ai/mcp-inspector ckrv-mcp
//! ```

use ckrv_mcp::run_stdio_transport;

fn main() {
    // Run the stdio transport loop
    run_stdio_transport();
}
