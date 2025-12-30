# Feature Specification: Cloud Executions Support

**Feature Branch**: `005-cloud-executions`  
**Created**: 2025-12-30  
**Status**: Draft  
**Input**: User description: "add support for cloud executions"

---

## Overview

Enable Chakravarti CLI users to run agent orchestration jobs on remote cloud infrastructure instead of their local machine. This allows users to offload computationally intensive AI agent tasks to scalable cloud resources, enabling background execution while the local machine is free for other work.

---

## Clarifications

### Session 2025-12-30

- Q: What is the cloud architecture model? → A: Managed Cloud Service (SaaS) - Chakravarti operates a hosted backend with managed workers to capture value.
- Q: What is the worker isolation model? → A: Orchestrator Pod (Kubernetes) - Each job runs as a K8s Pod with an orchestrator container (`ckrv`) that spawns agent containers (Claude Code, Gemini, etc.) dynamically via K8s API. Security hardening deferred to post-architecture phase.
- Q: What context is transferred to cloud workers? → A: Minimal - Only the spec file + git repository URL + user-specified base branch. Worker clones code from git. Environment variables are the user's responsibility via keyvault/secrets manager (good coding practices).
- Q: How does the cloud worker access private git repositories? → A: User-provided deploy key or personal access token (PAT). User uploads credentials to Chakravarti account settings. Stored encrypted. Used only during job execution.
- Q: What is the billing model? → A: Subscription Tiers - Monthly plans with included job quotas. Overage billed separately. Users pay for value of managed infrastructure, not raw compute cost (avoids cost+ model).
- Q: How are the repositories structured? → A: Split architecture - CLI (`chakravarti-cli`) is open source for transparency. Cloud API & workers (`chakravarti-cloud`) are private/proprietary. OpenAPI contract is the shared interface.

---

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Run Job in Cloud (Priority: P1)

As a developer, I want to execute my `ckrv run` command on a remote cloud worker so that I can continue working locally while the agent completes its task in the background.

**Why this priority**: This is the core value proposition of cloud executions. Without the ability to offload a job to the cloud, the entire feature is moot.

**Independent Test**: Run `ckrv run --cloud` on a valid spec and observe the job starting on a remote worker. The CLI should return immediately with a job ID while the execution continues remotely.

**Acceptance Scenarios**:

1. **Given** a valid spec exists and the user is authenticated with the cloud service, **When** the user runs `ckrv run --cloud`, **Then** the job is dispatched to a remote worker, a unique job ID is returned, and the CLI exits without blocking.
2. **Given** the user is not authenticated with the cloud service, **When** the user runs `ckrv run --cloud`, **Then** the CLI displays a clear error instructing them to authenticate.
3. **Given** the cloud service is unreachable, **When** the user runs `ckrv run --cloud`, **Then** the CLI displays a connection error and suggests running locally as a fallback.

---

### User Story 2 - Monitor Cloud Job Status (Priority: P1)

As a developer, I want to check the status of my running cloud jobs so that I know when they are complete and can review results.

**Why this priority**: Without visibility into running jobs, users cannot trust the system. This is essential for user confidence.

**Independent Test**: After dispatching a cloud job, run `ckrv status <job-id>` and observe accurate real-time status updates (e.g., "pending", "running", "succeeded", "failed").

**Acceptance Scenarios**:

1. **Given** a cloud job is running, **When** the user runs `ckrv status <job-id>`, **Then** the CLI displays the current phase (e.g., planning, executing, verifying), elapsed time, and estimated completion.
2. **Given** a cloud job has completed, **When** the user runs `ckrv status <job-id>`, **Then** the CLI displays "succeeded" or "failed" with a summary of the outcome.
3. **Given** an invalid or expired job ID, **When** the user runs `ckrv status <job-id>`, **Then** the CLI displays "Job not found" with suggestions.

---

### User Story 3 - Stream Cloud Job Logs (Priority: P2)

As a developer, I want to stream logs from a running cloud job to my terminal so that I can observe what the agent is doing in real-time.

**Why this priority**: Log streaming provides transparency, but users can function without it by polling status. It enhances the experience but is not strictly required for MVP.

**Independent Test**: Run `ckrv logs <job-id> --follow` and observe real-time log lines appearing in the terminal as the cloud agent executes.

**Acceptance Scenarios**:

1. **Given** a cloud job is running, **When** the user runs `ckrv logs <job-id> --follow`, **Then** log lines are streamed to the terminal in real-time.
2. **Given** a cloud job has completed, **When** the user runs `ckrv logs <job-id>`, **Then** the full historical log is displayed.

---

### User Story 4 - Retrieve Cloud Job Results (Priority: P1)

As a developer, I want to download the results (diff file, artifacts) of a completed cloud job so that I can review and promote the changes locally.

**Why this priority**: The final deliverable of a job is the diff. Without retrieval, cloud execution is useless.

**Independent Test**: After a cloud job succeeds, run `ckrv pull <job-id>` and observe the diff being applied to the local worktree.

**Acceptance Scenarios**:

1. **Given** a cloud job has succeeded, **When** the user runs `ckrv pull <job-id>`, **Then** the generated changes are merged into the local repository or a designated worktree.
2. **Given** a cloud job failed, **When** the user runs `ckrv pull <job-id>`, **Then** the CLI displays an error explaining no successful output exists.

---

### User Story 5 - Cloud Authentication (Priority: P1)

As a developer, I want to authenticate my CLI with the cloud execution service so that my jobs are secure and tied to my account.

**Why this priority**: Security and identity are foundational. Without authentication, the service cannot function.

**Independent Test**: Run `ckrv cloud login` and complete the authentication flow. Subsequent `ckrv run --cloud` commands should succeed without re-authentication.

**Acceptance Scenarios**:

1. **Given** the user is not logged in, **When** the user runs `ckrv cloud login`, **Then** a browser-based or CLI-based authentication flow is initiated and credentials are stored securely.
2. **Given** the user is logged in, **When** the user runs `ckrv cloud logout`, **Then** credentials are cleared and subsequent cloud commands require re-authentication.
3. **Given** the user is logged in, **When** the user runs `ckrv cloud whoami`, **Then** the CLI displays the authenticated user's identity.

---

### Edge Cases

- What happens when network connection is lost mid-job? (Job continues on cloud; user can reconnect to stream logs later)
- How does the system handle jobs that exceed time limits? (Job is terminated with a timeout error; partial results are not persisted)
- What happens if the user's cloud quota is exceeded? (CLI displays a quota error with instructions to upgrade or wait)
- How are secrets/credentials passed to the cloud worker? (Encrypted transfer; never stored beyond job lifetime)

---

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST support a `--cloud` flag on the `ckrv run` command to dispatch jobs to a remote worker.
- **FR-002**: System MUST return a unique job ID immediately upon successful cloud dispatch.
- **FR-003**: System MUST provide a `ckrv status <job-id>` command to query the current state of a cloud job.
- **FR-004**: System MUST provide a `ckrv logs <job-id>` command to retrieve logs from a cloud job.
- **FR-005**: System MUST provide a `ckrv pull <job-id>` command to download results of a completed cloud job.
- **FR-006**: System MUST support `ckrv cloud login`, `ckrv cloud logout`, and `ckrv cloud whoami` for authentication management.
- **FR-007**: System MUST securely store authentication credentials locally (e.g., in a system keychain or encrypted file).
- **FR-008**: System MUST display clear error messages when cloud service is unavailable or quota exceeded.
- **FR-009**: System MUST support streaming logs in real-time via `ckrv logs <job-id> --follow`.
- **FR-010**: System MUST transfer only the spec file, git repository URL, and user-specified base branch to the cloud worker. Worker MUST clone code from git and create a feature branch. Environment variables are the user's responsibility (keyvault integration recommended).
- **FR-011**: System MUST allow users to upload and manage git credentials (deploy keys or PATs) via `ckrv cloud credentials` commands. Credentials MUST be stored encrypted and used only during job execution.

### Key Entities

- **CloudJob**: Represents a single execution dispatched to the cloud. Attributes: job_id, spec_id, status, created_at, updated_at, result_url.
- **CloudWorker**: Represents a compute resource executing jobs. Attributes: worker_id, region, capacity.
- **CloudCredentials**: User authentication data. Attributes: access_token, refresh_token, expiry.

---

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can dispatch a job to the cloud in under 10 seconds.
- **SC-002**: Job status is accurately reflected within 5 seconds of any state change.
- **SC-003**: Log streaming has less than 2 seconds of latency from worker to terminal.
- **SC-004**: 95% of cloud jobs complete successfully without infrastructure-related failures.
- **SC-005**: Users can retrieve job results within 30 seconds of job completion.
- **SC-006**: Authentication flow completes in under 60 seconds for new users.

---

## Assumptions

- **Architecture Model**: Chakravarti operates a Managed Cloud Service (SaaS). The CLI authenticates to a central Chakravarti API that dispatches jobs to Chakravarti-managed workers. This enables value capture through the hosted service.
- **Worker Isolation**: Each job runs as a Kubernetes Pod containing an orchestrator container (running `ckrv`) that spawns agent containers (Claude Code, Gemini, etc.) dynamically via K8s API. Pods are isolated per job.
- Users have internet connectivity when using cloud features.
- The cloud execution environment mirrors the local Docker sandbox capabilities.
- Job results (diffs) are stored temporarily (e.g., 7 days) and then purged.
- Security hardening (e.g., Firecracker, gVisor, network policies) is deferred to post-MVP.
- **Billing Model**: Subscription tiers with monthly job quotas. Overage billed separately. Pricing reflects managed infrastructure value, not raw compute cost.
- **Repository Architecture**: Split architecture - CLI is open source (`/apps/chakravarti-cli`), Cloud API/workers are private (`/apps/chakravarti-cloud`). OpenAPI contract is the shared interface.
