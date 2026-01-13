# Research: CSS Theme Consolidation

**Feature**: 001-css-theme-consolidation  
**Date**: 2026-01-12  
**Status**: Complete

## Research Tasks

### 1. Tailwind CSS v4 @theme Directive Best Practices

**Decision**: Use `@theme inline` directive in `index.css` to define all custom colors as Tailwind utilities.

**Rationale**: 
- Tailwind v4 moves configuration from `tailwind.config.js` to CSS-first using `@theme`
- The `@theme inline` variant allows embedding theme values directly in CSS without generating CSS custom properties twice
- This approach is already partially implemented in the existing `index.css` (lines 470-509)

**Alternatives Considered**:
- `tailwind.config.js` extension - Rejected: Legacy approach in v4, less flexible
- CSS custom properties only - Rejected: Loses Tailwind utility class generation

**Implementation Pattern**:
```css
@theme inline {
  --color-accent-cyan: oklch(0.82 0.19 195);
  --color-accent-cyan-dim: oklch(0.82 0.19 195 / 15%);
  --color-accent-green: oklch(0.76 0.18 145);
  /* ... */
}
```

### 2. OKLCH Color Format Conversion

**Decision**: Convert all existing hex/rgba colors to OKLCH format for perceptual uniformity.

**Rationale**:
- OKLCH provides perceptually uniform color manipulation
- Better for generating color scales and accessibility
- Native CSS support in modern browsers (96%+ coverage)
- Required for compatibility with tweakcn theme exports

**Conversion Reference** (existing colors → OKLCH):
| Token | Current Value | OKLCH Equivalent |
|-------|---------------|------------------|
| accent-cyan | #22d3ee | oklch(0.82 0.19 195) |
| accent-green | #4ade80 | oklch(0.76 0.18 145) |
| accent-amber | #fbbf24 | oklch(0.84 0.18 85) |
| accent-red | #f87171 | oklch(0.70 0.19 25) |
| accent-purple | #a78bfa | oklch(0.70 0.17 290) |
| bg-primary | #0a0a0b | oklch(0.13 0.01 265) |
| bg-secondary | #111113 | oklch(0.15 0.01 265) |
| bg-tertiary | #18181b | oklch(0.18 0.01 265) |
| bg-elevated | #1f1f23 | oklch(0.21 0.01 265) |
| bg-surface | #27272a | oklch(0.24 0.01 265) |

**Tool**: Online converter or CSS color functions for precision

### 3. Dim Variant Pattern

**Decision**: Use OKLCH alpha channel for dim variants (15% opacity).

**Rationale**:
- Current pattern uses `rgba(r, g, b, 0.15)` for dim colors
- OKLCH supports native alpha: `oklch(L C H / 15%)`
- Maintains consistency with full-color counterparts

**Pattern**:
```css
@theme inline {
  --color-accent-cyan: oklch(0.82 0.19 195);
  --color-accent-cyan-dim: oklch(0.82 0.19 195 / 15%);
}
```

### 4. Tailwind Utility Class Migration Strategy

**Decision**: Replace inline `var(--*)` with semantic Tailwind classes.

**Rationale**:
- Improves IDE autocomplete and type safety
- Reduces bundle size (Tailwind can tree-shake)
- Decouples components from CSS implementation details

**Migration Mapping**:
| Current Pattern | Migrated Pattern |
|-----------------|------------------|
| `text-[var(--accent-cyan)]` | `text-accent-cyan` |
| `bg-[var(--accent-cyan-dim)]` | `bg-accent-cyan-dim` |
| `border-[var(--accent-cyan)]` | `border-accent-cyan` |
| `hover:shadow-[0_0_20px_var(--accent-cyan-dim)]` | `hover:shadow-accent-cyan` (custom utility) |

### 5. Shadow/Glow Utilities

**Decision**: Define custom shadow utilities using Tailwind's `@theme` color references.

**Rationale**:
- Current glow effects use arbitrary values: `shadow-[0_0_20px_var(--accent-cyan-dim)]`
- These can be standardized as reusable utilities

**Pattern**:
```css
/* In @theme */
--shadow-glow-cyan: 0 0 20px oklch(0.82 0.19 195 / 20%);
--shadow-glow-green: 0 0 20px oklch(0.76 0.18 145 / 20%);

/* Or as utility classes */
.glow-cyan { box-shadow: 0 0 20px oklch(0.82 0.19 195 / 20%); }
.glow-green { box-shadow: 0 0 20px oklch(0.76 0.18 145 / 20%); }
```

### 6. Dark Mode Architecture

**Decision**: Use `.dark` class selector with CSS custom properties for theme-aware colors.

**Rationale**:
- Already implemented in codebase (lines 511-543)
- Compatible with shadcn/ui components
- Allows programmatic theme switching

**Structure**:
```css
:root {
  /* Light mode defaults (optional, for future) */
}

.dark {
  --background: oklch(0.13 0.04 265);
  --foreground: oklch(0.98 0.00 248);
  /* ... all semantic tokens */
}
```

## Findings Summary

All technical unknowns resolved:

1. ✅ `@theme inline` is the correct directive for Tailwind 4
2. ✅ OKLCH conversion is straightforward with known formulas
3. ✅ Dim variants use OKLCH alpha channel syntax
4. ✅ Utility migration is a mechanical find-replace operation
5. ✅ Glow effects can be standardized as utility classes
6. ✅ Dark mode structure is already compatible

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Browser OKLCH support | Low | Medium | Modern browsers 96%+ support; hex fallbacks if needed |
| Visual regression | Low | High | Screenshot comparison before/after |
| Missing token mapping | Low | Low | Grep validation ensures no orphaned references |
| IDE autocomplete gaps | Low | Low | VSCode Tailwind extension handles @theme definitions |
