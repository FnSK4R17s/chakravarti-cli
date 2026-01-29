# Quickstart: Using GLM Coding Plan via CLI

**Feature**: 016-glm-cli-support  
**Date**: 2026-01-29

## Prerequisites

- Z.AI GLM Coding Plan subscription with API key
- Chakravarti CLI installed (`ckrv` command available)
- Docker running (for sandboxed execution)

## Step 1: Configure GLM Agent

Add your GLM agent to `~/.config/chakravarti/agents.yaml`:

```yaml
agents:
  - name: "my-glm-agent"
    agent_type: ClaudeGLM
    glm:
      api_key: "your-zai-api-key"
      model: "glm-4.7"
      timeout_ms: 3000000
```

## Step 2: Verify Agent Configuration

```bash
# List all configured agents
ckrv agents list

# Test GLM agent connectivity (optional)
ckrv agents test my-glm-agent
```

Expected output:
```
Agents:
  ✓ my-glm-agent [GLM] - glm-4.7
```

## Step 3: Run a Task with GLM Agent

### Option A: Single Command

```bash
ckrv task run --agent "my-glm-agent" -p "Create a hello.txt file with 'Hello World'"
```

### Option B: Full Workflow

```bash
# Initialize a spec
ckrv spec new "Add greeting feature"

# Generate tasks
ckrv spec tasks

# Generate plan with GLM agent
ckrv plan --agent "my-glm-agent"

# Execute with GLM agent
ckrv run --agent "my-glm-agent"
```

## Step 4: Verify GLM Execution

Check the logs to confirm GLM is being used:

```bash
ckrv run --agent "my-glm-agent" 2>&1 | grep "GLM"
```

Expected output:
```
Using GLM Coding Plan: glm-4.7
```

## Troubleshooting

### "Agent not found"
- Verify agent name matches exactly in `agents.yaml`
- Run `ckrv agents list` to see available agents

### "Authentication failed"
- Verify Z.AI API key is valid
- Check API key has GLM Coding Plan access

### "Model not found"
- Use valid model: `glm-4.7` or `glm-4.5-air`
- Check Z.AI documentation for available models
