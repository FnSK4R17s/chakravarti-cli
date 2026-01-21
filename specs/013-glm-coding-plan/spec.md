# Feature Specification: GLM Coding Plan Agent Support

**Feature Branch**: `013-glm-coding-plan`  
**Created**: 2026-01-21  
**Status**: Draft  
**Input**: User description: "Add GLM Coding Plan support as an agent just like OpenRouter + Claude Code. We need to add GLM Coding Plan + Claude Code integration using Z.AI API."

## Overview

Add support for Z.AI's GLM Coding Plan as a new agent type in Chakravarti CLI. This follows the same integration pattern as OpenRouter (using Claude Code CLI with API redirection) but uses Z.AI's API endpoint and authentication. GLM Coding Plan allows users to leverage GLM-4.7 and GLM-4.5-Air models through the familiar Claude Code interface.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Configure GLM Coding Plan Agent (Priority: P1)

A developer wants to use Z.AI's GLM models for code generation instead of Anthropic's native Claude. They configure a new agent in Chakravarti that uses their Z.AI API key and the GLM Coding Plan endpoint.

**Why this priority**: This is the core functionality - without agent configuration, no other features work. Users need to register their Z.AI credentials and select GLM models before any execution.

**Independent Test**: Can be fully tested by creating a new agent in the UI with type "GLM Coding Plan", entering Z.AI API key, and verifying the agent appears in the agents list with correct configuration.

**Acceptance Scenarios**:

1. **Given** the user opens the Agent Manager, **When** they select "Add Agent" and choose "GLM Coding Plan" type, **Then** a configuration form appears with fields for Z.AI API key and model selection (GLM-4.7, GLM-4.5-Air).

2. **Given** a GLM Coding Plan agent is configured with valid credentials, **When** the user saves the agent, **Then** the agent appears in the agents list with a distinctive badge/icon indicating Z.AI/GLM type.

3. **Given** a GLM Coding Plan agent exists, **When** the user clicks "Test Connection", **Then** the system validates the API key format and displays success/failure status.

---

### User Story 2 - Execute Tasks with GLM Agent (Priority: P1)

A developer selects their GLM Coding Plan agent and runs a task execution. The system invokes Claude Code CLI with Z.AI environment variables, routing all API calls through Z.AI's infrastructure.

**Why this priority**: Execution is the primary value proposition - users need to actually run coding tasks with GLM models.

**Independent Test**: Configure a GLM agent, run a simple task (e.g., "create a hello world function"), verify the task completes and the GLM model is used (visible in logs).

**Acceptance Scenarios**:

1. **Given** a configured GLM Coding Plan agent, **When** the user runs a batch execution with this agent selected, **Then** the system sets the correct environment variables (`ANTHROPIC_BASE_URL=https://api.z.ai/api/anthropic`, `ANTHROPIC_AUTH_TOKEN=<z.ai_api_key>`) before invoking Claude Code.

2. **Given** a task is running with GLM agent, **When** viewing execution logs, **Then** the logs clearly indicate "Using GLM Coding Plan" and show the model in use (e.g., "GLM-4.7").

3. **Given** an execution with GLM agent fails due to invalid API key, **When** viewing the error, **Then** the user sees a clear error message indicating authentication failure with Z.AI.

---

### User Story 3 - Interactive Terminal with GLM (Priority: P2)

A developer launches an interactive terminal session with their GLM agent selected. The Docker container is configured with Z.AI environment variables, allowing interactive Claude Code usage with GLM models.

**Why this priority**: Interactive mode is valuable but secondary to batch execution. Many users prefer the execution runner workflow.

**Independent Test**: Start terminal session with GLM agent, run `claude -p "hello"` inside container, verify response comes from GLM model.

**Acceptance Scenarios**:

1. **Given** a GLM Coding Plan agent is selected, **When** the user starts an interactive terminal session, **Then** the Docker container environment includes Z.AI-specific environment variables.

2. **Given** an active terminal session with GLM agent, **When** the user runs `/status` in Claude Code, **Then** the status shows GLM model names (e.g., "glm-4.7") instead of Claude model names.

---

### User Story 4 - Model Switching within GLM (Priority: P3)

A developer wants to switch between GLM-4.7 (high capability) and GLM-4.5-Air (faster, lighter) models for different tasks within their GLM Coding Plan subscription.

**Why this priority**: Model flexibility is a nice-to-have. Most users will stick with the default GLM-4.7.

**Independent Test**: Configure agent with GLM-4.7, run task, then switch to GLM-4.5-Air, run another task, verify different models used.

**Acceptance Scenarios**:

1. **Given** the user is configuring a GLM agent, **When** they select the model dropdown, **Then** they can choose from GLM-4.7, GLM-4.5-Air, or a custom model ID.

2. **Given** multiple GLM agents with different models, **When** viewing the agent list, **Then** each agent displays its configured model name.

---

### Edge Cases

- What happens when Z.AI API is rate-limited?
  - System should display rate limit error from Z.AI and allow retry.
  
- How does system handle expired/invalid Z.AI API keys?
  - Clear error message indicating authentication failure, prompt to update API key.

- What happens when user specifies an invalid model name?
  - Claude Code CLI will return model not found error; system displays this error to user.

- How does system handle timeout for long-running GLM requests?
  - Respect `API_TIMEOUT_MS` environment variable (default 3000000ms per Z.AI docs).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST support a new agent type "GLM Coding Plan" (`claude_glm` or `glm_coding_plan` internally).
- **FR-002**: System MUST store Z.AI API keys securely in the agents configuration file.
- **FR-003**: System MUST set `ANTHROPIC_BASE_URL` to `https://api.z.ai/api/anthropic` when executing GLM agents.
- **FR-004**: System MUST set `ANTHROPIC_AUTH_TOKEN` to the configured Z.AI API key.
- **FR-005**: System MUST set `ANTHROPIC_API_KEY` to empty string to prevent conflicts.
- **FR-006**: System MUST support GLM model selection via `ANTHROPIC_DEFAULT_SONNET_MODEL`, `ANTHROPIC_DEFAULT_OPUS_MODEL`, and `ANTHROPIC_DEFAULT_HAIKU_MODEL` environment variables.
- **FR-007**: System MUST support configurable timeout via `API_TIMEOUT_MS` environment variable (default: 3000000).
- **FR-008**: System MUST display GLM/Z.AI-specific badges in the agent list UI to distinguish from OpenRouter agents.
- **FR-009**: System MUST validate Z.AI API key format before saving (should start with typical API key patterns).
- **FR-010**: System MUST work with both batch execution (sandbox) and interactive terminal sessions.

### Key Entities

- **GLMConfig**: Configuration for GLM Coding Plan integration
  - `api_key`: Z.AI API key
  - `model`: Selected model (glm-4.7, glm-4.5-air, custom)
  - `timeout_ms`: Optional custom timeout
  - `base_url`: Z.AI API endpoint (fixed: `https://api.z.ai/api/anthropic`)

- **AgentType::ClaudeGLM**: New variant in the AgentType enum representing GLM Coding Plan agents

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can create and configure a GLM Coding Plan agent in under 2 minutes (API key + model selection).
- **SC-002**: Batch executions with GLM agent complete successfully with correct model attribution in logs.
- **SC-003**: Interactive terminal sessions with GLM agent allow full Claude Code functionality.
- **SC-004**: Agent test/validation accurately reports Z.AI connectivity status.
- **SC-005**: Users can distinguish GLM agents from OpenRouter and native Claude agents at a glance in the UI.

## Assumptions

- Users have access to Z.AI GLM Coding Plan subscription and API key.
- Z.AI's API endpoint follows Anthropic-compatible format (confirmed by documentation).
- Claude Code CLI version 2.0.14+ is installed in the Docker image.
- The existing Docker containers (ckrv-claude) work with Z.AI API redirection (no Claude-specific code paths block it).
