# Research: Cloud Executions

**Feature**: 005-cloud-executions  
**Date**: 2025-12-30  
**Status**: Complete

---

## 1. OAuth2 Device Authorization for CLI

### Decision
Use **OAuth2 Device Authorization Grant (RFC 8628)** for CLI-to-browser authentication.

### Rationale
- Designed specifically for devices without browsers (CLIs, IoT)
- User experience: CLI displays code, user visits URL in any browser, enters code
- No need for localhost callback server (avoids firewall/port issues)
- Widely supported by identity providers (Auth0, Okta, Cognito)

### Alternatives Considered
| Alternative | Rejected Because |
|-------------|------------------|
| Authorization Code + localhost | Firewall issues, requires port binding |
| API key manual copy | Poor UX, no refresh tokens |
| Username/password | Insecure, no MFA support |

### Implementation Notes
- CLI polls `/token` endpoint until user completes auth
- Store `access_token` + `refresh_token` in system keychain
- Refresh silently before expiry; re-auth only if refresh fails

---

## 2. Kubernetes Job Orchestration

### Decision
Use **Kubernetes Jobs with Pod Templates** for ephemeral job execution. Orchestrator runs as main container; agents are spawned as additional containers via K8s API from within the pod.

### Rationale
- Jobs provide built-in completion tracking and retry logic
- Pod template allows defining resource limits and secrets
- Orchestrator container can use K8s client libraries to spawn sidecar pods
- Native integration with K8s RBAC for security

### Alternatives Considered
| Alternative | Rejected Because |
|-------------|------------------|
| Raw Pod creation | No built-in completion tracking |
| Argo Workflows | Overkill for MVP; adds dependency |
| Docker-in-Docker | Security concerns with privileged mode |

### Implementation Notes
- Create K8s Job with orchestrator container on dispatch
- Orchestrator uses in-cluster service account to create agent pods
- Job completion triggers artifact upload webhook
- Use `activeDeadlineSeconds` for job timeouts

### Pod Architecture
```yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: ckrv-job-{job_id}
spec:
  template:
    spec:
      serviceAccountName: ckrv-orchestrator  # Has pod create permissions
      containers:
        - name: orchestrator
          image: chakravarti/ckrv-worker:latest
          env:
            - name: JOB_ID
              value: "{job_id}"
            - name: SPEC_CONTENT
              valueFrom:
                secretKeyRef: ...
      restartPolicy: Never
  backoffLimit: 0
```

---

## 3. Log Streaming Approach

### Decision
Use **Server-Sent Events (SSE)** for log streaming from Cloud API to CLI.

### Rationale
- Simpler than WebSocket (HTTP/1.1 compatible, no upgrade handshake)
- Unidirectional (server→client) is perfect for log streaming
- Built-in reconnection in browsers and easy to implement in CLI
- Works through most proxies without configuration

### Alternatives Considered
| Alternative | Rejected Because |
|-------------|------------------|
| WebSocket | Overkill for unidirectional; more complex |
| Long-polling | Higher latency, more requests |
| gRPC streaming | Adds protobuf dependency; overkill for MVP |

### Implementation Notes
- Cloud API endpoint: `GET /api/jobs/{job_id}/logs/stream`
- CLI uses `reqwest` with streaming response
- Each SSE event contains: `timestamp`, `level`, `message`, `source` (orchestrator/agent)
- Include `Last-Event-ID` for reconnection support

---

## 4. Cross-Platform Credential Storage

### Decision
Use **`keyring` crate** with fallback to encrypted file for systems without keychain.

### Rationale
- `keyring` crate provides unified API across macOS Keychain, Windows Credential Manager, Linux Secret Service
- Encrypted file fallback ensures CI/headless environments work
- Industry standard approach (used by AWS CLI, gh CLI, etc.)

### Alternatives Considered
| Alternative | Rejected Because |
|-------------|------------------|
| Plain text config file | Insecure |
| Environment variables only | Poor UX for interactive use |
| Platform-specific code | Maintenance burden |

### Implementation Notes
```rust
// Try keyring first, fallback to encrypted file
pub fn store_token(token: &str) -> Result<()> {
    match keyring::Entry::new("chakravarti", "access_token")?.set_password(token) {
        Ok(_) => Ok(()),
        Err(_) => store_encrypted_file(token),  // Fallback
    }
}
```
- Encryption key for fallback derived from machine ID + user salt
- Token format: `{"access_token": "...", "refresh_token": "...", "expires_at": "..."}`

---

## 5. Git Credentials in Kubernetes Pods

### Decision
Use **Kubernetes Secrets** injected as environment variables for git credentials.

### Rationale
- Simple and secure (secrets are base64 encoded, access controlled via RBAC)
- Works with any git provider (GitHub, GitLab, Bitbucket)
- No need for CSI drivers or external secret managers (MVP simplicity)

### Alternatives Considered
| Alternative | Rejected Because |
|-------------|------------------|
| Git credential helper with mounted secrets | More complex setup |
| External Secrets Operator | Adds dependency; overkill for MVP |
| SSH key volumes | Requires key management; PAT is simpler |

### Implementation Notes
1. User uploads PAT via `ckrv cloud credentials add --name github --token xxx`
2. Cloud API stores encrypted in database (per-user)
3. On job dispatch, inject as K8s Secret:
   ```yaml
   env:
     - name: GIT_CREDENTIALS
       valueFrom:
         secretKeyRef:
           name: ckrv-job-{job_id}-secrets
           key: git-token
   ```
4. Orchestrator configures git credential helper:
   ```bash
   git config --global credential.helper '!f() { echo "password=$GIT_CREDENTIALS"; }; f'
   ```
5. Secret is auto-deleted when Job completes

---

## Summary

All Phase 0 research items resolved. Key decisions:

| Topic | Decision |
|-------|----------|
| CLI Auth | OAuth2 Device Authorization Grant |
| K8s Execution | Jobs with orchestrator pod + dynamic agent pods |
| Log Streaming | Server-Sent Events (SSE) |
| Credential Storage | `keyring` crate with encrypted file fallback |
| Git in K8s | Secrets as environment variables |

**Ready for Phase 1**: Data model and API contract design.
