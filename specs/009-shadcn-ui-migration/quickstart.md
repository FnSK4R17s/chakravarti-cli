# Quickstart: shadcn/ui Migration

**Feature**: 009-shadcn-ui-migration  
**Date**: 2026-01-11

## Prerequisites

Before starting the migration, ensure you have:

1. Node.js 20+ installed
2. The frontend dev server stopped
3. A clean git working directory

---

## Step 1: Install Dependencies

```bash
cd crates/ckrv-ui/frontend

# Install shadcn dependencies
npm install clsx tailwind-merge tw-animate-css

# Install shadcn CLI (can also use npx directly)
npm install -D @shadcn/ui
```

---

## Step 2: Configure Path Aliases

### Update `tsconfig.app.json`:

```json
{
  "compilerOptions": {
    // ... existing config ...
    "baseUrl": ".",
    "paths": {
      "@/*": ["./src/*"]
    }
  }
}
```

### Update `vite.config.ts`:

```typescript
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'

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

## Step 3: Initialize shadcn/ui

```bash
npx shadcn@latest init
```

When prompted:
- **Style**: New York (default)
- **Base color**: Slate
- **CSS variables**: Yes
- **CSS file location**: src/index.css
- **Tailwind config**: tailwind.config.js
- **Components directory**: src/components/ui
- **RSC**: No
- **Aliases**: @/components and @/lib

This creates `components.json` configuration file.

---

## Step 4: Create Utility Function

Create `src/lib/utils.ts`:

```typescript
import { clsx, type ClassValue } from "clsx"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}
```

---

## Step 5: Update CSS Variables

Add shadcn variables to `src/index.css` (after existing variables):

```css
:root {
  /* ... existing tokens ... */

  /* shadcn/ui required variables */
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

## Step 6: Install Components

Install components in order of priority:

### Phase 1: Foundation
```bash
npx shadcn@latest add button card badge tooltip
```

### Phase 2: Forms
```bash
npx shadcn@latest add input select
```

### Phase 3: Overlays
```bash
npx shadcn@latest add dialog dropdown-menu
```

### Phase 4: Layout
```bash
npx shadcn@latest add tabs collapsible separator scroll-area
```

### Phase 5: Feedback
```bash
npx shadcn@latest add progress skeleton alert
```

---

## Step 7: Customize Badge Variants

Edit `src/components/ui/badge.tsx` to add custom variants:

```typescript
const badgeVariants = cva(
  "inline-flex items-center ...",
  {
    variants: {
      variant: {
        default: "...",
        secondary: "...",
        destructive: "...",
        outline: "...",
        // Add custom variants:
        success: "bg-[var(--accent-green-dim)] text-[var(--accent-green)] border-transparent",
        warning: "bg-[var(--accent-amber-dim)] text-[var(--accent-amber)] border-transparent",
        info: "bg-[var(--accent-purple-dim)] text-[var(--accent-purple)] border-transparent",
      },
    },
    // ...
  }
)
```

---

## Step 8: Migrate Components

For each component, follow this pattern:

### Before (custom Tailwind):
```tsx
<button className="inline-flex items-center gap-2 px-4 py-2 rounded-lg bg-cyan-500 ...">
  <Play size={16} />
  Run
</button>
```

### After (shadcn Button):
```tsx
import { Button } from "@/components/ui/button"

<Button>
  <Play size={16} />
  Run
</Button>
```

---

## Step 9: Remove Radix Themes

After all components are migrated:

1. Update `App.tsx`:
```tsx
// Before
import { Theme } from '@radix-ui/themes'
import '@radix-ui/themes/styles.css'

function App() {
  return (
    <Theme appearance="dark" ...>
      {/* children */}
    </Theme>
  )
}

// After
function App() {
  return (
    <div className="min-h-screen bg-background text-foreground">
      {/* children */}
    </div>
  )
}
```

2. Remove package:
```bash
npm uninstall @radix-ui/themes
```

---

## Step 10: Verify

1. Start dev server:
```bash
npm run dev
```

2. Check each page:
   - [ ] Dashboard - Navigation works
   - [ ] Agents - Cards, modals render
   - [ ] Specs - Editors work
   - [ ] Tasks - Filters, status changes work
   - [ ] Plan - DAG view renders
   - [ ] Runner - Execution starts
   - [ ] Diff - Branches selectable

3. Run E2E tests:
```bash
npm run test:e2e
```

---

## Troubleshooting

### "Cannot find module '@/components/ui/button'"
- Ensure path aliases are configured in both `tsconfig.app.json` AND `vite.config.ts`

### Colors don't match
- Check that shadcn CSS variables reference your existing tokens correctly
- Verify dark mode is applied (check for `dark:` prefix classes if using class-based dark mode)

### Animation issues
- Ensure `tw-animate-css` is imported in your CSS
- Check that `tailwindcss-animate` is not still referenced

### Focus rings not visible
- The existing `:focus-visible` styles in `index.css` should work
- If not, check that shadcn's focus styles aren't being overridden

---

## Quick Reference

| Old Pattern | New Pattern |
|-------------|-------------|
| `<button className="...">` | `<Button variant="...">` |
| `<div className="bg-gray-800 rounded-lg border ...">` | `<Card>` |
| `<span className="bg-green-900/30 text-green-400 px-2 py-0.5 rounded">` | `<Badge variant="success">` |
| Custom modal with backdrop | `<Dialog>` |
| Custom dropdown | `<Select>` or `<DropdownMenu>` |
| Custom tabs | `<Tabs>` |
| Custom collapsible section | `<Collapsible>` |
| Condition ? loading spinner : content | `{loading ? <Skeleton /> : content}` |
