---
description: Comprehensive frontend audit using all design skills. Detects theme drift, composition issues, and convention violations, then applies fixes.
---

# Frontend Overhaul

## User Input

```
$ARGUMENTS
```

**Optional flags:**
- `--component <name>` - Focus on a specific component (e.g., `ExecutionRunner`)
- `--audit-only` - Only report issues, don't apply fixes
- `--fix-all` - Apply all fixes without prompting
- `--theme <url>` - shadcn theme URL to install (e.g., `https://tweakcn.com/r/themes/darkmatter.json`)

By default, analyzes **all components** and **applies fixes**.

**Theme URL:** If theme issues are detected and no `--theme` flag is provided, you will be prompted:

> 🎨 Theme drift detected! Would you like to install a shadcn theme to fix consistency issues?
> 
> Provide a theme URL (e.g., from https://tweakcn.com or https://ui.shadcn.com/themes):
> 
> Example: `https://tweakcn.com/r/themes/darkmatter.json`

Popular theme sources:
- **tweakcn.com** - Community themes (darkmatter, neonpunk, etc.)
- **ui.shadcn.com/themes** - Official shadcn themes

## Goal

Perform a comprehensive frontend audit using all available design skills:
1. **FRONTEND_CONVENTIONS.md** - Project-specific conventions
2. **vercel-composition-patterns** - React composition best practices
3. **vercel-react-best-practices** - Performance optimization (57 rules)
4. **web-design-guidelines** - Vercel's Web Interface Guidelines

This workflow detects issues that cause:
- New components breaking
- Theme inconsistencies between components
- Performance issues causing visual glitches
- Maintainability problems

---

## Phase 0: Load All Skills & Conventions

Before any analysis, load all reference materials:

### Step 0.1: Load Project Conventions

// turbo
```bash
cat /apps/chakravarti-cli/crates/ckrv-ui/FRONTEND_CONVENTIONS.md
```

### Step 0.2: Load Composition Patterns Skill

// turbo
```bash
cat /apps/chakravarti-cli/.agent/skills/vercel-composition-patterns/SKILL.md
```

### Step 0.3: Load React Best Practices Skill

// turbo
```bash
cat /apps/chakravarti-cli/.agent/skills/vercel-react-best-practices/SKILL.md
```

### Step 0.4: Fetch Web Interface Guidelines

Fetch the latest guidelines from:
```
https://raw.githubusercontent.com/vercel-labs/web-interface-guidelines/main/command.md
```

Use WebFetch to retrieve fresh rules before auditing.

---

## Phase 1: Component Inventory

### Step 1.1: List All Components

// turbo
```bash
cd /apps/chakravarti-cli/crates/ckrv-ui/frontend/src/components && \
wc -l *.tsx 2>/dev/null | sort -rn
```

### Step 1.2: Categorize by Size

| Lines | Category | Priority |
|-------|----------|----------|
| > 600 | 🔴 Critical | Fix immediately |
| 400-600 | 🟠 Warning | Plan to refactor |
| 200-400 | ⚠️ Monitor | Document well |
| < 200 | ✅ Good | Maintain |

### Step 1.3: List All Hooks

// turbo
```bash
cd /apps/chakravarti-cli/crates/ckrv-ui/frontend/src/hooks && \
wc -l *.ts 2>/dev/null | sort -rn
```

---

## Phase 2: Theme Consistency Audit

**Goal:** Detect hardcoded colors/styles that cause theme drift.

### Step 2.1: Find Hardcoded Colors

// turbo
```bash
# Find inline color classes (should use design tokens)
grep -rn "text-\(red\|blue\|green\|gray\|yellow\|orange\)-[0-9]" \
  /apps/chakravarti-cli/crates/ckrv-ui/frontend/src/components/*.tsx \
  --include="*.tsx" 2>/dev/null | head -30
```

**Expected:** Colors should come from constants like `STATUS_COLORS`, not inline classes.

### Step 2.2: Find Missing Design Token Usage

// turbo
```bash
# Check if components import and use the design tokens
for file in /apps/chakravarti-cli/crates/ckrv-ui/frontend/src/components/*.tsx; do
  component=$(basename "$file" .tsx)
  
  # Check for inline styles (red flag)
  inline_styles=$(grep -c "style={{" "$file" 2>/dev/null || echo "0")
  
  # Check for cn() usage (good sign)
  cn_usage=$(grep -c "cn(" "$file" 2>/dev/null || echo "0")
  
  if [ "$inline_styles" -gt 0 ]; then
    echo "⚠️  $component: $inline_styles inline styles (prefer cn() utility)"
  fi
  
  if [ "$cn_usage" -eq 0 ]; then
    echo "❌ $component: No cn() usage - may have inconsistent styling"
  fi
done
```

### Step 2.3: Check shadcn/ui Component Usage

// turbo
```bash
# Verify components use shadcn/ui primitives
for file in /apps/chakravarti-cli/crates/ckrv-ui/frontend/src/components/*.tsx; do
  component=$(basename "$file" .tsx)
  
  # Check for raw HTML elements that should be shadcn/ui
  raw_buttons=$(grep -c "<button" "$file" 2>/dev/null || echo "0")
  raw_inputs=$(grep -c "<input" "$file" 2>/dev/null || echo "0")
  
  if [ "$raw_buttons" -gt 0 ]; then
    echo "⚠️  $component: $raw_buttons raw <button> elements (use shadcn/ui Button)"
  fi
  
  if [ "$raw_inputs" -gt 0 ]; then
    echo "⚠️  $component: $raw_inputs raw <input> elements (use shadcn/ui Input)"
  fi
done
```

### Step 2.4: Theme Installation Decision

**If theme issues were detected in Steps 2.1-2.3:**

1. **Check if `--theme` flag was provided:**
   - If yes, proceed to install the theme
   - If no, prompt the user:

> 🎨 **Theme drift detected!** Found hardcoded colors and inconsistent styling.
>
> Would you like to install a shadcn theme to establish a consistent design system?
>
> **Provide a theme URL** (or press Enter to skip):
> - tweakcn.com themes: `https://tweakcn.com/r/themes/<theme-name>.json`
> - shadcn themes: `https://ui.shadcn.com/themes`
>
> **Popular themes:**
> - `https://tweakcn.com/r/themes/darkmatter.json` - Dark, modern
> - `https://tweakcn.com/r/themes/neonpunk.json` - Vibrant cyberpunk
> - `https://tweakcn.com/r/themes/catppuccin.json` - Soft, pastel
> - `https://tweakcn.com/r/themes/rosepine.json` - Elegant rose tones

2. **If a theme URL is provided, install it:**

```bash
cd /apps/chakravarti-cli/crates/ckrv-ui/frontend && \
npx shadcn@latest add <THEME_URL>
```

**Example:**
```bash
cd /apps/chakravarti-cli/crates/ckrv-ui/frontend && \
npx shadcn@latest add https://tweakcn.com/r/themes/darkmatter.json
```

3. **After theme installation:**
   - The theme will update `globals.css` with CSS variables
   - Re-run Step 2.1 to verify hardcoded colors are now using theme tokens
   - Document the installed theme in `FRONTEND_CONVENTIONS.md`

### Step 2.5: Verify Theme Installation

// turbo
```bash
# Check if theme CSS variables are present
grep -c "hsl(var(--" /apps/chakravarti-cli/crates/ckrv-ui/frontend/src/index.css 2>/dev/null || echo "0"
```

If count > 10, theme is properly installed.

---

## Phase 3: Composition Patterns Audit

**Skill:** `vercel-composition-patterns`

### Step 3.1: Detect Boolean Prop Proliferation

// turbo
```bash
# Find components with many boolean props (anti-pattern)
for file in /apps/chakravarti-cli/crates/ckrv-ui/frontend/src/components/*.tsx; do
  component=$(basename "$file" .tsx)
  
  # Count boolean props in interface
  bool_props=$(grep -E "^\s+(is|has|should|can|show|hide|enable|disable)[A-Z][a-zA-Z]*\??\s*:" "$file" | wc -l)
  
  if [ "$bool_props" -gt 3 ]; then
    echo "🔴 $component: $bool_props boolean props (refactor to composition)"
    grep -E "^\s+(is|has|should|can|show|hide|enable|disable)[A-Z][a-zA-Z]*\??\s*:" "$file" | head -5 | sed 's/^/    /'
  fi
done
```

**Fix:** Create explicit variant components instead of boolean modes.

### Step 3.2: Check for Compound Component Patterns

// turbo
```bash
# Check if large components use compound patterns
for file in /apps/chakravarti-cli/crates/ckrv-ui/frontend/src/components/*.tsx; do
  component=$(basename "$file" .tsx)
  lines=$(wc -l < "$file" 2>/dev/null | tr -d ' ')
  
  # Only check large components
  if [ "$lines" -gt 400 ]; then
    # Check for context usage (good)
    has_context=$(grep -c "createContext\|useContext\|use(" "$file" 2>/dev/null || echo "0")
    
    # Check for compound export pattern (good)
    has_compound=$(grep -c "^const.*=.*{$\|Provider:\|Frame:\|Input:" "$file" 2>/dev/null || echo "0")
    
    if [ "$has_context" -eq 0 ] && [ "$has_compound" -eq 0 ]; then
      echo "⚠️  $component ($lines lines): Large component without compound pattern"
      echo "    Consider splitting into Compound Components (see vercel-composition-patterns)"
    fi
  fi
done
```

### Step 3.3: Check State Decoupling

// turbo
```bash
# Find UI components that directly call global state hooks
for file in /apps/chakravarti-cli/crates/ckrv-ui/frontend/src/components/*.tsx; do
  component=$(basename "$file" .tsx)
  
  # Check for direct global state usage in render functions
  direct_global=$(grep -E "useGlobal|useStore|zustand" "$file" 2>/dev/null | wc -l)
  
  if [ "$direct_global" -gt 2 ]; then
    echo "⚠️  $component: $direct_global direct global state calls"
    echo "    Consider wrapping in a Provider component (state-decouple-implementation)"
  fi
done
```

---

## Phase 4: React Best Practices Audit

**Skill:** `vercel-react-best-practices`

### Step 4.1: Check for Render Performance Issues

// turbo
```bash
# Find potential re-render issues
for file in /apps/chakravarti-cli/crates/ckrv-ui/frontend/src/components/*.tsx; do
  component=$(basename "$file" .tsx)
  
  # Check for inline object/array in JSX (causes re-renders)
  inline_objects=$(grep -E "={{.*}}" "$file" | grep -v "style={{" | wc -l)
  
  # Check for missing useCallback on handlers passed as props
  handlers_no_callback=$(grep -E "onClick={\([^)]*\)\s*=>" "$file" | wc -l)
  
  if [ "$inline_objects" -gt 3 ]; then
    echo "⚠️  $component: $inline_objects inline objects in JSX (rerender-memo-with-default-value)"
  fi
  
  if [ "$handlers_no_callback" -gt 2 ]; then
    echo "⚠️  $component: $handlers_no_callback inline arrow handlers (use useCallback)"
  fi
done
```

### Step 4.2: Check for Async Waterfalls

// turbo
```bash
# Find sequential awaits that should be parallel
for file in /apps/chakravarti-cli/crates/ckrv-ui/frontend/src/components/*.tsx \
            /apps/chakravarti-cli/crates/ckrv-ui/frontend/src/hooks/*.ts; do
  name=$(basename "$file")
  
  # Find multiple sequential awaits (should use Promise.all)
  sequential_awaits=$(grep -A1 "await " "$file" 2>/dev/null | grep -c "await" || echo "0")
  
  if [ "$sequential_awaits" -gt 2 ]; then
    echo "⚠️  $name: $sequential_awaits sequential awaits (async-parallel: use Promise.all)"
  fi
done
```

### Step 4.3: Check Bundle Size Patterns

// turbo
```bash
# Find heavy imports that should be dynamic
for file in /apps/chakravarti-cli/crates/ckrv-ui/frontend/src/components/*.tsx; do
  component=$(basename "$file" .tsx)
  
  # Check for heavy library imports
  heavy_imports=$(grep -E "^import.*from '(chart|monaco|ace-editor|lodash[^/]|moment)';" "$file" 2>/dev/null)
  
  if [ -n "$heavy_imports" ]; then
    echo "🔴 $component: Heavy import should use next/dynamic or lazy loading"
    echo "$heavy_imports" | sed 's/^/    /'
  fi
done
```

---

## Phase 5: Convention Compliance Audit

**Based on:** `FRONTEND_CONVENTIONS.md`

### Step 5.1: Import Order Check

// turbo
```bash
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

### Step 5.2: Naming Convention Check

// turbo
```bash
for file in /apps/chakravarti-cli/crates/ckrv-ui/frontend/src/components/*.tsx; do
  component=$(basename "$file" .tsx)
  
  # Check handler naming (should be handle*)
  bad_handlers=$(grep -E "const on[A-Z]|const click|const submit" "$file" 2>/dev/null | head -3)
  if [ -n "$bad_handlers" ]; then
    echo "⚠️  $component: Handlers should use 'handle' prefix"
  fi
  
  # Check boolean naming (should be is*/has*/should*/can*)
  bad_booleans=$(grep -E "const (loading|error|open|visible|active|disabled)\s*=" "$file" 2>/dev/null | \
                 grep -v "is\|has\|should\|can" | head -3)
  if [ -n "$bad_booleans" ]; then
    echo "⚠️  $component: Booleans should use is/has/should/can prefix"
  fi
done
```

### Step 5.3: Section Comments Check

// turbo
```bash
for file in /apps/chakravarti-cli/crates/ckrv-ui/frontend/src/components/*.tsx; do
  component=$(basename "$file" .tsx)
  lines=$(wc -l < "$file" 2>/dev/null | tr -d ' ')
  
  if [ "$lines" -gt 200 ]; then
    sections=$(grep -c "// ===" "$file" 2>/dev/null || echo "0")
    if [ "$sections" -lt 3 ]; then
      echo "⚠️  $component ($lines lines): Missing section comments (found $sections, want 3+)"
    fi
  fi
done
```

### Step 5.4: Module Header Check

// turbo
```bash
for file in /apps/chakravarti-cli/crates/ckrv-ui/frontend/src/components/*.tsx \
            /apps/chakravarti-cli/crates/ckrv-ui/frontend/src/hooks/*.ts; do
  name=$(basename "$file")
  
  if ! head -10 "$file" | grep -q "@module"; then
    echo "❌ $name: Missing @module header"
  fi
done
```

---

## Phase 6: Web Interface Guidelines Audit

**Skill:** `web-design-guidelines`

### Step 6.1: Fetch Latest Guidelines

Fetch from: `https://raw.githubusercontent.com/vercel-labs/web-interface-guidelines/main/command.md`

### Step 6.2: Apply Guidelines to Components

For each component, check against the fetched rules. Key areas:
- Accessibility (ARIA, keyboard navigation)
- Interaction patterns (hover states, focus indicators)
- Layout consistency (spacing, alignment)
- Typography (font sizes, line heights)

Output findings in `file:line` format as specified in the guidelines.

---

## Phase 7: Generate Comprehensive Report

Create a unified report at `crates/ckrv-ui/docs/overhaul-report.md`:

```markdown
# Frontend Overhaul Report

Generated: <CURRENT_DATE>
Commit: <CURRENT_COMMIT>

## Executive Summary

| Category | Issues | Fixed | Remaining |
|----------|--------|-------|-----------|
| Theme Consistency | X | Y | Z |
| Composition Patterns | X | Y | Z |
| React Best Practices | X | Y | Z |
| Convention Compliance | X | Y | Z |
| Web Guidelines | X | Y | Z |

## 🔴 Critical Issues (Fix Immediately)

### Components with Boolean Prop Proliferation
| Component | Bool Props | Recommended Fix |
|-----------|------------|-----------------|
| ... | ... | Create explicit variants |

### Large Components Needing Splitting
| Component | Lines | Recommended Split |
|-----------|-------|-------------------|
| ExecutionRunner | 1400 | See Phase 8.1 |

## 🟠 Theme Drift Issues

### Hardcoded Colors
| File | Line | Current | Should Use |
|------|------|---------|------------|
| ... | ... | text-red-500 | STATUS_COLORS.failed |

### Missing cn() Usage
| Component | Issue |
|-----------|-------|
| ... | No cn() utility - inconsistent styling |

## ⚠️ Convention Violations

### Import Order
| File | Issue |
|------|-------|
| ... | React should be first |

### Naming Conventions
| File | Line | Current | Should Be |
|------|------|---------|-----------|
| ... | ... | const loading | const isLoading |

## 📊 Component Health Matrix

| Component | Lines | Theme | Composition | Perf | Conventions | Overall |
|-----------|-------|-------|-------------|------|-------------|---------|
| ExecutionRunner | 1400 | ⚠️ | 🔴 | ⚠️ | ✅ | 🔴 |
| TaskCard | 180 | ✅ | ✅ | ✅ | ✅ | ✅ |
| ... | ... | ... | ... | ... | ... | ... |
```

---

## Phase 8: Apply Fixes (Unless --audit-only)

### Step 8.1: Split Large Components

For components over 600 lines:
1. Extract hooks into `use<ComponentName>.ts`
2. Extract subcomponents into separate files
3. Create a Provider if using shared state

**Target structure:**
```
ExecutionRunner/
├── index.tsx              # Main component (orchestration)
├── ExecutionRunner.tsx    # UI component
├── useExecutionState.ts   # State management hook
├── ExecutionControls.tsx  # Subcomponent
├── ExecutionBatchPanel.tsx
└── types.ts
```

### Step 8.2: Fix Theme Drift

**Step 8.2.1: Install Theme (if URL provided)**

If a theme URL was provided via `--theme` or during the prompt in Phase 2.4:

```bash
cd /apps/chakravarti-cli/crates/ckrv-ui/frontend && \
npx shadcn@latest add <THEME_URL>
```

**Step 8.2.2: Create Shared Theme Constants**

Create `src/lib/theme.ts` to centralize theme tokens:

```typescript
/**
 * @module theme
 * @description
 * Centralized theme tokens for consistent styling across components.
 * These map to CSS variables from the installed shadcn theme.
 * 
 * @see FRONTEND_CONVENTIONS.md for usage guidelines
 */

// Status colors using theme's semantic colors
export const STATUS_COLORS = {
  pending: 'text-muted-foreground',
  waiting: 'text-primary/70',
  running: 'text-primary',
  completed: 'text-green-500',  // or 'text-success' if theme has it
  failed: 'text-destructive',
} as const;

// Log colors for terminal/console output
export const LOG_COLORS = {
  error: 'text-destructive',
  success: 'text-green-500',
  warning: 'text-yellow-500',
  info: 'text-muted-foreground',
  batch: 'text-primary',
} as const;

// Background variants
export const BG_COLORS = {
  pending: 'bg-muted/50',
  running: 'bg-primary/10',
  completed: 'bg-green-500/10',
  failed: 'bg-destructive/10',
} as const;

// Border variants
export const BORDER_COLORS = {
  pending: 'border-muted',
  running: 'border-primary',
  completed: 'border-green-500',
  failed: 'border-destructive',
} as const;
```

**Step 8.2.3: Replace Hardcoded Colors**

For each hardcoded color found in Phase 2.1:
1. Import the theme constants: `import { STATUS_COLORS, LOG_COLORS } from '@/lib/theme';`
2. Replace inline color with constant
3. Use `cn()` utility for conditional classes

**Example fixes:**
```typescript
// Before
<span className="text-red-500">Error</span>
<div className="bg-blue-900/30 border-blue-500">Running</div>

// After
import { STATUS_COLORS, BG_COLORS, BORDER_COLORS } from '@/lib/theme';

<span className={STATUS_COLORS.failed}>Error</span>
<div className={cn(BG_COLORS.running, BORDER_COLORS.running)}>Running</div>
```

**Step 8.2.4: Document Theme in FRONTEND_CONVENTIONS.md**

Add installed theme info to conventions:

```markdown
## Installed Theme

- **Theme:** <theme-name>
- **Source:** <theme-url>
- **Installed:** <date>

The theme provides CSS variables in `globals.css`. Use `@/lib/theme.ts` 
for TypeScript constants that reference these variables.
```

### Step 8.3: Add Missing Module Headers

For each file missing `@module`:
1. Read the file to understand purpose
2. Add @module header with description, context, dependencies

### Step 8.4: Fix Naming Conventions

For each violation:
1. Rename variable/function to follow convention
2. Update all usages in the file
3. If exported, update imports in other files

### Step 8.5: Add Section Comments

For large components missing sections:
1. Identify logical sections (STATE, EFFECTS, HANDLERS, RENDER)
2. Add `// ===` separators before each section

---

## Phase 9: Verification

After applying fixes, verify by re-running audits:

// turbo
```bash
echo "=== Re-running Theme Audit ==="
# Theme check commands from Phase 2

echo "=== Re-running Composition Audit ==="
# Composition check commands from Phase 3

echo "=== Re-running Convention Audit ==="
# Convention check commands from Phase 5
```

**If any checks still fail, go back and fix.**

---

## Phase 10: Output Summary

```markdown
## Frontend Overhaul Complete

### Skills Applied
- ✅ FRONTEND_CONVENTIONS.md
- ✅ vercel-composition-patterns
- ✅ vercel-react-best-practices  
- ✅ web-design-guidelines

### Results

| Metric | Before | After |
|--------|--------|-------|
| Theme Consistency | 65% | 98% |
| Composition Score | 40% | 85% |
| Perf Best Practices | 70% | 95% |
| Convention Compliance | 55% | 100% |

### Files Modified
- X components updated
- Y hooks extracted
- Z convention fixes applied

### Reports Generated
- 📋 `crates/ckrv-ui/docs/overhaul-report.md`
- 📋 `crates/ckrv-ui/docs/drift-report.md`

### Recommended Follow-ups
1. Review compound component refactors
2. Add Storybook stories for new variants
3. Set up pre-commit hooks for convention checks
```

---

## Quick Reference: When to Use Each Skill

| Issue | Primary Skill | Rule to Read |
|-------|---------------|--------------|
| New component breaks | vercel-composition-patterns | architecture-compound-components |
| Different theme than existing | FRONTEND_CONVENTIONS.md | STATUS_COLORS section |
| Boolean props multiplying | vercel-composition-patterns | architecture-avoid-boolean-props |
| Component too large | FRONTEND_CONVENTIONS.md | Component Size Guidelines |
| Performance issues | vercel-react-best-practices | rerender-* rules |
| Accessibility issues | web-design-guidelines | Fetch and apply |

---

## Notes

- This workflow is **comprehensive** - run it before major releases or when onboarding new team members
- For quick checks, use `/docs.frontend` instead
- The `--audit-only` flag is useful for CI/CD pipelines
- All fixes are reversible - commit before running with `--fix-all`
