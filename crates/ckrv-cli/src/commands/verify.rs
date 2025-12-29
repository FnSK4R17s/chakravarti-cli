//! Verify command - run tests, lint, and quality checks.

use std::path::PathBuf;
use std::time::Instant;

use clap::Args;
use serde::Serialize;

use crate::ui::UiContext;
use crate::ui::Renderable;
use crate::ui::components::Banner;

/// Arguments for the verify command
#[derive(Args)]
pub struct VerifyArgs {
    /// Run only lint checks
    #[arg(long)]
    pub lint: bool,

    /// Run only type checks
    #[arg(long, name = "type")]
    pub typecheck: bool,

    /// Run only tests
    #[arg(long)]
    pub test: bool,

    /// Auto-fix issues where possible
    #[arg(long)]
    pub fix: bool,

    /// Continue on failure (run all checks even if some fail)
    #[arg(long)]
    pub continue_on_failure: bool,

    /// Save results to verification.yaml
    #[arg(long)]
    pub save: bool,
}

#[derive(Serialize, Clone)]
pub struct VerifyOutput {
    pub success: bool,
    pub checks: Vec<CheckResult>,
    pub total_duration_ms: u64,
    pub summary: VerifySummary,
}

#[derive(Serialize, Clone)]
pub struct CheckResult {
    pub name: String,
    pub passed: bool,
    pub duration_ms: u64,
    pub output: Option<String>,
    pub error: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct VerifySummary {
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
}

/// Project type detection
#[derive(Debug, Clone, PartialEq)]
enum ProjectType {
    Python,
    JavaScript,
    TypeScript,
    Rust,
    Go,
    Unknown,
}

/// Execute the verify command
pub async fn execute(args: VerifyArgs, json: bool, ui: &UiContext) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let start_time = Instant::now();

    if !json {
        println!("{}", Banner::new("CKRV VERIFY").subtitle("Quality Checks").render(&ui.theme));
    }

    // Detect project type
    let project_type = detect_project_type(&cwd);
    
    if !json {
        println!("📦 Detected project type: {:?}\n", project_type);
    }

    let mut checks: Vec<CheckResult> = Vec::new();
    let mut all_passed = true;
    let run_all = !args.lint && !args.typecheck && !args.test;

    // Run lint checks
    if run_all || args.lint {
        let result = run_lint_check(&cwd, &project_type, args.fix).await;
        if !result.passed {
            all_passed = false;
        }
        if !json {
            print_check_result(&result);
        }
        checks.push(result);
        
        if !all_passed && !args.continue_on_failure {
            return finish_verify(checks, all_passed, start_time, json, &cwd, args.save);
        }
    }

    // Run type checks
    if run_all || args.typecheck {
        let result = run_type_check(&cwd, &project_type).await;
        if !result.passed {
            all_passed = false;
        }
        if !json {
            print_check_result(&result);
        }
        checks.push(result);
        
        if !all_passed && !args.continue_on_failure {
            return finish_verify(checks, all_passed, start_time, json, &cwd, args.save);
        }
    }

    // Run tests
    if run_all || args.test {
        let result = run_tests(&cwd, &project_type).await;
        if !result.passed {
            all_passed = false;
        }
        if !json {
            print_check_result(&result);
        }
        checks.push(result);
    }

    finish_verify(checks, all_passed, start_time, json, &cwd, args.save)
}

fn finish_verify(
    checks: Vec<CheckResult>,
    all_passed: bool,
    start_time: Instant,
    json: bool,
    cwd: &PathBuf,
    save: bool,
) -> anyhow::Result<()> {
    let total_duration = start_time.elapsed().as_millis() as u64;
    
    let passed = checks.iter().filter(|c| c.passed).count();
    let failed = checks.iter().filter(|c| !c.passed).count();
    let skipped = 0; // Could track skipped checks

    let summary = VerifySummary { passed, failed, skipped };

    let output = VerifyOutput {
        success: all_passed,
        checks: checks.clone(),
        total_duration_ms: total_duration,
        summary: summary.clone(),
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("\n═══════════════════════════════════════════════════════════════");
        if all_passed {
            println!("✅ All checks passed! ({} checks in {}ms)", passed, total_duration);
        } else {
            println!("❌ {} check(s) failed ({} passed, {} failed)", failed, passed, failed);
        }
        println!("═══════════════════════════════════════════════════════════════\n");

        if all_passed {
            println!("💡 Next Steps:");
            println!("   ckrv diff       # Review changes");
            println!("   ckrv promote    # Create pull request");
        } else {
            println!("💡 To fix issues:");
            println!("   ckrv verify --fix   # Auto-fix where possible");
            println!("   Review errors above and fix manually");
        }
    }

    // Save to verification.yaml if requested
    if save {
        save_verification_results(cwd, &output)?;
        if !json {
            println!("\n📝 Results saved to .specs/<branch>/verification.yaml");
        }
    }

    if !all_passed {
        std::process::exit(1);
    }

    Ok(())
}

fn detect_project_type(cwd: &PathBuf) -> ProjectType {
    if cwd.join("Cargo.toml").exists() {
        ProjectType::Rust
    } else if cwd.join("pyproject.toml").exists() || cwd.join("requirements.txt").exists() || cwd.join("setup.py").exists() {
        ProjectType::Python
    } else if cwd.join("tsconfig.json").exists() {
        ProjectType::TypeScript
    } else if cwd.join("package.json").exists() {
        ProjectType::JavaScript
    } else if cwd.join("go.mod").exists() {
        ProjectType::Go
    } else {
        ProjectType::Unknown
    }
}

async fn run_lint_check(cwd: &PathBuf, project_type: &ProjectType, fix: bool) -> CheckResult {
    let start = Instant::now();
    let name = "Lint".to_string();

    let (cmd, args): (&str, Vec<&str>) = match project_type {
        ProjectType::Python => {
            if fix {
                ("ruff", vec!["check", "--fix", "."])
            } else {
                ("ruff", vec!["check", "."])
            }
        }
        ProjectType::JavaScript | ProjectType::TypeScript => {
            if fix {
                ("npx", vec!["eslint", "--fix", "."])
            } else {
                ("npx", vec!["eslint", "."])
            }
        }
        ProjectType::Rust => {
            ("cargo", vec!["clippy", "--", "-D", "warnings"])
        }
        ProjectType::Go => {
            ("golangci-lint", vec!["run"])
        }
        ProjectType::Unknown => {
            return CheckResult {
                name,
                passed: true,
                duration_ms: 0,
                output: Some("No linter configured for this project type".to_string()),
                error: None,
            };
        }
    };

    run_command(name, cmd, &args, cwd, start).await
}

async fn run_type_check(cwd: &PathBuf, project_type: &ProjectType) -> CheckResult {
    let start = Instant::now();
    let name = "Type Check".to_string();

    let (cmd, args): (&str, Vec<&str>) = match project_type {
        ProjectType::Python => {
            // Try mypy first, fall back to pyright
            if which_exists("mypy") {
                ("mypy", vec!["."])
            } else if which_exists("pyright") {
                ("pyright", vec![])
            } else {
                return CheckResult {
                    name,
                    passed: true,
                    duration_ms: 0,
                    output: Some("No type checker found (install mypy or pyright)".to_string()),
                    error: None,
                };
            }
        }
        ProjectType::TypeScript => {
            ("npx", vec!["tsc", "--noEmit"])
        }
        ProjectType::Rust => {
            ("cargo", vec!["check"])
        }
        ProjectType::Go => {
            ("go", vec!["build", "./..."])
        }
        _ => {
            return CheckResult {
                name,
                passed: true,
                duration_ms: 0,
                output: Some("No type checker for this project type".to_string()),
                error: None,
            };
        }
    };

    run_command(name, cmd, &args, cwd, start).await
}

async fn run_tests(cwd: &PathBuf, project_type: &ProjectType) -> CheckResult {
    let start = Instant::now();
    let name = "Tests".to_string();

    let (cmd, args): (&str, Vec<&str>) = match project_type {
        ProjectType::Python => {
            ("pytest", vec!["-v"])
        }
        ProjectType::JavaScript | ProjectType::TypeScript => {
            // Check for test script in package.json
            if cwd.join("package.json").exists() {
                ("npm", vec!["test", "--", "--passWithNoTests"])
            } else {
                return CheckResult {
                    name,
                    passed: true,
                    duration_ms: 0,
                    output: Some("No test configuration found".to_string()),
                    error: None,
                };
            }
        }
        ProjectType::Rust => {
            ("cargo", vec!["test"])
        }
        ProjectType::Go => {
            ("go", vec!["test", "./..."])
        }
        ProjectType::Unknown => {
            return CheckResult {
                name,
                passed: true,
                duration_ms: 0,
                output: Some("No test runner configured for this project type".to_string()),
                error: None,
            };
        }
    };

    run_command(name, cmd, &args, cwd, start).await
}

async fn run_command(name: String, cmd: &str, args: &[&str], cwd: &PathBuf, start: Instant) -> CheckResult {
    let output = std::process::Command::new(cmd)
        .args(args)
        .current_dir(cwd)
        .output();

    let duration_ms = start.elapsed().as_millis() as u64;

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            
            CheckResult {
                name,
                passed: out.status.success(),
                duration_ms,
                output: if stdout.is_empty() { None } else { Some(stdout) },
                error: if stderr.is_empty() || out.status.success() { None } else { Some(stderr) },
            }
        }
        Err(e) => {
            CheckResult {
                name,
                passed: false,
                duration_ms,
                output: None,
                error: Some(format!("Failed to run {}: {}", cmd, e)),
            }
        }
    }
}

fn print_check_result(result: &CheckResult) {
    let icon = if result.passed { "✅" } else { "❌" };
    println!("{} {} ({}ms)", icon, result.name, result.duration_ms);
    
    if let Some(ref error) = result.error {
        // Print first few lines of error
        for line in error.lines().take(10) {
            println!("   \x1b[31m{}\x1b[0m", line);
        }
        let total_lines = error.lines().count();
        if total_lines > 10 {
            println!("   ... and {} more lines", total_lines - 10);
        }
    }
    println!();
}

fn which_exists(cmd: &str) -> bool {
    std::process::Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn save_verification_results(cwd: &PathBuf, output: &VerifyOutput) -> anyhow::Result<()> {
    // Get current branch to find spec directory
    let branch_output = std::process::Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(cwd)
        .output()?;
    
    let branch = String::from_utf8_lossy(&branch_output.stdout).trim().to_string();
    
    let spec_dir = cwd.join(".specs").join(&branch);
    if spec_dir.exists() {
        let yaml = serde_yaml::to_string(output)?;
        std::fs::write(spec_dir.join("verification.yaml"), yaml)?;
    }
    
    Ok(())
}

