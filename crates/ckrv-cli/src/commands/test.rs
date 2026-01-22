//! Test command - run tests, plan and write new tests.

use std::path::PathBuf;

use clap::{Args, Subcommand};
use serde::Serialize;

use crate::services::{
    agent_lookup,
    diff_analyzer,
    report_generator,
    test_framework::{self, TestResult},
};
use crate::ui::components::Banner;
use crate::ui::{Renderable, UiContext};

/// Arguments for the test command
#[derive(Args)]
pub struct TestArgs {
    #[command(subcommand)]
    pub command: TestSubcommand,
}

/// Test subcommands
#[derive(Subcommand)]
pub enum TestSubcommand {
    /// Run existing tests in sandbox
    Run {
        /// Branch to compare against (default: main)
        #[arg(long, default_value = "main")]
        base: String,
    },
    
    /// Analyze changes and generate test plan
    Plan {
        /// Branch to compare against (default: main)
        #[arg(long, default_value = "main")]
        base: String,
    },
    
    /// Write new tests using test writer agent
    Write {
        /// Branch to compare against (default: main)
        #[arg(long, default_value = "main")]
        base: String,
        
        /// Run tests after writing
        #[arg(long)]
        run: bool,
    },
    
    /// Check test coverage of changed files
    Coverage {
        /// Branch to compare against (default: main)
        #[arg(long, default_value = "main")]
        base: String,
    },
}

/// Output for test run command
#[derive(Serialize)]
pub struct TestRunOutput {
    pub success: bool,
    pub result: TestResult,
}

/// Output for test plan command
#[derive(Serialize)]
pub struct TestPlanOutput {
    pub plan_id: String,
    pub base_branch: String,
    pub changed_files: Vec<ChangedFileInfo>,
    pub proposed_tests: Vec<ProposedTest>,
}

#[derive(Serialize)]
pub struct ChangedFileInfo {
    pub path: String,
    pub change_type: String,
    pub lines_added: u32,
    pub lines_removed: u32,
    pub has_tests: bool,
}

#[derive(Serialize)]
pub struct ProposedTest {
    pub target_file: String,
    pub test_file: String,
    pub description: String,
    pub priority: String,
}

/// Execute the test command
pub async fn execute(args: TestArgs, json: bool, ui: &UiContext) -> anyhow::Result<()> {
    match args.command {
        TestSubcommand::Run { base } => execute_run(&base, json, ui).await,
        TestSubcommand::Plan { base } => execute_plan(&base, json, ui).await,
        TestSubcommand::Write { base, run } => execute_write(&base, run, json, ui).await,
        TestSubcommand::Coverage { base } => execute_coverage(&base, json, ui).await,
    }
}

/// Execute test run
async fn execute_run(base: &str, json: bool, ui: &UiContext) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    
    if !json {
        println!("{}", Banner::new("CKRV TEST").subtitle("Run Tests").render(&ui.theme));
    }
    
    // Detect framework
    let framework = test_framework::detect_framework(&cwd);
    
    if !json {
        println!("📦 Detected framework: {}\n", framework.name());
    }
    
    // Run tests
    let result = test_framework::run_tests_local(&cwd).await;
    
    if json {
        let output = TestRunOutput {
            success: result.success,
            result: result.clone(),
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        // Get branch info for report
        let branch = diff_analyzer::get_current_branch().unwrap_or_else(|_| "unknown".to_string());
        let report = report_generator::generate_test_report(&result, &branch, base);
        println!("{}", report);
    }
    
    if !result.success {
        std::process::exit(1);
    }
    
    Ok(())
}

/// Execute test plan
async fn execute_plan(base: &str, json: bool, ui: &UiContext) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    
    if !json {
        println!("{}", Banner::new("CKRV TEST").subtitle("Plan Tests").render(&ui.theme));
    }
    
    // Get changed files
    let changed_files = match diff_analyzer::get_changed_files(base) {
        Ok(files) => files,
        Err(e) => {
            if !json {
                println!("❌ Failed to get changed files: {}", e);
                println!("\nMake sure you're on a feature branch with changes vs '{}'", base);
            }
            return Err(e);
        }
    };
    
    if changed_files.is_empty() {
        if !json {
            println!("ℹ️  No changes found compared to '{}'", base);
        }
        return Ok(());
    }
    
    // Analyze which files need tests
    let mut proposed_tests = Vec::new();
    let mut file_infos = Vec::new();
    
    for file in &changed_files {
        let path_str = file.path.to_string_lossy().to_string();
        let has_tests = check_has_tests(&cwd, &file.path);
        
        file_infos.push(ChangedFileInfo {
            path: path_str.clone(),
            change_type: format!("{:?}", file.change_type).to_lowercase(),
            lines_added: file.lines_added,
            lines_removed: file.lines_removed,
            has_tests,
        });
        
        // Suggest tests for files without them
        if !has_tests && is_testable_file(&file.path) {
            let test_file = suggest_test_file(&file.path);
            proposed_tests.push(ProposedTest {
                target_file: path_str.clone(),
                test_file,
                description: format!("Add unit tests for {}", file.path.file_name().unwrap_or_default().to_string_lossy()),
                priority: if file.lines_added > 50 { "high" } else { "medium" }.to_string(),
            });
        }
    }
    
    let output = TestPlanOutput {
        plan_id: format!("plan-{}", chrono::Utc::now().timestamp()),
        base_branch: base.to_string(),
        changed_files: file_infos,
        proposed_tests,
    };
    
    if json {
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("## Test Plan\n");
        println!("**Base**: {}\n", base);
        
        println!("### Changed Files\n");
        println!("| File | Status | +/- | Has Tests |");
        println!("|------|--------|-----|-----------|");
        for f in &output.changed_files {
            println!("| {} | {} | +{} -{} | {} |",
                f.path, f.change_type, f.lines_added, f.lines_removed,
                if f.has_tests { "✅" } else { "❌" }
            );
        }
        println!();
        
        if output.proposed_tests.is_empty() {
            println!("✅ All changed files have tests.\n");
        } else {
            println!("### Proposed Tests\n");
            for test in &output.proposed_tests {
                println!("- **{}** → `{}`", test.description, test.test_file);
                println!("  Priority: {}\n", test.priority);
            }
            println!("\nRun `ckrv test write` to have the test writer agent create these tests.");
        }
    }
    
    Ok(())
}

/// Execute test write
async fn execute_write(base: &str, run_after: bool, json: bool, ui: &UiContext) -> anyhow::Result<()> {
    if !json {
        println!("{}", Banner::new("CKRV TEST").subtitle("Write Tests").render(&ui.theme));
    }
    
    // Check for test writer agent
    let agent = match agent_lookup::find_test_writer_agent() {
        Some(a) => a,
        None => {
            if json {
                println!(r#"{{"error": "No test writer agent configured", "exit_code": 4}}"#);
            } else {
                println!("{}", agent_lookup::test_writer_missing_message());
            }
            std::process::exit(4);
        }
    };
    
    if !json {
        println!("🤖 Using agent: {} ({})\n", agent.name, agent.id);
        println!("📝 Analyzing changes vs '{}'...\n", base);
    }
    
    // Get changed files
    let changed_files = diff_analyzer::get_changed_files(base)?;
    
    if changed_files.is_empty() {
        if !json {
            println!("ℹ️  No changes found. Nothing to write tests for.");
        }
        return Ok(());
    }
    
    // For now, show what would be done
    // Full agent invocation would require Docker sandbox integration
    if !json {
        println!("🔧 Test writing would analyze these {} files:", changed_files.len());
        for file in &changed_files {
            println!("   - {}", file.path.display());
        }
        println!("\n⚠️  Agent invocation requires Docker sandbox (not yet integrated).");
        println!("   The test writer agent ({}) would create tests for uncovered code.", agent.name);
    }
    
    if run_after {
        if !json {
            println!("\n📋 Running tests after write...\n");
        }
        execute_run(base, json, ui).await?;
    }
    
    Ok(())
}

/// Execute test coverage
async fn execute_coverage(base: &str, json: bool, ui: &UiContext) -> anyhow::Result<()> {
    if !json {
        println!("{}", Banner::new("CKRV TEST").subtitle("Coverage").render(&ui.theme));
        println!("📊 Checking coverage of changes vs '{}'...\n", base);
    }
    
    // Get changed files
    let changed_files = diff_analyzer::get_changed_files(base)?;
    let cwd = std::env::current_dir()?;
    
    let mut covered = 0;
    let mut uncovered = 0;
    
    for file in &changed_files {
        if is_testable_file(&file.path) {
            if check_has_tests(&cwd, &file.path) {
                covered += 1;
            } else {
                uncovered += 1;
            }
        }
    }
    
    let total = covered + uncovered;
    let coverage = if total > 0 { (covered as f64 / total as f64) * 100.0 } else { 100.0 };
    
    if json {
        println!(r#"{{"total": {}, "covered": {}, "uncovered": {}, "coverage_percent": {:.1}}}"#,
            total, covered, uncovered, coverage);
    } else {
        println!("## Coverage Summary\n");
        println!("| Metric | Value |");
        println!("|--------|-------|");
        println!("| Testable files | {} |", total);
        println!("| With tests | {} |", covered);
        println!("| Without tests | {} |", uncovered);
        println!("| Coverage | {:.1}% |", coverage);
        println!();
        
        if coverage < 80.0 {
            println!("⚠️  Coverage below 80%. Run `ckrv test plan` to see what needs tests.");
        } else {
            println!("✅ Good coverage! All changed files have tests.");
        }
    }
    
    Ok(())
}

/// Check if a file has corresponding tests
fn check_has_tests(cwd: &PathBuf, file: &PathBuf) -> bool {
    let file_str = file.to_string_lossy();
    
    // Skip test files themselves
    if file_str.contains("test") || file_str.contains("spec") {
        return true;
    }
    
    // Check for common test file patterns
    let stem = file.file_stem().unwrap_or_default().to_string_lossy();
    let ext = file.extension().unwrap_or_default().to_string_lossy();
    
    // Rust: look for #[test] in same file or tests/
    if ext == "rs" {
        let tests_dir = cwd.join("tests");
        if tests_dir.join(format!("{}.rs", stem)).exists() {
            return true;
        }
        // Check for inline tests
        if let Ok(content) = std::fs::read_to_string(cwd.join(file)) {
            if content.contains("#[test]") || content.contains("#[cfg(test)]") {
                return true;
            }
        }
    }
    
    // TypeScript/JavaScript: look for .test.ts or .spec.ts
    if ext == "ts" || ext == "tsx" || ext == "js" || ext == "jsx" {
        let parent = file.parent().unwrap_or(file);
        let test_file = parent.join(format!("{}.test.{}", stem, ext));
        let spec_file = parent.join(format!("{}.spec.{}", stem, ext));
        if cwd.join(&test_file).exists() || cwd.join(&spec_file).exists() {
            return true;
        }
        // Check __tests__ directory
        let tests_dir = parent.join("__tests__");
        if cwd.join(tests_dir.join(format!("{}.test.{}", stem, ext))).exists() {
            return true;
        }
    }
    
    // Python: look for test_*.py or *_test.py
    if ext == "py" {
        let parent = file.parent().unwrap_or(file);
        let test_file = parent.join(format!("test_{}.py", stem));
        let test_file2 = parent.join(format!("{}_test.py", stem));
        if cwd.join(&test_file).exists() || cwd.join(&test_file2).exists() {
            return true;
        }
    }
    
    false
}

/// Check if a file is testable (source code, not config)
fn is_testable_file(file: &PathBuf) -> bool {
    let ext = file.extension().unwrap_or_default().to_string_lossy();
    let file_str = file.to_string_lossy();
    
    // Skip non-source files
    if !matches!(ext.as_ref(), "rs" | "ts" | "tsx" | "js" | "jsx" | "py" | "go" | "java" | "kt" | "swift") {
        return false;
    }
    
    // Skip test files
    if file_str.contains("test") || file_str.contains("spec") || file_str.contains("_test") {
        return false;
    }
    
    // Skip config and generated files
    if file_str.contains("node_modules") || file_str.contains("target/") || file_str.contains("dist/") {
        return false;
    }
    
    true
}

/// Suggest a test file path for a source file
fn suggest_test_file(file: &PathBuf) -> String {
    let ext = file.extension().unwrap_or_default().to_string_lossy();
    let stem = file.file_stem().unwrap_or_default().to_string_lossy();
    let parent = file.parent().unwrap_or(file);
    
    match ext.as_ref() {
        "rs" => format!("tests/{}.rs", stem),
        "ts" | "tsx" => format!("{}/{}.test.{}", parent.display(), stem, ext),
        "js" | "jsx" => format!("{}/{}.test.{}", parent.display(), stem, ext),
        "py" => format!("{}/test_{}.py", parent.display(), stem),
        "go" => format!("{}/{}_test.go", parent.display(), stem),
        _ => format!("{}/{}.test", parent.display(), stem),
    }
}
