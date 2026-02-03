# Quickstart: Frontend Code Documentation

## Prerequisites

- Access to `/apps/chakravarti-cli/crates/ckrv-ui/frontend/src/`
- FRONTEND_CONVENTIONS.md open for reference
- `/docs.frontend` workflow available

## Quick Reference

### Adding @module Header

Copy this template to the TOP of any component file (before imports):

```typescript
/**
 * @module [ComponentName]
 * @description
 * [2-4 sentences: What does this component do? Why does it exist?]
 *
 * @context
 * [Where is this used? What parent renders it? When is it shown?]
 *
 * @dependencies
 * - [ParentComponent]: [why it depends on this]
 * - [useHookName]: [what data it provides]
 * - [ExternalLib]: [what functionality it enables]
 *
 * @example
 * // Most common usage pattern
 * <[ComponentName] prop1={value} prop2={value} />
 */

// ============================================================
// IMPORTS
// ============================================================
import ...
```

### Adding Props JSDoc

Add before every Props interface:

```typescript
/**
 * Props for the [ComponentName] component.
 *
 * @example
 * const props: [ComponentName]Props = {
 *   requiredProp: value,
 *   optionalProp: value,
 * };
 */
interface [ComponentName]Props {
  /** Description of this prop. */
  requiredProp: Type;
  
  /**
   * Description of optional prop.
   * @default defaultValue
   */
  optionalProp?: Type;
}
```

### Adding Section Comments

For files >400 lines, add these separators:

```typescript
// === STATE ===
const [value, setValue] = useState(initialValue);

// === EFFECTS ===
useEffect(() => {
  // Effect logic
}, [dependencies]);

// === HANDLERS ===
const handleClick = useCallback(() => {
  // Handler logic
}, [dependencies]);

// === RENDER ===
return (
  <div>...</div>
);
```

### Hook Documentation

For hooks, add @param and @returns:

```typescript
/**
 * @module useCustomHook
 * @description
 * [What this hook provides and why it exists]
 *
 * @context
 * [Which components use this hook and when]
 *
 * @dependencies
 * - [Other hooks or stores it depends on]
 *
 * @example
 * const { data, isLoading, error } = useCustomHook('param');
 *
 * @param paramName - Description of the parameter
 * @returns Object containing { data, isLoading, error }
 */
export function useCustomHook(paramName: string) {
  // ...
}
```

## Implementation Order

1. **Phase 1**: Critical components (9 files >400 lines)
   - ExecutionRunner.tsx, AgentManager.tsx, PlanEditor.tsx, etc.

2. **Phase 2**: Medium components (12 files 200-400 lines)
   - WorkflowPanel.tsx, LogViewer.tsx, DiffViewer.tsx, etc.

3. **Phase 3**: Small components (6 files <200 lines)
   - BatchLogCarousel.tsx, ErrorBoundary.tsx, etc.

4. **Phase 4**: Hooks (12 files)
   - useSpec.ts, useLogStore.ts, etc.

5. **Phase 5**: Layout & README
   - Dashboard.tsx, frontend/README.md

6. **Phase 6**: Verification
   - Run `/docs.frontend` to verify 100% compliance

## Verification Commands

```bash
# Check for @module headers
grep -l "@module" crates/ckrv-ui/frontend/src/components/*.tsx | wc -l

# Find files missing @module
for f in crates/ckrv-ui/frontend/src/components/*.tsx; do
  grep -q "@module" "$f" || echo "Missing: $(basename $f)"
done

# Find large files needing section comments
wc -l crates/ckrv-ui/frontend/src/components/*.tsx | awk '$1 > 400'

# Check for section comments
grep -c "// ===" crates/ckrv-ui/frontend/src/components/ExecutionRunner.tsx
```

## Common Mistakes to Avoid

1. ❌ Putting @module AFTER imports (must be FIRST)
2. ❌ Missing @example in @module header
3. ❌ Missing @example in Props interface
4. ❌ Forgetting @default for optional props
5. ❌ Missing @param/@returns for hooks
6. ❌ Using `handle` without `handle*` prefix pattern
7. ❌ Boolean props without `is*/has*/should*/can*` prefix
