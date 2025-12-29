use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=frontend/src");
    println!("cargo:rerun-if-changed=frontend/package.json");

    let is_release = std::env::var("PROFILE").unwrap_or_else(|_| "debug".to_string()) == "release";

    let mut cmd = Command::new("npm");
    cmd.arg("run").arg("build");
    cmd.current_dir("frontend");

    if !cmd.status().expect("failed to execute npm run build").success() {
        if is_release {
             panic!("Frontend build failed");
        } else {
             println!("cargo:warning=Frontend build failed, but continuing in debug mode");
        }
    }
}
