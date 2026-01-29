// GLM CLI Support - Type Contracts
// Feature: 016-glm-cli-support

/// Extended RunnerConfig with GLM support
/// Location: crates/ckrv-core/src/runner.rs
interface RunnerConfig {
    // ... existing fields ...

    /** Z.AI API key for GLM Coding Plan */
    glm_api_key?: string;

    /** GLM model identifier (glm-4.7, glm-4.5-air) */
    glm_model?: string;

    /** Custom timeout in milliseconds (default: 3000000) */
    glm_timeout_ms?: number;
}

/// GLMConfig already exists in UI
/// Location: crates/ckrv-ui/src/api/agents.rs
interface GLMConfig {
    /** Z.AI API key */
    api_key?: string;

    /** Model identifier */
    model: string;

    /** Timeout in milliseconds */
    timeout_ms?: number;
}

/// AgentType enum - ClaudeGLM already exists
/// Location: crates/ckrv-ui/src/api/agents.rs
type AgentType =
    | "Claude"
    | "Codex"
    | "ClaudeOpenRouter"
    | "ClaudeGLM";  // Already exists

/// Environment variables to inject for GLM
interface GLMEnvironment {
    ANTHROPIC_BASE_URL: "https://api.z.ai/api/anthropic";
    ANTHROPIC_AUTH_TOKEN: string;  // glm_api_key
    ANTHROPIC_API_KEY: "";         // Must be empty
    API_TIMEOUT_MS: string;        // glm_timeout_ms or "3000000"
    ANTHROPIC_DEFAULT_SONNET_MODEL: string;  // glm_model
    ANTHROPIC_DEFAULT_OPUS_MODEL: string;    // glm_model
    ANTHROPIC_DEFAULT_HAIKU_MODEL: string;   // glm_model
}
