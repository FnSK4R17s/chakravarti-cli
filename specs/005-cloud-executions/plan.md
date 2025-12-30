# Implementation Plan: Cloud Executions Support

**Branch**: `005-cloud-executions` | **Date**: 2025-12-30 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/005-cloud-executions/spec.md`

## Summary

Enable Chakravarti CLI users to offload agent orchestration jobs to a Managed Cloud Service (SaaS). The CLI authenticates to a central Chakravarti API, dispatches jobs with minimal context (spec + git URL + base branch), and receives results asynchronously. Jobs execute on Kubernetes Pods with an orchestrator container (`ckrv`) spawning agent containers dynamically.

**Key Technical Decisions**:
- Architecture: Managed SaaS with Chakravarti-hosted Kubernetes infrastructure
- Isolation: Pod-per-job with orchestrator + dynamic agent containers
- Context Transfer: Minimal (spec file, git URL, base branch only)
- Auth: User-provided deploy keys/PATs for git access; OAuth2 for API auth
- Billing: Subscription tiers with job quotas

## Technical Context

**Language/Version**: Rust 1.75+ (CLI), TypeScript/Node.js (Cloud API)  
**Primary Dependencies**: 
- CLI: `reqwest` (HTTP client), `keyring` (credential storage), `tokio` (async)
- Cloud API: Express/Fastify, Kubernetes client, PostgreSQL
- Worker: Docker images with `ckrv` + agent runtimes (Claude Code, Gemini)

**Storage**: 
- CLI: Local keychain/encrypted file for credentials
- Cloud: PostgreSQL (jobs, users, quotas), S3-compatible (job artifacts, diffs)

**Testing**: 
- CLI: `cargo test` with mocked HTTP responses
- Cloud API: Jest/Vitest with database fixtures
- Integration: End-to-end tests with ephemeral K8s cluster

**Target Platform**: 
- CLI: Linux, macOS, Windows (cross-compile)
- Cloud: Kubernetes (GKE/EKS/self-hosted)

**Project Type**: Multi-component (CLI extension + new Cloud API + Worker image)

**Performance Goals**: 
- Job dispatch: <10 seconds
- Status sync: <5 seconds latency
- Log streaming: <2 seconds latency
- 95% job success rate (infrastructure-related)

**Constraints**: 
- Minimal context transfer (no large uploads)
- Encrypted credential storage
- Job results retention: 7 days
- Security hardening deferred to post-MVP

**Scale/Scope**: MVP targets 100 concurrent jobs, 1000 users

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Requirement | Status |
|-----------|-------------|--------|
| I. Code Quality Excellence | Full typing (Rust + TypeScript), zero lint errors, single responsibility | ✅ |
| II. Testing Standards | TDD planned, mocked HTTP for CLI tests, contract tests for API | ✅ |
| III. Reliability First | Explicit error handling, idempotent dispatch, job state recovery | ✅ |
| IV. Security by Default | Credentials via keychain, encrypted transfer, no secrets in code | ✅ |
| V. Deterministic CLI Behavior | JSON output for all commands, explicit exit codes, stderr for errors | ✅ |

## Project Structure

### Documentation (this feature)

```text
specs/005-cloud-executions/
├── spec.md              # Feature specification
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (OpenAPI specs)
│   ├── cloud-api.yaml   # Chakravarti Cloud API contract
│   └── events.yaml      # SSE event schemas
└── tasks.md             # Phase 2 output
```

### Source Code Structure

#### Repository 1: `chakravarti-cli` (Open Source - THIS REPO)

The CLI is the user-facing tool. Remains open source for transparency and community trust.

```text
crates/ckrv-cli/
├── src/
│   ├── commands/
│   │   ├── cloud/            # NEW: Cloud subcommand group
│   │   │   ├── mod.rs
│   │   │   ├── login.rs      # ckrv cloud login
│   │   │   ├── logout.rs     # ckrv cloud logout
│   │   │   ├── whoami.rs     # ckrv cloud whoami
│   │   │   └── credentials.rs # ckrv cloud credentials
│   │   ├── status.rs         # NEW: ckrv status <job-id>
│   │   ├── logs.rs           # NEW: ckrv logs <job-id>
│   │   └── pull.rs           # NEW: ckrv pull <job-id>
│   └── cloud/                # NEW: Cloud client library
│       ├── mod.rs
│       ├── client.rs         # HTTP client for Cloud API
│       ├── auth.rs           # OAuth2 device flow, token management
│       └── credentials.rs    # Keychain/encrypted file storage

# Tests for cloud integration (mocked HTTP)
crates/ckrv-cli/src/commands/cloud/__tests__/
```

#### Repository 2: `chakravarti-cloud` (Private/Proprietary)

**Workspace Path**: `/apps/chakravarti-cloud`

The Cloud API and infrastructure code. Private repository containing business logic.

```text
/apps/chakravarti-cloud/              # PRIVATE REPO IN WORKSPACE
├── api/                              # Cloud API service (TypeScript/Node.js)
│   ├── package.json
│   ├── tsconfig.json
│   ├── src/
│   │   ├── index.ts                  # Entry point
│   │   ├── routes/
│   │   │   ├── auth.ts               # /auth/* endpoints
│   │   │   ├── jobs.ts               # /jobs/* endpoints
│   │   │   ├── users.ts              # /users/* endpoints
│   │   │   └── billing.ts            # /billing/* endpoints
│   │   ├── services/
│   │   │   ├── job-dispatcher.ts     # K8s pod creation
│   │   │   ├── log-streamer.ts       # SSE log streaming
│   │   │   └── artifact-store.ts     # S3 artifact management
│   │   └── models/
│   │       ├── job.ts
│   │       ├── user.ts
│   │       └── subscription.ts
│   └── tests/
│
├── worker/                           # Worker Docker image
│   ├── Dockerfile
│   ├── entrypoint.sh
│   └── config/
│       └── agents.yaml               # Agent container definitions
│
├── infra/                            # Infrastructure as Code
│   ├── terraform/                    # Cloud infrastructure (GKE/EKS)
│   │   ├── main.tf
│   │   ├── variables.tf
│   │   └── k8s.tf
│   └── k8s/                          # Kubernetes manifests
│       ├── namespace.yaml
│       ├── deployment.yaml
│       └── service.yaml
│
├── contracts/                        # Shared API contracts (copied from CLI spec)
│   └── cloud-api.yaml                # OpenAPI spec (source of truth from CLI repo)
│
├── .env.example                      # Environment variable template
├── docker-compose.yaml               # Local development setup
├── Makefile                          # Build/deploy commands
└── README.md                         # Private repo documentation
```

**Repository Split Rationale**:

| Aspect | CLI Repo (Open Source) | Cloud Repo (Private) |
|--------|------------------------|----------------------|
| **Contains** | User-facing CLI, client library | API, workers, infra |
| **Visibility** | Public | Private |
| **Why** | Transparency, community trust, contributions | Proprietary business logic, security |
| **Versioning** | Semantic versioning, public releases | Internal versioning |
| **CI/CD** | GitHub Actions, public builds | Private CI, internal deployments |

**Contract Sharing**:
- The OpenAPI spec (`contracts/cloud-api.yaml`) is the **interface contract**
- Spec lives in the CLI repo (open source) so users can see the API contract
- Cloud repo copies/references this spec for implementation validation
- Both repos validate against the same contract

**Structure Decision**: **Split repository architecture**. The open-source CLI (`chakravarti-cli`) contains user-facing commands and the cloud client library. The private Cloud API (`chakravarti-cloud`) contains proprietary backend logic, workers, and infrastructure. The OpenAPI contract is the shared interface between repos.

## Complexity Tracking

> **No Constitution violations identified. All principles can be satisfied with standard practices.**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|--------------------------------------|
| N/A | — | — |

---

## Phase 0: Research Tasks

Based on Technical Context unknowns and integration requirements:

1. **OAuth2 Flow for CLI**: Research device authorization grant (RFC 8628) for CLI-to-browser auth
2. **Kubernetes Job API**: Best practices for ephemeral pod creation with sidecar containers
3. **SSE over WebSocket**: Evaluate log streaming approach (SSE simpler but WebSocket more robust)
4. **Credential Storage**: Cross-platform keychain access patterns (keyring crate capabilities)
5. **Git Clone in K8s**: Patterns for git credentials injection into pods (secrets, CSI drivers)

---

## Next Steps

Run **Phase 0** research to resolve technical unknowns, then proceed to:
- `research.md` - Consolidated findings
- `data-model.md` - Entity definitions
- `contracts/` - OpenAPI specs
- `quickstart.md` - Developer onboarding guide
