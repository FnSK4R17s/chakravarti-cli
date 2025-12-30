//! CLI command modules.

pub mod cloud;
pub mod diff;
pub mod fix;
pub mod init;
pub mod logs;
pub mod promote;
pub mod pull;
pub mod report;
pub mod run;
pub mod spec;
pub mod spec_structs;
pub mod status;
pub mod task;
pub mod ui;
pub mod verify;

/// Emit a JSON value to stdout if requested.
pub fn emit_json<T: serde::Serialize>(val: T, json: bool) {
    if json {
        if let Ok(json_str) = serde_json::to_string_pretty(&val) {
            println!("{}", json_str);
        }
    }
}
