# Feature Specification: GLM Coding Plan CLI Support

**Feature Branch**: `016-glm-cli-support`  
**Created**: 2026-01-29  
**Status**: Draft  
**Input**: User description: "Support GLM Coding Plan to run via CLI as well as UI, just like OpenRouter"

## Overview

Currently, GLM Coding Plan agents can only be used through the `ckrv ui` dashboard. This feature extends GLM support to CLI commands (`ckrv run`, `ckrv task`), matching the existing OpenRouter implementation pattern which works via both CLI and UI.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Run Task with GLM Agent via CLI (Priority: P1)

A developer wants to execute a coding task using their GLM Coding Plan agent directly from the command line without opening the web UI. They specify the agent by name or ID and the CLI handles all Z.AI environment variable configuration.

**Why this priority**: This is the core functionality. CLI users are blocked from using GLM agents entirely. Unblocking this restores CLI/UI parity that users expect.

**Independent Test**: Configure a GLM agent via the global config file, run `ckrv task run --agent "my-glm-agent" -p "create hello.txt"`, verify task completes and logs show "Using GLM Coding Plan".

**Acceptance Scenarios**:

1. **Given** a GLM Coding Plan agent configured in `~/.config/chakravarti/agents.yaml`, **When** the user runs `ckrv run --agent <glm-agent-name>`, **Then** the execution uses the GLM agent with correct Z.AI environment variables.

2. **Given** a GLM agent is selected for CLI execution, **When** viewing execution logs, **Then** logs clearly indicate "Using GLM Coding Plan: <model-name>" just like UI execution.

3. **Given** a GLM agent with invalid API key, **When** the user runs a task, **Then** a clear error message indicates Z.AI authentication failure.

---

### User Story 2 - GLM Agent Discovery in CLI (Priority: P2)

A developer wants to see their available GLM agents when listing agents via CLI, and select one for execution without manually editing config files.

**Why this priority**: Discoverability improves UX but is not blocking - users can still manually configure agents.

**Independent Test**: Run `ckrv agents list`, verify GLM agents appear with correct type badge and can be used in subsequent commands.

**Acceptance Scenarios**:

1. **Given** GLM agents exist in the config, **When** the user runs `ckrv agents list`, **Then** GLM agents appear with a distinctive indicator (e.g., "[GLM]" or "glm-coding-plan" type).

2. **Given** the user wants to test a GLM agent, **When** they run `ckrv agents test <agent-name>`, **Then** the system validates connectivity with Z.AI and reports success/failure.

---

### User Story 3 - Unified Agent Configuration Loading (Priority: P2)

The CLI and UI must load agent configuration from the same source to ensure consistency. A developer configures a GLM agent in the UI and expects it to be immediately available in CLI commands.

**Why this priority**: Configuration consistency is essential for a seamless experience but doesn't block basic functionality.

**Independent Test**: Add a GLM agent via `ckrv ui`, exit UI, run `ckrv agents list`, verify the new agent appears.

**Acceptance Scenarios**:

1. **Given** a GLM agent added via the UI, **When** the user runs CLI commands, **Then** the CLI reads from the same `agents.yaml` file and finds the agent.

2. **Given** a GLM agent configured via CLI/manual file edit, **When** the user opens the UI, **Then** the agent appears in the Agent Manager.

---

### Edge Cases

- What happens when the Z.AI API key is not configured?
  - System displays clear error: "GLM agent '<name>' requires Z.AI API key. Set in agents.yaml or via environment variable."

- What happens when the user specifies an invalid GLM model?
  - Claude Code CLI returns model not found error; system displays this to user.

- What happens when both CLI `--agent` flag and default agent are set?
  - CLI flag takes precedence over any default.

- How does the system handle timeout for long GLM requests?
  - Respect `API_TIMEOUT_MS` from GLM config (default: 3000000ms per Z.AI docs).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST support GLM Coding Plan agents in `ckrv run` command with identical behavior to UI execution.
- **FR-002**: System MUST support GLM Coding Plan agents in `ckrv task run` command.
- **FR-003**: System MUST load GLM agent configuration from `~/.config/chakravarti/agents.yaml` (same as UI).
- **FR-004**: System MUST set correct Z.AI environment variables when executing GLM agents:
  - `ANTHROPIC_BASE_URL=https://api.z.ai/api/anthropic`
  - `ANTHROPIC_AUTH_TOKEN=<api_key>`
  - `ANTHROPIC_API_KEY=""` (empty)
  - `API_TIMEOUT_MS=<timeout_ms>`
  - `ANTHROPIC_DEFAULT_*_MODEL=<model>`
- **FR-005**: System MUST display clear error messages for GLM-specific failures (invalid key, timeout, model not found).
- **FR-006**: System MUST log "Using GLM Coding Plan: <model>" when executing with a GLM agent.
- **FR-007**: Feature MUST be accessible via both CLI and UI with identical behavior (CLI/UI Parity).

### Key Entities

- **GLMConfig**: Configuration for a GLM Coding Plan agent
  - `api_key`: Z.AI API key (required)
  - `model`: Model identifier (glm-4.7, glm-4.5-air)
  - `timeout_ms`: Custom timeout (default: 3000000)

- **AgentType**: Must include `ClaudeGLM` variant for proper identification

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can execute tasks with GLM agents via CLI in under 30 seconds (same as UI).
- **SC-002**: 100% of GLM agent configurations created via UI are accessible via CLI.
- **SC-003**: Error messages for GLM failures include actionable guidance in 100% of cases.
- **SC-004**: CLI logs show identical agent identification as UI logs ("Using GLM Coding Plan: <model>").

## Assumptions

- Z.AI's API endpoint continues to follow Anthropic-compatible format.
- The existing agent configuration file format (`agents.yaml`) can accommodate GLM agents.
- Users have valid Z.AI API keys from their GLM Coding Plan subscription.
- The `ckrv-core` runner can be extended to support GLM without breaking existing Claude/OpenRouter functionality.
