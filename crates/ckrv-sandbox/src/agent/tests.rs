//! Tests for the agent module

use super::{
    create_agent, default_agent, AgentConfig, AgentOutput, AgentProvider, AgentType,
    ClaudeProvider, CodexProvider, GeminiProvider, KiloCodeProvider,
};
use std::path::Path;

#[test]
fn test_agent_type_from_str() {
    assert_eq!(AgentType::from_str("claude"), Some(AgentType::Claude));
    assert_eq!(AgentType::from_str("Claude"), Some(AgentType::Claude));
    assert_eq!(AgentType::from_str("claude-code"), Some(AgentType::Claude));
    assert_eq!(AgentType::from_str("codex"), Some(AgentType::Codex));
    assert_eq!(AgentType::from_str("Codex"), Some(AgentType::Codex));
    assert_eq!(AgentType::from_str("openai"), Some(AgentType::Codex));
    assert_eq!(AgentType::from_str("openai-codex"), Some(AgentType::Codex));
    assert_eq!(AgentType::from_str("kilo"), Some(AgentType::KiloCode));
    assert_eq!(AgentType::from_str("Kilo"), Some(AgentType::KiloCode));
    assert_eq!(AgentType::from_str("kilo-code"), Some(AgentType::KiloCode));
    assert_eq!(AgentType::from_str("kilocode"), Some(AgentType::KiloCode));
    assert_eq!(AgentType::from_str("gemini"), Some(AgentType::Gemini));
    assert_eq!(AgentType::from_str("Gemini"), Some(AgentType::Gemini));
    assert_eq!(AgentType::from_str("gemini-cli"), Some(AgentType::Gemini));
    assert_eq!(AgentType::from_str("unknown"), None);
}

#[test]
fn test_agent_type_default() {
    let default = AgentType::default();
    assert_eq!(default, AgentType::Claude);
}

#[test]
fn test_agent_type_display_name() {
    assert_eq!(AgentType::Claude.display_name(), "Claude Code");
    assert_eq!(AgentType::Codex.display_name(), "OpenAI Codex");
    assert_eq!(AgentType::KiloCode.display_name(), "Kilo Code");
    assert_eq!(AgentType::Gemini.display_name(), "Gemini CLI");
}

#[test]
fn test_agent_config_default() {
    let config = AgentConfig::default();
    assert_eq!(config.agent_type, AgentType::Claude);
    assert!(config.model.is_none());
    assert!(config.streaming);
}

#[test]
fn test_agent_config_builder() {
    let config = AgentConfig::new(AgentType::Codex)
        .with_model("gpt-4o")
        .with_streaming(false);

    assert_eq!(config.agent_type, AgentType::Codex);
    assert_eq!(config.model, Some("gpt-4o".to_string()));
    assert!(!config.streaming);
}

#[test]
fn test_create_agent_claude() {
    let agent = create_agent(AgentType::Claude);
    assert_eq!(agent.name(), "Claude Code");
    assert_eq!(agent.agent_type(), AgentType::Claude);
    assert!(agent.required_env_vars().contains(&"ANTHROPIC_API_KEY"));
}

#[test]
fn test_create_agent_codex() {
    let agent = create_agent(AgentType::Codex);
    assert_eq!(agent.name(), "OpenAI Codex");
    assert_eq!(agent.agent_type(), AgentType::Codex);
    assert!(agent.required_env_vars().contains(&"OPENAI_API_KEY"));
}

#[test]
fn test_claude_provider_build_command() {
    let provider = ClaudeProvider::new();
    let config = AgentConfig::new(AgentType::Claude);
    let workdir = Path::new("/workspace");

    let cmd = provider.build_command("test prompt", workdir, &config);

    assert!(cmd.contains(&"claude".to_string()));
    assert!(cmd.contains(&"--print".to_string()));
    assert!(cmd.contains(&"--dangerously-skip-permissions".to_string()));
    assert!(cmd.contains(&"test prompt".to_string()));
}

#[test]
fn test_codex_provider_build_command() {
    let provider = CodexProvider::new();
    let config = AgentConfig::new(AgentType::Codex);
    let workdir = Path::new("/workspace");

    let cmd = provider.build_command("test prompt", workdir, &config);

    assert!(cmd.contains(&"codex".to_string()));
    assert!(cmd.contains(&"--print".to_string()));
    assert!(cmd.contains(&"--full-auto".to_string()));
    assert!(cmd.contains(&"test prompt".to_string()));
}

#[test]
fn test_codex_provider_with_model() {
    let provider = CodexProvider::new();
    let config = AgentConfig::new(AgentType::Codex).with_model("gpt-5.2-codex");
    let workdir = Path::new("/workspace");

    let cmd = provider.build_command("test prompt", workdir, &config);

    assert!(cmd.contains(&"codex".to_string()));
    assert!(cmd.contains(&"--model".to_string()));
    assert!(cmd.contains(&"gpt-5.2-codex".to_string()));
}

#[test]
fn test_default_agent() {
    let agent = default_agent();
    assert_eq!(agent.agent_type(), AgentType::Claude);
}

#[test]
fn test_agent_output_default() {
    let output = AgentOutput::default();
    assert!(!output.success);
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
    assert_eq!(output.exit_code, 0);
}

#[test]
fn test_claude_parse_output_success() {
    let provider = ClaudeProvider::new();
    let result = provider.parse_output("success output", "", 0).unwrap();

    assert!(result.success);
    assert_eq!(result.stdout, "success output");
    assert_eq!(result.exit_code, 0);
}

#[test]
fn test_claude_parse_output_failure() {
    let provider = ClaudeProvider::new();
    let result = provider.parse_output("", "error message", 1).unwrap();

    assert!(!result.success);
    assert_eq!(result.stderr, "error message");
    assert_eq!(result.exit_code, 1);
}

#[test]
fn test_create_agent_kilo() {
    let agent = create_agent(AgentType::KiloCode);
    assert_eq!(agent.name(), "Kilo Code");
    assert_eq!(agent.agent_type(), AgentType::KiloCode);
    assert!(agent.required_env_vars().is_empty());
}

#[test]
fn test_create_agent_gemini() {
    let agent = create_agent(AgentType::Gemini);
    assert_eq!(agent.name(), "Gemini CLI");
    assert_eq!(agent.agent_type(), AgentType::Gemini);
    assert!(agent.required_env_vars().contains(&"GEMINI_API_KEY"));
}

#[test]
fn test_kilo_provider_build_command() {
    let provider = KiloCodeProvider::new();
    let config = AgentConfig::new(AgentType::KiloCode);
    let workdir = Path::new("/workspace");

    let cmd = provider.build_command("test prompt", workdir, &config);

    assert!(cmd.contains(&"kilo".to_string()));
    assert!(cmd.contains(&"run".to_string()));
    assert!(cmd.contains(&"--auto".to_string()));
    assert!(cmd.contains(&"test prompt".to_string()));
    // Streaming is enabled by default
    assert!(cmd.contains(&"--format".to_string()));
    assert!(cmd.contains(&"json".to_string()));
}

#[test]
fn test_kilo_provider_with_model() {
    let provider = KiloCodeProvider::new();
    let config = AgentConfig::new(AgentType::KiloCode).with_model("google/gemini-2.5-pro");
    let workdir = Path::new("/workspace");

    let cmd = provider.build_command("test prompt", workdir, &config);

    assert!(cmd.contains(&"kilo".to_string()));
    assert!(cmd.contains(&"--model".to_string()));
    assert!(cmd.contains(&"google/gemini-2.5-pro".to_string()));
}

#[test]
fn test_kilo_provider_no_streaming() {
    let provider = KiloCodeProvider::new();
    let config = AgentConfig::new(AgentType::KiloCode).with_streaming(false);
    let workdir = Path::new("/workspace");

    let cmd = provider.build_command("test prompt", workdir, &config);

    assert!(cmd.contains(&"kilo".to_string()));
    assert!(cmd.contains(&"--auto".to_string()));
    // Should NOT contain --format json when streaming is disabled
    assert!(!cmd.contains(&"--format".to_string()));
    assert!(!cmd.contains(&"json".to_string()));
}

#[test]
fn test_kilo_parse_output_success() {
    let provider = KiloCodeProvider::new();
    let result = provider.parse_output("success output", "", 0).unwrap();

    assert!(result.success);
    assert_eq!(result.stdout, "success output");
    assert_eq!(result.exit_code, 0);
}

#[test]
fn test_kilo_parse_output_failure() {
    let provider = KiloCodeProvider::new();
    let result = provider.parse_output("", "error message", 1).unwrap();

    assert!(!result.success);
    assert_eq!(result.stderr, "error message");
    assert_eq!(result.exit_code, 1);
}

#[test]
fn test_gemini_provider_build_command() {
    let provider = GeminiProvider::new();
    let config = AgentConfig::new(AgentType::Gemini);
    let workdir = Path::new("/workspace");

    let cmd = provider.build_command("test prompt", workdir, &config);

    assert!(cmd.contains(&"gemini".to_string()));
    assert!(cmd.contains(&"--prompt".to_string()));
    assert!(cmd.contains(&"--yolo".to_string()));
    assert!(cmd.contains(&"test prompt".to_string()));
}

#[test]
fn test_gemini_provider_with_model() {
    let provider = GeminiProvider::new();
    let config = AgentConfig::new(AgentType::Gemini).with_model("gemini-2.5-pro");
    let workdir = Path::new("/workspace");

    let cmd = provider.build_command("test prompt", workdir, &config);

    assert!(cmd.contains(&"gemini".to_string()));
    assert!(cmd.contains(&"--model".to_string()));
    assert!(cmd.contains(&"gemini-2.5-pro".to_string()));
}

#[test]
fn test_gemini_parse_output_success() {
    let provider = GeminiProvider::new();
    let result = provider.parse_output("success output", "", 0).unwrap();

    assert!(result.success);
    assert_eq!(result.stdout, "success output");
    assert_eq!(result.exit_code, 0);
}

#[test]
fn test_gemini_parse_output_failure() {
    let provider = GeminiProvider::new();
    let result = provider.parse_output("", "error message", 1).unwrap();

    assert!(!result.success);
    assert_eq!(result.stderr, "error message");
    assert_eq!(result.exit_code, 1);
}
