//! # Tauri Transport
//!
//! Tauri IPC command wrappers.
//!
//! ## Overview
//!
//! This module provides the `get_invoke_handlers()` function that returns
//! a function registry for Tauri command registration.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use ckrv_transport::tauri::get_invoke_handlers;
//!
//! // In Tauri app
//! tauri::Builder::default()
//!     .invoke_handler(tauri::generate_handler![
//!         // handlers from get_invoke_handlers()
//!     ])
//!     .run(tauri::generate_context!())
//!     .expect("error while running tauri application");
//! ```
//!
//! ## Note
//!
//! This module is a stub for Phase 6 (US4) implementation.
//! The actual Tauri commands will be added when the Tauri app is built.

pub mod agents;
pub mod status;

/// Placeholder for Tauri invoke handlers.
///
/// In the actual implementation, this would return a list of
/// command handlers that can be registered with Tauri.
///
/// ```rust,ignore
/// pub fn get_invoke_handlers() -> impl Fn(Invoke) {
///     tauri::generate_handler![
///         status::get_status,
///         agents::list_agents,
///         // ... more handlers
///     ]
/// }
/// ```
pub fn get_invoke_handlers_info() -> Vec<&'static str> {
    vec![
        "get_status",
        "check_docker",
        "get_cloud_status",
        "list_agents",
        "upsert_agent",
        "delete_agent",
        "set_default_agent",
        "list_specs",
        "get_spec",
        "create_spec",
        "update_spec",
        "delete_spec",
        "get_plan",
        "generate_plan",
        "list_tasks",
        "get_task",
        "list_runs",
        "get_run",
        "start_execution",
        "stop_execution",
        "get_execution_status",
        "run_command",
        "console_input",
        "get_diff",
        "qa_review",
        "qa_bugs",
        "qa_report",
        "create_session",
        "get_session",
        "stop_session",
        "test_run",
        "test_plan",
        "test_write",
    ]
}
