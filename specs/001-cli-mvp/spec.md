# Feature Specification: Chakravarti CLI MVP

**Feature Branch**: `001-cli-mvp`  
**Created**: 2025-12-12  
**Status**: Draft  
**Input**: User description: "Build Chakravarti CLI: a spec-driven agent orchestrator with planner/executor separation, worktree isolation, containerized execution, and cost/time metrics."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Initialize Project for Agent Orchestration (Priority: P1)

As a developer, I want to initialize my repository for Chakravarti so that I can start defining specifications and running agent-driven code changes in an isolated, auditable manner.

**Why this priority**: Foundation for all other functionality. Without initialization, no other commands can work. This is the gateway to using the entire system.

**Independent Test**: Run `ckrv init` in any git repository and verify the expected directory structure is created with proper configuration files.

**Acceptance Scenarios**:

1. **Given** a git repository without Chakravarti configuration, **When** I run `ckrv init`, **Then** the system creates `.specs/` and `.chakravarti/` directories with default configuration
2. **Given** a repository already initialized with Chakravarti, **When** I run `ckrv init`, **Then** the system informs me it's already initialized and does not overwrite existing configuration
3. **Given** a directory that is not a git repository, **When** I run `ckrv init`, **Then** the system displays an error explaining that Chakravarti requires a git repository

---

### User Story 2 - Create and Manage Specifications (Priority: P1)

As a developer, I want to create machine-readable specifications that define what code changes I need, so that agents can execute changes based on clear requirements rather than vague prompts.

**Why this priority**: Specs are the source of truth for all agent work. Without specs, there's nothing to plan or execute. This is fundamental to the "spec-first" philosophy.

**Independent Test**: Run `ckrv spec new <name>` and verify a valid spec template is created that can be parsed by the system.

**Acceptance Scenarios**:

1. **Given** an initialized Chakravarti repository, **When** I run `ckrv spec new add_rate_limiter`, **Then** a new spec file is created at `.specs/add_rate_limiter.yaml` with required fields (id, goal, constraints, acceptance)
2. **Given** an existing spec file, **When** I run `ckrv spec validate <spec_path>`, **Then** the system reports whether the spec is valid or lists specific validation errors
3. **Given** a spec with missing required fields, **When** I attempt to run a job with that spec, **Then** the system rejects it with a clear error message

---

### User Story 3 - Run Spec-Driven Code Changes (Priority: P1)

As a developer, I want to execute a specification and have the system automatically plan, execute, and verify code changes in an isolated worktree, so that my main branch is never touched until I explicitly approve the changes.

**Why this priority**: This is the core value proposition—converting specs into verified code diffs. Without this, the tool provides no value.

**Independent Test**: Run `ckrv run <spec>` with a valid spec and verify it produces a git diff in an isolated worktree without modifying the main branch.

**Acceptance Scenarios**:

1. **Given** a valid specification, **When** I run `ckrv run specs/add_rate_limiter.yaml`, **Then** the system creates a new worktree at `.worktrees/<job_id>/<attempt_id>/`, generates a plan, executes steps, runs verification, and produces a git diff
2. **Given** a running job, **When** verification fails, **Then** the system automatically retries (up to configured max attempts) or replans before giving up
3. **Given** a successful job, **When** execution completes, **Then** I can see the diff, cost report, and time report stored in `.chakravarti/runs/<job_id>/`
4. **Given** any job execution, **When** I check the main branch, **Then** it remains unmodified regardless of job success or failure

---

### User Story 4 - Inspect Job Status and Results (Priority: P2)

As a developer, I want to check the status of running or completed jobs and view their outputs, so that I can understand what changes were made and at what cost.

**Why this priority**: Observability is essential for trust and debugging, but it's secondary to actually being able to run jobs.

**Independent Test**: After running a job, use `ckrv status`, `ckrv diff`, and `ckrv report` commands to verify all outputs are accessible and accurate.

**Acceptance Scenarios**:

1. **Given** a running job, **When** I run `ckrv status <job_id>`, **Then** I see the current phase (planning/executing/verifying), attempt number, and elapsed time
2. **Given** a completed job, **When** I run `ckrv diff <job_id>`, **Then** I see the git diff produced by that job
3. **Given** a completed job, **When** I run `ckrv report <job_id>`, **Then** I see total wall-clock time, per-step latency, token usage per model, and estimated dollar cost

---

### User Story 5 - Promote Verified Changes to a Branch (Priority: P2)

As a developer, I want to promote a successful job's changes to a named branch, so that I can create merge requests and integrate the changes into my workflow.

**Why this priority**: This completes the git-native workflow, but users can manually extract diffs from worktrees if needed.

**Independent Test**: After a successful job, run `ckrv promote <job_id> --branch <name>` and verify a new branch is created with the changes.

**Acceptance Scenarios**:

1. **Given** a successful job, **When** I run `ckrv promote <job_id> --branch add-rate-limiter`, **Then** a new branch is created from the job's worktree containing all changes
2. **Given** a failed job, **When** I attempt to promote it, **Then** the system refuses with a clear error message
3. **Given** a branch name that already exists, **When** I try to promote to it, **Then** the system asks for confirmation or requires a `--force` flag

---

### User Story 6 - Configure Model Selection and Optimization (Priority: P3)

As a developer, I want to choose between optimizing for cost, time, or a balanced approach when running jobs, so that I can control resource usage based on my current priorities.

**Why this priority**: Important for enterprise adoption and cost control, but the system should work with sensible defaults first.

**Independent Test**: Run jobs with different `--optimize` flags and verify the model selection and behavior changes accordingly.

**Acceptance Scenarios**:

1. **Given** a job with `--optimize=cost`, **When** execution runs, **Then** the system prefers cheaper models and may take longer
2. **Given** a job with `--optimize=time`, **When** execution runs, **Then** the system prefers faster models even if more expensive
3. **Given** a job with `--optimize=balanced` (default), **When** execution runs, **Then** the system applies a heuristic balancing cost and time
4. **Given** explicit model flags like `--planner-model=gpt-4.1` and `--executor-model=gpt-4o-mini`, **When** the job runs, **Then** those specific models are used regardless of optimization setting

---

### Edge Cases

- What happens when the planner model API is unreachable during job execution?
- How does the system handle partial failures (some steps succeed, verification fails)?
- What happens when disk space is insufficient for creating worktrees?
- How does the system behave when the user cancels a running job (Ctrl+C)?
- What happens when the spec references files that don't exist in the repository?
- How does the system handle concurrent jobs on the same repository?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST initialize a repository with `.specs/` and `.chakravarti/` directories via a single command
- **FR-002**: System MUST create specification files with required schema (id, goal, constraints, acceptance criteria)
- **FR-003**: System MUST validate specifications before job execution and reject invalid specs with actionable error messages
- **FR-004**: System MUST generate execution plans using a planner model and store them as deterministic DAGs
- **FR-005**: System MUST execute plan steps in isolated git worktrees, never modifying the main branch
- **FR-006**: System MUST run verification (tests, acceptance criteria checks) after execution steps
- **FR-007**: System MUST automatically retry failed attempts up to a configurable maximum
- **FR-008**: System MUST produce git diffs as the primary output of successful jobs
- **FR-009**: System MUST track and report cost metrics (token usage, estimated dollar cost per model)
- **FR-010**: System MUST track and report time metrics (wall-clock time, per-step latency)
- **FR-011**: System MUST store job metadata locally at `.chakravarti/runs/<job_id>/`
- **FR-012**: System MUST support promoting successful job changes to named branches
- **FR-013**: System MUST support configurable model selection for planner and executor roles
- **FR-014**: System MUST support optimization modes (cost, time, balanced) affecting model routing
- **FR-015**: System MUST execute all operations locally without requiring cloud services (except model APIs)
- **FR-016**: System MUST provide structured (JSON) output alongside human-readable output for all commands
- **FR-017**: System MUST enforce a tool allow-list for executor operations to prevent unsafe actions
- **FR-018**: System MUST inject secrets via environment variables only, never storing them in files

### Key Entities

- **Spec**: The source of truth defining a desired code change. Contains id, goal, constraints, and acceptance criteria. Stored in `.specs/` directory.
- **Plan**: A deterministic DAG of execution steps generated by the planner model. Includes step order, dependencies, and expected tools.
- **Job**: A single attempt to execute a spec. Has a unique job_id, tracks attempts, status, and results.
- **Attempt**: One execution cycle within a job. Created in a fresh worktree. May succeed, fail, or trigger retry/replan.
- **Worktree**: An isolated git worktree where execution happens. Path pattern: `.worktrees/<job_id>/<attempt_id>/`
- **Metrics**: Cost and time data for a job. Includes token counts, dollar estimates, wall-clock time, and per-step latency.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can go from `ckrv init` to a verified code diff in under 10 minutes for simple specs
- **SC-002**: System produces identical outputs when given identical specs and repository state (deterministic reruns)
- **SC-003**: Cost per task is at least 30% lower than equivalent single-model agent approaches
- **SC-004**: Main branch remains unmodified across all job executions (100% isolation)
- **SC-005**: All job data (diffs, metrics, logs) is inspectable after completion without external dependencies
- **SC-006**: System works fully offline except for model API calls
- **SC-007**: Zero user code, prompts, or diffs are transmitted beyond the configured model APIs
- **SC-008**: Users can complete the full workflow (init → spec → run → inspect → promote) using only CLI commands with no UI required

## Assumptions

- Users have git installed and initialized in their repository
- Users have access to at least one LLM API (OpenAI, Anthropic, or equivalent) with valid credentials
- Model APIs are accessible via standard HTTPS endpoints
- Repositories are single-repo (multi-repo support is explicitly out of scope for MVP)
- Container runtime (Docker or equivalent) is available for isolated execution
- User environment supports git worktrees (git 2.5+)
