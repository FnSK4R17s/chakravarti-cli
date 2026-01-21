# Research: Persistent Runner Logs

**Feature**: 010-persistent-runner-logs  
**Date**: 2026-01-15

## Research Summary

This feature is primarily an internal implementation leveraging existing patterns in the codebase. No external research was required.

## Decisions

### 1. Log Storage Format

**Decision**: JSONL (JSON Lines) format

**Rationale**:
- Append-only writes are atomic and safe for concurrent access
- Each line is independently parseable (corruption-resistant)
- Easy to stream line-by-line during replay
- Human-readable for debugging
- Already used for structured log output in engine.rs

**Alternatives Considered**:
- SQLite: Overkill for append-only logs, adds dependency
- Single JSON file: Requires rewriting entire file on each append
- Binary format: Less debuggable, marginal performance gain

### 2. Storage Location

**Decision**: `.ckrv/logs/{execution_id}/log.jsonl`

**Rationale**:
- `.ckrv/` is already used for local state (config files)
- Execution ID as folder allows easy cleanup per-execution
- JSONL extension clearly indicates format
- Folder structure allows future expansion (metadata, artifacts)

**Alternatives Considered**:
- `~/.cache/ckrv/logs/`: Cross-project, harder to manage
- In-memory with async write: Risk of data loss on crash
- Per-spec folder: Would mix logs with spec documents

### 3. History Fetch Mechanism

**Decision**: REST endpoint with offset/limit pagination

**Rationale**:
- Simple request/response for historical data
- Easier caching than WebSocket
- Frontend can request specific ranges for scroll loading
- WebSocket reserved for real-time streaming only

**Alternatives Considered**:
- WebSocket-only: Complex state management for history
- Cursor-based pagination: JSONL line numbers work better here
- Full file download: Memory issues with large logs

### 4. Real-Time Display Strategy

**Decision**: Show only tail 10 logs during real-time streaming (per clarification)

**Rationale**:
- User explicitly requested this behavior
- Reduces UI rendering overhead during high-volume logging
- Full history accessible via scroll-up gesture
- Clear mental model: "recent = live, scroll = history"

### 5. Auto-Cleanup Trigger

**Decision**: Delete logs when all associated worktrees are merged

**Rationale**:
- Matches user's mental model of "done with feature"
- Ties into existing merge_all_branches flow
- No arbitrary time-based retention (per clarification)
- User can manually delete if needed earlier

**Alternatives Considered**:
- Time-based retention: User rejected this
- Manual-only: User wanted auto-cleanup on merge
- Size-based rotation: Complicates simple append model

### 6. WebSocket Reconnection Strategy

**Decision**: On reconnect, send `last_timestamp` to request missed logs

**Rationale**:
- Frontend tracks last received log timestamp
- Backend fetches logs from file where `timestamp > last_timestamp`
- Merges seamlessly with WebSocket stream
- No duplicate detection needed (timestamp-ordered)

## Technology Compatibility

| Component | Compatibility | Notes |
|-----------|--------------|-------|
| Tokio async | ✅ Full | Already used in execution engine |
| Axum REST | ✅ Full | Add new route, follows existing patterns |
| WebSocket | ✅ Full | Extend existing handler |
| React Query | ✅ Full | Use for history fetch with caching |
| File I/O | ✅ Full | Standard std::fs with tokio for async |

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Disk space exhaustion | Low | Medium | Logs auto-cleanup on merge; add size warning |
| Slow history load | Low | Medium | Pagination, lazy loading, virtualization |
| File corruption | Very Low | Low | JSONL is line-atomic; skip corrupt lines |
| Race condition on write | Low | Medium | Append-only with file locking |

## Conclusion

No blocking unknowns. Implementation can proceed using standard patterns already established in the codebase.
