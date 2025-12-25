# Quickstart: Chakravarti CLI

**Feature**: 001-cli-mvp  
**Date**: 2025-12-12

---

## Prerequisites

- **Rust** (stable, 1.75+): Install via [rustup](https://rustup.rs/)
- **Git** (2.5+): For worktree support
- **Docker** or **Podman**: For sandboxed execution
- **Model API Key**: OpenAI, Anthropic, or compatible endpoint

---

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/FnSK4R17s/chakravarti-cli.git
cd chakravarti-cli

# Build release binary
cargo build --release

# Add to PATH (or symlink)
export PATH="$PATH:$(pwd)/target/release"

# Verify installation
ckrv --version
```

### From Cargo (once published)

```bash
cargo install chakravarti-cli
```

---

## Configuration

### 1. Set API Keys

```bash
# OpenAI (default)
export OPENAI_API_KEY="sk-..."

# Or Anthropic
export ANTHROPIC_API_KEY="sk-ant-..."

# Or custom endpoint
export CKRV_MODEL_ENDPOINT="http://localhost:8080"
export CKRV_MODEL_API_KEY="your-key"
```

### 2. Initialize Repository

```bash
cd your-project
ckrv init
```

This creates:
```
.specs/           # Specification files
.chakravarti/     # Configuration and run data
  └── config.json # Local configuration
```

---

## Basic Workflow

### Step 1: Create a Spec

```bash
ckrv spec new add_rate_limiter
```

Edit `.specs/add_rate_limiter.yaml`:

```yaml
id: add_rate_limiter
goal: Add rate limiting to login endpoint

constraints:
  - No breaking API changes
  - Must include unit tests
  - Use existing middleware pattern

acceptance:
  - Returns 429 after 5 failed attempts in 1 minute
  - Rate limit resets after 1 minute
  - Successful login resets counter
```

### Step 2: Validate the Spec

```bash
ckrv spec validate .specs/add_rate_limiter.yaml
```

### Step 3: Run the Job

```bash
ckrv run .specs/add_rate_limiter.yaml
```

Watch the output:
```
▶ Running spec: add_rate_limiter
  Job ID: a1b2c3d4

📋 Planning...
  Generated 5 steps

🔄 Attempt 1/3
  [1/5] analyze_login_flow ✓
  [2/5] add_rate_limit_middleware ✓
  [3/5] update_config ✓
  [4/5] add_tests ✓
  [5/5] run_tests ✓

✅ Verification passed

→ Diff available: ckrv diff a1b2c3d4
```

### Step 4: Review the Changes

```bash
# View the diff
ckrv diff a1b2c3d4

# See statistics
ckrv diff a1b2c3d4 --stat

# View cost report
ckrv report a1b2c3d4
```

### Step 5: Promote to Branch

```bash
ckrv promote a1b2c3d4 --branch feature/add-rate-limiter

# Optionally push to remote
ckrv promote a1b2c3d4 --branch feature/add-rate-limiter --push
```

---

## Advanced Options

### Optimization Modes

```bash
# Optimize for cost (slower, cheaper models)
ckrv run .specs/add_rate_limiter.yaml --optimize=cost

# Optimize for time (faster, more expensive models)
ckrv run .specs/add_rate_limiter.yaml --optimize=time

# Balanced (default)
ckrv run .specs/add_rate_limiter.yaml --optimize=balanced
```

### Model Overrides

```bash
# Use specific models
ckrv run .specs/add_rate_limiter.yaml \
  --planner-model=gpt-4.1 \
  --executor-model=gpt-4o-mini
```

### Retry Configuration

```bash
# Allow more attempts
ckrv run .specs/add_rate_limiter.yaml --max-attempts=5
```

### JSON Output

```bash
# All commands support JSON output
ckrv run .specs/add_rate_limiter.yaml --json 2>&1 | jq

# Useful for scripting
JOB_ID=$(ckrv run .specs/add_rate_limiter.yaml --json | jq -r '.job_id')
```

---

## Directory Structure

After running jobs:

```
.specs/
└── add_rate_limiter.yaml       # Your spec

.chakravarti/
├── config.json                  # Configuration
└── runs/
    └── a1b2c3d4/               # Job data
        ├── job.json            # Job state
        ├── plan.json           # Generated plan
        ├── metrics.json        # Cost/time metrics
        └── logs/               # Execution logs

.worktrees/                      # Temporary (cleaned up)
└── a1b2c3d4/
    └── 1/                      # Attempt worktree
```

---

## CI/CD Integration (GitLab)

Example `.gitlab-ci.yml`:

```yaml
stages:
  - prepare
  - agent
  - review

variables:
  CKRV_OPTIMIZE: balanced
  CKRV_MAX_ATTEMPTS: "3"

run-spec:
  stage: agent
  image: rust:latest
  services:
    - docker:dind
  before_script:
    - cargo install chakravarti-cli
  script:
    - ckrv init --force
    - ckrv run $SPEC_FILE --json > result.json
    - |
      if [ "$(jq -r '.success' result.json)" = "true" ]; then
        JOB_ID=$(jq -r '.job_id' result.json)
        ckrv promote $JOB_ID --branch agent/$CI_PIPELINE_ID --push
      fi
  artifacts:
    paths:
      - result.json
      - .chakravarti/runs/
  only:
    - merge_requests
```

---

## Troubleshooting

### Container Runtime Not Found

```
Error: Container runtime not available
```

Ensure Docker or Podman is running:
```bash
docker ps  # or: podman ps
```

### API Key Missing

```
Error: Model API key not configured
```

Set your API key:
```bash
export OPENAI_API_KEY="sk-..."
```

### Git Worktree Failed

```
Error: Failed to create worktree
```

Ensure git version supports worktrees:
```bash
git --version  # Should be 2.5+
```

### Verification Failed

Check the job report for details:
```bash
ckrv report <job_id>
ckrv status <job_id> --verbose
```

---

## Next Steps

1. **Read the Spec Guide**: Learn to write effective specifications
2. **Explore Reports**: Understand cost/time optimization
3. **CI Integration**: Automate agent-driven changes in your pipeline
4. **Model Configuration**: Fine-tune model selection for your needs
