//! Build script for ckrv-ui: compiles the frontend if not pre-built.
#![allow(clippy::expect_used, clippy::panic)]

use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=frontend/src");
    println!("cargo:rerun-if-changed=frontend/package.json");

    // Skip if frontend is already built (CI/justfile pre-builds it)
    if Path::new("frontend/dist/index.html").exists() {
        return;
    }

    let is_release = std::env::var("PROFILE").unwrap_or_else(|_| "debug".to_string()) == "release";

    // Windows needs "npm.cmd", unix uses "npm"
    let npm = if cfg!(windows) { "npm.cmd" } else { "npm" };

    let mut cmd = Command::new(npm);
    cmd.arg("run").arg("build");
    cmd.current_dir("frontend");

    if !cmd
        .status()
        .expect("failed to execute npm run build")
        .success()
    {
        if is_release {
            panic!("Frontend build failed");
        } else {
            println!("cargo:warning=Frontend build failed, but continuing in debug mode");
        }
    }
}
