//! CLI command modules.

pub mod diff;
pub mod init;
pub mod promote;
pub mod report;
pub mod run;
pub mod spec;
pub mod status;
pub mod task;

/// Emit a JSON value to stdout if requested.
pub fn emit_json<T: serde::Serialize>(val: T, json: bool) {
    if json {
        if let Ok(json_str) = serde_json::to_string_pretty(&val) {
            println!("{}", json_str);
        }
    }
}
