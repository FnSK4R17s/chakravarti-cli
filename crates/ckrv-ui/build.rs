use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=frontend/src");
    println!("cargo:rerun-if-changed=frontend/package.json");

    let is_release = std::env::var("PROFILE").unwrap_or_else(|_| "debug".to_string()) == "release";

    // Ensure frontend dependencies are present for release installs.
    let has_node_modules = std::path::Path::new("frontend/node_modules").exists();
    let has_tsc = std::path::Path::new("frontend/node_modules/typescript/bin/tsc").exists();
    if !has_node_modules || !has_tsc {
        let mut install = Command::new("npm");
        install.args(["install", "--include=dev"]);
        install.current_dir("frontend");
        if !install
            .status()
            .expect("failed to execute npm install")
            .success()
        {
            if is_release {
                panic!("Frontend dependency install failed");
            } else {
                println!("cargo:warning=Frontend dependency install failed in debug mode");
                return;
            }
        }
    }

    let mut cmd = Command::new("npm");
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
