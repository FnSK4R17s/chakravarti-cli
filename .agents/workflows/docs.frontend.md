---
description: Analyze frontend health and apply component documentation for ckrv-ui.
---

# Frontend Documentation & Convention Application

## User Input

```
$ARGUMENTS
```

**Optional:** `--component <name>` to focus on a specific component (e.g., `ExecutionRunner`).
By default, analyzes **all components**.

## Goal

Detect missing frontend documentation and apply it in a single pass. No report files are generated — issues are fixed inline and a summary is printed at the end.

**This workflow:**
- ✅ Adds `@module` headers to components and hooks
- ✅ Adds JSDoc to Props interfaces
- ✅ Adds section separators (`// ===...`)
- ✅ Adds state and effect documentation
- ✅ Updates `crates/ckrv-ui/frontend/README.md` and `crates/ckrv-ui/docs/api-reference.md`
- ⚠️ Warns about unfixable issues (component too large, needs splitting)
- ❌ Does NOT modify component logic
- ❌ Does NOT generate report files

## Frontend Location

```
crates/ckrv-ui/frontend/src/
```

---

## Phase 0: Load Conventions

Before any work, load the conventions file for reference:

```bash
cat /apps/chakravarti-cli/crates/ckrv-ui/FRONTEND_CONVENTIONS.md 2>/dev/null || echo "⚠️ FRONTEND_CONVENTIONS.md not found"
```

---

## Phase 1: Discover Components

### Step 1.1: List All Components

<!-- turbo -->
```bash
find /apps/chakravarti-cli/crates/ckrv-ui/frontend/src/components -name "*.tsx" 2>/dev/null | head -50
```

### Step 1.2: List All Hooks

<!-- turbo -->
```bash
find /apps/chakravarti-cli/crates/ckrv-ui/frontend/src/hooks -name "*.ts" 2>/dev/null | head -50
```

If `--component <name>` was specified, filter to just that component.

### Step 1.3: Priority Order

Process files in this order (highest impact first):
1. **Large components** (>400 lines)
2. **All hooks**
3. **Remaining components**

---

## Phase 2: Detect & Fix (Per File)

**For each file, detect issues and fix them immediately.**

### Step 2.1: Add Missing Module Headers

<!-- turbo -->
```bash
for file in /apps/chakravarti-cli/crates/ckrv-ui/frontend/src/components/*.tsx \
            /apps/chakravarti-cli/crates/ckrv-ui/frontend/src/hooks/*.ts; do
  name=$(basename "$file")
  if ! head -30 "$file" | grep -q "@module"; then
    echo "FIX $name - Missing @module header"
  fi
done
```

**For each file that says FIX:**
1. Read the file to understand what it does
2. **Edit the file** — insert `@module` block at line 1 (before imports):

```typescript
/**
 * @module ComponentName
 * @description
 * <Infer description from component name and JSX content>
 *
 * @context
 * <Describe where this component is used>
 *
 * @dependencies
 * <List key imports like hooks and external libs>
 */
```

### Step 2.2: Add Missing Props Documentation

For each component with an undocumented Props interface:

<!-- turbo -->
```bash
for file in /apps/chakravarti-cli/crates/ckrv-ui/frontend/src/components/*.tsx; do
  name=$(basename "$file" .tsx)
  
  # Check if Props interface exists but lacks JSDoc
  if grep -q "interface.*Props" "$file"; then
    if ! grep -B1 "interface.*Props" "$file" | grep -q "/\*\*"; then
      echo "FIX $name - Props interface missing JSDoc"
    fi
  fi
done
```

**For each file that says FIX:**
1. Find the Props interface in the file
2. **Edit the file** — add JSDoc block above it and above each prop:

```typescript
/**
 * Props for ComponentName.
 * <Description inferred from component purpose>
 */
interface ComponentNameProps {
  /** <Description of this prop> */
  propName: string;
  /** <Description> @default <value if optional> */
  optionalProp?: boolean;
}
```

### Step 2.3: Add Missing State Documentation

For components with undocumented `useState` calls:

<!-- turbo -->
```bash
for file in /apps/chakravarti-cli/crates/ckrv-ui/frontend/src/components/*.tsx; do
  name=$(basename "$file" .tsx)
  
  documented=$(grep -B 1 "useState" "$file" 2>/dev/null | grep -c "/\*\*\|//" 2>/dev/null | tr -d '\n')
  documented=${documented:-0}
  actual=$(grep -c "useState" "$file" 2>/dev/null | tr -d '\n')
  actual=${actual:-0}
  
  if [ "$actual" -gt 0 ] 2>/dev/null; then
    undoc=$((actual - documented))
    if [ "$undoc" -gt 0 ] 2>/dev/null; then
      echo "FIX $name: $undoc/$actual useState undocumented"
    fi
  fi
done
```

**For each file that says FIX:**
1. Find the undocumented `useState` calls
2. **Edit the file** — add a comment above each:

```typescript
/** Current execution status: idle, running, paused, or completed */
const [status, setStatus] = useState<ExecutionStatus>('idle');
```

For components with 5+ useState calls, group related state together:

```typescript
// ============================================================
// EXECUTION STATE
// ============================================================

/** Current execution status */
const [status, setStatus] = useState('idle');
/** Error from most recent attempt */
const [error, setError] = useState<Error | null>(null);
```

### Step 2.4: Add Missing Effect Documentation

For components with undocumented `useEffect` calls:

<!-- turbo -->
```bash
for file in /apps/chakravarti-cli/crates/ckrv-ui/frontend/src/components/*.tsx; do
  name=$(basename "$file" .tsx)
  
  undocumented=$(awk '
    /useEffect/ {
      if (prev !~ /\/\*\*|\/\//) {
        count++
      }
    }
    { prev = $0 }
    END { print count+0 }
  ' "$file" 2>/dev/null | tr -d '\n')
  undocumented=${undocumented:-0}
  
  if [ "$undocumented" -gt 0 ] 2>/dev/null; then
    echo "FIX $name: $undocumented useEffect undocumented"
  fi
done
```

**For each file that says FIX:**
1. Find the undocumented `useEffect` calls
2. **Edit the file** — add a comment above each:

```typescript
/** Fetch task data when specId changes */
useEffect(() => {
  // ...
}, [specId]);
```

### Step 2.5: Add Missing Section Separators

For components > 200 lines without proper section organization:

<!-- turbo -->
```bash
for file in /apps/chakravarti-cli/crates/ckrv-ui/frontend/src/components/*.tsx; do
  name=$(basename "$file" .tsx)
  lines=$(wc -l < "$file" 2>/dev/null | tr -d ' \n')
  lines=${lines:-0}
  
  if [ "$lines" -gt 200 ] 2>/dev/null; then
    sections=$(grep -c "// ===" "$file" 2>/dev/null | tr -d '\n')
    sections=${sections:-0}
    if [ "$sections" -lt 3 ] 2>/dev/null; then
      echo "FIX $name ($lines lines): needs section separators (found $sections, want 3+)"
    fi
  fi
done
```

**For each file that says FIX:**
1. Read the file to identify logical sections
2. **Edit the file** — add `// ===` separator comments:

```typescript
// ============================================================
// STATE
// ============================================================

// ============================================================
// EFFECTS
// ============================================================

// ============================================================
// HANDLERS
// ============================================================

// ============================================================
// RENDER HELPERS
// ============================================================

// ============================================================
// MAIN RENDER
// ============================================================
```

Ensure at least STATE, EFFECTS, HANDLERS, RENDER sections are marked.

### Step 2.6: Warn About Large Components (Cannot Auto-Fix)

<!-- turbo -->
```bash
for file in /apps/chakravarti-cli/crates/ckrv-ui/frontend/src/components/*.tsx; do
  lines=$(wc -l < "$file" 2>/dev/null | tr -d ' \n')
  lines=${lines:-0}
  name=$(basename "$file" .tsx)
  if [ "$lines" -gt 500 ] 2>/dev/null; then
    echo "⚠️ WARN $name ($lines lines): consider splitting"
  fi
done
```

These are logged in the summary but **not fixed** — splitting components requires logic changes.

---

## Phase 3: Update Documentation Files

### Step 3.1: Update frontend/README.md

<!-- turbo -->
```bash
cat /apps/chakravarti-cli/crates/ckrv-ui/frontend/README.md
```

If the README is Vite boilerplate or outdated, **replace it** with a proper project README:

```markdown
# ckrv-ui Frontend

React frontend for the Chakravarti CLI web dashboard.

## Tech Stack

- **React 18** with TypeScript
- **Vite** for bundling
- **shadcn/ui** components
- **TanStack Query** for data fetching
- **WebSocket** for real-time updates

## Development

```bash
cd crates/ckrv-ui/frontend
pnpm install
pnpm dev
```

The dev server runs on http://localhost:5173 and proxies API requests to the Rust backend on :3000.

## Project Structure

```
src/
├── components/     # React components
├── hooks/          # Custom React hooks
├── lib/            # Utilities and API client
├── pages/          # Page components
└── App.tsx         # Main application
```

## Key Components

<list major components from src/components/>

## API Integration

The frontend communicates with the Rust backend via:
- REST API (`/api/*`) for CRUD operations
- WebSocket (`/ws`) for real-time execution updates

See `crates/ckrv-ui/docs/api-reference.md` for endpoint documentation.
```

### Step 3.2: Update api-reference.md

<!-- turbo -->
```bash
cat /apps/chakravarti-cli/crates/ckrv-ui/docs/api-reference.md 2>/dev/null || echo "File needs to be created"
```

<!-- turbo -->
```bash
# Extract API routes from Rust backend
grep -r "web::\|.route(" /apps/chakravarti-cli/crates/ckrv-ui/src/api/*.rs /apps/chakravarti-cli/crates/ckrv-transport/src/**/*.rs 2>/dev/null | grep -E "get|post|put|delete" | head -30
```

Generate/update `api-reference.md` from the Rust API handlers with endpoint tables for each resource group (Agents, Executions, Specs, etc.) and WebSocket message types.

---

## Phase 4: Post-Edit Verification

**After all edits are complete, re-check every modified file to confirm the applied documentation follows conventions.** This catches mistakes — especially from smaller models that may generate wrong formats.

### Step 4.1: Verify Module Header Format

<!-- turbo -->
```bash
for file in /apps/chakravarti-cli/crates/ckrv-ui/frontend/src/components/*.tsx \
            /apps/chakravarti-cli/crates/ckrv-ui/frontend/src/hooks/*.ts; do
  name=$(basename "$file")
  
  if head -30 "$file" | grep -q "@module"; then
    # Check required fields: @module, @description, @context, @dependencies
    missing=""
    head -30 "$file" | grep -q "@description" || missing="$missing @description"
    head -30 "$file" | grep -q "@context"     || missing="$missing @context"
    head -30 "$file" | grep -q "@dependencies" || missing="$missing @dependencies"
    
    if [ -n "$missing" ]; then
      echo "REFIX $name - @module header missing fields:$missing"
    fi
  fi
done
```

**Convention check:** Every `@module` header must include `@description`, `@context`, and `@dependencies` fields. If any say REFIX, edit the file to add the missing fields.

### Step 4.2: Verify Props JSDoc Format

<!-- turbo -->
```bash
for file in /apps/chakravarti-cli/crates/ckrv-ui/frontend/src/components/*.tsx; do
  name=$(basename "$file" .tsx)
  
  if grep -q "interface.*Props" "$file"; then
    # Check that Props interface has JSDoc above it
    if ! grep -B1 "interface.*Props" "$file" | grep -q "\*/"; then
      echo "REFIX $name - Props interface missing JSDoc block"
      continue
    fi
    
    # Check that individual props have /** */ comments
    prop_count=$(sed -n '/interface.*Props/,/^}/p' "$file" | grep -cE "^\s+\w+\??\s*:" 2>/dev/null || echo 0)
    doc_props=$(sed -n '/interface.*Props/,/^}/p' "$file" | grep -c "/\*\*" 2>/dev/null || echo 0)
    
    if [ "$prop_count" -gt 0 ] && [ "$doc_props" -eq 0 ]; then
      echo "REFIX $name - Props have interface JSDoc but individual props need /** */ comments"
    fi
  fi
done
```

**Convention check:** Each Props interface needs a JSDoc block above it AND each individual prop should have a `/** description */` comment. If any say REFIX, edit to add missing docs.

### Step 4.3: Verify Section Separator Format

<!-- turbo -->
```bash
for file in /apps/chakravarti-cli/crates/ckrv-ui/frontend/src/components/*.tsx; do
  name=$(basename "$file" .tsx)
  lines=$(wc -l < "$file" 2>/dev/null | tr -d ' \n')
  lines=${lines:-0}
  
  if [ "$lines" -gt 200 ] 2>/dev/null; then
    # Check separator format: must be exactly "// ============..." (60 =)
    bad_separators=$(grep "// ===" "$file" | grep -vc "// ============================================================" 2>/dev/null || echo 0)
    if [ "$bad_separators" -gt 0 ] 2>/dev/null; then
      echo "REFIX $name - $bad_separators section separators have wrong format (need 60 = characters)"
    fi
    
    # Check separator labels are UPPERCASE
    lowercase_labels=$(grep "// ============" "$file" -A1 | grep "^// " | grep -vc "^// [A-Z ]" 2>/dev/null || echo 0)
    if [ "$lowercase_labels" -gt 0 ] 2>/dev/null; then
      echo "REFIX $name - Section separator labels must be UPPERCASE"
    fi
  fi
done
```

**Convention check:** Separators must use exactly 60 `=` characters and labels must be UPPERCASE (e.g., `// STATE`, not `// state`). If any say REFIX, edit to correct.

### Step 4.4: Verify State/Effect Comments

<!-- turbo -->
```bash
for file in /apps/chakravarti-cli/crates/ckrv-ui/frontend/src/components/*.tsx; do
  name=$(basename "$file" .tsx)
  
  # Verify useState comments use /** */ format (not just //)
  bad_state_comments=$(grep -B1 "useState" "$file" | grep "^\s*//" | grep -vc "/\*\*" 2>/dev/null || echo 0)
  if [ "$bad_state_comments" -gt 0 ] 2>/dev/null; then
    echo "REFIX $name - $bad_state_comments useState comments use // instead of /** */ format"
  fi
done
```

**Convention check:** State documentation must use `/** description */` JSDoc format, not bare `//` comments. If any say REFIX, convert `//` to `/** */`.

### Step 4.5: TypeScript Check

<!-- turbo -->
```bash
cd /apps/chakravarti-cli/crates/ckrv-ui/frontend && npx tsc --noEmit 2>&1 | tail -10
```

Confirm no type errors were introduced by documentation changes. If errors appear, fix them before proceeding.

---

## Phase 5: Output Summary

After processing all files, provide a combined summary **in the conversation** (no file generated):

```markdown
## Frontend Documentation Applied

### Files Processed
- Components: 27
- Hooks: 8
- Documentation files: 2

### Changes Made

| Fix Type | Count | Files |
|----------|:-----:|-------|
| @module headers added | 5 | AgentManager, PlanEditor, ... |
| Props JSDoc added | 3 | TaskEditor, SpecEditor, ... |
| State documented | 12 | ExecutionRunner (5), AgentManager (4), ... |
| Effects documented | 4 | ExecutionRunner (2), TestRunner (2) |
| Section separators added | 3 | ExecutionRunner, AgentManager, PlanEditor |

### Files Modified
- `ExecutionRunner.tsx` - Added @module, 5 useState docs, 2 useEffect docs, sections
- `AgentManager.tsx` - Added @module, Props JSDoc, 4 useState docs
- `crates/ckrv-ui/frontend/README.md` - Replaced Vite boilerplate with project docs
- ...

### Post-Edit Verification
- ✅ @module header format: all have @description, @context, @dependencies
- ✅ Props JSDoc format: all interfaces and props documented
- ✅ Section separator format: all correct (60 =, UPPERCASE labels)
- ✅ State/effect comment format: all use /** */ JSDoc
- ✅ `tsc --noEmit`: no type errors

### ⚠️ Warnings (Manual Action Needed)
- `ExecutionRunner.tsx` (680 lines) - Consider splitting
- `AgentManager.tsx` (550 lines) - Consider splitting
```

---

## Notes

- **Detect-and-fix in one pass**: No separate analysis phase — issues are fixed as they're found
- **No report files**: Summary is printed in conversation, not saved to `drift-report.md` or `health-report.md`
- **Documentation only**: Adds comments, JSDoc, and section separators — never changes component logic
- **Warnings for unfixable issues**: Large components that need splitting are warned about but not touched
- **Convention compliance**: Follows patterns from `FRONTEND_CONVENTIONS.md`
- **Idempotent**: Safe to run multiple times — already-documented items are skipped

---

## Next Workflow

Read the **docs-order** skill to determine what workflow to run next based on what was changed.
