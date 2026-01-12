# Theme Variable Schema

**Feature**: 001-css-theme-consolidation  
**Date**: 2026-01-12

## Overview

This contract defines the required CSS custom properties that any theme must provide for Chakravarti UI compatibility.

## Required Theme Variables

A valid theme MUST define all of the following CSS custom properties in OKLCH format.

### Accent Colors (Required)

```css
/* Primary accent palette - all OKLCH format */
--accent-cyan: oklch(L C H);
--accent-cyan-dim: oklch(L C H / 15%);
--accent-green: oklch(L C H);
--accent-green-dim: oklch(L C H / 15%);
--accent-amber: oklch(L C H);
--accent-amber-dim: oklch(L C H / 15%);
--accent-red: oklch(L C H);
--accent-red-dim: oklch(L C H / 15%);
--accent-purple: oklch(L C H);
--accent-purple-dim: oklch(L C H / 15%);
```

### Background Colors (Required)

```css
/* Surface hierarchy - from darkest to lightest */
--bg-primary: oklch(L C H);    /* Root background */
--bg-secondary: oklch(L C H);  /* Sidebar, header */
--bg-tertiary: oklch(L C H);   /* Muted areas */
--bg-elevated: oklch(L C H);   /* Cards */
--bg-surface: oklch(L C H);    /* Popovers */
```

### Border Colors (Required)

```css
--border-subtle: oklch(L C H);   /* Inactive */
--border-default: oklch(L C H);  /* Standard */
--border-strong: oklch(L C H);   /* Hover */
```

### Text Colors (Required)

```css
--text-primary: oklch(L C H);    /* Headings */
--text-secondary: oklch(L C H);  /* Secondary */
--text-muted: oklch(L C H);      /* Disabled */
```

### Semantic Mappings (Auto-Derived)

These are automatically derived from the above tokens and should NOT be manually overridden:

```css
/* Derived from accent/background tokens */
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
--border: var(--border-default);
--input: var(--border-default);
--ring: var(--accent-cyan);
```

## Tailwind Theme Block

The `@theme inline` block MUST expose these color utilities:

```css
@theme inline {
  /* Accent utilities */
  --color-accent-cyan: var(--accent-cyan);
  --color-accent-cyan-dim: var(--accent-cyan-dim);
  --color-accent-green: var(--accent-green);
  --color-accent-green-dim: var(--accent-green-dim);
  --color-accent-amber: var(--accent-amber);
  --color-accent-amber-dim: var(--accent-amber-dim);
  --color-accent-red: var(--accent-red);
  --color-accent-red-dim: var(--accent-red-dim);
  --color-accent-purple: var(--accent-purple);
  --color-accent-purple-dim: var(--accent-purple-dim);
  
  /* Background utilities */
  --color-bg-primary: var(--bg-primary);
  --color-bg-secondary: var(--bg-secondary);
  --color-bg-tertiary: var(--bg-tertiary);
  --color-bg-elevated: var(--bg-elevated);
  --color-bg-surface: var(--bg-surface);
  
  /* Existing shadcn mappings preserved */
  --color-background: var(--background);
  --color-foreground: var(--foreground);
  /* ... etc */
}
```

## Generated Utility Classes

After theme consolidation, components can use these Tailwind classes:

### Text Colors
- `text-accent-cyan`, `text-accent-green`, `text-accent-amber`, `text-accent-red`, `text-accent-purple`
- `text-foreground`, `text-muted-foreground`

### Background Colors
- `bg-accent-cyan`, `bg-accent-cyan-dim`
- `bg-accent-green`, `bg-accent-green-dim`
- `bg-accent-amber`, `bg-accent-amber-dim`
- `bg-accent-red`, `bg-accent-red-dim`
- `bg-accent-purple`, `bg-accent-purple-dim`
- `bg-background`, `bg-card`, `bg-muted`, `bg-accent`

### Border Colors
- `border-accent-cyan`, `border-accent-green`, `border-accent-amber`, `border-accent-red`, `border-accent-purple`
- `border-border`

## Validation

A theme can be validated by checking:

1. All required variables are defined
2. All values use OKLCH format
3. Dim variants have 15% opacity
4. Lightness (L) values form proper hierarchy for backgrounds:
   - `bg-primary` L is lowest (darkest)
   - `bg-surface` L is highest (lightest)

## Example: Default Dark Theme

```css
:root {
  /* Accents */
  --accent-cyan: oklch(0.82 0.19 195);
  --accent-cyan-dim: oklch(0.82 0.19 195 / 15%);
  --accent-green: oklch(0.76 0.18 145);
  --accent-green-dim: oklch(0.76 0.18 145 / 15%);
  --accent-amber: oklch(0.84 0.18 85);
  --accent-amber-dim: oklch(0.84 0.18 85 / 15%);
  --accent-red: oklch(0.70 0.19 25);
  --accent-red-dim: oklch(0.70 0.19 25 / 15%);
  --accent-purple: oklch(0.70 0.17 290);
  --accent-purple-dim: oklch(0.70 0.17 290 / 15%);
  
  /* Backgrounds */
  --bg-primary: oklch(0.13 0.01 265);
  --bg-secondary: oklch(0.15 0.01 265);
  --bg-tertiary: oklch(0.18 0.01 265);
  --bg-elevated: oklch(0.21 0.01 265);
  --bg-surface: oklch(0.24 0.01 265);
  
  /* Borders */
  --border-subtle: oklch(0.24 0.01 265);
  --border-default: oklch(0.32 0.01 265);
  --border-strong: oklch(0.39 0.01 265);
  
  /* Text */
  --text-primary: oklch(0.98 0.00 0);
  --text-secondary: oklch(0.70 0.01 265);
  --text-muted: oklch(0.52 0.01 265);
}
```
