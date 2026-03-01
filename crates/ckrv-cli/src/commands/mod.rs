//! CLI command modules.
//!
//! Each submodule implements a top-level CLI command (e.g., `ckrv init`, `ckrv run`).

/// Cloud execution commands (login, logout, whoami, credentials).
pub mod cloud;
/// Code workflow commands (spec, tasks, plan, run, diff).
pub mod code;
/// Diff command - view changes between branches.
pub mod diff;
/// Fix command - AI-powered error fixing.
pub mod fix;
/// Init command - initialize Chakravarti in a repository.
pub mod init;
/// Logs command - stream or view cloud job logs.
pub mod logs;
/// Plan command - generate execution plans from tasks.
pub mod plan;
/// Promote command - create pull/merge requests.
pub mod promote;
/// Pull command - download cloud job results.
pub mod pull;
/// QA command - code review and bug analysis.
pub mod qa;
/// Report command - view job metrics.
pub mod report;
/// Run command - execute jobs from specifications.
pub mod run;
/// Spec command - create and manage specifications.
pub mod spec;
/// Spec data structures shared across commands.
pub mod spec_structs;
/// Status command - check job status.
pub mod status;
/// Task command - execute workflow-based agent tasks.
pub mod task;
/// Term command - spawn interactive AI agent terminals.
pub mod term;
/// Test command - run tests, plan and write new tests.
pub mod test;
/// UI command - launch the web dashboard.
pub mod ui;
/// Verify command - run tests, lint, and quality checks.
pub mod verify;

/// Emit a JSON value to stdout if requested.
pub fn emit_json<T: serde::Serialize>(val: T, json: bool) {
    if json {
        if let Ok(json_str) = serde_json::to_string_pretty(&val) {
            println!("{}", json_str);
        }
    }
}
