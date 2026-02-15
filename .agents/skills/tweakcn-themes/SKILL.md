---
name: tweakcn-themes
description: Manage shadcn/ui themes from tweakcn.com. Use when installing themes, updating terminal colors, or switching the project's visual theme.
license: MIT
compatibility: Node.js projects with shadcn/ui
metadata:
  author: FnSK4R17s
  version: "1.0"
---

# TweakCN Theme Management

This skill covers installing shadcn/ui themes from tweakcn.com and keeping terminal
colors in sync with the theme.

## Prerequisites

- Node.js and npm installed
- shadcn/ui initialized in the frontend project
- Project structure: `crates/ckrv-ui/frontend/`

## Quick Reference

| Task | Command |
|------|---------|
| Install theme | `npx shadcn@latest add <theme-url>` |
| Extract terminal colors | `npx ts-node --esm scripts/extract-terminal-colors.ts` |
| Browse themes | https://tweakcn.com |

## Installing a New Theme

### Step 1: Choose a Theme

Browse themes at https://tweakcn.com and find one you like. Each theme has a JSON URL.

Popular themes:
- **darkmatter** (current): `https://tweakcn.com/r/themes/darkmatter.json`
- **catppuccin**: `https://tweakcn.com/r/themes/catppuccin-mocha.json`
- **rose-pine**: `https://tweakcn.com/r/themes/rose-pine.json`

### Step 2: Install the Theme

```bash
cd crates/ckrv-ui/frontend
npx shadcn@latest add https://tweakcn.com/r/themes/<theme-name>.json
```

This overwrites `src/index.css` with the new theme's CSS variables.

### Step 3: Update Terminal Colors

xterm.js requires hex color values, not oklch. Run the extraction script:

```bash
npx ts-node --esm scripts/extract-terminal-colors.ts
```

Sample output:
```
============================================================
TERMINAL THEME COLORS
============================================================

🌙 DARK MODE (from .dark):
────────────────────────────────────────
  --background: oklch(0.1822 0 0)
    → #121212
  --foreground: oklch(0.9288 0.0126 255.5078)
    → #e2e8f0

📋 COPY-PASTE CONFIG FOR theme.ts:
────────────────────────────────────────

dark: {
    background: '#121212',
    foreground: '#e2e8f0',
    cursor: '#e2e8f0',
    cursorAccent: '#121212',
    // ... ANSI colors remain the same
},
```

### Step 4: Update theme.ts

Open `src/lib/theme.ts` and update the `TERMINAL_THEMES` constant:

1. Find the `TERMINAL_THEMES` section (around line 263)
2. Replace the `background`, `foreground`, `cursor`, and `cursorAccent` values
3. Update the comment header with the new theme name and date

Example:
```typescript
// Current theme: catppuccin-mocha (from tweakcn.com)
// Last synced: 2026-02-03

export const TERMINAL_THEMES = {
    dark: {
        background: '#1e1e2e',    // from script output
        foreground: '#cdd6f4',    // from script output
        cursor: '#cdd6f4',
        cursorAccent: '#1e1e2e',
        // ... keep ANSI colors unchanged
    },
    // ...
};
```

### Step 5: Rebuild

```bash
npm run build
```

## Script Details

### extract-terminal-colors.ts

Locations:
- `crates/ckrv-ui/frontend/scripts/extract-terminal-colors.ts` (main)
- `.agent/skills/tweakcn-themes/scripts/extract-terminal-colors.ts` (skill copy)

What it does:
1. Reads `src/index.css`
2. Parses `:root` (light mode) and `.dark` (dark mode) CSS variables
3. Converts `oklch()` values to hex using color space math
4. Outputs copy-paste ready config

The oklch → hex conversion uses:
- oklch → oklab → linear sRGB → sRGB → hex

### Variables Extracted

| Variable | Purpose |
|----------|---------|
| `--background` | Terminal background color |
| `--foreground` | Terminal text color |
| `--muted` | Muted UI elements |
| `--muted-foreground` | Muted text |
| `--primary` | Primary accent color |

## Theme File Structure

After installing a theme, `index.css` contains:

```css
/* Light mode */
:root {
  --background: oklch(0.9911 0 0);
  --foreground: oklch(0.2046 0 0);
  /* ... more variables */
}

/* Dark mode */
.dark {
  --background: oklch(0.1822 0 0);
  --foreground: oklch(0.9288 0.0126 255.5078);
  /* ... more variables */
}
```

## Semantic Color Mapping

The project uses semantic colors that map to shadcn variables:

| Semantic | shadcn Variable | Usage |
|----------|-----------------|-------|
| `text-success` | `--success` | Green for completed states |
| `text-warning` | `--warning` | Amber for warnings |
| `text-error` | `--error` | Red for errors |
| `text-info` | `--info` | Teal/blue for info |
| `text-primary` | `--primary` | Theme primary color |

These are defined in `index.css` under the "Custom semantic colors" section.

## Troubleshooting

### Theme not applying

1. Hard refresh the browser (Ctrl+Shift+R)
2. Clear Vite cache: `rm -rf node_modules/.vite`
3. Rebuild: `npm run build`

### Terminal colors look wrong

1. Re-run the extraction script
2. Verify `getTerminalTheme()` is called in xterm initialization
3. Check that the theme toggle is working (dark class on `<html>`)

### Light mode terminal has dark background

The terminal uses `getTerminalTheme()` which checks `document.documentElement.classList.contains('dark')`.
Make sure:
1. Theme toggle updates the `<html>` element's class
2. Terminal is re-initialized on theme change (or page refresh)

## Full Workflow Example

```bash
# 1. Install new theme
cd crates/ckrv-ui/frontend
npx shadcn@latest add https://tweakcn.com/r/themes/catppuccin-mocha.json

# 2. Extract terminal colors
npx ts-node --esm scripts/extract-terminal-colors.ts

# 3. Update theme.ts with the output values
# (edit src/lib/theme.ts manually)

# 4. Rebuild and test
npm run build

# 5. Start dev server and verify
npm run dev
```

## References

- [tweakcn.com](https://tweakcn.com) - Community shadcn themes
- [shadcn/ui Themes](https://ui.shadcn.com/themes) - Official themes
- [OKLCH Color Space](https://oklch.com) - Color picker and converter
