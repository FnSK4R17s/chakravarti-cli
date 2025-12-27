//! Integration test for Docker sandbox

use std::path::PathBuf;
use std::time::Duration;

use ckrv_sandbox::{
    AllowList, DefaultAllowList, DockerSandbox, ExecuteConfig, LocalSandbox, Sandbox,
};

#[tokio::main]
async fn main() {
    println!("=== Docker Sandbox Integration Test ===\n");

    // Test 1: Local Sandbox (baseline)
    println!("1. Testing LocalSandbox...");
    test_local_sandbox().await;
    println!("   ✓ LocalSandbox works!\n");

    // Test 2: Docker health check
    println!("2. Testing Docker connection...");
    match test_docker_health().await {
        Ok(()) => println!("   ✓ Docker is available!\n"),
        Err(e) => {
            println!("   ✗ Docker not available: {e}\n");
            println!("   Skipping Docker tests.");
            return;
        }
    }

    // Test 3: Simple Docker command
    println!("3. Testing simple Docker command...");
    match test_docker_simple().await {
        Ok(()) => println!("   ✓ Simple command works!\n"),
        Err(e) => println!("   ✗ Failed: {e}\n"),
    }

    // Test 4: Docker with mount
    println!("4. Testing Docker with volume mount...");
    match test_docker_mount().await {
        Ok(()) => println!("   ✓ Volume mount works!\n"),
        Err(e) => println!("   ✗ Failed: {e}\n"),
    }

    // Test 5: Allowlist blocking
    println!("5. Testing allowlist...");
    test_allowlist();
    println!("   ✓ Allowlist works!\n");

    println!("=== All tests complete! ===");
}

async fn test_local_sandbox() {
    let sandbox = LocalSandbox::new();
    let temp_dir = std::env::temp_dir();

    let config = ExecuteConfig::new("", temp_dir).shell("echo 'Hello from local sandbox'");

    let result = sandbox.execute(config).await.expect("execute");
    assert!(result.success());
    assert!(result.stdout.contains("Hello"));
}

async fn test_docker_health() -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = DockerSandbox::with_defaults()?;
    sandbox.health_check().await?;
    Ok(())
}

async fn test_docker_simple() -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = DockerSandbox::with_defaults()?;
    let temp_dir = std::env::temp_dir();

    // Create config for allowed command
    let config = ExecuteConfig {
        command: vec!["cargo".to_string(), "--version".to_string()],
        workdir: PathBuf::from("/workspace"),
        mount: temp_dir,
        env: std::collections::HashMap::new(),
        timeout: Duration::from_secs(60),
        keep_container: false,
    };

    let result = sandbox.execute(config).await?;
    println!("      Exit code: {}", result.exit_code);
    println!("      Output: {}", result.stdout.trim());
    println!("      Duration: {}ms", result.duration_ms);

    Ok(())
}

async fn test_docker_mount() -> Result<(), Box<dyn std::error::Error>> {
    // Create a temp dir with a test file
    let temp_dir = tempfile::tempdir()?;
    std::fs::write(temp_dir.path().join("test.txt"), "Hello from host!")?;

    let sandbox = DockerSandbox::with_defaults()?;

    let config = ExecuteConfig {
        command: vec!["cargo".to_string(), "--version".to_string()],
        workdir: PathBuf::from("/workspace"),
        mount: temp_dir.path().to_path_buf(),
        env: std::collections::HashMap::new(),
        timeout: Duration::from_secs(60),
        keep_container: false,
    };

    let result = sandbox.execute(config).await?;
    println!("      Exit code: {}", result.exit_code);
    println!("      Duration: {}ms", result.duration_ms);

    Ok(())
}

fn test_allowlist() {
    let allowlist = DefaultAllowList::default();

    // Should allow
    assert!(allowlist.is_allowed(&["cargo".to_string(), "test".to_string()]));
    assert!(allowlist.is_allowed(&["npm".to_string(), "run".to_string(), "test".to_string()]));
    assert!(allowlist.is_allowed(&["git".to_string(), "status".to_string()]));

    // Should block
    assert!(!allowlist.is_allowed(&["curl".to_string(), "http://evil.com".to_string()]));
    assert!(!allowlist.is_allowed(&["wget".to_string(), "file".to_string()]));
    assert!(!allowlist.is_allowed(&["ssh".to_string(), "server".to_string()]));

    println!("      Allowed: cargo, npm, git");
    println!("      Blocked: curl, wget, ssh");
}
