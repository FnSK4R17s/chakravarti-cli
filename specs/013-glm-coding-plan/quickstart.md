# Quickstart: GLM Coding Plan Agent

## Prerequisites

1. **Z.AI Account**: Sign up at [z.ai/subscribe](https://z.ai/subscribe)
2. **API Key**: Generate at [z.ai/manage-apikey](https://z.ai/manage-apikey/apikey-list)
3. **Chakravarti CLI**: Running with `ckrv ui`

## Configuration Steps

### 1. Open Agent Manager

Navigate to the Agents section in the Chakravarti UI.

### 2. Add New Agent

Click "Add Agent" and select:
- **Type**: GLM Coding Plan
- **Name**: e.g., "GLM-4.7 Agent"
- **API Key**: Paste your Z.AI API key
- **Model**: Select from:
  - `glm-4.7` (recommended for coding tasks)
  - `glm-4.5-air` (faster, lighter option)

### 3. Save and Test

Click "Save" then "Test Connection" to verify the configuration.

## Using the Agent

### Batch Execution

1. Select your GLM agent from the agent dropdown
2. Create or select a spec with batches
3. Click "Run" to execute with GLM

### Interactive Terminal

1. Select your GLM agent
2. Start a terminal session
3. Use Claude Code commands as normal - they will route through Z.AI

## Verification

Run `/status` in Claude Code to confirm:
```
Model: glm-4.7
Provider: Z.AI
```

## Troubleshooting

| Issue | Solution |
|-------|----------|
| "Authentication failed" | Verify API key is correct and has credits |
| "Timeout" | Try increasing `timeout_ms` in agent config |
| "Model not found" | Check model name spelling (case-sensitive) |
