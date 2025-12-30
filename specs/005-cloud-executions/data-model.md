# Data Model: Cloud Executions

**Feature**: 005-cloud-executions  
**Date**: 2025-12-30

---

## Entities

### User

Represents an authenticated user of the Chakravarti Cloud Service.

| Field | Type | Description | Constraints |
|-------|------|-------------|-------------|
| `id` | UUID | Unique identifier | Primary key |
| `email` | String | User's email address | Unique, validated format |
| `name` | String | Display name | Optional |
| `created_at` | Timestamp | Account creation time | Auto-set |
| `updated_at` | Timestamp | Last modification time | Auto-updated |
| `subscription_tier` | Enum | Current plan (free/pro/enterprise) | Default: free |
| `job_quota_remaining` | Integer | Jobs remaining this billing cycle | Tracked per cycle |
| `billing_cycle_end` | Timestamp | When current quota resets | Monthly |

**Relationships**:
- Has many `GitCredential`
- Has many `CloudJob`

---

### GitCredential

User-provided credentials for accessing private git repositories.

| Field | Type | Description | Constraints |
|-------|------|-------------|-------------|
| `id` | UUID | Unique identifier | Primary key |
| `user_id` | UUID | Owner of credential | Foreign key → User |
| `name` | String | User-assigned label (e.g., "github-work") | Unique per user |
| `provider` | Enum | Git provider (github/gitlab/bitbucket/generic) | Required |
| `credential_type` | Enum | Type (pat/deploy_key/ssh) | Required |
| `encrypted_value` | Bytes | Encrypted credential data | Server-side encrypted |
| `created_at` | Timestamp | Creation time | Auto-set |
| `last_used_at` | Timestamp | Last used in a job | Updated on use |

**Constraints**:
- Credentials are **never** returned in API responses (write-only)
- Decrypted only during job execution, then discarded

---

### CloudJob

Represents a single agent orchestration job dispatched to the cloud.

| Field | Type | Description | Constraints |
|-------|------|-------------|-------------|
| `id` | UUID | Unique job identifier | Primary key |
| `user_id` | UUID | Job owner | Foreign key → User |
| `status` | Enum | Current state | See state machine below |
| `spec_content` | Text | Full spec file content | Required, immutable after creation |
| `git_repo_url` | String | Repository to clone | Required, validated URL |
| `git_base_branch` | String | Branch to fork from | Default: "main" |
| `git_credential_id` | UUID | Credential to use for clone | Foreign key → GitCredential, nullable |
| `feature_branch_name` | String | Generated feature branch | Auto-generated |
| `created_at` | Timestamp | Job creation time | Auto-set |
| `started_at` | Timestamp | Execution start time | Set on transition to `running` |
| `completed_at` | Timestamp | Execution end time | Set on terminal state |
| `result_status` | Enum | Outcome (succeeded/failed/timeout) | Set on completion |
| `result_summary` | Text | Human-readable outcome | Optional |
| `diff_artifact_url` | String | S3 URL to generated diff | Set on success |
| `log_artifact_url` | String | S3 URL to full logs | Always set on completion |
| `error_message` | Text | Error details if failed | Optional |
| `k8s_job_name` | String | Kubernetes job name | Internal reference |
| `k8s_namespace` | String | Kubernetes namespace | Internal reference |

**State Machine**:
```
              ┌─────────┐
              │ pending │
              └────┬────┘
                   │ worker picks up
                   ▼
              ┌─────────┐
              │ running │
              └────┬────┘
         ┌─────────┼─────────┐
         │         │         │
         ▼         ▼         ▼
   ┌─────────┐ ┌────────┐ ┌─────────┐
   │succeeded│ │ failed │ │ timeout │
   └─────────┘ └────────┘ └─────────┘
```

**Constraints**:
- Jobs are immutable after creation (status updates only)
- Artifacts retained for 7 days, then purged

---

### Subscription

Represents billing tier configuration (reference data).

| Field | Type | Description | Constraints |
|-------|------|-------------|-------------|
| `tier` | Enum | Plan name (free/pro/enterprise) | Primary key |
| `monthly_job_quota` | Integer | Included jobs per month | Required |
| `overage_price_cents` | Integer | Price per overage job (cents) | Required |
| `features` | JSONB | Feature flags | Extensible |

**Seed Data**:
| Tier | Monthly Quota | Overage Price |
|------|---------------|---------------|
| free | 10 | N/A (blocked) |
| pro | 100 | $0.50/job |
| enterprise | 1000 | $0.25/job |

---

### JobLog (Transient)

Log entries streamed during job execution. Stored ephemerally, then archived to S3.

| Field | Type | Description | Constraints |
|-------|------|-------------|-------------|
| `job_id` | UUID | Associated job | Foreign key → CloudJob |
| `sequence` | Integer | Ordering within job | Auto-increment |
| `timestamp` | Timestamp | Log entry time | Millisecond precision |
| `level` | Enum | Log level (debug/info/warn/error) | Required |
| `source` | String | Origin (orchestrator/agent-name) | Required |
| `message` | Text | Log content | Required |
| `metadata` | JSONB | Additional context | Optional |

**Constraints**:
- Stored in Redis or similar during streaming
- Flushed to S3 as JSON lines file on job completion
- Database only stores reference to archived log URL

---

## Relationships Diagram

```
┌──────────────┐       ┌─────────────────┐       ┌──────────────┐
│     User     │───────│  GitCredential  │       │ Subscription │
└──────────────┘  1:N  └─────────────────┘       └──────────────┘
       │                                                │
       │ 1:N                                           │ 1:N
       ▼                                               │
┌──────────────┐                                       │
│   CloudJob   │◄──────────────────────────────────────┘
└──────────────┘       (User has tier from Subscription)
       │
       │ 1:N (streaming)
       ▼
┌──────────────┐
│   JobLog     │ (transient, archived to S3)
└──────────────┘
```

---

## Validation Rules

### User
- `email`: Valid email format, unique across system
- `subscription_tier`: Must be valid enum value
- `job_quota_remaining`: Cannot be negative

### GitCredential
- `name`: 1-50 characters, alphanumeric + hyphens
- `encrypted_value`: Must be encrypted with server key

### CloudJob
- `git_repo_url`: Valid git URL (https or git@ format)
- `spec_content`: Non-empty, valid YAML/Markdown
- `status`: Only valid transitions per state machine

---

## Indexes

| Table | Index | Purpose |
|-------|-------|---------|
| `users` | `email` (unique) | Login lookup |
| `git_credentials` | `user_id, name` (unique) | Credential lookup by name |
| `cloud_jobs` | `user_id, created_at DESC` | User's job history |
| `cloud_jobs` | `status` | Queue processing |
| `cloud_jobs` | `k8s_job_name` | Webhook reconciliation |
