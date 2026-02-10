//! Containerized execution for Chakravarti CLI.
//!
//! This crate provides sandboxed command execution using Docker/Podman.

pub mod agent;
pub mod allowlist;
pub mod docker;
pub mod env;
pub mod error;
pub mod executor;

pub use agent::{create_agent, default_agent, AgentConfig, AgentOutput, AgentProvider, AgentType};
pub use allowlist::{AllowList, DefaultAllowList};
pub use docker::DockerClient;
pub use env::{detect_env, EnvConfig};
pub use error::SandboxError;
pub use executor::{BindMount, DockerSandbox, ExecuteConfig, ExecuteResult, LocalSandbox, Sandbox};
