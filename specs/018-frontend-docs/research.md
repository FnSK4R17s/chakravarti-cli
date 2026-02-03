# Research: Frontend Code Documentation

**Branch**: `018-frontend-docs` | **Date**: 2026-02-03

## Existing Pattern Analysis

### Similar Feature Search

**Feature**: Frontend code documentation  
**Search Commands**:
```bash
grep -r "@module" crates/ckrv-ui/frontend/src --include="*.tsx" -l
grep -r "\/\*\*" crates/ckrv-ui/frontend/src/components --include="*.tsx" | head -20
```

**Docs Consulted**:
- `crates/ckrv-ui/FRONTEND_CONVENTIONS.md` - Primary reference for documentation patterns
- `crates/docs/architecture.md` - System overview (ckrv-ui is the UI crate)

**Conventions Applied**:
- `FRONTEND_CONVENTIONS.md` - Defines all documentation patterns for this feature

### Current Documentation State

| Metric | Count | Notes |
|--------|-------|-------|
| Components with @module | 0 | None have module headers |
| Hooks with @module | 0 | None have module headers |
| Props with JSDoc | ~5 | Some have basic JSDoc |
| Files with section comments | 0 | No `// ===` separators found |
| Files >400 lines | 9 | Need section comments |
| Files 200-400 lines | 12 | Should have section comments |

### Implementation Locations

| Location | Files | Purpose |
|----------|-------|---------|
| components/ | 27 .tsx | React components requiring documentation |
| hooks/ | 12 .ts | Custom hooks requiring documentation |
| layouts/ | 1 .tsx | Dashboard layout |
| frontend/README.md | 1 | Project description (currently boilerplate) |

### CLI/UI Parity Check

N/A - Documentation feature only. The `/docs.frontend` workflow is the automation mechanism.

## Reference Patterns

### @module Header Pattern (from FRONTEND_CONVENTIONS.md)

```typescript
/**
 * @module ComponentName
 * @description
 * Brief description of what this module does and why it exists.
 * This should be 2-4 sentences explaining the "what" and "why".
 *
 * @context
 * Where this fits in the application architecture.
 * What triggers this component/hook to be used.
 *
 * @dependencies
 * - ParentComponent: renders this component when X happens
 * - useSpecStore: provides spec data
 * - WebSocket: receives real-time updates
 *
 * @example
 * // Most common usage pattern
 * <TaskCard task={task} onRetry={handleRetry} />
 */
```

### Props Interface Pattern (from FRONTEND_CONVENTIONS.md)

```typescript
/**
 * Props for the TaskCard component.
 *
 * @example
 * const props: TaskCardProps = {
 *   task: { id: 'task_1', title: 'Implement auth', status: 'running' },
 *   onRetry: () => executor.retry('task_1'),
 *   expanded: true
 * };
 */
interface TaskCardProps {
  /** The task to display. Must have id, title, and status at minimum. */
  task: Task;

  /**
   * Called when user clicks retry on a failed task.
   * Only shown when task.status === 'failed'.
   */
  onRetry?: () => void;

  /**
   * Whether to show expanded view with full logs.
   * @default false
   */
  expanded?: boolean;
}
```

### Section Comment Pattern (from FRONTEND_CONVENTIONS.md)

```typescript
// === STATE ===
const [isOpen, setIsOpen] = useState(false);

// === EFFECTS ===
useEffect(() => { ... }, []);

// === HANDLERS ===
const handleClick = () => { ... };

// === RENDER ===
return ( ... );
```

### Hook Documentation Pattern (FR-002a)

```typescript
/**
 * @module useSpec
 * @description
 * Manages spec state and operations for the spec editor.
 *
 * @context
 * Used by SpecEditor and related components to access spec data.
 *
 * @dependencies
 * - useLogStore: For logging spec operations
 * - WebSocket: For real-time spec updates
 *
 * @example
 * const { spec, updateSpec, isLoading } = useSpec('spec-123');
 *
 * @param specId - The unique identifier for the spec to load
 * @returns Object containing spec data, update functions, and loading state
 */
export function useSpec(specId: string) { ... }
```

## Decisions

### D1: @example is REQUIRED everywhere

**Decision**: @example blocks are mandatory in both @module headers and Props interfaces.

**Rationale**: AI agents benefit most from concrete usage examples. Without examples, agents must infer usage from context, which leads to errors.

**Alternatives Considered**: Making @example optional - rejected because it defeats the purpose of self-contained documentation.

### D2: Section comments use flexible format

**Decision**: Use `// === SECTION_NAME ===` pattern (flexible length).

**Rationale**: Easy to recognize, flexible enough for different editors. Exact character count (e.g., 60 chars) is too rigid.

**Alternatives Considered**: Fixed 60-char separators like `// ========...========` - rejected for being too prescriptive.

### D3: Naming conventions included in scope

**Decision**: Added FR-013 to require `handle*` for handlers and `is*/has*/should*/can*` for booleans.

**Rationale**: Naming is part of self-documenting code. Consistent naming reduces AI confusion.

**Alternatives Considered**: Exclude naming from docs feature - rejected because it's closely related to code readability.

### D4: 100% compliance required

**Decision**: Feature is incomplete until all files pass verification (SC-008).

**Rationale**: Partial documentation creates inconsistency. Either the codebase is documented or it isn't.

**Alternatives Considered**: 95% threshold - rejected because it creates ambiguity about which files can be skipped.

## Unknowns Resolved

| Unknown | Resolution |
|---------|-----------|
| What makes a valid @module header? | Must have all 5 sections: @module, @description, @context, @dependencies, @example |
| Are Props @example required? | Yes, per FR-003 |
| What about 200-400 line files? | SHOULD have section comments (FR-006a) |
| How to document hooks differently? | Add @param/@returns (FR-002a) |
| What about import ordering? | Out of scope for this feature |
| What about file naming conventions? | Deferred (existing practice) |

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Documentation becomes stale | Medium | Medium | Add to PR review checklist |
| Over-documentation (too verbose) | Low | Low | Follow 2-4 sentence guideline |
| Inconsistent application | Medium | High | 100% verification requirement (SC-008) |
| Break existing code | Very Low | High | Documentation-only changes (no logic) |
