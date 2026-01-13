# Research: shadcn/ui Migration

**Feature**: 009-shadcn-ui-migration  
**Date**: 2026-01-11

## Research Summary

This document captures research findings for migrating the Chakravarti UI frontend from custom Tailwind components to shadcn/ui.

---

## Decision 1: Tailwind CSS v4 Compatibility

### Question
Is shadcn/ui compatible with Tailwind CSS v4 (currently v4.1.18)?

### Decision
**Yes, shadcn/ui is fully compatible with Tailwind CSS v4**

### Rationale
- All shadcn/ui components have been updated to support Tailwind v4
- shadcn/ui CLI can initialize projects with Tailwind v4 directly
- The main differences involve CSS-first configuration via `@theme` directive instead of `tailwind.config.js`

### Key Changes Required
1. Use `tw-animate-css` instead of deprecated `tailwindcss-animate`
2. CSS variables may need to wrap color values with `hsl()` or `oklch()` directly in CSS
3. HSL colors are automatically converted to OKLCH (non-breaking)
4. `forwardRef` has been removed from components (React 19 compatibility)

### Alternatives Considered
- **Stay with Radix Themes**: Rejected because only using Theme provider, not component library
- **Build custom component library**: Rejected due to effort and maintenance burden

---

## Decision 2: React 19 Compatibility

### Question
Is shadcn/ui compatible with React 19.2.0?

### Decision
**Yes, shadcn/ui fully supports React 19**

### Rationale
- Components have been updated to remove `forwardRef` (deprecated in React 19)
- Type definitions adjusted for React 19 compatibility
- New shadcn/ui CLI generates React 19 compatible code

### Implications
- No need to downgrade React
- Components will use React 19's native ref handling

---

## Decision 3: Installation Method

### Question
How should shadcn/ui be installed in an existing Vite + React project?

### Decision
**Use `npx shadcn@latest init` with manual configuration**

### Rationale
The existing project has:
- Vite 7.2.4 with React plugin
- TypeScript with strict configuration
- Tailwind CSS v4.1.18 already installed
- Custom CSS variables in `index.css`

### Installation Steps
1. Run `npx shadcn@latest init` in the frontend directory
2. Choose options:
   - Style: "new-york" (default, modern look)
   - Base color: Keep existing (slate/cyan)
   - CSS variables: Yes
   - React Server Components: No (this is a client-side app)
3. Configure `components.json` to use `@/components/ui` path alias
4. Add path alias to `tsconfig.app.json` and `vite.config.ts`

### Path Alias Configuration
```json
// tsconfig.app.json - add to compilerOptions:
{
  "baseUrl": ".",
  "paths": {
    "@/*": ["./src/*"]
  }
}
```

```typescript
// vite.config.ts
import path from "path"
export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
})
```

---

## Decision 4: CSS Variable Integration

### Question
How should shadcn/ui CSS variables integrate with existing design tokens?

### Decision
**Map shadcn variables to existing CSS variables**

### Rationale
The project has a comprehensive design token system in `index.css`:
- Background colors: `--bg-primary`, `--bg-secondary`, etc.
- Border colors: `--border-subtle`, `--border-default`, etc.
- Text colors: `--text-primary`, `--text-secondary`, etc.
- Accent colors: `--accent-cyan`, `--accent-green`, etc.

### Mapping Strategy
Configure shadcn/ui CSS variables to reference existing tokens:

```css
:root {
  /* Map shadcn to existing tokens */
  --background: var(--bg-primary);
  --foreground: var(--text-primary);
  --card: var(--bg-elevated);
  --card-foreground: var(--text-primary);
  --popover: var(--bg-surface);
  --popover-foreground: var(--text-primary);
  --primary: var(--accent-cyan);
  --primary-foreground: var(--bg-primary);
  --secondary: var(--bg-surface);
  --secondary-foreground: var(--text-primary);
  --muted: var(--bg-tertiary);
  --muted-foreground: var(--text-muted);
  --accent: var(--bg-elevated);
  --accent-foreground: var(--text-primary);
  --destructive: var(--accent-red);
  --destructive-foreground: var(--text-primary);
  --border: var(--border-default);
  --input: var(--border-default);
  --ring: var(--accent-cyan);
  --radius: var(--card-radius);
}
```

---

## Decision 5: Component Installation Strategy

### Question
Should all components be installed at once or incrementally?

### Decision
**Incremental installation per migration phase**

### Rationale
- Reduces initial bundle impact
- Allows testing each component type before moving to next
- shadcn/ui CLI supports individual component installation

### Phase Order
1. **Foundation**: button, card, badge, tooltip
2. **Forms**: input, select, textarea
3. **Overlays**: dialog, dropdown-menu
4. **Layout**: tabs, collapsible, separator, scroll-area
5. **Feedback**: progress, skeleton, alert

---

## Decision 6: Existing `@radix-ui/themes` Removal

### Question
When should `@radix-ui/themes` be removed?

### Decision
**Remove after all components are migrated**

### Rationale
- Currently only using `<Theme>` wrapper for dark mode context
- shadcn/ui components have their own dark mode styling
- Safe to remove once all custom components use shadcn equivalents

### Steps
1. Complete all component migrations
2. Replace `<Theme>` wrapper with standard `<div>` or nothing
3. Remove `@radix-ui/themes` from `package.json`
4. Remove `@radix-ui/themes/styles.css` import from `App.tsx`

---

## Decision 7: Custom Components to Preserve

### Question
Which custom components should NOT be replaced with shadcn?

### Decision
**Preserve specialized visualization components**

### Components to Keep
| Component | Reason | Styling Update |
|-----------|--------|----------------|
| `ProgressRing` | Custom SVG circular progress | Use design tokens |
| `DagView` | Custom SVG dependency graph | Use design tokens |
| `LogTerminal` (xterm.js) | Third-party terminal | Wrap in Card |
| `EmbeddedTerminal` | Third-party terminal | Wrap in Dialog + Card |

### Rationale
No shadcn equivalent exists for these specialized visualizations. They work correctly and just need styling consistency.

---

## Decision 8: Animation Library

### Question
Which animation library should be used?

### Decision
**Use `tw-animate-css` (replaces deprecated `tailwindcss-animate`)**

### Rationale
- `tailwindcss-animate` is being deprecated
- `tw-animate-css` is the recommended replacement
- Required for shadcn/ui animation utilities

### Installation
```bash
npm install tw-animate-css
```

Add to CSS:
```css
@import "tw-animate-css";
```

---

## Decision 9: `cn()` Utility Setup

### Question
How should the `cn()` utility function be set up?

### Decision
**Create standard `lib/utils.ts` file**

### Implementation
```typescript
// src/lib/utils.ts
import { clsx, type ClassValue } from "clsx"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}
```

### Dependencies
```bash
npm install clsx tailwind-merge
```

---

## Decision 10: Testing Strategy

### Question
How should the migration be tested?

### Decision
**Visual regression + E2E test preservation**

### Strategy
1. **Before migration**: Run existing E2E tests to establish baseline
2. **During migration**: Migrate component-by-component with visual verification
3. **After migration**: Re-run all E2E tests to ensure no functional regression
4. **Optional**: Add visual regression tests using Playwright screenshots

### Rationale
- E2E tests (Playwright) already exist in `tests/` directory
- Tests verify functionality, not specific styling
- Visual verification done manually during development

---

## Technology Stack Summary

| Technology | Version | Purpose |
|------------|---------|---------|
| React | 19.2.0 | UI framework |
| Vite | 7.2.4 | Build tool |
| TypeScript | 5.9.3 | Type safety |
| Tailwind CSS | 4.1.18 | Styling |
| shadcn/ui | latest | Component library |
| tw-animate-css | latest | Animation utilities |
| clsx | latest | Class name utility |
| tailwind-merge | latest | Tailwind class merging |
| lucide-react | 0.562.0 | Icons (already installed) |
| @tanstack/react-query | 5.90.12 | Data fetching (keep) |
| @xterm/xterm | 6.0.0 | Terminal (keep) |
