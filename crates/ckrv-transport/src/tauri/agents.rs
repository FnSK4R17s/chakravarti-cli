//! # Agents Tauri Commands
//!
//! Tauri command stubs for agent handlers.
//!
//! Note: This is a placeholder for Phase 6 implementation.

/// List agents command stub.
pub fn list_agents_info() -> &'static str {
    "list_agents: Returns all configured agents"
}

/// Upsert agent command stub.
pub fn upsert_agent_info() -> &'static str {
    "upsert_agent: Creates or updates an agent"
}

/// Delete agent command stub.
pub fn delete_agent_info() -> &'static str {
    "delete_agent: Deletes an agent by name"
}

/// Set default agent command stub.
pub fn set_default_agent_info() -> &'static str {
    "set_default_agent: Sets the default agent"
}
