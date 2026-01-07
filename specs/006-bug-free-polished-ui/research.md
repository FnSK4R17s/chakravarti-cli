# Research: Bug-Free and Polished Chakravarti CLI UI

**Feature**: 006-bug-free-polished-ui  
**Date**: 2026-01-05  
**Status**: Complete

## Technology Research

### E2E Testing Framework

**Decision**: Playwright  
**Rationale**: 
- First-class TypeScript support
- Cross-browser testing (Chrome, Firefox, Safari, Edge) per spec requirements
- Built-in retry mechanisms for flaky test prevention
- Parallel test execution for fast CI feedback
- Good WebSocket support for testing real-time features
- Visual comparison capabilities for regression testing

**Alternatives Considered**:
- Cypress: Excellent DX but weaker cross-browser support, no native WebSocket testing
- Puppeteer: Chrome-only, lower-level API

### WebSocket Message Throttling

**Decision**: Use `requestAnimationFrame` batching + React 18 batched updates  
**Rationale**:
- Prevents DOM thrashing during rapid WebSocket message bursts
- Maintains 60fps rendering even at >10 messages/second
- React 18 automatic batching already helps, but requestAnimationFrame provides additional frame-aligned batching

**Alternatives Considered**:
- Throttle with lodash: Fixed intervals don't align with browser rendering cycles
- Web Worker: Overkill for this use case, adds complexity

### Auto-Retry Pattern

**Decision**: Exponential backoff with visible countdown (3 attempts, starting at 5s)  
**Rationale**:
- Handles transient network issues automatically
- User visibility prevents confusion about connection state
- 3 attempts is reasonable before requiring manual intervention
- Exponential backoff (5s → 10s → 20s) prevents server overload

### Test Fixture Strategy

**Decision**: Pre-seeded YAML fixtures in `/frontend/tests/fixtures/`  
**Rationale**:
- Deterministic tests per Constitution Principle II
- Fast setup (no API calls during test initialization)
- Version-controlled for traceability
- Real data structure matches production format

### Test Isolation Strategy (CRITICAL)

**Decision**: Each E2E test runs in an isolated temporary directory  
**Rationale**:
- CLI commands (`spec-new`, `tasks`, `run`, etc.) modify the filesystem by creating/updating files
- Running tests against the real codebase would corrupt working specs, plans, and Git state
- Constitution Principle II mandates "Test Isolation: Each test MUST be independent; no shared mutable state between tests"
- Temporary directories provide complete isolation between test runs

**Implementation**:
- Playwright custom fixture creates temp directory via `mkdtemp()`
- Fresh Git repository initialized in temp folder
- Fixture files copied from `/frontend/tests/fixtures/sample-project/`
- Backend server started with `CKRV_PROJECT_ROOT` pointing to temp directory
- Temp directory cleaned up after each test via `rm -rf`

**Alternatives Considered**:
- Running against real codebase with Git stash/restore: Too fragile, race conditions possible
- Docker containers per test: Too slow for fast CI feedback requirement (60s target)
- In-memory filesystem mock: Wouldn't test real file operations

## Codebase Analysis

### Key Components Analyzed

| Component | Lines | Priority | Complexity | Notes |
|-----------|-------|----------|------------|-------|
| ExecutionRunner.tsx | 961 | P1 | High | Core execution UI, WebSocket handling |
| AgentManager.tsx | 1003 | P3 | Medium | Agent CRUD, OpenRouter integration |
| CommandPalette.tsx | 618 | P2 | Medium | Command execution, state management |
| SpecEditor.tsx | 670 | P2 | Medium | Spec viewing/editing |
| TaskEditor.tsx | 768 | P2 | Medium | Task management, status updates |
| PlanEditor.tsx | 640 | P2 | Medium | Plan visualization, DAG view |
| WorkflowPanel.tsx | 498 | P3 | Low | Pipeline visualization |
| LogTerminal.tsx | 104 | P2 | Low | XTerm.js wrapper |
| DiffViewer.tsx | 398 | P3 | Low | Diff display |
| StatusWidget.tsx | 350 | P3 | Low | Status indicators |

### Backend API Endpoints

| Endpoint | File | Method | Purpose |
|----------|------|--------|---------|
| `/api/execution/start` | execution.rs | POST | Start execution run |
| `/api/execution/stop` | execution.rs | POST | Stop execution run |
| `/api/execution/ws` | execution.rs | WS | Stream execution logs |
| `/api/specs` | specs.rs | GET | List specifications |
| `/api/specs/:name` | specs.rs | GET | Get spec details |
| `/api/tasks` | tasks.rs | GET | List tasks |
| `/api/tasks/:spec` | tasks.rs | GET | Get tasks for spec |
| `/api/plan/:spec` | plans.rs | GET | Get execution plan |
| `/api/agents` | agents.rs | GET | List agent configs |
| `/api/command/*` | commands.rs | POST | Execute CLI commands |

### Design System (CSS Custom Properties)

The following CSS custom properties are defined in `index.css` and should be used consistently:

```css
/* Backgrounds */
--bg-primary: #0a0a0b
--bg-secondary: #111113
--bg-tertiary: #18181b
--bg-elevated: #1f1f23
--bg-surface: #27272a

/* Borders */
--border-subtle: #27272a
--border-default: #3f3f46
--border-strong: #52525b

/* Text */
--text-primary: #fafafa
--text-secondary: #a1a1aa
--text-muted: #71717a

/* Accents */
--accent-cyan: #22d3ee
--accent-cyan-dim: rgba(34, 211, 238, 0.15)
--accent-green: #4ade80
--accent-green-dim: rgba(74, 222, 128, 0.15)
--accent-amber: #fbbf24
--accent-amber-dim: rgba(251, 191, 36, 0.15)
--accent-red: #f87171
--accent-red-dim: rgba(248, 113, 113, 0.15)
--accent-purple: #a78bfa
--accent-purple-dim: rgba(167, 139, 250, 0.15)
```

## Research Summary

All NEEDS CLARIFICATION items have been resolved:
1. ✅ Testing framework: Playwright
2. ✅ WebSocket throttling: requestAnimationFrame batching
3. ✅ Auto-retry pattern: Exponential backoff with countdown
4. ✅ Test fixtures: Pre-seeded YAML files
5. ✅ Design system: CSS custom properties documented
