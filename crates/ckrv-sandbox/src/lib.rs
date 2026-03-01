//! Containerized execution for Chakravarti CLI.
//!
//! This crate provides sandboxed command execution using Docker/Podman.
//! It includes agent provider abstractions, command allowlists, environment
//! configuration, and both Docker-backed and local sandbox implementations.

/// Agent provider abstractions for multiple AI coding assistants.
pub mod agent;
/// Command allowlist for restricting sandbox execution.
pub mod allowlist;
/// Docker/Podman client wrapper for container lifecycle management.
pub mod docker;
/// Environment variable injection and language-specific defaults.
pub mod env;
/// Sandbox error types.
pub mod error;
/// Sandboxed command execution configuration and implementations.
pub mod executor;

pub use agent::{create_agent, default_agent, AgentConfig, AgentOutput, AgentProvider, AgentType};
pub use allowlist::{AllowList, DefaultAllowList};
pub use docker::DockerClient;
pub use env::{detect_env, EnvConfig};
pub use error::SandboxError;
pub use executor::{BindMount, DockerSandbox, ExecuteConfig, ExecuteResult, LocalSandbox, Sandbox};
