//! Tests for the agent module.

// ============================================================
// IMPORTS
// ============================================================

use super::{
    create_agent, default_agent, AgentConfig, AgentOutput, AgentProvider, AgentType,
    AmpProvider, ClaudeProvider, CodexProvider, CursorProvider, FactoryDroidProvider,
    GeminiProvider, GithubCopilotProvider, KiloCodeProvider, MistralVibeProvider,
    OpencodeProvider, QwenProvider,
};
use std::path::Path;

// ============================================================
// AGENT TYPE TESTS
// ============================================================

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
    assert_eq!(AgentType::from_str("cursor"), Some(AgentType::Cursor));
    assert_eq!(AgentType::from_str("cursor-cli"), Some(AgentType::Cursor));
    assert_eq!(AgentType::from_str("amp"), Some(AgentType::Amp));
    assert_eq!(AgentType::from_str("ampcode"), Some(AgentType::Amp));
    assert_eq!(AgentType::from_str("qwen"), Some(AgentType::Qwen));
    assert_eq!(AgentType::from_str("qwen-code"), Some(AgentType::Qwen));
    assert_eq!(AgentType::from_str("qwencode"), Some(AgentType::Qwen));
    assert_eq!(AgentType::from_str("opencode"), Some(AgentType::Opencode));
    assert_eq!(AgentType::from_str("open-code"), Some(AgentType::Opencode));
    assert_eq!(AgentType::from_str("factory"), Some(AgentType::FactoryDroid));
    assert_eq!(AgentType::from_str("factory-droid"), Some(AgentType::FactoryDroid));
    assert_eq!(AgentType::from_str("factory_droid"), Some(AgentType::FactoryDroid));
    assert_eq!(AgentType::from_str("github-copilot"), Some(AgentType::GithubCopilot));
    assert_eq!(AgentType::from_str("copilot"), Some(AgentType::GithubCopilot));
    assert_eq!(AgentType::from_str("gh-copilot"), Some(AgentType::GithubCopilot));
    assert_eq!(AgentType::from_str("mistral-vibe"), Some(AgentType::MistralVibe));
    assert_eq!(AgentType::from_str("mistral_vibe"), Some(AgentType::MistralVibe));
    assert_eq!(AgentType::from_str("vibe"), Some(AgentType::MistralVibe));
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
    assert_eq!(AgentType::Cursor.display_name(), "Cursor");
    assert_eq!(AgentType::Amp.display_name(), "Amp");
    assert_eq!(AgentType::Qwen.display_name(), "Qwen Code");
    assert_eq!(AgentType::Opencode.display_name(), "Opencode");
    assert_eq!(AgentType::FactoryDroid.display_name(), "Factory Droid");
    assert_eq!(AgentType::GithubCopilot.display_name(), "GitHub Copilot");
    assert_eq!(AgentType::MistralVibe.display_name(), "Mistral Vibe");
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

// ============================================================
// PROVIDER TESTS
// ============================================================

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

// ============================================================
// KILO CODE TESTS
// ============================================================

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

// ============================================================
// CURSOR TESTS
// ============================================================

#[test]
fn test_create_agent_cursor() {
    let agent = create_agent(AgentType::Cursor);
    assert_eq!(agent.name(), "Cursor");
    assert_eq!(agent.agent_type(), AgentType::Cursor);
    assert!(agent.required_env_vars().is_empty());
}

#[test]
fn test_cursor_provider_build_command() {
    let provider = CursorProvider::new();
    let config = AgentConfig::new(AgentType::Cursor);
    let workdir = Path::new("/workspace");

    let cmd = provider.build_command("test prompt", workdir, &config);

    assert!(cmd.contains(&"cursor".to_string()));
    assert!(cmd.contains(&"--print".to_string()));
    assert!(cmd.contains(&"test prompt".to_string()));
}

#[test]
fn test_cursor_parse_output_success() {
    let provider = CursorProvider::new();
    let result = provider.parse_output("success output", "", 0).unwrap();

    assert!(result.success);
    assert_eq!(result.stdout, "success output");
    assert_eq!(result.exit_code, 0);
}

#[test]
fn test_cursor_parse_output_failure() {
    let provider = CursorProvider::new();
    let result = provider.parse_output("", "error message", 1).unwrap();

    assert!(!result.success);
    assert_eq!(result.stderr, "error message");
    assert_eq!(result.exit_code, 1);
}

// ============================================================
// AMP TESTS
// ============================================================

#[test]
fn test_create_agent_amp() {
    let agent = create_agent(AgentType::Amp);
    assert_eq!(agent.name(), "Amp");
    assert_eq!(agent.agent_type(), AgentType::Amp);
    assert!(agent.required_env_vars().is_empty());
}

#[test]
fn test_amp_provider_build_command() {
    let provider = AmpProvider::new();
    let config = AgentConfig::new(AgentType::Amp);
    let workdir = Path::new("/workspace");

    let cmd = provider.build_command("test prompt", workdir, &config);

    assert!(cmd.contains(&"amp".to_string()));
    assert!(cmd.contains(&"--execute".to_string()));
    assert!(cmd.contains(&"test prompt".to_string()));
}

#[test]
fn test_amp_parse_output_success() {
    let provider = AmpProvider::new();
    let result = provider.parse_output("success output", "", 0).unwrap();

    assert!(result.success);
    assert_eq!(result.stdout, "success output");
    assert_eq!(result.exit_code, 0);
}

#[test]
fn test_amp_parse_output_failure() {
    let provider = AmpProvider::new();
    let result = provider.parse_output("", "error message", 1).unwrap();

    assert!(!result.success);
    assert_eq!(result.stderr, "error message");
    assert_eq!(result.exit_code, 1);
}

// ============================================================
// QWEN TESTS
// ============================================================

#[test]
fn test_create_agent_qwen() {
    let agent = create_agent(AgentType::Qwen);
    assert_eq!(agent.name(), "qwen-code");
    assert_eq!(agent.agent_type(), AgentType::Qwen);
}

#[test]
fn test_qwen_provider_build_command() {
    let provider = QwenProvider::new();
    let config = AgentConfig::new(AgentType::Qwen);
    let workdir = Path::new("/workspace");

    let cmd = provider.build_command("test prompt", workdir, &config);

    assert!(cmd.contains(&"qwen".to_string()));
    assert!(cmd.contains(&"test prompt".to_string()));
}

#[test]
fn test_qwen_parse_output_success() {
    let provider = QwenProvider::new();
    let result = provider.parse_output("success output", "", 0).unwrap();

    assert!(result.success);
    assert_eq!(result.stdout, "success output");
    assert_eq!(result.exit_code, 0);
}

#[test]
fn test_qwen_parse_output_failure() {
    let provider = QwenProvider::new();
    let result = provider.parse_output("", "error message", 1).unwrap();

    assert!(!result.success);
    assert_eq!(result.stderr, "error message");
    assert_eq!(result.exit_code, 1);
}

// ============================================================
// OPENCODE TESTS
// ============================================================

#[test]
fn test_create_agent_opencode() {
    let agent = create_agent(AgentType::Opencode);
    assert_eq!(agent.name(), "Opencode");
    assert_eq!(agent.agent_type(), AgentType::Opencode);
    assert!(agent.required_env_vars().is_empty());
}

#[test]
fn test_opencode_provider_build_command() {
    let provider = OpencodeProvider::new();
    let config = AgentConfig::new(AgentType::Opencode);
    let workdir = Path::new("/workspace");

    let cmd = provider.build_command("test prompt", workdir, &config);

    assert!(cmd.contains(&"opencode".to_string()));
    assert!(cmd.contains(&"test prompt".to_string()));
}

#[test]
fn test_opencode_parse_output_success() {
    let provider = OpencodeProvider::new();
    let result = provider.parse_output("success output", "", 0).unwrap();

    assert!(result.success);
    assert_eq!(result.stdout, "success output");
    assert_eq!(result.exit_code, 0);
}

#[test]
fn test_opencode_parse_output_failure() {
    let provider = OpencodeProvider::new();
    let result = provider.parse_output("", "error message", 1).unwrap();

    assert!(!result.success);
    assert_eq!(result.stderr, "error message");
    assert_eq!(result.exit_code, 1);
}

// ============================================================
// FACTORY DROID TESTS
// ============================================================

#[test]
fn test_create_agent_factory_droid() {
    let agent = create_agent(AgentType::FactoryDroid);
    assert_eq!(agent.name(), "Factory Droid");
    assert_eq!(agent.agent_type(), AgentType::FactoryDroid);
    assert!(agent.required_env_vars().contains(&"FACTORY_API_KEY"));
}

#[test]
fn test_factory_droid_provider_build_command() {
    let provider = FactoryDroidProvider::new();
    let config = AgentConfig::new(AgentType::FactoryDroid);
    let workdir = Path::new("/workspace");

    let cmd = provider.build_command("test prompt", workdir, &config);

    assert!(cmd.contains(&"droid".to_string()));
    assert!(cmd.contains(&"test prompt".to_string()));
}

#[test]
fn test_factory_droid_parse_output_success() {
    let provider = FactoryDroidProvider::new();
    let result = provider.parse_output("success output", "", 0).unwrap();

    assert!(result.success);
    assert_eq!(result.stdout, "success output");
    assert_eq!(result.exit_code, 0);
}

#[test]
fn test_factory_droid_parse_output_failure() {
    let provider = FactoryDroidProvider::new();
    let result = provider.parse_output("", "error message", 1).unwrap();

    assert!(!result.success);
    assert_eq!(result.stderr, "error message");
    assert_eq!(result.exit_code, 1);
}

// ============================================================
// GITHUB COPILOT TESTS
// ============================================================

#[test]
fn test_create_agent_github_copilot() {
    let agent = create_agent(AgentType::GithubCopilot);
    assert_eq!(agent.name(), "GitHub Copilot");
    assert_eq!(agent.agent_type(), AgentType::GithubCopilot);
    assert!(agent.required_env_vars().is_empty());
}

#[test]
fn test_github_copilot_provider_build_command() {
    let provider = GithubCopilotProvider::new();
    let config = AgentConfig::new(AgentType::GithubCopilot);
    let workdir = Path::new("/workspace");

    let cmd = provider.build_command("test prompt", workdir, &config);

    assert!(cmd.contains(&"gh".to_string()));
    assert!(cmd.contains(&"test prompt".to_string()));
}

#[test]
fn test_github_copilot_parse_output_success() {
    let provider = GithubCopilotProvider::new();
    let result = provider.parse_output("success output", "", 0).unwrap();

    assert!(result.success);
    assert_eq!(result.stdout, "success output");
    assert_eq!(result.exit_code, 0);
}

#[test]
fn test_github_copilot_parse_output_failure() {
    let provider = GithubCopilotProvider::new();
    let result = provider.parse_output("", "error message", 1).unwrap();

    assert!(!result.success);
    assert_eq!(result.stderr, "error message");
    assert_eq!(result.exit_code, 1);
}

// ============================================================
// MISTRAL VIBE TESTS
// ============================================================

#[test]
fn test_create_agent_mistral_vibe() {
    let agent = create_agent(AgentType::MistralVibe);
    assert_eq!(agent.name(), "Mistral Vibe");
    assert_eq!(agent.agent_type(), AgentType::MistralVibe);
    assert!(agent.required_env_vars().contains(&"MISTRAL_API_KEY"));
}

#[test]
fn test_mistral_vibe_provider_build_command() {
    let provider = MistralVibeProvider::new();
    let config = AgentConfig::new(AgentType::MistralVibe);
    let workdir = Path::new("/workspace");

    let cmd = provider.build_command("test prompt", workdir, &config);

    assert!(cmd.contains(&"vibe".to_string()));
    assert!(cmd.contains(&"test prompt".to_string()));
}

#[test]
fn test_mistral_vibe_parse_output_success() {
    let provider = MistralVibeProvider::new();
    let result = provider.parse_output("success output", "", 0).unwrap();

    assert!(result.success);
    assert_eq!(result.stdout, "success output");
    assert_eq!(result.exit_code, 0);
}

#[test]
fn test_mistral_vibe_parse_output_failure() {
    let provider = MistralVibeProvider::new();
    let result = provider.parse_output("", "error message", 1).unwrap();

    assert!(!result.success);
    assert_eq!(result.stderr, "error message");
    assert_eq!(result.exit_code, 1);
}
