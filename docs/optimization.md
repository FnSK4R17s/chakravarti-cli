# Optimization Modes

Chakravarti supports three optimization modes that control model selection and execution strategy:

## Usage

```bash
# Cost optimization - minimize API costs
ckrv run .specs/feature.yaml --optimize cost

# Time optimization - minimize execution time
ckrv run .specs/feature.yaml --optimize time

# Balanced (default) - balance between cost and time
ckrv run .specs/feature.yaml --optimize balanced
```

## Mode Details

### Cost Optimization (`--optimize cost`)

Prioritizes lower API costs by selecting cheaper models:

| Task Type | Model | Approx. Cost |
|-----------|-------|--------------|
| Planning | gpt-4o-mini | $0.15/M in, $0.60/M out |
| Execution | gpt-4o-mini | $0.15/M in, $0.60/M out |
| Verification | gpt-4o-mini | $0.15/M in, $0.60/M out |

**Best for:**
- Exploratory work
- Simple specifications
- High-volume batch processing
- Development/testing

### Time Optimization (`--optimize time`)

Prioritizes faster completion by using more capable models that require fewer retries:

| Task Type | Model | Approx. Cost |
|-----------|-------|--------------|
| Planning | gpt-4o | $2.50/M in, $10/M out |
| Execution | gpt-4o | $2.50/M in, $10/M out |
| Verification | gpt-4o-mini | $0.15/M in, $0.60/M out |

**Best for:**
- Production deployments
- Complex specifications
- Time-sensitive changes
- When quality matters most

### Balanced Optimization (`--optimize balanced`)

Uses task-appropriate models based on complexity:

| Task Type | Model | Rationale |
|-----------|-------|-----------|
| Planning | gpt-4o | Strong reasoning for plan generation |
| Execution | gpt-4o-mini | Fast code generation for simple edits |
| Verification | gpt-4o-mini | Quick verification checks |

**Best for:**
- General use (default)
- Unknown specification complexity
- Mixed workloads

## Model Overrides

You can override model selection with explicit flags:

```bash
# Use specific planner model
ckrv run .specs/feature.yaml --planner-model claude-3-5-sonnet

# Use specific executor model
ckrv run .specs/feature.yaml --executor-model gpt-4-turbo

# Combine overrides with optimization mode
ckrv run .specs/feature.yaml --optimize cost --executor-model gpt-4o
```

## Budget Tracking

Chakravarti tracks token usage and costs per job:

```bash
# View cost report for a job
ckrv report <job_id>

# Example output:
# Total Tokens: 15,234
# Est. Cost: $0.0045
# By Model:
#   gpt-4o-mini: $0.0042
#   gpt-4o: $0.0003
```

## Supported Models

### OpenAI
- `gpt-4o` - Best for complex reasoning and code generation
- `gpt-4o-mini` - Fast and cost-effective for simpler tasks
- `gpt-4-turbo` - High capability with longer context
- `o1` - Advanced reasoning (currently not recommended)
- `o1-mini` - Reasoning at lower cost

### Anthropic
- `claude-3-5-sonnet` - Balanced capability and speed
- `claude-3-5-haiku` - Fast and economical
- `claude-3-opus` - Highest capability (expensive)

### Custom Endpoints

Configure custom OpenAI-compatible endpoints:

```bash
export CKRV_MODEL_API_KEY="your-api-key"
export CKRV_MODEL_ENDPOINT="https://your-endpoint.com/v1"
```

## Price Estimates (as of December 2024)

| Model | Input ($/M) | Output ($/M) | Context |
|-------|-------------|--------------|---------|
| gpt-4o | $2.50 | $10.00 | 128K |
| gpt-4o-mini | $0.15 | $0.60 | 128K |
| claude-3-5-sonnet | $3.00 | $15.00 | 200K |
| claude-3-5-haiku | $0.80 | $4.00 | 200K |

*Prices may change. Check provider documentation for current rates.*
