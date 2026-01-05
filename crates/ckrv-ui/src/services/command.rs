use std::process::{Command, Stdio};
use crate::state::{AppState, SystemMode};
use crate::hub::OrchestrationEvent;
use chrono::Utc;
use tokio::io::{AsyncBufReadExt, BufReader};

pub struct CommandService;

impl CommandService {
    pub fn new() -> Self {
        Self
    }

    fn timestamp() -> String {
        Utc::now().to_rfc3339()
    }

    fn emit_log(state: &AppState, message: &str) {
        state.hub.broadcast(OrchestrationEvent::Log {
            message: message.to_string(),
            timestamp: Self::timestamp(),
            metadata: None,
        });
    }

    fn emit_error(state: &AppState, message: &str) {
        state.hub.broadcast(OrchestrationEvent::Error {
            message: message.to_string(),
            timestamp: Self::timestamp(),
        });
    }

    fn emit_success(state: &AppState, message: &str) {
        state.hub.broadcast(OrchestrationEvent::Success {
            message: message.to_string(),
            timestamp: Self::timestamp(),
        });
    }

    fn emit_step_start(state: &AppState, step_name: &str) {
        state.hub.broadcast(OrchestrationEvent::StepStart {
            step_name: step_name.to_string(),
            timestamp: Self::timestamp(),
        });
    }

    fn emit_step_end(state: &AppState, step_name: &str, status: &str) {
        state.hub.broadcast(OrchestrationEvent::StepEnd {
            step_name: step_name.to_string(),
            timestamp: Self::timestamp(),
            status: status.to_string(),
        });
    }

    /// Determine if a stderr line is an actual error or just an info/debug log
    fn is_error_line(line: &str) -> bool {
        // Check for structured log levels - these are informational
        let is_info_log = line.contains(" INFO ") 
            || line.contains(" WARN ") 
            || line.contains(" DEBUG ")
            || line.contains(" TRACE ");
        
        // It's an error if:
        // 1. It contains " ERROR " (structured error log)
        // 2. It's not an info log AND contains "Error:" or "error:" (error message)
        line.contains(" ERROR ") 
            || (!is_info_log && (line.contains("Error:") || line.contains("error:")))
    }

    /// Emit stderr line as either error or log based on content
    fn emit_stderr_line(state: &AppState, line: &str) {
        if Self::is_error_line(line) {
            Self::emit_error(state, line);
        } else {
            Self::emit_log(state, line);
        }
    }

    pub async fn run_init(state: &AppState) -> Result<String, String> {
        Self::emit_step_start(state, "Initialize Repository");
        Self::emit_log(state, "Starting repository initialization...");

        // Update mode to running
        {
            let mut status = state.status.write().await;
            status.mode = SystemMode::Running;
        }

        // Get current directory
        let cwd = std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?;

        Self::emit_log(state, &format!("Working directory: {}", cwd.display()));

        // Run ckrv init command
        let exe = std::env::current_exe()
            .map_err(|e| format!("Failed to get executable path: {}", e))?;

        Self::emit_log(state, &format!("Running: {} init", exe.display()));

        let output = Command::new(&exe)
            .arg("init")
            .arg("--json")
            .current_dir(&cwd)
            .output();

        match output {
            Ok(result) => {
                let stdout = String::from_utf8_lossy(&result.stdout);
                let stderr = String::from_utf8_lossy(&result.stderr);

                // Log any output
                if !stdout.is_empty() {
                    for line in stdout.lines() {
                        Self::emit_log(state, line);
                    }
                }

                if !stderr.is_empty() {
                    for line in stderr.lines() {
                        Self::emit_stderr_line(state, line);
                    }
                }

                if result.status.success() {
                    // Try to parse JSON output for better status
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                        if let Some(success) = json.get("success").and_then(|v| v.as_bool()) {
                            if success {
                                // Update status
                                {
                                    let mut status = state.status.write().await;
                                    status.is_ready = true;
                                    status.mode = SystemMode::Idle;
                                }
                                Self::emit_success(state, "Repository initialized successfully");
                                Self::emit_step_end(state, "Initialize Repository", "success");
                                return Ok("Initialized successfully".to_string());
                            }
                        }
                    }

                    // Non-JSON success
                    {
                        let mut status = state.status.write().await;
                        status.is_ready = true;
                        status.mode = SystemMode::Idle;
                    }
                    Self::emit_success(state, "Repository initialized");
                    Self::emit_step_end(state, "Initialize Repository", "success");
                    Ok("Initialized".to_string())
                } else {
                    // Command failed
                    {
                        let mut status = state.status.write().await;
                        status.mode = SystemMode::Idle;
                    }
                    let error_msg = if stderr.is_empty() {
                        format!("Init failed with exit code: {:?}", result.status.code())
                    } else {
                        stderr.to_string()
                    };
                    Self::emit_error(state, &error_msg);
                    Self::emit_step_end(state, "Initialize Repository", "failed");
                    Err(error_msg)
                }
            }
            Err(e) => {
                {
                    let mut status = state.status.write().await;
                    status.mode = SystemMode::Idle;
                }
                let error_msg = format!("Failed to execute init command: {}", e);
                Self::emit_error(state, &error_msg);
                Self::emit_step_end(state, "Initialize Repository", "failed");
                Err(error_msg)
            }
        }
    }

    pub async fn run_git_init(state: &AppState) -> Result<String, String> {
        Self::emit_step_start(state, "Git Initialize");
        Self::emit_log(state, "Initializing git repository...");

        // Get current directory
        let cwd = std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?;

        Self::emit_log(state, &format!("Working directory: {}", cwd.display()));

        // Check if already a git repo
        let git_dir = cwd.join(".git");
        if git_dir.exists() {
            Self::emit_log(state, "Git repository already exists");
            Self::emit_success(state, "Already a git repository");
            Self::emit_step_end(state, "Git Initialize", "success");
            return Ok("Already initialized".to_string());
        }

        Self::emit_log(state, "Running: git init");

        let output = Command::new("git")
            .arg("init")
            .current_dir(&cwd)
            .output();

        match output {
            Ok(result) => {
                let stdout = String::from_utf8_lossy(&result.stdout);
                let stderr = String::from_utf8_lossy(&result.stderr);

                // Log output
                if !stdout.is_empty() {
                    for line in stdout.lines() {
                        Self::emit_log(state, line);
                    }
                }

                if !stderr.is_empty() {
                    for line in stderr.lines() {
                        Self::emit_log(state, line);
                    }
                }

                if result.status.success() {
                    // Configure git user for the repo
                    Self::emit_log(state, "Configuring git user...");
                    
                    let _ = Command::new("git")
                        .args(["config", "user.email", "chakravarti@local"])
                        .current_dir(&cwd)
                        .output();
                    
                    let _ = Command::new("git")
                        .args(["config", "user.name", "Chakravarti"])
                        .current_dir(&cwd)
                        .output();

                    // Update status
                    {
                        let mut status = state.status.write().await;
                        status.active_branch = "main".to_string();
                    }

                    Self::emit_success(state, "Git repository initialized successfully");
                    Self::emit_step_end(state, "Git Initialize", "success");
                    Ok("Git initialized".to_string())
                } else {
                    let error_msg = if stderr.is_empty() {
                        format!("git init failed with exit code: {:?}", result.status.code())
                    } else {
                        stderr.to_string()
                    };
                    Self::emit_error(state, &error_msg);
                    Self::emit_step_end(state, "Git Initialize", "failed");
                    Err(error_msg)
                }
            }
            Err(e) => {
                let error_msg = format!("Failed to execute git init: {}", e);
                Self::emit_error(state, &error_msg);
                Self::emit_step_end(state, "Git Initialize", "failed");
                Err(error_msg)
            }
        }
    }

    pub async fn run_spec_new(state: &AppState, description: &str, name: Option<&str>) -> Result<String, String> {
        Self::emit_step_start(state, "Create Specification");
        Self::emit_log(state, &format!("Creating new specification: \"{}\"", description));

        // Update mode to running
        {
            let mut status = state.status.write().await;
            status.mode = SystemMode::Planning;
        }

        // Get current directory
        let cwd = std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?;

        Self::emit_log(state, &format!("Working directory: {}", cwd.display()));

        // Build the command
        let exe = std::env::current_exe()
            .map_err(|e| format!("Failed to get executable path: {}", e))?;

        let mut args = vec!["spec", "new", "--json"];
        
        // Add name if provided
        let name_args: Vec<String>;
        if let Some(n) = name {
            name_args = vec!["-n".to_string(), n.to_string()];
            args.push("-n");
            args.push(name_args[1].as_str());
        }
        
        // Description must be last
        args.push(description);

        Self::emit_log(state, &format!("Running: {} {}", exe.display(), args.join(" ")));

        let output = Command::new(&exe)
            .args(&args)
            .current_dir(&cwd)
            .output();

        // Reset mode
        {
            let mut status = state.status.write().await;
            status.mode = SystemMode::Idle;
        }

        match output {
            Ok(result) => {
                let stdout = String::from_utf8_lossy(&result.stdout);
                let stderr = String::from_utf8_lossy(&result.stderr);

                // Log any output
                if !stdout.is_empty() {
                    for line in stdout.lines() {
                        Self::emit_log(state, line);
                    }
                }

                if !stderr.is_empty() {
                    for line in stderr.lines() {
                        Self::emit_stderr_line(state, line);
                    }
                }

                if result.status.success() {
                    // Try to parse JSON output for better messaging
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                        if let Some(spec_file) = json.get("spec_file").and_then(|v| v.as_str()) {
                            Self::emit_success(state, &format!("Specification created: {}", spec_file));
                        } else {
                            Self::emit_success(state, "Specification created successfully");
                        }
                    } else {
                        Self::emit_success(state, "Specification created successfully");
                    }
                    Self::emit_step_end(state, "Create Specification", "success");
                    Ok("Specification created".to_string())
                } else {
                    // Try to get error message from JSON output
                    let error_msg = if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                        if let Some(err) = json.get("error").and_then(|v| v.as_str()) {
                            if !err.is_empty() {
                                err.to_string()
                            } else {
                                json.get("code")
                                    .and_then(|v| v.as_str())
                                    .map(|c| format!("Command failed: {}", c))
                                    .unwrap_or_else(|| format!("spec new failed with exit code: {:?}", result.status.code()))
                            }
                        } else {
                            format!("spec new failed with exit code: {:?}", result.status.code())
                        }
                    } else {
                        format!("spec new failed with exit code: {:?}", result.status.code())
                    };
                    Self::emit_error(state, &error_msg);
                    Self::emit_step_end(state, "Create Specification", "failed");
                    Err(error_msg)
                }
            }
            Err(e) => {
                let error_msg = format!("Failed to execute spec new command: {}", e);
                Self::emit_error(state, &error_msg);
                Self::emit_step_end(state, "Create Specification", "failed");
                Err(error_msg)
            }
        }
    }

    pub async fn run_spec_tasks(state: &AppState) -> Result<String, String> {
        Self::emit_step_start(state, "Generate Tasks");
        Self::emit_log(state, "Generating implementation tasks from specification...");

        // Update mode to planning
        {
            let mut status = state.status.write().await;
            status.mode = SystemMode::Planning;
        }

        // Get current directory
        let cwd = std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?;

        Self::emit_log(state, &format!("Working directory: {}", cwd.display()));

        // Build the command
        let exe = std::env::current_exe()
            .map_err(|e| format!("Failed to get executable path: {}", e))?;

        let args = vec!["spec", "tasks", "--json"];

        Self::emit_log(state, &format!("Running: {} {}", exe.display(), args.join(" ")));

        let output = Command::new(&exe)
            .args(&args)
            .current_dir(&cwd)
            .output();

        // Reset mode
        {
            let mut status = state.status.write().await;
            status.mode = SystemMode::Idle;
        }

        match output {
            Ok(result) => {
                let stdout = String::from_utf8_lossy(&result.stdout);
                let stderr = String::from_utf8_lossy(&result.stderr);

                // Log any output
                if !stdout.is_empty() {
                    for line in stdout.lines() {
                        Self::emit_log(state, line);
                    }
                }

                if !stderr.is_empty() {
                    for line in stderr.lines() {
                        Self::emit_stderr_line(state, line);
                    }
                }

                if result.status.success() {
                    // Try to parse JSON output
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                        if let Some(count) = json.get("task_count").and_then(|v| v.as_u64()) {
                            Self::emit_success(state, &format!("Generated {} tasks", count));
                        } else {
                            Self::emit_success(state, "Tasks generated successfully");
                        }
                    } else {
                        Self::emit_success(state, "Tasks generated successfully");
                    }
                    Self::emit_step_end(state, "Generate Tasks", "success");
                    Ok("Tasks generated".to_string())
                } else {
                    // Try to get error message from JSON output
                    let error_msg = if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                        json.get("error")
                            .and_then(|v| v.as_str())
                            .filter(|s| !s.is_empty())
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| format!("spec tasks failed with exit code: {:?}", result.status.code()))
                    } else {
                        format!("spec tasks failed with exit code: {:?}", result.status.code())
                    };
                    Self::emit_error(state, &error_msg);
                    Self::emit_step_end(state, "Generate Tasks", "failed");
                    Err(error_msg)
                }
            }
            Err(e) => {
                let error_msg = format!("Failed to execute spec tasks command: {}", e);
                Self::emit_error(state, &error_msg);
                Self::emit_step_end(state, "Generate Tasks", "failed");
                Err(error_msg)
            }
        }
    }

    pub async fn run_run(state: &AppState, dry_run: bool) -> Result<String, String> {
        let step_name = if dry_run { "Plan Execution (Dry Run)" } else { "Execute Tasks" };
        Self::emit_step_start(state, step_name);
        Self::emit_log(state, if dry_run { "Running execution plan (dry-run)..." } else { "Running AI agents to execute tasks..." });

        // Update mode to running
        {
            let mut status = state.status.write().await;
            status.mode = SystemMode::Running;
        }

        // Get current directory
        let cwd = std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?;

        Self::emit_log(state, &format!("Working directory: {}", cwd.display()));

        // Build the command
        let exe = std::env::current_exe()
            .map_err(|e| format!("Failed to get executable path: {}", e))?;

        let cmd_str = if dry_run { format!("{} run --dry-run", exe.display()) } else { format!("{} run", exe.display()) };
        Self::emit_log(state, &format!("Running: {}", cmd_str));

        // Use tokio Command for async streaming
        let mut cmd = tokio::process::Command::new(&exe);
        cmd.arg("run");
        if dry_run {
            cmd.arg("--dry-run");
        }
        
        let mut child = cmd
            .current_dir(&cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                let error_msg = format!("Failed to spawn run command: {}", e);
                Self::emit_error(state, &error_msg);
                Self::emit_step_end(state, step_name, "failed");
                // Reset mode
                let state_clone = state.clone();
                tokio::spawn(async move {
                    let mut status = state_clone.status.write().await;
                    status.mode = SystemMode::Idle;
                });
                error_msg
            })?;

        // Stream stdout
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        
        let state_stdout = state.clone();
        let state_stderr = state.clone();

        // Spawn task to read stdout
        let stdout_handle = if let Some(stdout) = stdout {
            Some(tokio::spawn(async move {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    CommandService::emit_log(&state_stdout, &line);
                }
            }))
        } else {
            None
        };

        // Spawn task to read stderr
        let stderr_handle = if let Some(stderr) = stderr {
            Some(tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    CommandService::emit_stderr_line(&state_stderr, &line);
                }
            }))
        } else {
            None
        };

        // Wait for the process to complete
        let status = child.wait().await;

        // Wait for output readers to finish
        if let Some(handle) = stdout_handle {
            let _ = handle.await;
        }
        if let Some(handle) = stderr_handle {
            let _ = handle.await;
        }

        // Reset mode
        {
            let mut sys_status = state.status.write().await;
            sys_status.mode = SystemMode::Idle;
        }

        match status {
            Ok(exit_status) => {
                if exit_status.success() {
                    Self::emit_success(state, "Tasks executed successfully");
                    Self::emit_step_end(state, "Execute Tasks", "success");
                    Ok("Tasks executed".to_string())
                } else {
                    let error_msg = format!("run failed with exit code: {:?}", exit_status.code());
                    Self::emit_error(state, &error_msg);
                    Self::emit_step_end(state, "Execute Tasks", "failed");
                    Err(error_msg)
                }
            }
            Err(e) => {
                let error_msg = format!("Failed to wait for run command: {}", e);
                Self::emit_error(state, &error_msg);
                Self::emit_step_end(state, "Execute Tasks", "failed");
                Err(error_msg)
            }
        }
    }

    pub async fn run_diff(
        state: &AppState,
        base: Option<&str>,
        stat: bool,
        files: bool,
        summary: bool,
    ) -> Result<serde_json::Value, String> {
        Self::emit_step_start(state, "View Diff");
        Self::emit_log(state, "Getting diff between branches...");

        let cwd = std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?;

        let exe = std::env::current_exe()
            .map_err(|e| format!("Failed to get executable path: {}", e))?;

        let mut args = vec!["diff", "--json"];
        
        if let Some(b) = base {
            args.push("--base");
            args.push(b);
        }
        if stat {
            args.push("--stat");
        }
        if files {
            args.push("--files");
        }
        if summary {
            args.push("--summary");
        }

        Self::emit_log(state, &format!("Running: {} {}", exe.display(), args.join(" ")));

        let output = Command::new(&exe)
            .args(&args)
            .current_dir(&cwd)
            .output();

        match output {
            Ok(result) => {
                let stdout = String::from_utf8_lossy(&result.stdout);
                let stderr = String::from_utf8_lossy(&result.stderr);

                if !stderr.is_empty() {
                    for line in stderr.lines() {
                        Self::emit_stderr_line(state, line);
                    }
                }

                if result.status.success() {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                        // Log diff summary
                        if let Some(current) = json.get("current_branch").and_then(|v| v.as_str()) {
                            if let Some(base) = json.get("base_branch").and_then(|v| v.as_str()) {
                                Self::emit_log(state, &format!("Comparing: {} → {}", base, current));
                            }
                        }

                        // Log statistics
                        let files_changed = json.get("files_changed").and_then(|v| v.as_u64()).unwrap_or(0);
                        let lines_added = json.get("lines_added").and_then(|v| v.as_u64()).unwrap_or(0);
                        let lines_removed = json.get("lines_removed").and_then(|v| v.as_u64()).unwrap_or(0);
                        
                        Self::emit_log(state, "");
                        Self::emit_log(state, "📊 Diff Summary:");
                        Self::emit_log(state, &format!("   {} files changed", files_changed));
                        Self::emit_log(state, &format!("   +{} insertions", lines_added));
                        Self::emit_log(state, &format!("   -{} deletions", lines_removed));

                        // Log changed files
                        if let Some(files) = json.get("files").and_then(|v| v.as_array()) {
                            if !files.is_empty() {
                                Self::emit_log(state, "");
                                Self::emit_log(state, "📁 Changed Files:");
                                for file in files.iter().take(20) {
                                    if let Some(filename) = file.get("file").and_then(|v| v.as_str()) {
                                        let status = file.get("status").and_then(|v| v.as_str()).unwrap_or("modified");
                                        let ins = file.get("insertions").and_then(|v| v.as_u64()).unwrap_or(0);
                                        let del = file.get("deletions").and_then(|v| v.as_u64()).unwrap_or(0);
                                        let icon = match status {
                                            "added" => "+",
                                            "deleted" => "-",
                                            "renamed" => "→",
                                            _ => "~",
                                        };
                                        Self::emit_log(state, &format!("   {} {} (+{}/-{})", icon, filename, ins, del));
                                    }
                                }
                                if files.len() > 20 {
                                    Self::emit_log(state, &format!("   ... and {} more files", files.len() - 20));
                                }
                            }
                        }

                        Self::emit_log(state, "");
                        Self::emit_success(state, "Diff retrieved successfully");
                        Self::emit_step_end(state, "View Diff", "success");
                        Ok(json)
                    } else {
                        Self::emit_success(state, "Diff retrieved");
                        Self::emit_step_end(state, "View Diff", "success");
                        Ok(serde_json::json!({ "output": stdout.to_string() }))
                    }
                } else {
                    let error_msg = format!("diff failed with exit code: {:?}", result.status.code());
                    Self::emit_error(state, &error_msg);
                    Self::emit_step_end(state, "View Diff", "failed");
                    Err(error_msg)
                }
            }
            Err(e) => {
                let error_msg = format!("Failed to execute diff command: {}", e);
                Self::emit_error(state, &error_msg);
                Self::emit_step_end(state, "View Diff", "failed");
                Err(error_msg)
            }
        }
    }

    pub async fn run_verify(
        state: &AppState,
        lint: bool,
        typecheck: bool,
        test: bool,
        fix: bool,
    ) -> Result<serde_json::Value, String> {
        Self::emit_step_start(state, "Verify Code");
        Self::emit_log(state, "Running verification checks...");

        // Update mode to running
        {
            let mut status = state.status.write().await;
            status.mode = SystemMode::Running;
        }

        let cwd = std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?;

        let exe = std::env::current_exe()
            .map_err(|e| format!("Failed to get executable path: {}", e))?;

        let mut args = vec!["verify", "--json"];
        
        if lint {
            args.push("--lint");
        }
        if typecheck {
            args.push("--type");
        }
        if test {
            args.push("--test");
        }
        if fix {
            args.push("--fix");
        }

        Self::emit_log(state, &format!("Running: {} {}", exe.display(), args.join(" ")));

        // Use streaming for verify as it can take a while
        let mut child = tokio::process::Command::new(&exe)
            .args(&args)
            .current_dir(&cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                let error_msg = format!("Failed to spawn verify command: {}", e);
                Self::emit_error(state, &error_msg);
                Self::emit_step_end(state, "Verify Code", "failed");
                error_msg
            })?;

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        let state_stderr = state.clone();

        // Stream stderr for progress
        let stderr_handle = if let Some(stderr) = stderr {
            Some(tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    CommandService::emit_stderr_line(&state_stderr, &line);
                }
            }))
        } else {
            None
        };

        // Collect stdout for JSON result
        let stdout_content = if let Some(stdout) = stdout {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            let mut content = String::new();
            while let Ok(Some(line)) = lines.next_line().await {
                content.push_str(&line);
                content.push('\n');
            }
            content
        } else {
            String::new()
        };

        let status = child.wait().await;

        if let Some(handle) = stderr_handle {
            let _ = handle.await;
        }

        // Reset mode
        {
            let mut sys_status = state.status.write().await;
            sys_status.mode = SystemMode::Idle;
        }

        match status {
            Ok(exit_status) => {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout_content) {
                    // Log detailed check results
                    Self::emit_log(state, "");
                    Self::emit_log(state, "🔍 Verification Results:");
                    
                    if let Some(checks) = json.get("checks").and_then(|v| v.as_array()) {
                        for check in checks {
                            let name = check.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown");
                            let passed = check.get("passed").and_then(|v| v.as_bool()).unwrap_or(false);
                            let duration = check.get("duration_ms").and_then(|v| v.as_u64()).unwrap_or(0);
                            
                            let icon = if passed { "✅" } else { "❌" };
                            Self::emit_log(state, &format!("   {} {} ({}ms)", icon, name, duration));
                            
                            // Log error details if check failed
                            if !passed {
                                if let Some(error) = check.get("error").and_then(|v| v.as_str()) {
                                    // Log first few lines of error
                                    for (i, line) in error.lines().take(10).enumerate() {
                                        if i == 0 {
                                            Self::emit_log(state, &format!("      └─ {}", line));
                                        } else {
                                            Self::emit_log(state, &format!("         {}", line));
                                        }
                                    }
                                    let total_lines = error.lines().count();
                                    if total_lines > 10 {
                                        Self::emit_log(state, &format!("         ... and {} more lines", total_lines - 10));
                                    }
                                }
                            }
                        }
                    }

                    // Log summary
                    if let Some(summary) = json.get("summary") {
                        let passed = summary.get("passed").and_then(|v| v.as_u64()).unwrap_or(0);
                        let failed = summary.get("failed").and_then(|v| v.as_u64()).unwrap_or(0);
                        let duration = json.get("total_duration_ms").and_then(|v| v.as_u64()).unwrap_or(0);
                        
                        Self::emit_log(state, "");
                        Self::emit_log(state, &format!("📊 Summary: {} passed, {} failed ({}ms total)", passed, failed, duration));
                    }

                    Self::emit_log(state, "");
                    
                    if exit_status.success() {
                        Self::emit_success(state, "All checks passed");
                        Self::emit_step_end(state, "Verify Code", "success");
                    } else {
                        Self::emit_error(state, "Some checks failed");
                        Self::emit_step_end(state, "Verify Code", "failed");
                    }
                    Ok(json)
                } else {
                    if exit_status.success() {
                        Self::emit_success(state, "Verification complete");
                        Self::emit_step_end(state, "Verify Code", "success");
                        Ok(serde_json::json!({ "success": true }))
                    } else {
                        Self::emit_error(state, "Verification failed");
                        Self::emit_step_end(state, "Verify Code", "failed");
                        Err("Verification failed".to_string())
                    }
                }
            }
            Err(e) => {
                let error_msg = format!("Failed to wait for verify command: {}", e);
                Self::emit_error(state, &error_msg);
                Self::emit_step_end(state, "Verify Code", "failed");
                Err(error_msg)
            }
        }
    }

    pub async fn run_promote(
        state: &AppState,
        base: Option<&str>,
        draft: bool,
        push: bool,
    ) -> Result<serde_json::Value, String> {
        Self::emit_step_start(state, "Create Pull Request");
        Self::emit_log(state, "Creating pull request...");

        let cwd = std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?;

        let exe = std::env::current_exe()
            .map_err(|e| format!("Failed to get executable path: {}", e))?;

        let mut args = vec!["promote", "--json"];
        
        if let Some(b) = base {
            args.push("--base");
            args.push(b);
        }
        if draft {
            args.push("--draft");
        }
        if push {
            args.push("--push");
        }

        Self::emit_log(state, &format!("Running: {} {}", exe.display(), args.join(" ")));

        let output = Command::new(&exe)
            .args(&args)
            .current_dir(&cwd)
            .output();

        match output {
            Ok(result) => {
                let stdout = String::from_utf8_lossy(&result.stdout);
                let stderr = String::from_utf8_lossy(&result.stderr);

                if !stdout.is_empty() {
                    for line in stdout.lines() {
                        Self::emit_log(state, line);
                    }
                }

                if !stderr.is_empty() {
                    for line in stderr.lines() {
                        Self::emit_stderr_line(state, line);
                    }
                }

                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                    if result.status.success() {
                        if let Some(url) = json.get("pr_url").and_then(|v| v.as_str()) {
                            Self::emit_success(state, &format!("PR created: {}", url));
                        } else {
                            Self::emit_success(state, "Ready to create PR");
                        }
                        Self::emit_step_end(state, "Create Pull Request", "success");
                    } else {
                        let msg = json.get("message").and_then(|v| v.as_str()).unwrap_or("Failed");
                        Self::emit_error(state, msg);
                        Self::emit_step_end(state, "Create Pull Request", "failed");
                    }
                    Ok(json)
                } else {
                    if result.status.success() {
                        Self::emit_success(state, "Promote complete");
                        Self::emit_step_end(state, "Create Pull Request", "success");
                        Ok(serde_json::json!({ "success": true }))
                    } else {
                        let error_msg = format!("promote failed with exit code: {:?}", result.status.code());
                        Self::emit_error(state, &error_msg);
                        Self::emit_step_end(state, "Create Pull Request", "failed");
                        Err(error_msg)
                    }
                }
            }
            Err(e) => {
                let error_msg = format!("Failed to execute promote command: {}", e);
                Self::emit_error(state, &error_msg);
                Self::emit_step_end(state, "Create Pull Request", "failed");
                Err(error_msg)
            }
        }
    }

    pub async fn run_fix(
        state: &AppState,
        lint: bool,
        typecheck: bool,
        test: bool,
        check: bool,
        error: Option<&str>,
    ) -> Result<serde_json::Value, String> {
        Self::emit_step_start(state, "Fix with AI");
        Self::emit_log(state, "Running AI to fix verification errors...");

        // Update mode to running
        {
            let mut status = state.status.write().await;
            status.mode = SystemMode::Running;
        }

        let cwd = std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?;

        let exe = std::env::current_exe()
            .map_err(|e| format!("Failed to get executable path: {}", e))?;

        let mut args = vec!["fix", "--json"];
        
        if lint {
            args.push("--lint");
        }
        if typecheck {
            args.push("--type");
        }
        if test {
            args.push("--test");
        }
        if check {
            args.push("--check");
        }
        
        // Store the error string so it lives long enough
        let error_string;
        if let Some(e) = error {
            error_string = e.to_string();
            args.push("--error");
            args.push(&error_string);
        }

        Self::emit_log(state, &format!("Running: {} {}", exe.display(), args.join(" ")));

        // Use streaming for fix as it can take a while (Claude interaction)
        let mut child = tokio::process::Command::new(&exe)
            .args(&args)
            .current_dir(&cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                let error_msg = format!("Failed to spawn fix command: {}", e);
                Self::emit_error(state, &error_msg);
                Self::emit_step_end(state, "Fix with AI", "failed");
                error_msg
            })?;

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        let state_stdout = state.clone();
        let state_stderr = state.clone();

        // Stream stdout for Claude output
        let stdout_handle = if let Some(stdout) = stdout {
            Some(tokio::spawn(async move {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();
                let mut json_content = String::new();
                while let Ok(Some(line)) = lines.next_line().await {
                    // Try to detect if it's JSON output (starts with { or is part of JSON)
                    if line.starts_with('{') || line.starts_with('"') {
                        json_content.push_str(&line);
                        json_content.push('\n');
                    } else {
                        // Log non-JSON lines (progress output)
                        if !line.trim().is_empty() {
                            CommandService::emit_log(&state_stdout, &format!("🤖 {}", line));
                        }
                    }
                }
                json_content
            }))
        } else {
            None
        };

        // Stream stderr for progress
        let stderr_handle = if let Some(stderr) = stderr {
            Some(tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    CommandService::emit_stderr_line(&state_stderr, &line);
                }
            }))
        } else {
            None
        };

        // Get the stdout content
        let stdout_content = if let Some(handle) = stdout_handle {
            handle.await.unwrap_or_default()
        } else {
            String::new()
        };

        let status = child.wait().await;

        if let Some(handle) = stderr_handle {
            let _ = handle.await;
        }

        // Reset mode
        {
            let mut sys_status = state.status.write().await;
            sys_status.mode = SystemMode::Idle;
        }

        match status {
            Ok(exit_status) => {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout_content) {
                    // Log results
                    Self::emit_log(state, "");
                    
                    let success = json.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
                    let fixes = json.get("fixes_applied").and_then(|v| v.as_u64()).unwrap_or(0);
                    let remaining = json.get("errors_remaining").and_then(|v| v.as_u64()).unwrap_or(0);
                    
                    if success {
                        Self::emit_log(state, &format!("✅ {} fix(es) applied", fixes));
                        if remaining > 0 {
                            Self::emit_log(state, &format!("⚠️  {} error(s) remaining", remaining));
                        }
                        Self::emit_success(state, "Fixes applied successfully");
                        Self::emit_step_end(state, "Fix with AI", "success");
                    } else {
                        if let Some(msg) = json.get("message").and_then(|v| v.as_str()) {
                            Self::emit_error(state, msg);
                        }
                        Self::emit_step_end(state, "Fix with AI", "failed");
                    }
                    
                    Ok(json)
                } else {
                    if exit_status.success() {
                        Self::emit_success(state, "Fix complete");
                        Self::emit_step_end(state, "Fix with AI", "success");
                        Ok(serde_json::json!({ "success": true }))
                    } else {
                        Self::emit_error(state, "Fix command failed");
                        Self::emit_step_end(state, "Fix with AI", "failed");
                        Err("Fix failed".to_string())
                    }
                }
            }
            Err(e) => {
                let error_msg = format!("Failed to wait for fix command: {}", e);
                Self::emit_error(state, &error_msg);
                Self::emit_step_end(state, "Fix with AI", "failed");
                Err(error_msg)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{AppState, SystemStatus};
    use crate::hub::Hub;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    #[tokio::test]
    async fn test_run_init() {
        let state = AppState {
            status: Arc::new(RwLock::new(SystemStatus::default())),
            hub: Arc::new(Hub::new()),
        };
        
        // This test may fail if not in a git repo, but that's expected
        let _result = CommandService::run_init(&state).await;
        // Just verify it doesn't panic
    }
}
