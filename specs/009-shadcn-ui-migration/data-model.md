# Data Model: shadcn/ui Migration

**Feature**: 009-shadcn-ui-migration  
**Date**: 2026-01-11

## Overview

This document defines the component hierarchy and relationships for the shadcn/ui migration. Since this is a UI component migration (not a data/API feature), the "data model" represents the component structure and design token system.

---

## Component Inventory

### shadcn/ui Components to Install

| Component | Primitives Used | Variants Needed |
|-----------|-----------------|-----------------|
| **button** | Radix Slot | default, destructive, outline, secondary, ghost, link |
| **card** | - | Card, CardHeader, CardContent, CardFooter, CardTitle, CardDescription |
| **badge** | - | default, secondary, destructive, outline |
| **dialog** | Radix Dialog | Dialog, DialogTrigger, DialogContent, DialogHeader, DialogFooter, DialogTitle, DialogDescription |
| **input** | - | - |
| **select** | Radix Select | Select, SelectTrigger, SelectContent, SelectItem, SelectGroup, SelectLabel |
| **tabs** | Radix Tabs | Tabs, TabsList, TabsTrigger, TabsContent |
| **tooltip** | Radix Tooltip | Tooltip, TooltipTrigger, TooltipContent, TooltipProvider |
| **collapsible** | Radix Collapsible | Collapsible, CollapsibleTrigger, CollapsibleContent |
| **scroll-area** | Radix ScrollArea | ScrollArea, ScrollBar |
| **dropdown-menu** | Radix DropdownMenu | Full menu primitives |
| **progress** | Radix Progress | - |
| **skeleton** | - | - |
| **alert** | - | Alert, AlertTitle, AlertDescription |
| **separator** | Radix Separator | - |

---

## Component Migration Map

### Layout Components

```
Dashboard.tsx (layouts/)
├── Sidebar
│   ├── NavIcon → Button[variant=ghost] + Tooltip
│   └── StatusIndicators
│       ├── ConnectionIndicator → Badge + Tooltip
│       ├── DockerIndicator → Badge + Tooltip  
│       └── CloudIndicator → Badge + Tooltip
└── Header
    └── (minimal, no changes)
```

### Page: Agents

```
AgentManager.tsx
├── Header
│   └── Add Agent Button → Button[variant=default]
├── AgentCard (repeated)
│   ├── Container → Card
│   ├── Header → CardHeader
│   ├── Status Badge → Badge
│   ├── Expand/Collapse → Collapsible
│   ├── Action Buttons → Button[variant=ghost] + Tooltip
│   └── Details → CardContent
└── AgentModal
    ├── Container → Dialog
    ├── Header → DialogHeader + DialogTitle
    ├── Form
    │   ├── Agent Type → Select
    │   ├── Name → Input
    │   ├── Model → Select
    │   └── Other Fields → Input
    └── Actions → DialogFooter + Button
```

### Page: Specs

```
SpecEditor.tsx
├── SpecListView
│   └── Spec Card (repeated) → Card
├── Header
│   ├── Back Button → Button[variant=ghost]
│   ├── View Toggle → Tabs
│   └── Save Button → Button[variant=default]
├── Section (repeated) → Collapsible
│   ├── Trigger → CollapsibleTrigger + Button
│   └── Content → CollapsibleContent
├── EditableText → Input (inline editing)
├── PriorityBadge → Badge
└── RequirementRow → Card (small)
```

### Page: Tasks

```
TaskEditor.tsx
├── SpecListView → Card (repeated)
├── Header
│   ├── View Toggle → Tabs
│   ├── FilterBar → Select (multiple)
│   └── Action Buttons → Button
├── PhaseGroup (repeated) → Collapsible
│   └── TaskCard (repeated) → Card
│       ├── StatusBadge → Badge
│       ├── RiskBadge → Badge
│       ├── ModelTierBadge → Badge
│       └── ComplexityDots → (custom)
├── KanbanColumn → Card (column container)
└── TaskDetailModal → Dialog
    ├── Task Info → Card sections
    ├── AgentSelector → Select
    ├── EmbeddedTerminal → Card (wrapper for xterm)
    └── Action Buttons → Button
```

### Page: Plan

```
PlanEditor.tsx
├── SpecListView → Card (repeated)
├── Header
│   ├── View Toggle → Tabs
│   └── Save Button → Button
├── BatchCard (repeated) → Card
│   ├── ModelBadge → Badge
│   ├── StrategyBadge → Badge
│   └── Task List → (internal list)
├── DagView → (custom SVG, keep)
└── BatchDetail (sidebar) → Card
```

### Page: Runner (Execution)

```
ExecutionRunner.tsx
├── SpecListView → Card (repeated)
├── Header
│   ├── Status → Badge
│   ├── Timer → (custom)
│   └── Actions → Button
├── OrchestratorPanel
│   ├── ProgressRing → (custom SVG, keep)
│   ├── Batch Cards → Card (grid)
│       ├── Status → Badge
│       ├── Log Panel → ScrollArea
│       └── Actions → Button[variant=ghost]
│   └── Elapsed Time → (custom)
├── LogTerminal → Card (wrapper for xterm)
├── MergeBranchesPanel → Card
│   ├── Branch List → (list)
│   └── Merge Button → Button
└── CompletionSummary → Card + Alert
```

### Page: Diff

```
DiffViewer.tsx
├── Header
│   ├── BranchSelector (x2) → Select
│   ├── Stats → Badge (multiple)
│   └── Actions → Button
├── FileDiffView (repeated)
│   ├── Container → Card + Collapsible
│   ├── Header → CollapsibleTrigger
│   ├── Status Icon → (icon)
│   └── Diff Content → ScrollArea
```

### Shared Components

```
CommandPalette.tsx
├── Container → Card
├── Section Headers → CardHeader
├── Command Cards → Card (small)
│   ├── Icon → (lucide icon)
│   ├── Title → CardTitle
│   ├── Badge → Badge
│   └── Action → Button
└── CreateSpec Modal
    ├── Container → Dialog
    ├── Form → Input (multiple)
    └── Actions → Button

StatusWidget.tsx
├── Container → Card
├── Header → CardHeader
├── StatusRow (repeated)
│   ├── Label → (text)
│   ├── Value → Badge OR (text)
│   └── Hint → Tooltip
└── GitInit Button → Button

WorkflowPanel.tsx
├── PipelineStage (repeated) → Card
│   ├── Header (icon + count) → CardHeader + Badge
│   ├── Status → Badge
│   └── Items → ScrollArea
├── Connector (arrows) → (custom)
└── EmptyState → (content)

LogViewer.tsx
├── Container → Card
├── Header → CardHeader
│   ├── Tabs (All/Errors) → Tabs
│   └── Actions → Button[variant=ghost]
├── Log Content → ScrollArea
└── EmptyLogs → (custom branding)

ErrorBoundary.tsx
└── Error Display → Alert[variant=destructive]
```

---

## Design Token Relationships

### Color Mapping (shadcn → existing tokens)

| shadcn Variable | Maps To | Usage |
|-----------------|---------|-------|
| `--background` | `--bg-primary` | Page background |
| `--foreground` | `--text-primary` | Primary text |
| `--card` | `--bg-elevated` | Card backgrounds |
| `--card-foreground` | `--text-primary` | Card text |
| `--popover` | `--bg-surface` | Popover/dropdown bg |
| `--popover-foreground` | `--text-primary` | Popover text |
| `--primary` | `--accent-cyan` | Primary actions |
| `--primary-foreground` | `--bg-primary` | Primary action text |
| `--secondary` | `--bg-surface` | Secondary actions |
| `--secondary-foreground` | `--text-primary` | Secondary text |
| `--muted` | `--bg-tertiary` | Muted backgrounds |
| `--muted-foreground` | `--text-muted` | Muted text |
| `--accent` | `--bg-elevated` | Accent backgrounds |
| `--accent-foreground` | `--text-primary` | Accent text |
| `--destructive` | `--accent-red` | Destructive actions |
| `--destructive-foreground` | `--text-primary` | Destructive text |
| `--border` | `--border-default` | Default borders |
| `--input` | `--border-default` | Input borders |
| `--ring` | `--accent-cyan` | Focus ring |
| `--radius` | `--card-radius` | Border radius |

### Badge Variant Mapping

| Badge Variant | Color Token | Use Case |
|---------------|-------------|----------|
| `default` | `--accent-cyan` | Primary status |
| `secondary` | `--bg-surface` | Neutral status |
| `destructive` | `--accent-red` | Error/failed |
| `outline` | transparent | Subtle indicators |
| *custom* `success` | `--accent-green` | Completed/success |
| *custom* `warning` | `--accent-amber` | Warning/pending |
| *custom* `info` | `--accent-purple` | Information |

---

## State Transitions

### Button States
```
idle → hover → focus → active → disabled
                     ↓
                  loading
```

### Dialog States
```
closed → opening (animation) → open → closing (animation) → closed
```

### Collapsible States
```
collapsed → expanding (animation) → expanded → collapsing (animation) → collapsed
```

---

## File Structure After Migration

```
frontend/src/
├── components/
│   ├── ui/                    # shadcn/ui components
│   │   ├── button.tsx
│   │   ├── card.tsx
│   │   ├── badge.tsx
│   │   ├── dialog.tsx
│   │   ├── input.tsx
│   │   ├── select.tsx
│   │   ├── tabs.tsx
│   │   ├── tooltip.tsx
│   │   ├── collapsible.tsx
│   │   ├── scroll-area.tsx
│   │   ├── dropdown-menu.tsx
│   │   ├── progress.tsx
│   │   ├── skeleton.tsx
│   │   ├── alert.tsx
│   │   └── separator.tsx
│   ├── AgentManager.tsx       # Migrated
│   ├── AgentCliModal.tsx      # Migrated
│   ├── CommandPalette.tsx     # Migrated
│   ├── CompletionSummary.tsx  # Migrated
│   ├── DiffViewer.tsx         # Migrated
│   ├── ErrorBoundary.tsx      # Migrated
│   ├── ExecutionRunner.tsx    # Migrated
│   ├── LogTerminal.tsx        # Partially (wrapper only)
│   ├── LogViewer.tsx          # Migrated
│   ├── PlanEditor.tsx         # Migrated
│   ├── RunHistoryPanel.tsx    # Migrated
│   ├── SpecEditor.tsx         # Migrated
│   ├── StatusWidget.tsx       # Migrated
│   ├── TaskDetailModal.tsx    # Migrated
│   ├── TaskEditor.tsx         # Migrated
│   └── WorkflowPanel.tsx      # Migrated
├── layouts/
│   └── Dashboard.tsx          # Migrated
├── lib/
│   └── utils.ts               # cn() utility
├── index.css                  # Updated with shadcn variables
└── App.tsx                    # Theme provider removed
```
