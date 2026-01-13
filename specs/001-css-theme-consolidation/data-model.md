# Data Model: Theme Token Inventory

**Feature**: 001-css-theme-consolidation  
**Date**: 2026-01-12

## Overview

This document catalogs all design tokens in the Chakravarti UI, their current locations, and target OKLCH values for the consolidated theme.

## Color Tokens

### Accent Colors (Primary Palette)

| Token Name | Current Value | OKLCH Value | Usage |
|------------|---------------|-------------|-------|
| `accent-cyan` | `#22d3ee` | `oklch(0.82 0.19 195)` | Primary brand, links, focus rings |
| `accent-cyan-dim` | `rgba(34, 211, 238, 0.15)` | `oklch(0.82 0.19 195 / 15%)` | Subtle backgrounds, hover states |
| `accent-green` | `#4ade80` | `oklch(0.76 0.18 145)` | Success states, running indicators |
| `accent-green-dim` | `rgba(74, 222, 128, 0.15)` | `oklch(0.76 0.18 145 / 15%)` | Success backgrounds |
| `accent-amber` | `#fbbf24` | `oklch(0.84 0.18 85)` | Warnings, in-progress states |
| `accent-amber-dim` | `rgba(251, 191, 36, 0.15)` | `oklch(0.84 0.18 85 / 15%)` | Warning backgrounds |
| `accent-red` | `#f87171` | `oklch(0.70 0.19 25)` | Errors, destructive actions |
| `accent-red-dim` | `rgba(248, 113, 113, 0.15)` | `oklch(0.70 0.19 25 / 15%)` | Error backgrounds |
| `accent-purple` | `#a78bfa` | `oklch(0.70 0.17 290)` | Special states, promoting |
| `accent-purple-dim` | `rgba(167, 139, 250, 0.15)` | `oklch(0.70 0.17 290 / 15%)` | Purple backgrounds |
| `accent-pink` | (implied) | `oklch(0.70 0.19 350)` | Agent type indicator |

### Background Colors (Surface Hierarchy)

| Token Name | Current Value | OKLCH Value | Usage |
|------------|---------------|-------------|-------|
| `bg-primary` | `#0a0a0b` | `oklch(0.13 0.01 265)` | Root background |
| `bg-secondary` | `#111113` | `oklch(0.15 0.01 265)` | Sidebar, header |
| `bg-tertiary` | `#18181b` | `oklch(0.18 0.01 265)` | Muted areas |
| `bg-elevated` | `#1f1f23` | `oklch(0.21 0.01 265)` | Cards, elevated surfaces |
| `bg-surface` | `#27272a` | `oklch(0.24 0.01 265)` | Popovers, dropdowns |

### Border Colors

| Token Name | Current Value | OKLCH Value | Usage |
|------------|---------------|-------------|-------|
| `border-subtle` | `#27272a` | `oklch(0.24 0.01 265)` | Inactive borders |
| `border-default` | `#3f3f46` | `oklch(0.32 0.01 265)` | Standard borders |
| `border-strong` | `#52525b` | `oklch(0.39 0.01 265)` | Hover borders |

### Text Colors

| Token Name | Current Value | OKLCH Value | Usage |
|------------|---------------|-------------|-------|
| `text-primary` | `#fafafa` | `oklch(0.98 0.00 0)` | Headings, primary text |
| `text-secondary` | `#a1a1aa` | `oklch(0.70 0.01 265)` | Secondary text |
| `text-muted` | `#71717a` | `oklch(0.52 0.01 265)` | Disabled, placeholder |

## Semantic Token Mappings

These map raw tokens to shadcn/ui semantic variables:

| Semantic Variable | Maps To | Description |
|-------------------|---------|-------------|
| `--background` | `bg-primary` | Page background |
| `--foreground` | `text-primary` | Default text |
| `--card` | `bg-elevated` | Card backgrounds |
| `--card-foreground` | `text-primary` | Card text |
| `--popover` | `bg-surface` | Dropdown/popover bg |
| `--popover-foreground` | `text-primary` | Popover text |
| `--primary` | `accent-cyan` | Primary brand color |
| `--primary-foreground` | `bg-primary` | Text on primary |
| `--secondary` | `bg-surface` | Secondary actions |
| `--secondary-foreground` | `text-primary` | Secondary text |
| `--muted` | `bg-tertiary` | Muted backgrounds |
| `--muted-foreground` | `text-muted` | Muted text |
| `--accent` | `bg-elevated` | Accent backgrounds |
| `--accent-foreground` | `text-primary` | Accent text |
| `--destructive` | `accent-red` | Destructive actions |
| `--border` | `border-default` | Default borders |
| `--input` | `border-default` | Input borders |
| `--ring` | `accent-cyan` | Focus rings |

## Shadow Tokens

| Token Name | Current Value | OKLCH Value |
|------------|---------------|-------------|
| `glow-cyan` | `0 0 20px rgba(34, 211, 238, 0.2)` | `0 0 20px oklch(0.82 0.19 195 / 20%)` |
| `glow-green` | `0 0 20px rgba(74, 222, 128, 0.2)` | `0 0 20px oklch(0.76 0.18 145 / 20%)` |
| `glow-amber` | `0 0 8px var(--accent-amber)` | `0 0 8px oklch(0.84 0.18 85 / 50%)` |
| `glow-purple` | `0 0 8px var(--accent-purple)` | `0 0 8px oklch(0.70 0.17 290 / 50%)` |

## Chart Colors

| Token Name | Maps To |
|------------|---------|
| `--chart-1` | `accent-cyan` |
| `--chart-2` | `accent-green` |
| `--chart-3` | `accent-amber` |
| `--chart-4` | `accent-purple` |
| `--chart-5` | `accent-red` |

## Animation Tokens (Non-Color)

These remain as CSS variables, not converted to OKLCH:

| Token Name | Value | Usage |
|------------|-------|-------|
| `--duration-fast` | `150ms` | Micro-interactions |
| `--duration-normal` | `200ms` | Standard transitions |
| `--duration-slow` | `300ms` | Panel transitions |
| `--duration-focus` | `5000ms` | Auto-collapse delay |
| `--ease-default` | `cubic-bezier(0.4, 0, 0.2, 1)` | Standard easing |
| `--ease-in` | `cubic-bezier(0.4, 0, 1, 1)` | Accelerate |
| `--ease-out` | `cubic-bezier(0, 0, 0.2, 1)` | Decelerate |
| `--ease-bounce` | `cubic-bezier(0.68, -0.55, 0.265, 1.55)` | Playful bounce |

## Spacing & Layout Tokens (Non-Color)

| Token Name | Value | Usage |
|------------|-------|-------|
| `--space-xs` | `0.25rem` | 4px spacing |
| `--space-sm` | `0.5rem` | 8px spacing |
| `--space-md` | `1rem` | 16px spacing |
| `--space-lg` | `1.5rem` | 24px spacing |
| `--space-xl` | `2rem` | 32px spacing |
| `--space-2xl` | `3rem` | 48px spacing |
| `--header-height` | `3.5rem` | 56px header |
| `--sidebar-width` | `280px` | Sidebar width |
| `--panel-radius` | `0.75rem` | 12px radius |
| `--card-radius` | `0.5rem` | 8px radius |
| `--button-radius` | `0.5rem` | 8px radius |
| `--radius` | `0.5rem` | Default radius |

## Files With Inline References

| File | Reference Count | Tokens Used |
|------|-----------------|-------------|
| `CommandPalette.tsx` | 18 | accent-cyan, accent-green, accent-amber, accent-purple, accent-*-dim |
| `LogViewer.tsx` | 12 | accent-red, accent-green, accent-amber, accent-cyan, accent-purple |
| `DiffViewer.tsx` | 10 | accent-green, accent-cyan, accent-amber |
| `WorkflowPanel.tsx` | 15 | accent-green, accent-amber, accent-purple, accent-cyan, accent-red |
| `AgentManager.tsx` | 8 | accent-amber, accent-purple, accent-cyan, accent-green, accent-pink |
| `StatusWidget.tsx` | 6 | accent-cyan, accent-green, accent-amber |
| `TaskEditor.tsx` | 12 | accent-green, accent-amber, accent-red, accent-cyan |
| `SpecEditor.tsx` | 8 | accent-green, accent-cyan, accent-amber |
| `PlanEditor.tsx` | 10 | accent-cyan, accent-green, accent-amber, accent-purple |
| `TaskDetailModal.tsx` | 6 | accent-green, accent-amber, accent-red |
| `RunHistoryPanel.tsx` | 8 | accent-green, accent-red, accent-amber |
| `CompletionSummary.tsx` | 4 | accent-green, accent-red |
| `badge.tsx` | 6 | accent-green, accent-amber, accent-red, accent-cyan |
| `LoadingOverlay.tsx` | 4 | bg-primary, bg-tertiary |
| `Dashboard.tsx` | 5 | accent-cyan, accent-purple, bg-primary |

**Total**: ~132 inline references across 15 files
