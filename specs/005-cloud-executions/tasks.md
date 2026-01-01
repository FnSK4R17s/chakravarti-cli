# Tasks: Cloud Executions Support

**Input**: Design documents from `/apps/chakravarti-cli/specs/005-cloud-executions/`  
**Prerequisites**: plan.md ✅, spec.md ✅, research.md ✅, data-model.md ✅, contracts/ ✅

**Tests**: Tests will be included as TDD is a constitutional requirement (Principle II).

**Organization**: Tasks are split by repository and organized by user story priority.

## Format: `[ID] [P?] [Story?] [Repo] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story (US1-US5)
- **[CLI]**: Task in `/apps/chakravarti-cli` (open source)
- **[CLOUD]**: Task in `/apps/chakravarti-cloud` (private)

---

## Phase 1: Setup (Both Repos)

**Purpose**: Initialize project structures for both repositories

### CLI Repository (`/apps/chakravarti-cli`)

- [x] T001 [CLI] Create cloud subcommand module structure at `crates/ckrv-cli/src/commands/cloud/mod.rs`
- [x] T002 [P] [CLI] Create cloud client library structure at `crates/ckrv-cli/src/cloud/mod.rs`
- [x] T003 [P] [CLI] Add dependencies to `crates/ckrv-cli/Cargo.toml`: `reqwest`, `keyring`, `oauth2`
- [x] T004 [CLI] Register cloud subcommand in `crates/ckrv-cli/src/commands/mod.rs`
- [x] T005 [CLI] Register `status`, `logs`, `pull` commands in `crates/ckrv-cli/src/commands/mod.rs`

### Cloud Repository (`/apps/chakravarti-cloud`)

- [x] T006 [CLOUD] Initialize Node.js/TypeScript project at `/apps/chakravarti-cloud/api/`
- [x] T007 [P] [CLOUD] Create `package.json` with dependencies: `fastify`, `@fastify/cors`, `pg`, `@kubernetes/client-node`
- [x] T008 [P] [CLOUD] Create `tsconfig.json` with strict mode at `/apps/chakravarti-cloud/api/`
- [x] T009 [P] [CLOUD] Create environment config template at `/apps/chakravarti-cloud/.env.example`
- [x] T010 [P] [CLOUD] Create Makefile with build/dev/test targets at `/apps/chakravarti-cloud/Makefile`
- [x] T011 [P] [CLOUD] Copy OpenAPI contract from CLI specs to `/apps/chakravarti-cloud/contracts/cloud-api.yaml`
- [x] T012 [CLOUD] Create `docker-compose.yaml` for local dev (Postgres, Redis) at `/apps/chakravarti-cloud/`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

### CLI Foundational (`/apps/chakravarti-cli`)

- [x] T013 [CLI] Implement HTTP client base in `crates/ckrv-cli/src/cloud/client.rs` with retry logic and error handling
- [x] T014 [P] [CLI] Implement token storage in `crates/ckrv-cli/src/cloud/credentials.rs` using `keyring` crate
- [x] T015 [P] [CLI] Define cloud-specific error types in `crates/ckrv-cli/src/cloud/error.rs`
- [x] T016 [CLI] Implement config loading for cloud API URL in `crates/ckrv-cli/src/cloud/config.rs`

### Cloud API Foundational (`/apps/chakravarti-cloud`)

- [x] T017 [CLOUD] Create API entry point at `/apps/chakravarti-cloud/api/src/index.ts`
- [x] T018 [P] [CLOUD] Setup database connection pool in `/apps/chakravarti-cloud/api/src/db.ts`
- [x] T019 [P] [CLOUD] Create User model in `/apps/chakravarti-cloud/api/src/models/user.ts`
- [x] T020 [P] [CLOUD] Create GitCredential model in `/apps/chakravarti-cloud/api/src/models/git-credential.ts`
- [x] T021 [P] [CLOUD] Create CloudJob model in `/apps/chakravarti-cloud/api/src/models/job.ts`
- [x] T022 [P] [CLOUD] Create Subscription model in `/apps/chakravarti-cloud/api/src/models/subscription.ts`
- [x] T023 [CLOUD] Create database migration for all models in `/apps/chakravarti-cloud/api/migrations/`
- [x] T024 [CLOUD] Implement JWT validation middleware in `/apps/chakravarti-cloud/api/src/middleware/auth.ts`
- [x] T025 [P] [CLOUD] Implement error handling middleware in `/apps/chakravarti-cloud/api/src/middleware/error.ts`
- [x] T026 [CLOUD] Setup route registration in `/apps/chakravarti-cloud/api/src/routes/index.ts`

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 5 - Cloud Authentication (Priority: P1) 🎯 MVP-FIRST

**Goal**: Enable users to authenticate CLI with cloud service via OAuth2 device flow

**Independent Test**: Run `ckrv cloud login`, complete browser auth, verify `ckrv cloud whoami` shows user

**Why First**: Authentication is a prerequisite for ALL other cloud operations

### Tests for US5

- [ ] T027 [P] [US5] [CLI] Unit test for device auth flow in `crates/ckrv-cli/src/cloud/auth_test.rs`
- [ ] T028 [P] [US5] [CLOUD] Integration test for auth endpoints in `/apps/chakravarti-cloud/api/tests/auth.test.ts`

### CLI Implementation (US5)

- [x] T029 [US5] [CLI] Implement OAuth2 device flow in `crates/ckrv-cli/src/cloud/auth.rs`
- [x] T030 [US5] [CLI] Implement `ckrv cloud login` command in `crates/ckrv-cli/src/commands/cloud/login.rs`
- [x] T031 [P] [US5] [CLI] Implement `ckrv cloud logout` command in `crates/ckrv-cli/src/commands/cloud/logout.rs`
- [x] T032 [P] [US5] [CLI] Implement `ckrv cloud whoami` command in `crates/ckrv-cli/src/commands/cloud/whoami.rs`
- [x] T033 [US5] [CLI] Register cloud subcommands in `crates/ckrv-cli/src/main.rs`

### Cloud API Implementation (US5)

- [ ] T034 [US5] [CLOUD] Implement device code generation in `/apps/chakravarti-cloud/api/src/services/auth.ts`
- [x] T035 [US5] [CLOUD] Implement `/auth/device` endpoint in `/apps/chakravarti-cloud/api/src/routes/auth.ts`
- [x] T036 [US5] [CLOUD] Implement `/auth/token` endpoint in `/apps/chakravarti-cloud/api/src/routes/auth.ts`
- [x] T037 [P] [US5] [CLOUD] Implement `/auth/refresh` endpoint in `/apps/chakravarti-cloud/api/src/routes/auth.ts`
- [x] T038 [US5] [CLOUD] Implement `/users/me` endpoint in `/apps/chakravarti-cloud/api/src/routes/users.ts`

**Checkpoint**: Authentication complete. Users can log in and verify identity.

---

## Phase 4: User Story 1 - Run Job in Cloud (Priority: P1)

**Goal**: Dispatch jobs to cloud workers via `ckrv run --cloud`

**Independent Test**: Run `ckrv run --cloud --spec my-spec.md` and receive a job ID

### Tests for US1

- [ ] T039 [P] [US1] [CLI] Unit test for job dispatch in `crates/ckrv-cli/src/commands/run_test.rs`
- [ ] T040 [P] [US1] [CLOUD] Integration test for job creation in `/apps/chakravarti-cloud/api/tests/jobs.test.ts`

### CLI Implementation (US1)

- [x] T041 [US1] [CLI] Add `--cloud` flag to `run` command in `crates/ckrv-cli/src/commands/run.rs`
- [x] T042 [US1] [CLI] Implement job dispatch logic in `crates/ckrv-cli/src/cloud/jobs.rs`
- [x] T043 [US1] [CLI] Add spec file reading and serialization for cloud dispatch
- [x] T044 [US1] [CLI] Implement job ID display and exit behavior for `--cloud` mode

### Cloud API Implementation (US1)

- [ ] T045 [US1] [CLOUD] Implement job creation service in `/apps/chakravarti-cloud/api/src/services/job-service.ts`
- [x] T046 [US1] [CLOUD] Implement `POST /jobs` endpoint in `/apps/chakravarti-cloud/api/src/routes/jobs.ts`
- [x] T047 [US1] [CLOUD] Implement quota checking before job creation
- [x] T048 [US1] [CLOUD] Implement K8s job dispatcher in `/apps/chakravarti-cloud/api/src/services/job-dispatcher.ts`

### Worker Implementation (US1)

- [x] T049 [US1] [CLOUD] Create base Dockerfile at `/apps/chakravarti-cloud/worker/Dockerfile`
- [x] T050 [US1] [CLOUD] Create entrypoint script at `/apps/chakravarti-cloud/worker/entrypoint.sh`
- [x] T051 [US1] [CLOUD] Create K8s Job template at `/apps/chakravarti-cloud/infra/k8s/job-template.yaml`

**Checkpoint**: Jobs can be dispatched. `ckrv run --cloud` creates a K8s job.

---

## Phase 5: User Story 2 - Monitor Cloud Job Status (Priority: P1)

**Goal**: Check job status via `ckrv status <job-id>`

**Independent Test**: After job dispatch, run `ckrv status <job-id>` and see current state

### Tests for US2

- [ ] T052 [P] [US2] [CLI] Unit test for status command in `crates/ckrv-cli/src/commands/status_test.rs`
- [ ] T053 [P] [US2] [CLOUD] Integration test for status endpoint in `/apps/chakravarti-cloud/api/tests/jobs-status.test.ts`

### CLI Implementation (US2)

- [x] T054 [US2] [CLI] Create `status` command in `crates/ckrv-cli/src/commands/status.rs`
- [x] T055 [US2] [CLI] Implement status fetching in `crates/ckrv-cli/src/cloud/jobs.rs`
- [x] T056 [US2] [CLI] Format status output with phase, elapsed time, ETA

### Cloud API Implementation (US2)

- [x] T057 [US2] [CLOUD] Implement `GET /jobs/:id` endpoint in `/apps/chakravarti-cloud/api/src/routes/jobs.ts`
- [x] T058 [US2] [CLOUD] Implement job status sync from K8s in `/apps/chakravarti-cloud/api/src/services/job-sync.ts`

**Checkpoint**: Users can monitor job progress.

---

## Phase 6: User Story 4 - Retrieve Cloud Job Results (Priority: P1)

**Goal**: Download job results via `ckrv pull <job-id>`

**Independent Test**: After job succeeds, run `ckrv pull <job-id>` and see diff applied

### Tests for US4

- [ ] T059 [P] [US4] [CLI] Unit test for pull command in `crates/ckrv-cli/src/commands/pull_test.rs`
- [ ] T060 [P] [US4] [CLOUD] Integration test for artifact download in `/apps/chakravarti-cloud/api/tests/artifacts.test.ts`

### CLI Implementation (US4)

- [x] T061 [US4] [CLI] Create `pull` command in `crates/ckrv-cli/src/commands/pull.rs`
- [x] T062 [US4] [CLI] Implement diff download in `crates/ckrv-cli/src/cloud/artifacts.rs`
- [x] T063 [US4] [CLI] Implement git diff apply logic
- [x] T064 [US4] [CLI] Display applied changes summary

### Cloud API Implementation (US4)

- [x] T065 [US4] [CLOUD] Implement artifact storage service in `/apps/chakravarti-cloud/api/src/services/artifact-store.ts`
- [x] T066 [US4] [CLOUD] Implement `GET /jobs/:id/artifacts/diff` endpoint
- [x] T067 [US4] [CLOUD] Implement artifact upload from worker on job completion

### Worker Implementation (US4)

- [x] T068 [US4] [CLOUD] Implement diff generation in worker entrypoint
- [x] T069 [US4] [CLOUD] Implement artifact upload to S3/storage on job success

**Checkpoint**: P1 user stories complete. MVP functional end-to-end.

---

## Phase 7: User Story 3 - Stream Cloud Job Logs (Priority: P2)

**Goal**: Stream real-time logs via `ckrv logs <job-id> --follow`

**Independent Test**: Run `ckrv logs <job-id> --follow` during job execution, see live logs

### Tests for US3

- [ ] T070 [P] [US3] [CLI] Unit test for logs command in `crates/ckrv-cli/src/commands/logs_test.rs`
- [ ] T071 [P] [US3] [CLOUD] Integration test for SSE streaming in `/apps/chakravarti-cloud/api/tests/logs-stream.test.ts`

### CLI Implementation (US3)

- [x] T072 [US3] [CLI] Create `logs` command in `crates/ckrv-cli/src/commands/logs.rs`
- [x] T073 [US3] [CLI] Implement SSE client for log streaming in `crates/ckrv-cli/src/cloud/logs.rs`
- [x] T074 [US3] [CLI] Implement `--follow` flag with real-time output
- [x] T075 [US3] [CLI] Implement historical log fetch (without `--follow`)

### Cloud API Implementation (US3)

- [x] T076 [US3] [CLOUD] Implement log streaming service in `/apps/chakravarti-cloud/api/src/services/log-streamer.ts`
- [x] T077 [US3] [CLOUD] Implement `GET /jobs/:id/logs` with SSE support
- [x] T078 [US3] [CLOUD] Implement log aggregation from worker pods

### Worker Implementation (US3)

- [x] T079 [US3] [CLOUD] Implement log forwarding to API in worker

**Checkpoint**: Log streaming works. Full observability achieved.

---

## Phase 8: Git Credentials Management (Enhancement to US5)

**Goal**: Allow users to store git credentials for private repo access

**Independent Test**: Run `ckrv cloud credentials add`, then dispatch job to private repo

### CLI Implementation

- [x] T080 [US5] [CLI] Implement `ckrv cloud credentials` subcommand in `crates/ckrv-cli/src/commands/cloud/credentials.rs`
- [x] T081 [P] [US5] [CLI] Implement `add`, `list`, `remove` subcommands
- [x] T082 [US5] [CLI] Add `--credential` flag to `ckrv run --cloud`

### Cloud API Implementation

- [x] T083 [US5] [CLOUD] Implement `POST /credentials` endpoint
- [x] T084 [P] [US5] [CLOUD] Implement `GET /credentials` endpoint (names only, no values)
- [x] T085 [P] [US5] [CLOUD] Implement `DELETE /credentials/:name` endpoint
- [x] T086 [US5] [CLOUD] Implement credential encryption service
- [x] T087 [US5] [CLOUD] Inject credentials as K8s secrets during job dispatch

**Checkpoint**: Private repository access enabled.

---

## Phase 9: Infrastructure Setup

**Purpose**: Deploy cloud infrastructure

### Kubernetes Infrastructure

- [x] T088 [CLOUD] Create namespace manifest at `/apps/chakravarti-cloud/infra/k8s/namespace.yaml`
- [x] T089 [P] [CLOUD] Create API deployment at `/apps/chakravarti-cloud/infra/k8s/deployment.yaml`
- [x] T090 [P] [CLOUD] Create API service at `/apps/chakravarti-cloud/infra/k8s/service.yaml`
- [x] T091 [P] [CLOUD] Create ingress for API at `/apps/chakravarti-cloud/infra/k8s/ingress.yaml`
- [x] T092 [CLOUD] Create RBAC for worker pods at `/apps/chakravarti-cloud/infra/k8s/rbac.yaml`

### Terraform Infrastructure

- [x] T093 [CLOUD] Create main Terraform config at `/apps/chakravarti-cloud/infra/terraform/main.tf`
- [x] T094 [P] [CLOUD] Create variables file at `/apps/chakravarti-cloud/infra/terraform/variables.tf`
- [x] T095 [P] [CLOUD] Create K8s cluster config at `/apps/chakravarti-cloud/infra/terraform/k8s.tf`
- [x] T096 [CLOUD] Create S3/storage bucket config at `/apps/chakravarti-cloud/infra/terraform/storage.tf`

---

## Phase 10: Polish & Cross-Cutting Concerns

**Purpose**: Documentation, cleanup, and final validation

- [ ] T097 [P] [CLI] Update README with cloud commands documentation
- [ ] T098 [P] [CLI] Add `--help` text for all new commands
- [ ] T099 [P] [CLOUD] Create README at `/apps/chakravarti-cloud/README.md`
- [ ] T100 [P] [CLOUD] Create API documentation from OpenAPI spec
- [ ] T101 [CLI] Run quickstart.md validation (end-to-end test)
- [ ] T102 [CLOUD] Security review: credential handling, API auth
- [ ] T103 [CLI] Address compiler warnings in cloud modules
- [ ] T104 [CLOUD] Performance testing: job dispatch latency, log streaming

---

## Dependencies & Execution Order

### Phase Dependencies

```
Phase 1 (Setup) ─────────────────────────────► No dependencies
       │
       ▼
Phase 2 (Foundational) ──────────────────────► Blocks ALL user stories
       │
       ▼
Phase 3 (US5: Auth) ─────────────────────────► First user story (unblocks others)
       │
       ├──────► Phase 4 (US1: Run Job) ──────► Requires Auth
       │              │
       │              ▼
       │        Phase 5 (US2: Status) ───────► Requires Jobs exist
       │              │
       │              ▼
       │        Phase 6 (US4: Pull) ─────────► Requires Jobs complete
       │
       └──────► Phase 7 (US3: Logs) ─────────► Can run parallel to US4
                      │
                      ▼
              Phase 8 (Credentials) ─────────► Enhancement, can be parallel
                      │
                      ▼
              Phase 9 (Infrastructure) ──────► Deploy when ready
                      │
                      ▼
              Phase 10 (Polish) ─────────────► Final cleanup
```

### User Story Dependencies

| Story | Can Start After | Dependencies |
|-------|-----------------|--------------|
| US5 (Auth) | Phase 2 | None |
| US1 (Run Job) | US5 | Auth required |
| US2 (Status) | US1 | Jobs must exist |
| US4 (Pull) | US1 | Jobs must complete |
| US3 (Logs) | US1 | Jobs must run |

---

## Parallel Opportunities

### CLI Tasks (can be parallelized)

```bash
# After Phase 2, these can run in parallel:
T013, T014, T015, T016  # Cloud client foundation
T030, T031, T032        # Login, logout, whoami commands
```

### Cloud API Tasks (can be parallelized)

```bash
# During Phase 2, these can run in parallel:
T019, T020, T021, T022  # All models
T024, T025              # Middleware

# During Phase 4, these can run in parallel:
T045, T048              # Job service + K8s dispatcher
```

---

## Implementation Strategy

### MVP First (Phases 1-6)

1. Complete Phase 1: Setup both repos
2. Complete Phase 2: Foundational
3. Complete Phase 3: Auth (US5) → **First demo: login works**
4. Complete Phase 4: Run Job (US1) → **Second demo: jobs dispatch**
5. Complete Phase 5: Status (US2) → **Third demo: can monitor jobs**
6. Complete Phase 6: Pull (US4) → **MVP Complete: end-to-end works**

### Incremental Delivery

| Milestone | Stories | Demo |
|-----------|---------|------|
| Auth MVP | US5 | Login/logout/whoami |
| Dispatch MVP | US5 + US1 | Jobs start in cloud |
| Monitor MVP | US5 + US1 + US2 | See job status |
| Full MVP | US5 + US1 + US2 + US4 | Pull results |
| Enhanced | + US3 | Stream logs |
| Complete | + Credentials | Private repos |

---

## Summary

| Metric | Count |
|--------|-------|
| **Total Tasks** | 104 |
| **CLI Tasks** | 42 |
| **Cloud Tasks** | 62 |
| **Phase 1 (Setup)** | 12 |
| **Phase 2 (Foundational)** | 14 |
| **US5 (Auth)** | 12 |
| **US1 (Run Job)** | 13 |
| **US2 (Status)** | 7 |
| **US4 (Pull)** | 11 |
| **US3 (Logs)** | 10 |
| **Credentials** | 8 |
| **Infrastructure** | 9 |
| **Polish** | 8 |

**Suggested MVP Scope**: Phases 1-6 (US5 + US1 + US2 + US4) = 59 tasks
