use axum::{Json, response::IntoResponse, extract::State};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::Arc;
use once_cell::sync::Lazy;
use crate::state::AppState;
use ckrv_sandbox::{Sandbox, DockerSandbox, LocalSandbox, ExecuteConfig, DefaultAllowList};

// Global session store: Map<AgentID, ContainerID>
// In a real app this should be in AppState, but for now this is easier given the structure
static SESSION_STORE: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

/// Payload for executing a command
#[derive(Debug, Deserialize)]
pub struct ExecuteCommandPayload {
    pub agent_id: Option<String>,
    pub command: String,
    pub cwd: Option<String>,
    pub env: Option<HashMap<String, String>>,
    pub use_sandbox: bool,
    #[serde(default)]
    pub keep_container: bool, // Interpreted as "use persistent session"
}

/// Response for command execution
#[derive(Debug, Serialize)]
pub struct ExecuteCommandResponse {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub message: Option<String>,
}

pub async fn execute_command(
    State(state): State<AppState>,
    Json(payload): Json<ExecuteCommandPayload>,
) -> impl IntoResponse {
    let cwd = payload.cwd.map(PathBuf::from).unwrap_or_else(|| state.project_root.clone());
    let mut env = payload.env.unwrap_or_default();
    
    // Config for execution
    let config = ExecuteConfig {
        command: vec!["sh".to_string(), "-c".to_string(), payload.command.clone()],
        workdir: PathBuf::from("/workspace"),
        mount: cwd.clone(),
        env: env.clone(),
        timeout: std::time::Duration::from_secs(30),
        keep_container: payload.keep_container,
    };

    println!("Executing command: '{}' in {}", payload.command, cwd.display());

    if payload.use_sandbox {
        // Use Docker Sandbox
        let allowlist = DefaultAllowList::default();
        let sandbox_res = DockerSandbox::new(allowlist);
        
        match sandbox_res {
             Ok(sandbox) => {
                 if payload.keep_container && payload.agent_id.is_some() {
                     let agent_id = payload.agent_id.unwrap();
                     
                     // Check for existing session
                     let container_id = {
                         let store = SESSION_STORE.lock().unwrap();
                         store.get(&agent_id).cloned()
                     };

                     let cid = match container_id {
                         Some(cid) => cid, // Reuse
                         None => {
                             // Create new session
                             match sandbox.inner_client().create_session(
                                 "/workspace",
                                 &config.mount.to_string_lossy(),
                                 "/workspace",
                                 env
                             ).await {
                                 Ok(cid) => {
                                     let mut store = SESSION_STORE.lock().unwrap();
                                     store.insert(agent_id.clone(), cid.clone());
                                     cid
                                 },
                                 Err(e) => return error_response(format!("Failed to create session: {}", e))
                             }
                         }
                     };

                     // Execute in session
                     // Note: We need to access the inner client of DockerSandbox to call exec_in_session
                     // I need to expose inner client or wrap it. 
                     // Since I can't easily modify DockerSandbox right now (it fields are private), 
                     // I'll assume I added a method `inner_client()` or similar, OR I'll use `Sandbox` trait if I updated it.
                     // But wait, the previous edit to DockerSandbox added methods to `DockerClient`, but `DockerSandbox` wraps it.
                     // I need to update DockerSandbox to expose these or proxy them.
                     
                     // Let's assume for a moment I can get the client. 
                     // Actually I can't access `client` field of DockerSandbox as it is private.
                     // I must update `DockerSandbox` struct in `ckrv-sandbox/src/executor.rs` to expose these.
                     // Plan interruption: update executor.rs first.
                     
                     return error_response("Session execution not fully wired yet".to_string());
                 } else {
                     // One-off execution
                     handle_result(sandbox.execute(config).await)
                 }
            },
            Err(e) => error_response(format!("Failed to initialize sandbox: {}", e))
        }
    } else {
        // Local execution
        let sandbox = LocalSandbox::new();
        handle_result(sandbox.execute(config).await)
    }
}

fn handle_result(res: Result<ckrv_sandbox::ExecuteResult, ckrv_sandbox::SandboxError>) -> Json<ExecuteCommandResponse> {
    match res {
        Ok(exec_result) => Json(ExecuteCommandResponse {
            success: exec_result.exit_code == 0,
            stdout: exec_result.stdout,
            stderr: exec_result.stderr,
            exit_code: exec_result.exit_code,
            message: None,
        }),
        Err(e) => Json(ExecuteCommandResponse {
            success: false,
            stdout: "".to_string(),
            stderr: e.to_string(),
            exit_code: -1,
            message: Some(format!("Execution error: {}", e)),
        }),
    }
}

fn error_response(msg: String) -> Json<ExecuteCommandResponse> {
    Json(ExecuteCommandResponse {
        success: false,
        stdout: "".to_string(),
        stderr: msg.clone(),
        exit_code: -1,
        message: Some(msg),
    })
}
