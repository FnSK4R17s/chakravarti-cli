---
last_commit: 5160ff1
last_updated: 2026-01-29
related_files:
  - src/lib.rs
  - src/router.rs
  - src/provider.rs
  - src/pricing.rs
---

# ckrv-model

Model gateway and routing for Chakravarti.

> [!WARNING]
> **This crate is currently NOT USED.** Chakravarti executes code generation via Claude Code and Codex through Docker sandboxes (`ckrv-sandbox`), not direct API calls. This crate was built as infrastructure for a planned "bring your own key" feature that has not been integrated. It remains in the workspace for potential future use.

## Overview

This crate provides model provider abstraction, routing logic, and token/cost accounting for LLM interactions. It supports multiple providers (Anthropic, OpenAI) and intelligent model selection.

## Key Types

| Type | Module | Purpose |
|------|--------|---------|
| `ModelRouter` | router.rs | Routes requests, manages providers |
| `RoutingContext` | router.rs | Task type + optimization mode |
| `ModelSelection` | router.rs | Selection result with details |
| `TaskType` | router.rs | Planning, Execution, Verification |
| `BudgetTracker` | router.rs | Cost limit enforcement |
| `ModelProvider` | provider.rs | Async provider trait |
| `CompletionRequest` | provider.rs | Messages + model + params |
| `CompletionResponse` | provider.rs | Content + usage + model |
| `Message` | provider.rs | Role + content for chat |
| `AnthropicProvider` | anthropic.rs | Claude API implementation |
| `OpenAIProvider` | openai.rs | GPT API implementation |
| `PricingCatalog` | pricing.rs | Model pricing data |
| `ModelPricing` | pricing.rs | Per-model cost info |
| `TokenUsage` | accounting.rs | Input/output token counts |
| `UsageAccumulator` | accounting.rs | Aggregate token tracking |
| `ModelError` | error.rs | Error types |

## Module Structure

```
src/
├── lib.rs         # Public exports
├── router.rs      # Request routing (14KB)
├── provider.rs    # Provider trait
├── anthropic.rs   # Anthropic/Claude API
├── openai.rs      # OpenAI API
├── pricing.rs     # Cost calculation (9KB)
├── accounting.rs  # Token tracking
└── error.rs       # Error types
```

## Usage

### Basic Completion

```rust
use ckrv_model::{ModelRouter, CompletionRequest, Message};

let router = ModelRouter::new()?;

let response = router.complete(CompletionRequest {
    model: "claude-3-5-sonnet".to_string(),
    messages: vec![Message {
        role: "user".to_string(),
        content: "Write a function that...".to_string(),
    }],
    max_tokens: Some(4096),
    temperature: Some(0.7),
}).await?;

println!("Response: {}", response.content);
println!("Tokens: {} in, {} out", response.usage.input_tokens, response.usage.output_tokens);
```

### Model Selection

```rust
use ckrv_model::{ModelRouter, RoutingContext, TaskType};
use ckrv_core::OptimizeMode;

let router = ModelRouter::new()?;

// Select based on task type and optimization mode
let context = RoutingContext {
    task_type: TaskType::Execution,
    max_tokens: 4096,
    optimize: OptimizeMode::Cost,
};

let selection = router.select(&context);
println!("Selected: {} from {}", selection.model, selection.provider);

// Or get just the model name
let model = router.select_model(&context);
```

### Budget Tracking

```rust
use ckrv_model::{ModelRouter, BudgetTracker};

let router = ModelRouter::new()?;

// Set budget limit
router.set_budget(10.0); // $10 USD

// Check budget before request
let budget = router.budget();
if budget.lock().unwrap().has_budget(0.50) {
    // Make request...
}

// Track remaining
println!("Remaining: ${:.2}", budget.lock().unwrap().remaining());
```

### Pricing Information

```rust
use ckrv_model::{PricingCatalog, ModelPricing};

let catalog = PricingCatalog::new();

// Get pricing for a model
if let Some(pricing) = catalog.get("claude-3-5-sonnet") {
    let cost = pricing.calculate_cost(1000, 500);
    println!("Cost: ${:.4}", cost);
}

// Find cheapest for a provider
let cheapest = catalog.cheapest(Some("anthropic"));
```

## Routing Strategy

The router selects models based on:
- **Task type**: Planning (needs reasoning), Execution (needs code gen), Verification (needs analysis)
- **Optimization mode**: Cost, Time, Balanced (from `ckrv-core::OptimizeMode`)
- **Budget remaining**: Downgrades to cheaper models as budget depletes

## Traits

### ModelProvider

```rust
#[async_trait]
pub trait ModelProvider: Send + Sync {
    fn name(&self) -> &str;
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, ModelError>;
}
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `ckrv-core` | `OptimizeMode` enum |
| `reqwest` | HTTP client |
| `serde` | Serialization |
| `async-trait` | Async trait support |
| `tokio` | Async runtime |
