# Feature Specification: Agent Orchestration

**Feature Branch**: `003-agent-orchestration`  
**Created**: 2025-12-25  
**Status**: Draft  
**Input**: User description: "Build a meta agent for code implementation similar to Rover, porting its orchestration capabilities to Chakravarti in Rust"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Task Execution with SWE Workflow (Priority: P1)

A developer wants to modify their codebase using a natural language request (e.g., "Add a new route for user login"). They use Chakravarti to initiate a task. The system automatically creates a plan, generates code, and applies it to the project, mimicking the "Software Engineering" (SWE) workflow.

**Why this priority**: this is the core value proposition: automating the code-generation loop (Plan -> Code) which Rover provides. Without this, the agent is just a REPL.

**Independent Test**: Can be tested by running `ckrv task "Create a generic hello world file"` and verifying that a file is created with the correct content and a `plan.md` artifact exists.

**Acceptance Scenarios**:

1. **Given** a clean git repository, **When** the user runs `ckrv task "create hello.txt with 'world' content"`, **Then** the system spins up a sandbox, executes the planning step, executes the implementation step, and the file `hello.txt` exists with "world" inside.
2. **Given** a specific workflow (SWE), **When** a task is started, **Then** the intermediate artifacts (`plan.md`, `summary.md`) are generated in the `.rover/tasks/<id>` (or equivalent) directory.

---

### User Story 2 - Sandboxed Execution (Priority: P1)

The agent runs in a secure, isolated environment to prevent accidental damage to the host system and ensure reproducibility, while still having access to the project source code.

**Why this priority**: Security and safety (Red Flags check). Executing LLM-generated commands directly on one's machine is dangerous. Sandbox is essential for "Enterprise" readiness.

**Independent Test**: Can be tested by asking the agent to run a destructive command (e.g., `rm -rf /`) and verifying it only affects the disposable container, not the host (though testing strict isolation is complex, testing that it *runs* in Docker is sufficient for Acceptance). Also verifying correct volume mounts.

**Acceptance Scenarios**:

1. **Given** a task request, **When** execution starts, **Then** a Docker (or Podman) container is created.
2. **Given** project credentials (e.g., specific dotfiles), **When** the container starts, **Then** these credentials are accessible inside the container so the agent can authenticate with APIs.
3. **Given** the task completes, **When** the user checks running containers, **Then** the sandbox container is stopped/removed.

---

### User Story 3 - Workflow Customization (Priority: P2)

The user wants to define or modify the steps the agent takes (e.g., adding a "Review" step or changing the prompting strategy) without recompiling the CLI.

**Why this priority**: Flexibility. Hardcoding the Plan/Act loop makes it brittle. Porting Rover implies porting its configurability via `.yml` workflows.

**Independent Test**: Create a custom `custom.yml` workflow and run a task using it.

**Acceptance Scenarios**:

1. **Given** a custom workflow definition file, **When** the user runs a task specifying this workflow, **Then** the system executes the specific steps defined in that file instead of the default.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST support defining multi-step agent workflows via configuration files (e.g., YAML), specifying prompts, inputs, and outputs for each step.
- **FR-002**: System MUST support variable substitution in prompts (templating), allowing outputs from previous steps to be injected into subsequent steps.
- **FR-003**: System MUST provide a "Runner" mechanism that iterates through workflow steps, invokes the configured AI agent tool (e.g., Claude Code), and captures its output.
- **FR-004**: System MUST execute the agent workflow within an isolated container environment (Docker), strictly separating agent execution from the host OS.
- **FR-005**: System MUST mount the target project workspace into the sandbox container to allow the agent to read and write code.
- **FR-006**: System MUST securely mount necessary user credentials (e.g., API keys, auth tokens) from the host into the sandbox container.
- **FR-007**: System MUST support parsing structured output from the agent (e.g., JSON blocks, file creation requests) to facilitate data passing between steps.
- **FR-008**: System MUST maintain task state (ID, status, artifacts) on the host filesystem (e.g., in a `.ckrv` or `.rover` directory) for observability and resumption.
- **FR-009**: System MUST support the creation of a temporary git worktree for the task to ensure changes are isolated from the user's working directory until approved/merged.
- **FR-010**: System MUST support machine-readable output via a `--json` flag, outputting structured JSON to stdout for CI/CD integration and scripting.

## Clarifications

### Session 2025-12-25

- Q: Should the `ckrv task` command support a `--json` flag for machine-readable output? → A: Yes, add `--json` flag (outputs structured JSON to stdout)

### Key Entities *(include if feature involves data)*

- **Task**: A specific instance of a user request with a unique ID, status, and associated artifacts.
- **Workflow**: A static definition of a process, composed of a sequence of Steps.
- **Step**: A single unit of execution in a workflow (e.g., "Planning"), containing a prompt template and expected outputs.
- **Sandbox**: The ephemeral container environment where the Agent operates.
- **Agent**: The external LLM-driven tool (e.g., `claude`, `gemini`) invoked to perform cognitive work.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: User can successfully execute a "Hello World" code modification task (Plan + Implement) using the orchestrator in under 2 minutes (assuming average API latency).
- **SC-002**: System correctly parses and executes a reference workflow (like Rover's `swe.yml`) with at least 2 connected steps (Data passing works).
- **SC-003**: 100% of agent execution commands occur within the container boundary (verified by container inspections).
- **SC-004**: System cleanly handles container creation and teardown without leaving orphaned artifacts in 99% of runs.
