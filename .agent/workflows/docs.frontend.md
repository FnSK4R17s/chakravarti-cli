---
description: Analyze frontend health and generate component documentation for ckrv-ui.
---

# Frontend Documentation & Drift Detection

## User Input

```
$ARGUMENTS
```

**Optional:** `--component <name>` to focus on a specific component (e.g., `ExecutionRunner`).
By default, analyzes **all components**.

## Goal

Analyze frontend codebase health, detect documentation drift, apply missing documentation, and ensure code follows established conventions.

**Files maintained:**
- Component/hook JSDoc and module headers
- `crates/ckrv-ui/frontend/README.md` - Frontend project readme
- `crates/ckrv-ui/docs/api-reference.md` - Backend API documentation

## Frontend Location

```
crates/ckrv-ui/frontend/src/
```

---

## Phase 0: Load Conventions

Before any analysis, load the conventions file for reference:

```bash
cat /apps/chakravarti-cli/crates/ckrv-ui/FRONTEND_CONVENTIONS.md 2>/dev/null || echo "⚠️ FRONTEND_CONVENTIONS.md not found"
```

---

## Phase 1: Health Analysis

*(Existing health checks - kept for completeness)*

### Step 1.1: Component Size Analysis

```bash
cd /apps/chakravarti-cli/crates/ckrv-ui/frontend/src/components && \
wc -l *.tsx 2>/dev/null | sort -rn | head -30
```

| Lines | Rating | Action |
|-------|--------|--------|
| < 200 | ✅ Good | Maintain |
| 200-500 | ⚠️ Warning | Consider splitting |
| > 500 | 🔴 Critical | Must refactor |

### Step 1.2-1.8: Standard Health Checks

*(Run existing checks from original workflow)*

---

## Phase 2: Documentation Drift Detection 🆕

This phase compares what's documented against what's actually implemented.

### Step 2.1: Extract Documented Props vs Actual Props

For each component, compare the Props interface documentation against actual usage:

```bash
#!/bin/bash
# drift-props.sh - Detect Props documentation drift

COMPONENTS_DIR="/apps/chakravarti-cli/crates/ckrv-ui/frontend/src/components"

for file in "$COMPONENTS_DIR"/*.tsx; do
  component=$(basename "$file" .tsx)
  
  echo "=== $component ==="
  
  # Extract documented props (from interface JSDoc)
  echo "📝 Documented Props:"
  grep -A 50 "interface.*Props" "$file" | \
    grep -E "^\s*\/\*\*|^\s*\*|^\s*[a-zA-Z]+\??\s*:" | \
    head -30
  
  # Extract actual props used in component
  echo ""
  echo "💻 Actually Used Props:"
  # Find destructured props in function signature
  grep -E "^export (function|const)" "$file" | head -1
  
  echo ""
  echo "---"
done
```

### Step 2.2: Detect JSDoc Parameter Drift

Check if function parameters match JSDoc @param annotations:

```bash
#!/bin/bash
# drift-params.sh - Detect @param drift

HOOKS_DIR="/apps/chakravarti-cli/crates/ckrv-ui/frontend/src/hooks"

for file in "$HOOKS_DIR"/*.ts; do
  hook=$(basename "$file" .ts)
  
  # Find functions with JSDoc
  grep -B 20 "^export function\|^export const.*=" "$file" | \
  awk '
    /\/\*\*/ { in_jsdoc=1; jsdoc="" }
    in_jsdoc { jsdoc = jsdoc "\n" $0 }
    /\*\// { in_jsdoc=0 }
    /^export (function|const)/ {
      # Extract @param names from JSDoc
      n = split(jsdoc, lines, "\n")
      printf "📝 %s - Documented params: ", $0
      for (i=1; i<=n; i++) {
        if (match(lines[i], /@param\s+(\{[^}]+\}\s+)?([a-zA-Z_]+)/, arr)) {
          printf "%s ", arr[2]
        }
      }
      printf "\n"
    }
  '
done 2>/dev/null
```

### Step 2.3: Detect State Documentation Drift

Compare documented state variables against actual useState calls:

```bash
#!/bin/bash
# drift-state.sh - Detect state documentation drift

for file in /apps/chakravarti-cli/crates/ckrv-ui/frontend/src/components/*.tsx; do
  component=$(basename "$file" .tsx)
  
  # Count documented state (comments before useState)
  documented=$(grep -B 1 "useState" "$file" 2>/dev/null | grep -c "/\*\*\|//" 2>/dev/null | tr -d '\n')
  documented=${documented:-0}
  
  # Count actual useState calls
  actual=$(grep -c "useState" "$file" 2>/dev/null | tr -d '\n')
  actual=${actual:-0}
  
  if [ "$actual" -gt 0 ] 2>/dev/null; then
    coverage=$((documented * 100 / actual))
    if [ "$coverage" -lt 100 ]; then
      echo "⚠️  $component: $documented/$actual useState documented ($coverage%)"
    else
      echo "✅ $component: All $actual useState documented"
    fi
  fi
done
```

### Step 2.4: Detect Effect Documentation Drift

Check if useEffect hooks have explanatory comments:

```bash
#!/bin/bash
# drift-effects.sh - Detect effect documentation drift

for file in /apps/chakravarti-cli/crates/ckrv-ui/frontend/src/components/*.tsx; do
  component=$(basename "$file" .tsx)
  
  # Find useEffect without preceding comment
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
  
  total=$(grep -c "useEffect" "$file" 2>/dev/null | tr -d '\n')
  total=${total:-0}
  
  if [ "$total" -gt 0 ] 2>/dev/null && [ "$undocumented" -gt 0 ] 2>/dev/null; then
    echo "⚠️  $component: $undocumented/$total useEffect undocumented"
  fi
done
```

### Step 2.5: Module Header Check

Verify each file has a proper @module header:

```bash
#!/bin/bash
# drift-module-header.sh - Check for @module documentation

for file in /apps/chakravarti-cli/crates/ckrv-ui/frontend/src/components/*.tsx \
            /apps/chakravarti-cli/crates/ckrv-ui/frontend/src/hooks/*.ts; do
  name=$(basename "$file")
  
  # Check for @module tag in first 30 lines
  if head -30 "$file" | grep -q "@module"; then
    echo "✅ $name"
  else
    echo "❌ $name - Missing @module header"
  fi
done
```

### Step 2.6: Example Code Validation

Check if @example blocks exist and reference real types:

```bash
#!/bin/bash
# drift-examples.sh - Check @example coverage

for file in /apps/chakravarti-cli/crates/ckrv-ui/frontend/src/components/*.tsx; do
  component=$(basename "$file" .tsx)
  
  # Count exported functions/components
  exports=$(grep -c "^export " "$file" 2>/dev/null | tr -d '\n')
  exports=${exports:-0}
  
  # Count @example blocks
  examples=$(grep -c "@example" "$file" 2>/dev/null | tr -d '\n')
  examples=${examples:-0}
  
  if [ "$exports" -gt 0 ] 2>/dev/null; then
    if [ "$examples" -eq 0 ] 2>/dev/null; then
      echo "❌ $component: No @example blocks ($exports exports)"
    elif [ "$examples" -lt "$exports" ] 2>/dev/null; then
      echo "⚠️  $component: $examples examples for $exports exports"
    else
      echo "✅ $component: $examples examples"
    fi
  fi
done
```

---

## Phase 3: Convention Compliance Check 🆕

Check code against CONVENTIONS.md patterns:

### Step 3.1: Import Order Check

```bash
#!/bin/bash
# Check if imports follow convention order:
# 1. React 2. External 3. Internal components 4. Hooks 5. Utils 6. Relative 7. Types

for file in /apps/chakravarti-cli/crates/ckrv-ui/frontend/src/components/*.tsx; do
  component=$(basename "$file" .tsx)
  
  # Extract import section
  imports=$(sed -n '1,/^[^import]/p' "$file" | grep "^import")
  
  # Check if React is first
  first_import=$(echo "$imports" | head -1)
  if ! echo "$first_import" | grep -q "from 'react'"; then
    echo "⚠️  $component: React import should be first"
  fi
  
  # Check if type imports are last
  last_import=$(echo "$imports" | tail -1)
  type_imports=$(echo "$imports" | grep "import type")
  if [ -n "$type_imports" ]; then
    if ! echo "$last_import" | grep -q "import type"; then
      echo "⚠️  $component: Type imports should be last"
    fi
  fi
done
```

### Step 3.2: Naming Convention Check

```bash
#!/bin/bash
# Check naming conventions

for file in /apps/chakravarti-cli/crates/ckrv-ui/frontend/src/components/*.tsx; do
  component=$(basename "$file" .tsx)
  
  # Check handler naming (should be handle*)
  bad_handlers=$(grep -E "const on[A-Z]|const click|const submit" "$file" | head -5)
  if [ -n "$bad_handlers" ]; then
    echo "⚠️  $component: Handlers should use 'handle' prefix:"
    echo "$bad_handlers" | sed 's/^/    /'
  fi
  
  # Check boolean naming (should be is*/has*/should*/can*)
  bad_booleans=$(grep -E "const (loading|error|open|visible|active|disabled)\s*=" "$file" | \
                 grep -v "is\|has\|should\|can" | head -5)
  if [ -n "$bad_booleans" ]; then
    echo "⚠️  $component: Booleans should use is/has/should/can prefix:"
    echo "$bad_booleans" | sed 's/^/    /'
  fi
done
```

### Step 3.3: Section Comment Check

```bash
#!/bin/bash
# Check for section organization comments

for file in /apps/chakravarti-cli/crates/ckrv-ui/frontend/src/components/*.tsx; do
  component=$(basename "$file" .tsx)
  lines=$(wc -l < "$file" 2>/dev/null | tr -d ' \n')
  lines=${lines:-0}
  
  # Only check files > 200 lines
  if [ "$lines" -gt 200 ] 2>/dev/null; then
    sections=$(grep -c "// ===" "$file" 2>/dev/null | tr -d '\n')
    sections=${sections:-0}
    if [ "$sections" -lt 3 ] 2>/dev/null; then
      echo "⚠️  $component ($lines lines): Missing section comments (found $sections, want 3+)"
    fi
  fi
done
```

---

## Phase 4: Generate Drift Report 🆕

Compile all drift findings into a structured report:

```markdown
# Documentation Drift Report

Generated: <CURRENT_DATE>
Commit: <CURRENT_COMMIT>

## Summary

| Check | Passing | Failing | Coverage |
|-------|---------|---------|----------|
| Module Headers | X | Y | Z% |
| Props Documentation | X | Y | Z% |
| State Documentation | X | Y | Z% |
| Effect Documentation | X | Y | Z% |
| Example Blocks | X | Y | Z% |
| Import Order | X | Y | Z% |
| Naming Conventions | X | Y | Z% |

## 🔴 Critical Drift (Code Changed, Docs Stale)

| File | Issue | Line | Current Code | Documented As |
|------|-------|------|--------------|---------------|
| ExecutionRunner.tsx | Missing prop doc | 45 | `batchSize: number` | (undocumented) |
| ... | ... | ... | ... | ... |

## ⚠️ Missing Documentation

| File | Missing |
|------|---------|
| AgentManager.tsx | @module header, 5 useState, 2 useEffect |
| ... | ... |

## 📋 Convention Violations

| File | Violation | Line |
|------|-----------|------|
| TaskEditor.tsx | Handler should use 'handle' prefix | 142 |
| ... | ... | ... |

## Recommended Fixes

### Priority 1: Module Headers
Add @module headers to these files:
- [ ] ExecutionRunner.tsx
- [ ] AgentManager.tsx
- [ ] PlanEditor.tsx

### Priority 2: Props Documentation
- [ ] Add JSDoc to TaskEditorProps interface
- [ ] Document optional props with @default

### Priority 3: State Documentation
- [ ] Add comments before useState in ExecutionRunner
- [ ] Group related state with section comments
```

Save to: `crates/ckrv-ui/docs/drift-report.md`

---

## Phase 5: Apply Documentation (MANDATORY)

**This phase MUST edit files to add missing documentation. Do not just report - actually make the changes.**

### Step 5.1: Priority Order

Apply documentation in this order (highest impact first):

1. **Critical components** (>600 lines): ExecutionRunner, AgentManager, PlanEditor, TestRunner, TaskEditor, SpecEditor
2. **Warning components** (400-600 lines): TaskDetailModal, QAReviewer, BarebonesExecutor
3. **All hooks** in `src/hooks/`
4. **Remaining components**

### Step 5.2: Add Module Headers

**For EACH file missing @module header, edit the file and add this at the very top:**

```typescript
/**
 * @module ${ComponentName}
 * @description
 * ${INFER_DESCRIPTION_FROM_COMPONENT_NAME_AND_JSX}
 * 
 * Example: "Manages agent configuration and displays available AI coding agents."
 *
 * @context
 * ${DESCRIBE_WHERE_THIS_COMPONENT_IS_USED}
 *
 * @dependencies
 * ${LIST_KEY_IMPORTS_LIKE_HOOKS_AND_EXTERNAL_LIBS}
 */
```

**Action for each file:**
1. Read the file to understand what it does
2. Edit the file - insert the @module block at line 1 (before imports)
3. Move to next file

### Step 5.3: Add Props Documentation

**For EACH Props interface without JSDoc, edit the file and add documentation:**

Find patterns like:
```typescript
interface ExecutionRunnerProps {
  specId: string;
  onComplete?: () => void;
}
```

Replace with:
```typescript
/**
 * Props for ExecutionRunner.
 * Controls the execution of a spec through the agent orchestration pipeline.
 */
interface ExecutionRunnerProps {
  /** The ID of the spec to execute */
  specId: string;
  /** Callback fired when execution completes (success or failure) */
  onComplete?: () => void;
}
```

**Action for each interface:**
1. Find the interface in the file
2. Add JSDoc block above it
3. Add JSDoc comment above each prop

### Step 5.4: Add Section Comments to Large Components

**For components >400 lines, add section organization comments:**

```typescript
// ============================================================
// STATE
// ============================================================

const [isLoading, setIsLoading] = useState(false);
// ... more state

// ============================================================
// EFFECTS
// ============================================================

useEffect(() => {
  // ...
}, []);

// ============================================================
// HANDLERS
// ============================================================

const handleSubmit = () => {
  // ...
};

// ============================================================
// RENDER HELPERS
// ============================================================

const renderTaskList = () => {
  // ...
};

// ============================================================
// MAIN RENDER
// ============================================================

return (
  // ...
);
```

**Action:**
1. Identify logical sections in the component
2. Add `// ====...` separator comments before each section
3. Ensure at least: STATE, EFFECTS, HANDLERS, RENDER sections are marked

### Step 5.5: Document Complex State

**For components with >5 useState calls, add comments explaining each:**

```typescript
// ============================================================
// EXECUTION STATE
// ============================================================
// These states track the current execution lifecycle

/** Current execution status: idle, running, paused, or completed */
const [status, setStatus] = useState<ExecutionStatus>('idle');

/** Error from the most recent execution attempt, null if successful */
const [error, setError] = useState<Error | null>(null);

/** IDs of tasks currently being executed in parallel */
const [runningTaskIds, setRunningTaskIds] = useState<string[]>([]);
```

**Action:**
1. Group related useState calls together
2. Add section comment explaining the group
3. Add inline comment above each useState explaining its purpose

### Step 5.6: Verification

After applying documentation, verify by running:

```bash
# Check module headers were added
for file in /apps/chakravarti-cli/crates/ckrv-ui/frontend/src/components/*.tsx; do
  name=$(basename "$file")
  if head -10 "$file" | grep -q "@module"; then
    echo "✅ $name"
  else
    echo "❌ $name - STILL MISSING @module header"
  fi
done
```

**If any files still show ❌, go back and add the missing documentation.**

---

## Phase 5.5: Update Frontend Documentation Files

### Step 5.5.1: Update frontend/README.md

The frontend README should describe the project, not be Vite boilerplate.

// turbo
```bash
cat crates/ckrv-ui/frontend/README.md
```

Generate a proper README:

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

### Step 5.5.2: Update api-reference.md

// turbo
```bash
cat crates/ckrv-ui/docs/api-reference.md 2>/dev/null || echo "File needs to be created"
```

// turbo
```bash
# Extract API routes from Rust backend
grep -r "web::" crates/ckrv-ui/src/api/*.rs | grep -E "get|post|put|delete" | head -30
```

Generate/update api-reference.md from the Rust API handlers:

```markdown
# ckrv-ui API Reference

## REST Endpoints

### Agents

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/agents` | List all configured agents |
| POST | `/api/agents` | Add a new agent |
| PUT | `/api/agents/:id` | Update agent configuration |
| DELETE | `/api/agents/:id` | Remove an agent |

### Executions

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/executions` | List execution history |
| POST | `/api/executions` | Start new execution |
| GET | `/api/executions/:id` | Get execution details |

### Specs

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/specs` | List all specs |
| GET | `/api/specs/:name` | Get spec content |

## WebSocket

Connect to `/ws` for real-time updates.

### Message Types

| Type | Direction | Payload |
|------|-----------|---------|
| `execution_started` | Server → Client | `{ id, spec, agents }` |
| `batch_update` | Server → Client | `{ batch_id, status, logs }` |
| `execution_complete` | Server → Client | `{ id, result }` |
```

---

## Phase 6: Output Summary

```markdown
## Frontend Documentation Audit Complete

### Health Report
📋 `crates/ckrv-ui/docs/frontend-health-report.md`

### Drift Report  
📋 `crates/ckrv-ui/docs/drift-report.md`

### Statistics

| Metric | Before | After |
|--------|--------|------------------|
| Module Headers | 10/27 (37%) | 27/27 (100%) |
| Props Documented | 17/27 (63%) | 27/27 (100%) |
| State Documented | 45/120 (38%) | 120/120 (100%) |
| Convention Compliance | 60% | 95% |

### Files Modified
- ExecutionRunner.tsx (+45 lines documentation)
- AgentManager.tsx (+32 lines documentation)
- ...

### Next Steps
1. Review generated documentation for accuracy
2. Run `--drift` weekly to catch new drift
3. Add pre-commit hook to enforce conventions
```

---
---

## Notes

- **This workflow APPLIES documentation** - Phase 5 actually edits files to add missing docs
- **Drift detection** finds discrepancies between what's documented and what exists
- **Convention compliance** ensures new code follows patterns that make LLM editing easier
- **Review after running** - generated docs need human review for accuracy
- Run this workflow regularly (weekly or in CI) to catch documentation rot early
- The goal is making every file **self-contained context** for LLMs

### What This Workflow Modifies

| File Type | What Gets Added |
|-----------|-----------------|
| `*.tsx` components | @module headers, Props JSDoc, section comments, state comments |
| `*.ts` hooks | @module headers, @returns documentation |
| `frontend/README.md` | Complete project documentation (replaces Vite boilerplate) |
| `docs/drift-report.md` | Generated analysis report |

---

## Next Workflow

Read the **docs-order** skill to determine what workflow to run next based on what was changed.
