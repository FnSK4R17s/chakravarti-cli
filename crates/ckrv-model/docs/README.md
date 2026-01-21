---
last_commit: c1bb442
last_updated: 2026-01-21
related_files:
  - src/lib.rs
  - src/router.rs
  - src/provider.rs
---

# ckrv-model

Model gateway and routing for Chakravarti.

## Overview

This crate provides model provider abstraction, routing logic, and token/cost accounting for LLM interactions.

## Key Types

| Type | Purpose |
|------|---------|
| `ModelRouter` | Routes requests to providers |
| `ModelProvider` | Provider interface |
| `AnthropicProvider` | Anthropic/Claude API |
| `OpenAIProvider` | OpenAI API |
| `BudgetTracker` | Cost tracking |
| `PricingCatalog` | Model pricing data |

## Usage

```rust
use ckrv_model::{ModelRouter, RoutingContext, TaskType};

let router = ModelRouter::new(config);

let response = router.complete(CompletionRequest {
    prompt: "Generate code...",
    context: RoutingContext {
        task_type: TaskType::Coding,
        budget_remaining: 10.0,
    },
})?;
```

## Module Structure

```
src/
├── router.rs       # Request routing
├── provider.rs     # Provider trait
├── anthropic.rs    # Anthropic implementation
├── openai.rs       # OpenAI implementation
├── pricing.rs      # Cost calculation
├── accounting.rs   # Token tracking
└── error.rs        # Error types
```

## Routing Strategy

The router selects providers based on:
- Task type (coding, planning, review)
- Remaining budget
- Model availability
- Performance requirements

## Dependencies

| Crate | Purpose |
|-------|---------|
| `reqwest` | HTTP client |
| `serde_json` | JSON parsing |
| `tokio` | Async runtime |
