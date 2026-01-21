# Agent Configuration API Contracts

## GLMConfig Type

    ```typescript
interface GLMConfig {
  /** Z.AI API key for authentication */
  api_key?: string;
  
  /** Model identifier (e.g., "glm-4.7", "glm-4.5-air") */
  model: string;
  
  /** API timeout in milliseconds (default: 3000000) */
  timeout_ms?: number;
}
```

## AgentType Enum(Extended)

    ```typescript
type AgentType =
  | "claude"
  | "claude_open_router"
  | "claude_glm"  // NEW: GLM Coding Plan
  | "gemini"
  | "codex"
  | "cursor"
  | "amp"
  | "qwen_code"
  | "opencode"
  | "factory_droid"
  | "copilot";
```

## AgentConfig Type(Extended)

    ```typescript
interface AgentConfig {
  id: string;
  name: string;
  agent_type: AgentType;
  level: number;  // 1-5, capability tier
  is_default: boolean;
  is_qa_agent: boolean;
  is_test_writer: boolean;
  enabled: boolean;
  description?: string;
  
  // OpenRouter configuration (for claude_open_router type)
  openrouter?: OpenRouterConfig;
  
  // GLM configuration (for claude_glm type) - NEW
  glm?: GLMConfig;
  
  binary_path?: string;
  extra_args?: string[];
  env_vars?: Record<string, string>;
}
```

## API Endpoints

### POST / api / agents / test

Test agent connection.

** Request **:
```typescript
interface TestAgentRequest {
  agent: AgentConfig;
}
```

    ** Response ** (ClaudeGLM case):
```typescript
interface TestAgentResponse {
  success: boolean;
  message: string;  // e.g., "GLM Coding Plan config valid for model: glm-4.7"
}
```

### GET / api / agents

List all configured agents.

** Response **:
```typescript
interface ListAgentsResponse {
  success: boolean;
  agents: AgentConfig[];  // Includes ClaudeGLM agents
}
```
