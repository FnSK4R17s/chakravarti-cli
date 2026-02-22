# Supercharged `ckrv term` — Bugfix Tasks (03)

**Brainstorm**: [notes.md](./notes.md)
**Created**: 2026-02-16
**Source**: Manual QA — Docker containers run as root, breaking agent CLI security checks

## Bugfix Overview

| # | Bug | Severity | Estimate |
|---|-----|----------|----------|
| BF-07 | Docker containers run as root — agent CLIs reject privileged flags | Critical | 40m |
| BF-08 | Agent guide docs missing `USER` requirement for Dockerfiles | Medium | 15m |

**Severity breakdown**: 1 Critical, 1 Medium
**Total estimate**: ~55m

---

## BF-07: Docker containers run as root — agent CLIs reject privileged flags

**Severity**: Critical
**File(s)**: `docker/Dockerfile.claude`, `docker/Dockerfile.codex`, `docker/Dockerfile.kilo`, `docker/Dockerfile.agent`, `docker/Dockerfile.ckrv`
**Estimate**: 40m

### Problem

All five Dockerfiles have **no `USER` directive**, so containers run as root (UID 0). This causes agent CLIs to reject security-sensitive flags:

```
$ ckrv term --sandbox
✔ Agent options · Skip permissions
▌ ✔ Container Started
--dangerously-skip-permissions cannot be used with root/sudo privileges for security reasons
```

Claude Code explicitly blocks `--dangerously-skip-permissions` when running as root. This is Claude Code's own safety check — not ours. Other agents may have similar restrictions.

### Affected Dockerfiles

| Dockerfile | Creates user dirs at | HOME set to | Missing |
|-----------|---------------------|-------------|---------|
| `Dockerfile.claude` | `/home/claude` (line 18) | `/home/claude` (line 27) | `USER` directive |
| `Dockerfile.codex` | `/home/codex` (line 18) | `/home/codex` (line 28) | `USER` directive |
| `Dockerfile.kilo` | `/home/kilo` (line 19) | `/home/kilo` (line 28) | `USER` directive |
| `Dockerfile.agent` | `/home/claude` (line 24) | N/A (not set) | `USER` directive + `ENV HOME` |
| `Dockerfile.ckrv` | N/A | N/A | Non-root user entirely |

Each Dockerfile already creates a home directory with `chmod -R 777` and sets `ENV HOME` — they were clearly **designed** to run as a non-root user, but the `USER` switch was never added.

### Fix

Add a non-root user and `USER` directive to each Dockerfile. The user should be created **after** all `RUN` commands that need root (apt-get, npm install) and **before** the `CMD`.

#### `Dockerfile.claude`

```dockerfile
# Chakravarti Claude Agent Container
# Contains Claude Code CLI for sandboxed agent execution

FROM node:22-slim

# Install dependencies
RUN apt-get update && apt-get install -y \
    git \
    curl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Install Claude Code CLI globally
RUN npm install -g @anthropic-ai/claude-code

# Create non-root user with home directory
RUN useradd -m -s /bin/bash -d /home/claude claude && \
    mkdir -p /home/claude/.claude && \
    chown -R claude:claude /home/claude

# Create workspace directory
RUN mkdir -p /workspace && chown claude:claude /workspace

WORKDIR /workspace

# Set HOME environment variable
ENV HOME=/home/claude

# Verify Claude is installed (must run before USER switch)
RUN claude --version || true

# Switch to non-root user
USER claude

# Default command
CMD ["/bin/bash"]
```

#### `Dockerfile.codex`

```dockerfile
# Chakravarti Codex Agent Container
# Contains OpenAI Codex CLI for sandboxed agent execution

FROM node:22-slim

# Install dependencies
RUN apt-get update && apt-get install -y \
    git \
    curl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Install OpenAI Codex CLI globally
RUN npm install -g @openai/codex

# Create non-root user with home directory
RUN useradd -m -s /bin/bash -d /home/codex codex && \
    mkdir -p /home/codex/.codex && \
    mkdir -p /home/codex/.config/openai && \
    chown -R codex:codex /home/codex

# Create workspace directory
RUN mkdir -p /workspace && chown codex:codex /workspace

WORKDIR /workspace

# Set HOME environment variable
ENV HOME=/home/codex

# Verify Codex is installed (must run before USER switch)
RUN codex --version || true

# Switch to non-root user
USER codex

# Default command
CMD ["/bin/bash"]
```

#### `Dockerfile.kilo`

```dockerfile
# Chakravarti Kilo Code Agent Container
# Contains Kilo Code CLI for sandboxed agent execution
# Supports 30+ AI providers through a single interface

FROM node:22-slim

# Install dependencies
RUN apt-get update && apt-get install -y \
    git \
    curl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Install Kilo Code CLI globally
RUN npm install -g @kilocode/cli

# Create non-root user with home directory
RUN useradd -m -s /bin/bash -d /home/kilo kilo && \
    mkdir -p /home/kilo/.config/kilo && \
    chown -R kilo:kilo /home/kilo

# Create workspace directory
RUN mkdir -p /workspace && chown kilo:kilo /workspace

WORKDIR /workspace

# Set HOME environment variable
ENV HOME=/home/kilo

# Verify Kilo is installed (must run before USER switch)
RUN kilo --version || true

# Switch to non-root user
USER kilo

# Default command
CMD ["/bin/bash"]
```

#### `Dockerfile.agent`

```dockerfile
# Chakravarti Agent Container
# Contains Claude Code CLI, OpenAI Codex CLI, and Kilo Code CLI for sandboxed agent execution

FROM node:22-slim

# Install dependencies and agent CLIs
RUN apt-get update && apt-get install -y \
    git \
    curl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Install Claude Code CLI globally
RUN npm install -g @anthropic-ai/claude-code

# Install OpenAI Codex CLI globally
RUN npm install -g @openai/codex

# Install Kilo Code CLI globally
RUN npm install -g @kilocode/cli

# Create non-root user with home directory
RUN useradd -m -s /bin/bash -d /home/agent agent && \
    mkdir -p /home/agent/.claude && \
    mkdir -p /home/agent/.codex && \
    mkdir -p /home/agent/.config/openai && \
    mkdir -p /home/agent/.config/kilo && \
    chown -R agent:agent /home/agent

# Create workspace directory
RUN mkdir -p /workspace && chown agent:agent /workspace

WORKDIR /workspace

# Set HOME environment variable
ENV HOME=/home/agent

# Verify all CLIs are installed (must run before USER switch)
RUN claude --version || true
RUN codex --version || true
RUN kilo --version || true

# Switch to non-root user
USER agent

# Default command
CMD ["/bin/bash"]
```

**Note on `Dockerfile.agent`**: The current version uses `/home/claude` as the home directory even for Codex and Kilo. The fix renames this to `/home/agent` for clarity and also sets `ENV HOME` which was missing.

#### `Dockerfile.ckrv`

```dockerfile
# Chakravarti CLI Agent Container
# Contains ckrv CLI for sandboxed batch execution

FROM rust:1.83-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    git \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy source code
COPY . .

# Build release binary
RUN cargo build --release --package ckrv-cli

# Runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    git \
    curl \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /build/target/release/ckrv /usr/local/bin/ckrv

# Create non-root user
RUN useradd -m -s /bin/bash -d /home/ckrv ckrv

# Create workspace directory
RUN mkdir -p /workspace && chown ckrv:ckrv /workspace

WORKDIR /workspace

ENV HOME=/home/ckrv

# Verify ckrv is installed (must run before USER switch)
RUN ckrv --version || true

# Switch to non-root user
USER ckrv

# Default command
CMD ["/bin/bash"]
```

### Credential Mount Permissions

When switching to non-root, credential bind mounts (e.g., `~/.claude.json`) must be readable by the new user. Since the files are bind-mounted from the host, two scenarios:

1. **Host user matches container user UID** — works automatically
2. **Host user UID differs** — the file may not be readable

The current `config_mounts()` in `ckrv-sandbox/src/agent/claude.rs` already sets `read_only: Some(true)`. As long as the host files are world-readable (which credential files usually are for the host user), this should work. If issues arise, we may need to add `--user $(id -u):$(id -g)` to the `docker run` command, but that's a separate concern.

### Acceptance Criteria

- [ ] All 5 Dockerfiles have a `USER` directive switching to a non-root user
- [ ] Non-root user is created with `useradd -m -s /bin/bash`
- [ ] All `RUN` commands that need root (apt-get, npm install) happen BEFORE `USER` switch
- [ ] `--version` verification runs BEFORE `USER` switch (still as root)
- [ ] `ENV HOME` is set in all Dockerfiles (including `Dockerfile.agent` which was missing it)
- [ ] `chmod 777` replaced with `chown user:user` (proper ownership instead of world-writable)
- [ ] `Dockerfile.agent` uses `/home/agent` instead of `/home/claude` for consistency
- [ ] Docker images build successfully: `docker build -t ckrv-claude:latest -f docker/Dockerfile.claude docker/`
- [ ] `ckrv term --sandbox` with `--dangerously-skip-permissions` no longer errors with root/sudo message
- [ ] Agent credential mounts are still readable inside the container

---

## BF-08: Agent guide docs missing `USER` requirement for Dockerfiles

**Severity**: Medium
**File(s)**: `crates/docs/agent-guide.md`, `crates/ckrv-sandbox/docs/README.md`
**Estimate**: 15m

### Problem

The agent guide's "Step 4: Add to Docker Image" section (line 236-245) shows how to add a new agent CLI to the Dockerfile but does not mention:

1. The requirement to set a non-root `USER` in the Dockerfile
2. Why running as root breaks agent CLIs (specifically Claude Code's `--dangerously-skip-permissions` restriction)
3. The correct ordering: install as root → create user → switch to `USER` → `CMD`

Additionally, `crates/ckrv-sandbox/docs/README.md` mentions "Execution Isolation" (line 30-36) but doesn't mention the non-root user requirement.

### Fix

**Part A**: Update `crates/docs/agent-guide.md` — expand Step 4 with user setup:

Add after the existing Step 4 content (after line 245):

```markdown
### Step 4: Add to Docker Image

If your agent requires a CLI, create a Dockerfile in `docker/`:

```dockerfile
FROM node:22-slim

# Install system dependencies (as root)
RUN apt-get update && apt-get install -y \
    git curl ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Install your agent CLI (as root)
RUN npm install -g your-agent-cli

# Create non-root user — REQUIRED
# Many agent CLIs (e.g., Claude Code) refuse to run certain flags as root.
# Always create a dedicated user and switch to it.
RUN useradd -m -s /bin/bash -d /home/youragent youragent && \
    mkdir -p /home/youragent/.youragent && \
    chown -R youragent:youragent /home/youragent

# Create workspace with correct ownership
RUN mkdir -p /workspace && chown youragent:youragent /workspace

WORKDIR /workspace
ENV HOME=/home/youragent

# Verify install (before USER switch, still root)
RUN your-agent-cli --version || true

# Switch to non-root user
USER youragent

CMD ["/bin/bash"]
```

> **⚠️ Important**: Always include a `USER` directive in your Dockerfile.
> Running as root causes agent CLIs to reject security-sensitive flags.
> For example, Claude Code blocks `--dangerously-skip-permissions` when
> running as root/sudo for security reasons.
```

**Part B**: Update `crates/ckrv-sandbox/docs/README.md` — add non-root note to "Execution Isolation" section:

Add after line 36:

```markdown
## Execution Isolation

All agent execution runs inside Docker containers:
- Isolated filesystem
- No network by default
- Command allow-list enforced
- Secrets via env vars only
- **Non-root user** — containers run as a dedicated user, not root

> **⚠️ Important**: All agent Docker containers must run as a non-root user.
> Agent CLIs like Claude Code enforce security restrictions when running as
> root (e.g., blocking `--dangerously-skip-permissions`). Each Dockerfile
> creates a dedicated user and switches to it via the `USER` directive.
```

### Acceptance Criteria

- [ ] `crates/docs/agent-guide.md` Step 4 shows `useradd` + `USER` directive in the Dockerfile example
- [ ] Agent guide includes a warning callout explaining why non-root is required
- [ ] Agent guide mentions Claude Code's `--dangerously-skip-permissions` root rejection as a concrete example
- [ ] `crates/ckrv-sandbox/docs/README.md` "Execution Isolation" section mentions non-root user
- [ ] Sandbox docs include a warning callout about the `USER` directive requirement
- [ ] Dockerfile ordering is documented: install as root → create user → `USER` switch → `CMD`

---

## Verification

After all bugfixes are applied:

- [ ] All 5 Docker images build successfully
- [ ] `ckrv term --sandbox` spawns container as non-root user
- [ ] `docker exec <container> whoami` returns the agent user name (not `root`)
- [ ] `ckrv term --sandbox` with skip-permissions flag works without root/sudo error
- [ ] `ckrv run` batch execution still works (agents in sandbox)
- [ ] Agent credential mounts readable inside container
- [ ] Agent guide and sandbox docs updated with non-root user requirement
- [ ] `make install` succeeds (rebuilds Docker images)

## Notes

- **BF-07 must be done first** — it's the critical fix. BF-08 is documentation.
- All Dockerfiles follow the same pattern: install as root → create user → set HOME → verify → USER switch → CMD
- The `Dockerfile.agent` (multi-agent) had an additional issue: HOME was not set and home dir was named `/home/claude` even for non-Claude agents
- `chmod 777` (world-writable) is replaced with `chown user:user` (proper ownership) — more secure
- If bind-mounted credential files have restrictive host permissions, we may need to add `--user $(id -u):$(id -g)` to docker run in a future task
