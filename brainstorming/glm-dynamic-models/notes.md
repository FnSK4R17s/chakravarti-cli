# Dynamic Model Fetching for GLM Coding Plan

**Created**: 2026-02-12
**Status**: Draft

## Problem Statement

GLM Coding Plan model selection is hardcoded to just 2 models (`glm-4.7` and `glm-4.5-air`) in the UI, while Z.AI actually offers 9+ models including GLM-5, GLM-4.6, GLM-4.7-Flash, etc. OpenRouter and Kilo Code both fetch models dynamically — GLM should follow the same pattern.

## Current State

**What exists today** (`AgentManager.tsx:1158-1174`):
```tsx
<SelectItem value="glm-4.7">GLM-4.7 (Recommended)</SelectItem>
<SelectItem value="glm-4.5-air">GLM-4.5-Air (Faster)</SelectItem>
```

Two hardcoded options. No context window info, no pricing, no dynamic discovery. When Z.AI ships new models (they've gone from GLM-4.5 → GLM-5 already), we have to manually update the code.

**How OpenRouter does it** (`handlers/agents.rs:638-707`):
- `GET https://openrouter.ai/api/v1/models` → parses `{ data: [{ id, name, context_length, pricing }] }`
- Falls back to curated list on failure
- Frontend fetches via `GET /api/agents/models`

**How Kilo Code does it** (`handlers/agents.rs:504-584`):
- Runs `kilo models` CLI command → parses stdout lines
- Falls back to curated list on failure
- Frontend fetches via `GET /api/agents/kilo-models`

## Known Z.AI Models (as of 2026-02)

From Mastra docs and Z.AI Coding Plan documentation:

| Model ID | Context Window | Notes |
|----------|---------------|-------|
| `glm-5` | 205K | Flagship, agentic engineering |
| `glm-4.7` | 205K | Current recommended |
| `glm-4.7-flash` | 200K | Fast variant |
| `glm-4.6` | 205K | Previous gen |
| `glm-4.6v` | 128K | Multimodal (vision) |
| `glm-4.5` | 131K | Older gen |
| `glm-4.5-air` | 131K | Fast/cheap |
| `glm-4.5-flash` | 131K | Fastest of 4.5 |
| `glm-4.5v` | 64K | Multimodal (vision) |

**Coding Plan endpoint**: `https://api.z.ai/api/coding/paas/v4`
**Standard endpoint**: `https://api.z.ai/api/paas/v4`

## Technical Approach

### Option A: OpenAI-Compatible `/models` Endpoint (Preferred)

Z.AI is OpenAI-compatible. Roo Code [confirms](https://docs.roocode.com/providers/zai) they "automatically fetch all available models from Z AI's API." The standard OpenAI `/models` endpoint should work:

```
GET https://api.z.ai/api/paas/v4/models
Authorization: Bearer {api_key}
```

Or for the Coding Plan variant:
```
GET https://api.z.ai/api/coding/paas/v4/models
Authorization: Bearer {api_key}
```

**Implementation flow:**

1. **New Rust type** in `ckrv-transport/src/types/agents.rs`:
```rust
pub struct GlmModel {
    pub id: String,          // "glm-4.7"
    pub name: String,        // "GLM-4.7"
    pub context_length: Option<u32>,
    pub category: Option<String>,  // "text", "multimodal"
}
```

2. **New handler** in `ckrv-transport/src/handlers/agents.rs`:
```rust
pub async fn get_glm_models_handler(api_key: Option<String>) -> Result<Vec<GlmModel>, TransportError> {
    match fetch_glm_models(api_key).await {
        Ok(models) => Ok(models),
        Err(_) => Ok(get_fallback_glm_models()),
    }
}

async fn fetch_glm_models(api_key: Option<String>) -> Result<Vec<GlmModel>, TransportError> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let mut req = client
        .get("https://api.z.ai/api/paas/v4/models")
        .header("Accept", "application/json");

    if let Some(key) = api_key {
        req = req.header("Authorization", format!("Bearer {key}"));
    }

    let response = req.send().await?;
    // Parse OpenAI-compatible response: { data: [{ id, object, created, owned_by }] }
    // Filter to coding-relevant models (exclude image/video models)
}
```

3. **New route** in `ckrv-transport/src/axum/agents.rs`:
```rust
.route("/agents/glm-models", get(get_glm_models))
```

4. **Frontend query** in `AgentManager.tsx`:
```typescript
const fetchGlmModels = async (): Promise<{ models: GlmModel[] }> => {
    const res = await fetch('/api/agents/glm-models');
    return res.json();
};
const { data: glmModelsData } = useQuery({
    queryKey: ['glm-models'],
    queryFn: fetchGlmModels,
});
```

5. **Replace hardcoded Select** with dynamic dropdown matching OpenRouter/Kilo patterns.

| Pros | Cons |
|------|------|
| Consistent with OpenRouter pattern | Requires API key to fetch (can't browse before configuring) |
| Always up-to-date with Z.AI offerings | Z.AI `/models` endpoint isn't explicitly documented |
| Gets real model metadata | May need auth header unlike OpenRouter (which is public) |

### Option B: Curated List with Periodic Refresh

Maintain a hardcoded list in Rust that we update when Z.AI releases new models. No API call needed.

```rust
fn get_glm_models() -> Vec<GlmModel> {
    vec![
        GlmModel { id: "glm-5", name: "GLM-5 (Flagship)", context_length: Some(205_000), .. },
        GlmModel { id: "glm-4.7", name: "GLM-4.7 (Recommended)", context_length: Some(205_000), .. },
        GlmModel { id: "glm-4.7-flash", name: "GLM-4.7 Flash", context_length: Some(200_000), .. },
        // ...
    ]
}
```

| Pros | Cons |
|------|------|
| No API call, works offline | Gets stale when Z.AI adds models |
| No auth needed to browse models | Manual maintenance burden |
| Instant — no loading state | Less impressive UX |

### Option C: Hybrid — Try API, Fall Back to Curated List

Best of both worlds. This is exactly what OpenRouter and Kilo Code do.

1. If user has entered an API key → try `GET /models` with auth
2. If no key or API fails → show curated fallback list
3. Fallback list covers the 9 known models with context window info

| Pros | Cons |
|------|------|
| Works with or without API key | Slightly more code |
| Auto-discovers new models when key is set | Need to maintain fallback list too |
| Follows existing OpenRouter/Kilo pattern exactly | — |

### Decision

**Option C (Hybrid)** — it's the established pattern in this codebase and handles both connected/disconnected states gracefully.

## Implementation Notes

### API Key Chicken-and-Egg Problem

Unlike OpenRouter (public `/models`) and Kilo Code (local CLI), Z.AI likely requires authentication to list models. This creates a UX challenge: user can't browse models before entering their API key.

**Solutions:**
1. Show fallback list immediately, upgrade to live list once API key is entered
2. Trigger model fetch when API key field loses focus (onBlur)
3. Add a "Refresh Models" button that fetches with current key

The onBlur approach is cleanest — user types their key, tabs away, models auto-populate.

### Filtering Vision/Non-Coding Models

Not all Z.AI models make sense for coding tasks. The `glm-4.6v` and `glm-4.5v` are multimodal/vision models that probably shouldn't be the default for code generation. Options:
- Show all but tag vision models with a badge
- Filter out non-text models by default
- Let users see all but sort coding models first

### Coding Plan vs Standard Endpoint

Z.AI has two API paths:
- Standard: `https://api.z.ai/api/paas/v4`
- Coding Plan: `https://api.z.ai/api/coding/paas/v4`

The Coding Plan endpoint is what we route through (via `ANTHROPIC_BASE_URL`). We should try the coding endpoint for model listing first, fall back to the standard one.

### Changes Summary

| File | Change |
|------|--------|
| `ckrv-transport/src/types/agents.rs` | Add `GlmModel` struct |
| `ckrv-transport/src/handlers/agents.rs` | Add `get_glm_models_handler()`, `fetch_glm_models()`, `get_fallback_glm_models()` |
| `ckrv-transport/src/axum/agents.rs` | Add `/agents/glm-models` route |
| `ckrv-transport/src/tauri/agents.rs` | Add `get_glm_models` Tauri command |
| `ckrv-ui/frontend/src/components/AgentManager.tsx` | Add `useQuery` for GLM models, replace hardcoded Select with dynamic dropdown |

## Open Questions

- [ ] Does `GET https://api.z.ai/api/paas/v4/models` actually work? Need to test with a real API key
- [ ] Does the coding plan endpoint (`/api/coding/paas/v4/models`) return a different model set than the standard one?
- [ ] Does the API require auth for the `/models` endpoint, or is it public like OpenRouter?
- [ ] Should we filter out vision models (`glm-4.6v`, `glm-4.5v`) or show them with a badge?
- [ ] Should we pass the API key from the form to the backend for model fetching, or use a stored key?

## Success Criteria

| Metric | Target |
|--------|--------|
| Model list completeness | All 9 Z.AI models shown (vs current 2) |
| Fallback reliability | Works without API key using curated list |
| Load time | Models appear within 2s of API key entry |
| Consistency | UX matches OpenRouter/Kilo Code model selection pattern |

## Next Steps

- [ ] Test `GET https://api.z.ai/api/paas/v4/models` with a real Z.AI API key
- [ ] If `/models` works: implement Option C (hybrid fetch + fallback)
- [ ] If `/models` doesn't exist: implement Option B (expanded curated list — still a win going from 2 → 9 models)
- [ ] Add context window display in model info card
- [ ] Wire up onBlur API key → model refresh flow

## References

- [Z.AI API Docs](https://docs.z.ai/guides/overview/quick-start)
- [Z.AI Models on Mastra (Standard)](https://mastra.ai/models/providers/zai)
- [Z.AI Models on Mastra (Coding Plan)](https://mastra.ai/models/providers/zai-coding-plan)
- [Roo Code Z.AI Integration](https://docs.roocode.com/providers/zai) — confirms dynamic model fetching works
- [OpenRouter pattern](../../crates/ckrv-transport/src/handlers/agents.rs) — lines 630-707
- [Kilo Code pattern](../../crates/ckrv-transport/src/handlers/agents.rs) — lines 504-584
- [Current GLM UI](../../crates/ckrv-ui/frontend/src/components/AgentManager.tsx) — lines 1149-1206
