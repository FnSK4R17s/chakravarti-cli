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
  documented=$(grep -B 1 "useState" "$file" | grep -c "\/\*\*\|\/\/")
  
  # Count actual useState calls
  actual=$(grep -c "useState" "$file")
  
  if [ "$actual" -gt 0 ]; then
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
        print NR": "$0
      }
    }
    { prev = $0 }
    END { print "TOTAL:"count }
  ' "$file" | tail -1 | cut -d: -f2)
  
  total=$(grep -c "useEffect" "$file")
  
  if [ "$total" -gt 0 ] && [ "${undocumented:-0}" -gt 0 ]; then
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
  exports=$(grep -c "^export " "$file")
  
  # Count @example blocks
  examples=$(grep -c "@example" "$file")
  
  if [ "$exports" -gt 0 ]; then
    if [ "$examples" -eq 0 ]; then
      echo "❌ $component: No @example blocks ($exports exports)"
    elif [ "$examples" -lt "$exports" ]; then
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
  lines=$(wc -l < "$file")
  
  # Only check files > 200 lines
  if [ "$lines" -gt 200 ]; then
    sections=$(grep -c "// ===" "$file")
    if [ "$sections" -lt 3 ]; then
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

## Phase 5: Apply Documentation

### Step 5.1: Generate Module Headers

For files missing @module headers, generate them:

```typescript
/**
 * @module ${ComponentName}
 * @description
 * ${INFERRED_FROM_CODE}
 *
 * @context
 * ${INFERRED_FROM_IMPORTS_AND_USAGE}
 *
 * @dependencies
 * ${EXTRACTED_FROM_IMPORTS}
 */
```

### Step 5.2: Generate Props Documentation

For undocumented Props interfaces:

```typescript
/**
 * Props for ${ComponentName}.
 * ${INFERRED_PURPOSE}
 */
interface ${ComponentName}Props {
  /** ${INFERRED_FROM_USAGE_OR_NAME} */
  ${propName}: ${propType};
}
```

### Step 5.3: Generate State Comments

For undocumented useState:

```typescript
/** ${INFERRED_FROM_VARIABLE_NAME_AND_USAGE} */
const [${stateName}, set${StateName}] = useState<${Type}>(${initialValue});
```

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

## Automation: Pre-Commit Hook

Add to `.husky/pre-commit`:

```bash
#!/bin/bash

# Quick drift check on staged files
staged_tsx=$(git diff --cached --name-only | grep "\.tsx$")

for file in $staged_tsx; do
  # Check for @module header
  if ! head -30 "$file" | grep -q "@module"; then
    echo "❌ $file: Missing @module header"
    echo "   Run: /docs.frontend --component $(basename $file .tsx)"
    exit 1
  fi
  
  # Check Props interface has JSDoc
  if grep -q "interface.*Props" "$file"; then
    if ! grep -B 5 "interface.*Props" "$file" | grep -q "/\*\*"; then
      echo "❌ $file: Props interface missing JSDoc"
      exit 1
    fi
  fi
done

echo "✅ Documentation checks passed"
```

---

## Notes

- **Drift detection focuses on discrepancies** between what's documented and what exists
- **Convention compliance** ensures new code follows patterns that make LLM editing easier
- **Generated docs are stubs** - need human review for accuracy
- Run this workflow regularly (weekly or in CI) to catch documentation rot early
- The goal is making every file **self-contained context** for LLMs

---

## Next Workflow

Read the **docs-order** skill to determine what workflow to run next based on what was changed.
