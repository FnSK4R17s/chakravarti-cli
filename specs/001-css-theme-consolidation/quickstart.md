# Quickstart: Theme Swapping Guide

**Feature**: 001-css-theme-consolidation  
**Date**: 2026-01-12

## Overview

After the CSS theme consolidation, you can swap themes by editing a single file: `crates/ckrv-ui/frontend/src/index.css`.

## Quick Theme Swap (< 5 minutes)

### Step 1: Export Theme from tweakcn

1. Go to [tweakcn.com](https://tweakcn.com) or your theme generator
2. Design or select a theme
3. Export as "CSS Variables (OKLCH)"
4. Copy the generated CSS

### Step 2: Locate Theme Section

Open `src/index.css` and find the `:root` section (around lines 8-125):

```css
:root {
  /* === THEME COLORS START === */
  --accent-cyan: oklch(0.82 0.19 195);
  --accent-cyan-dim: oklch(0.82 0.19 195 / 15%);
  /* ... */
  /* === THEME COLORS END === */
}
```

### Step 3: Replace Colors

Replace the values between the `THEME COLORS START/END` markers with your exported theme values:

```css
/* Example: Switching to a warm orange theme */
--accent-cyan: oklch(0.78 0.16 50);      /* Orange instead of cyan */
--accent-cyan-dim: oklch(0.78 0.16 50 / 15%);
--accent-green: oklch(0.72 0.14 160);    /* Teal instead of green */
/* ... */
```

### Step 4: Verify

1. Save the file
2. Vite hot-reloads automatically
3. Check all pages for visual consistency

## Required Variables Checklist

Your theme MUST define all of these variables:

### Accent Colors (10 required)
- [ ] `--accent-cyan` and `--accent-cyan-dim`
- [ ] `--accent-green` and `--accent-green-dim`
- [ ] `--accent-amber` and `--accent-amber-dim`
- [ ] `--accent-red` and `--accent-red-dim`
- [ ] `--accent-purple` and `--accent-purple-dim`

### Background Colors (5 required)
- [ ] `--bg-primary` (root background)
- [ ] `--bg-secondary` (sidebar/header)
- [ ] `--bg-tertiary` (muted areas)
- [ ] `--bg-elevated` (cards)
- [ ] `--bg-surface` (popovers)

### Border Colors (3 required)
- [ ] `--border-subtle`
- [ ] `--border-default`
- [ ] `--border-strong`

### Text Colors (3 required)
- [ ] `--text-primary`
- [ ] `--text-secondary`
- [ ] `--text-muted`

## OKLCH Format Reference

All colors must use OKLCH format:

```
oklch(L C H)
oklch(L C H / alpha%)
```

Where:
- **L** (Lightness): 0-1 (0 = black, 1 = white)
- **C** (Chroma): 0-0.4 (0 = gray, higher = more saturated)
- **H** (Hue): 0-360 degrees (color wheel angle)
- **alpha**: Optional, 0-100%

### Common Hue Values
| Color | Hue (H) |
|-------|---------|
| Red | 25 |
| Orange | 50 |
| Amber | 85 |
| Yellow | 100 |
| Green | 145 |
| Teal | 180 |
| Cyan | 195 |
| Blue | 240 |
| Indigo | 265 |
| Purple | 290 |
| Pink | 350 |

## Theme Presets

### Default Dark (Current)
```css
--accent-cyan: oklch(0.82 0.19 195);
--bg-primary: oklch(0.13 0.01 265);
```

### Warm Dark
```css
--accent-cyan: oklch(0.80 0.18 50);   /* Warm orange */
--bg-primary: oklch(0.12 0.02 30);    /* Warm black */
```

### Cool Dark
```css
--accent-cyan: oklch(0.75 0.15 250);  /* Cool blue */
--bg-primary: oklch(0.12 0.02 240);   /* Cool black */
```

### High Contrast
```css
--accent-cyan: oklch(0.90 0.22 195);  /* Brighter cyan */
--text-primary: oklch(1.0 0.00 0);    /* Pure white */
--bg-primary: oklch(0.08 0.00 0);     /* Deeper black */
```

## Validation Script

After changing themes, verify no broken references:

```bash
# From frontend directory
grep -r "var(--accent-\|var(--bg-\|var(--text-\|var(--border-" \
  --include="*.tsx" src/components

# Should return 0 results after migration
```

## Troubleshooting

### Colors Not Updating
- Check that Vite dev server is running
- Clear browser cache
- Verify OKLCH syntax is correct (spaces, not commas)

### Missing Color
- Ensure all 21 required variables are defined
- Check for typos in variable names
- Dim variants must have ` / 15%` alpha

### Visual Regression
- Compare screenshots before/after
- Check contrast ratios for accessibility
- Verify hover states and focus rings
