# Agent Usage Collection Notes

This document captures practical usage collection options for newly added agents.

## Current implementation in `ckrv usage --agents`

- Primary source: **local job metrics model attribution** (`.chakravarti/metrics/*.json`)
- Safety behavior: if no attributable usage is found, ckrv returns a structured fallback reason (`usage_status: "fallback"`, `fallback_reason`) instead of failing.

## Agent-specific practical sources

### Gemini
- Primary in ckrv: model names containing `gemini`
- Additional practical source: Gemini CLI/API usage dashboard or usage endpoints when available.

### Cursor
- Primary in ckrv: model names containing `cursor`
- Additional practical source: Cursor local session/app logs (environment-dependent).

### Amp
- Primary in ckrv: model names containing `amp`
- Additional practical source: Amp local config/cache (`~/.config/amp`, `~/.cache/amp`) if usage traces are present.

### Qwen Code
- Primary in ckrv: model names containing `qwen`
- Additional practical source: Qwen local state/logs (`~/.qwen`) when present.

### Opencode
- Primary in ckrv: model names containing `opencode`/`open-code`
- Additional practical source: Opencode local logs/state (`~/.opencode`) when present.

### Factory Droid
- Primary in ckrv: model names containing `factory`
- Additional practical source: Factory service usage APIs/dashboard if available for account.

### GitHub Copilot
- Primary in ckrv: model names containing `copilot`
- Additional practical source: `gh copilot` session output and GitHub/Copilot account usage pages.

### Mistral Vibe
- Primary in ckrv: model names containing `vibe`/`mistral`
- Additional practical source: Mistral account/API usage dashboard endpoints.

## Why local attribution first

- Works offline and consistently across providers.
- Uses already-recorded ckrv job token/cost telemetry.
- Avoids brittle provider-specific parsing during routine usage checks.
