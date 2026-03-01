//! Tauri command modules
//!
//! Each module wraps the transport handlers for Tauri IPC invocation.

pub mod agents;
pub mod cli;
pub mod diff;
pub mod execution;
pub mod history;
pub mod plans;
pub mod project;
pub mod qa;
pub mod specs;
pub mod status;
pub mod terminal;
pub mod test;
pub mod update;
