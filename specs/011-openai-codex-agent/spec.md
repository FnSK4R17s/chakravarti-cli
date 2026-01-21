# Feature Specification: OpenAI Codex CLI Agent

**Feature Branch**: `011-openai-codex-agent`  
**Created**: 2026-01-21  
**Status**: Draft  
**Input**: User description: "add openai codex cli as an agent"

## Overview

Chakravarti currently supports Claude Code CLI as its primary agent for executing tasks within Docker sandboxes. This feature adds OpenAI Codex CLI as an alternative agent option, allowing users to select their preferred AI coding assistant when running executions. This creates a multi-agent architecture that supports different AI providers interchangeably.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Configure Codex as Default Agent (Priority: P1)

A developer wants to use OpenAI Codex CLI instead of Claude Code for their batch executions. They configure Codex as their preferred agent and run an execution.

**Why this priority**: Core functionality - without agent configuration, users cannot use Codex at all.

**Independent Test**: Can be fully tested by configuring a Codex agent in the config file and running a simple batch execution that creates a file.

**Acceptance Scenarios**:

1. **Given** a user has OpenAI credentials configured, **When** they set codex as the default agent in their config, **Then** all batch executions use Codex CLI instead of Claude Code
2. **Given** Codex is configured as the default agent, **When** the user runs a batch execution, **Then** the execution logs identify Codex as the active agent
3. **Given** no agent is explicitly configured, **When** the user runs an execution, **Then** the system falls back to Claude Code (backward compatibility)

---

### User Story 2 - Per-Execution Agent Selection (Priority: P2)

A developer wants to compare Claude Code and Codex outputs for the same task. They run one execution with Claude and another with Codex without changing global configuration.

**Why this priority**: Improves developer experience by enabling A/B testing and flexibility.

**Independent Test**: Can be tested by running the same spec twice with different `--agent` flag values and comparing outputs.

**Acceptance Scenarios**:

1. **Given** both Claude and Codex are configured, **When** the user runs `ckrv run --agent=codex`, **Then** only Codex CLI is used for that execution
2. **Given** both Claude and Codex are configured, **When** the user runs `ckrv run --agent=claude`, **Then** only Claude Code is used for that execution
3. **Given** a user specifies an unconfigured agent, **When** they run an execution, **Then** the system shows a clear error message with setup instructions

---

### User Story 3 - Agent-Specific Model Selection (Priority: P2)

A developer using Codex wants to use a specific OpenAI model (e.g., GPT-4, GPT-4o) for their executions.

**Why this priority**: Model selection affects quality and cost; users need control over this.

**Independent Test**: Can be tested by configuring different models and verifying the correct model appears in API calls/logs.

**Acceptance Scenarios**:

1. **Given** Codex is configured as the agent, **When** the user specifies a model in config or CLI flags, **Then** that specific OpenAI model is used for execution
2. **Given** no model is specified, **When** Codex runs, **Then** a sensible default model is used (e.g., gpt-4o)

---

### User Story 4 - Agent Status in UI (Priority: P3)

A developer using the web UI wants to see which agent is configured and active for their project.

**Why this priority**: UI visibility is helpful but not blocking for core functionality.

**Independent Test**: Can be tested by checking the UI displays the correct agent name after configuration changes.

**Acceptance Scenarios**:

1. **Given** the user opens the Chakravarti web UI, **When** they view the execution panel, **Then** the currently configured agent is visible
2. **Given** a batch is running, **When** the user views the batch log, **Then** the log header shows which agent is executing that batch

---

### Edge Cases

- What happens when OpenAI API credentials are invalid or expired?
- How does the system handle Codex CLI not being installed in the Docker image?
- What happens if the Codex CLI command syntax differs from Claude Code?
- How does the system handle rate limiting from OpenAI?
- What happens if a user tries to use Codex with a model that doesn't support code generation?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST support at least two agent types: Claude Code CLI and OpenAI Codex CLI
- **FR-002**: System MUST allow users to configure their preferred agent via configuration file (`agents.yaml`)
- **FR-003**: System MUST allow users to override the default agent via CLI flag (`--agent=codex|claude`)
- **FR-004**: System MUST validate agent credentials before attempting execution
- **FR-005**: System MUST install the correct CLI tool in the Docker sandbox based on configured agent
- **FR-006**: System MUST support agent-specific environment variables (OPENAI_API_KEY for Codex, ANTHROPIC_API_KEY for Claude)
- **FR-007**: System MUST log which agent is being used for each batch execution
- **FR-008**: System MUST support model selection for each agent type
- **FR-009**: System MUST provide clear error messages when an agent fails to execute
- **FR-010**: System MUST maintain backward compatibility - existing Claude Code configurations continue to work unchanged

### Key Entities

- **Agent**: Represents an AI coding assistant (type, credentials, default model, CLI command)
- **AgentConfig**: User's agent configuration (selected agent, API keys, model preferences)
- **ExecutionContext**: Runtime context that includes which agent to use for a specific execution

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can successfully complete a batch execution using Codex CLI within the same time frame as Claude Code (±15%)
- **SC-002**: Agent switching requires only configuration change - no code modifications needed by users
- **SC-003**: 100% of existing Claude Code executions continue to work without configuration changes
- **SC-004**: Agent selection is reflected in UI and logs within 5 seconds of execution start
- **SC-005**: Users can configure and use Codex within 5 minutes following documentation

## Assumptions

- OpenAI Codex CLI has a similar interface to Claude Code CLI (accepts prompts, outputs to files)
- Users have valid OpenAI API credentials with sufficient quota
- The Docker sandbox environment can install either CLI tool
- Codex CLI supports non-interactive/batch execution mode similar to Claude's `--print` flag
- Both agents can work with the same task prompt format
