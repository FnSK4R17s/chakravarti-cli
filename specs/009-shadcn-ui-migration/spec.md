# Feature Specification: shadcn/ui Migration

**Feature Branch**: `009-shadcn-ui-migration`  
**Created**: 2026-01-11  
**Status**: Draft  
**Input**: User description: "Migrate the frontend to shadcn/ui, replacing all custom Tailwind components with standard shadcn/ui components for a polished, production-ready customer-facing UI"

## Executive Summary

The Chakravarti UI frontend currently uses a hybrid approach with `@radix-ui/themes` (only for the Theme provider) and raw Tailwind CSS for all component styling. This results in:
- **Inconsistent visual polish** across components
- **Missing accessibility features** that come standard with proper component libraries
- **No cohesive design system** — each component is hand-styled
- **Not production-ready** for customer-facing release

This migration replaces all custom-styled components with shadcn/ui, providing a polished, accessible, and consistent design system.

---

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Core Navigation & Layout Polish (Priority: P1)

Users navigate through the Chakravarti dashboard, sidebar, and pages. The UI should feel premium with consistent spacing, animations, and visual hierarchy.

**Why this priority**: The layout (sidebar, header, navigation) is visible on every page. Polishing this first creates immediate visual impact across the entire application.

**Independent Test**: Navigate through all pages (Dashboard, Agents, Specs, Tasks, Plan, Runner, Diff) and verify consistent styling, smooth transitions, and proper responsive behavior.

**Acceptance Scenarios**:

1. **Given** a user is on any page, **When** they view the sidebar navigation, **Then** navigation icons have consistent hover/active states with smooth animations
2. **Given** a user clicks a navigation item, **When** the page transitions, **Then** the transition is smooth with proper loading states
3. **Given** a user views the status indicators (Connection, Docker, Cloud), **When** status changes, **Then** indicators animate smoothly with appropriate colors

---

### User Story 2 - Form Inputs & Buttons Consistency (Priority: P1)

Users interact with buttons, inputs, selects, and form controls throughout the application. All interactive elements should have consistent styling and behavior.

**Why this priority**: Buttons and inputs are the primary interaction points. Inconsistent styling here directly impacts user confidence in the product quality.

**Independent Test**: Interact with all form elements across different pages and verify consistent styling, focus states, and loading behaviors.

**Acceptance Scenarios**:

1. **Given** a user hovers over any button, **When** the hover state activates, **Then** the visual feedback is consistent with the design system
2. **Given** a user tabs through form elements, **When** each element receives focus, **Then** visible focus indicators appear consistently
3. **Given** a button is in loading state, **When** the user views it, **Then** a consistent loading spinner appears with optional loading text

---

### User Story 3 - Modal & Dialog Interactions (Priority: P2)

Users open modals for agent configuration, task details, and confirmations. Modals should have consistent styling, animations, and accessibility.

**Why this priority**: Modals are critical for configuration and detailed views. Poor modal UX breaks user flow.

**Independent Test**: Open all modal types (AgentModal, AgentCliModal, TaskDetailModal) and verify consistent appearance, keyboard navigation, and close behavior.

**Acceptance Scenarios**:

1. **Given** a modal is opened, **When** the user views it, **Then** a proper overlay appears with consistent animation
2. **Given** a modal is open, **When** the user presses Escape, **Then** the modal closes
3. **Given** a modal is open, **When** the user clicks outside, **Then** the modal closes (where appropriate)

---

### User Story 4 - Cards & Data Display (Priority: P2)

Users view information in cards (agent cards, spec cards, batch cards, task cards). Cards should have consistent elevation, borders, and hover states.

**Why this priority**: Cards are the primary container for content. Consistent card styling creates visual cohesion.

**Independent Test**: View all card types across pages and verify consistent styling, hover effects, and content layout.

**Acceptance Scenarios**:

1. **Given** cards are displayed, **When** the user hovers, **Then** elevation/border changes indicate interactivity
2. **Given** cards contain badges or status indicators, **When** displayed, **Then** badge styling is consistent across all card types

---

### User Story 5 - Execution & Real-time Feedback (Priority: P3)

Users run executions and see real-time progress in the ExecutionRunner. Progress indicators, batch status, and log panels should be polished.

**Why this priority**: This is the core functionality but involves complex real-time updates. Polishing this after basic components ensures a stable foundation.

**Independent Test**: Run an execution and verify progress rings, batch status badges, log panels, and completion summary display correctly.

**Acceptance Scenarios**:

1. **Given** an execution is running, **When** progress updates, **Then** progress indicators animate smoothly
2. **Given** a batch completes, **When** status changes, **Then** visual transition to completed state is clear

---

### User Story 6 - Editors & Complex Views (Priority: P3)

Users edit specs, tasks, and plans in dedicated editors. Collapsible sections, inline editing, and code views should be polished.

**Why this priority**: Editors are power-user features. Core polish comes before advanced editing UX.

**Independent Test**: Edit content in SpecEditor, TaskEditor, and PlanEditor. Verify collapsible sections, inline editing, and view toggles work with consistent styling.

**Acceptance Scenarios**:

1. **Given** a collapsible section is present, **When** user clicks to expand/collapse, **Then** animation is smooth
2. **Given** inline editing is available, **When** user enters edit mode, **Then** input styling matches the design system

---

### Edge Cases

- What happens when components render before shadcn styles load?
  - *Assumption*: Use proper CSS loading order and critical CSS to prevent flash of unstyled content
- How does the system handle components that don't have shadcn equivalents (e.g., LogTerminal with xterm.js)?
  - *Assumption*: Keep xterm.js but wrap in shadcn Card container with consistent styling
- How does the system handle custom visualizations (DAG view, progress rings)?
  - *Assumption*: Keep custom SVG-based visualizations but use shadcn design tokens for colors and spacing

---

## Requirements *(mandatory)*

### Functional Requirements

#### Setup & Infrastructure
- **FR-001**: System MUST install and configure shadcn/ui with the project's existing Tailwind CSS v4 setup
- **FR-002**: System MUST remove `@radix-ui/themes` dependency after migration is complete
- **FR-003**: System MUST preserve existing CSS variables and design tokens in `index.css`
- **FR-004**: System MUST configure shadcn/ui to use the existing dark theme palette

#### Core Components to Install
- **FR-005**: System MUST provide Button component with variants: default, destructive, outline, secondary, ghost, link
- **FR-006**: System MUST provide Card component (Card, CardHeader, CardContent, CardFooter, CardTitle, CardDescription)
- **FR-007**: System MUST provide Dialog component for modals with proper accessibility
- **FR-008**: System MUST provide Badge component for status indicators
- **FR-009**: System MUST provide Input component with proper focus states
- **FR-010**: System MUST provide Select component for dropdowns
- **FR-011**: System MUST provide Tabs component for view switching
- **FR-012**: System MUST provide Tooltip component for icon buttons and abbreviated content
- **FR-013**: System MUST provide Separator component for visual dividers
- **FR-014**: System MUST provide ScrollArea component for custom scrollbars
- **FR-015**: System MUST provide DropdownMenu component for action menus
- **FR-016**: System MUST provide Toggle component for on/off switches
- **FR-017**: System MUST provide Progress component for linear progress bars
- **FR-018**: System MUST provide Alert component for notifications
- **FR-019**: System MUST provide Skeleton component for loading states
- **FR-020**: System MUST provide Collapsible component for expandable sections

#### Component Migration Mapping

##### Layout Components
- **FR-021**: `Dashboard.tsx` sidebar navigation MUST use shadcn Tooltip on icon buttons
- **FR-022**: `Dashboard.tsx` status indicators MUST use shadcn Badge and Tooltip
- **FR-023**: Page headers MUST use consistent spacing from shadcn design tokens

##### Button Components
- **FR-024**: All action buttons MUST be replaced with shadcn Button component
- **FR-025**: `LoadingButton.tsx` MUST be replaced/integrated with shadcn Button + loading prop pattern
- **FR-026**: Icon-only buttons MUST use shadcn Button variant="ghost" with Tooltip

##### Card Components  
- **FR-027**: `AgentCard` in AgentManager MUST use shadcn Card
- **FR-028**: `BatchCard` in PlanEditor MUST use shadcn Card
- **FR-029**: `TaskCard` in TaskEditor MUST use shadcn Card
- **FR-030**: Spec list items MUST use shadcn Card
- **FR-031**: `StatusWidget` MUST use shadcn Card with consistent row layout
- **FR-032**: `WorkflowPanel` pipeline stages MUST use shadcn Card

##### Modal Components
- **FR-033**: `AgentModal` MUST use shadcn Dialog
- **FR-034**: `AgentCliModal` MUST use shadcn Dialog
- **FR-035**: `TaskDetailModal` MUST use shadcn Dialog

##### Form Components
- **FR-036**: All `<input>` elements MUST use shadcn Input
- **FR-037**: All `<textarea>` elements MUST use shadcn Textarea (if installed) or styled Input
- **FR-038**: All `<select>` elements MUST use shadcn Select
- **FR-039**: `BranchSelector` in DiffViewer MUST use shadcn Select

##### Badge Components
- **FR-040**: `RiskBadge` MUST use shadcn Badge
- **FR-041**: `ModelTierBadge` MUST use shadcn Badge
- **FR-042**: `StatusBadge` (task status) MUST use shadcn Badge
- **FR-043**: `StrategyBadge` MUST use shadcn Badge
- **FR-044**: `PriorityBadge` MUST use shadcn Badge

##### View Toggle Components
- **FR-045**: `ViewToggle` in SpecEditor MUST use shadcn Tabs
- **FR-046**: `ViewToggle` in TaskEditor MUST use shadcn Tabs
- **FR-047**: View mode toggles in PlanEditor MUST use shadcn Tabs

##### Collapsible Components
- **FR-048**: `Section` component in SpecEditor MUST use shadcn Collapsible
- **FR-049**: `PhaseGroup` in TaskEditor MUST use shadcn Collapsible
- **FR-050**: Expandable batch cards MUST use shadcn Collapsible

##### Progress & Status Components
- **FR-051**: `ProgressRing` custom component MAY be retained (no shadcn equivalent) but MUST use design tokens
- **FR-052**: Linear progress indicators MUST use shadcn Progress

##### Specialty Components (Keep with Styling Updates)
- **FR-053**: `LogTerminal` (xterm.js wrapper) MUST be wrapped in shadcn Card but keep internal terminal styling
- **FR-054**: `LogViewer` MUST use shadcn Card for container and ScrollArea for scrolling
- **FR-055**: `DiffViewer` file diff visualization MAY be retained but MUST use shadcn Card containers
- **FR-056**: `DagView` visualization MAY be retained but MUST use design tokens

### Key Entities *(include if feature involves data)*

- **shadcn Component**: A pre-built, accessible React component from the shadcn/ui library that is copied into the project's `components/ui/` directory
- **Design Token**: A CSS variable defined in `index.css` that controls colors, spacing, animations, and other design values
- **Component Variant**: A pre-defined styling variation of a shadcn component (e.g., Button variants: default, destructive, outline)

---

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All interactive elements (buttons, inputs, selects) have visible focus indicators for keyboard navigation
- **SC-002**: All modals can be closed via Escape key and support screen reader announcements
- **SC-003**: Visual styling is consistent across all 16 component files (verified by visual inspection)
- **SC-004**: No `@radix-ui/themes` imports remain in the codebase after migration
- **SC-005**: Component styling matches across pages — a button on the Dashboard looks identical to a button in the Runner
- **SC-006**: Loading states use consistent spinner styling across all components
- **SC-007**: Hover states transition smoothly with consistent timing (200ms)
- **SC-008**: Existing E2E tests pass after migration (no functional regression)

---

## Component Inventory & Migration Plan

### Files to Migrate (by priority)

| File | Size | Priority | shadcn Components Needed |
|------|------|----------|--------------------------|
| `layouts/Dashboard.tsx` | 14KB | P1 | Tooltip, Badge |
| `ui/LoadingButton.tsx` | 3.5KB | P1 | Button (replace) |
| `StatusWidget.tsx` | 12KB | P1 | Card, Badge, Tooltip |
| `CommandPalette.tsx` | 24KB | P1 | Card, Button, Badge, Skeleton |
| `AgentManager.tsx` | 49KB | P2 | Card, Button, Badge, Dialog, Input, Select, Tabs, Collapsible |
| `SpecEditor.tsx` | 28KB | P2 | Card, Tabs, Collapsible, Input, Badge |
| `TaskEditor.tsx` | 33KB | P2 | Card, Tabs, Badge, Collapsible, Select |
| `PlanEditor.tsx` | 29KB | P2 | Card, Tabs, Badge, Select |
| `ExecutionRunner.tsx` | 59KB | P3 | Card, Button, Badge, Progress, ScrollArea, Skeleton |
| `WorkflowPanel.tsx` | 18KB | P3 | Card, Badge |
| `LogViewer.tsx` | 16KB | P3 | Card, Button, ScrollArea |
| `DiffViewer.tsx` | 13KB | P3 | Card, Select, Collapsible |
| `TaskDetailModal.tsx` | 27KB | P3 | Dialog, Button, Badge, Select, Tabs |
| `AgentCliModal.tsx` | 10KB | P3 | Dialog, Button |
| `CompletionSummary.tsx` | 11KB | P3 | Card, Badge |
| `RunHistoryPanel.tsx` | 8KB | P3 | Card, Badge, ScrollArea |
| `ErrorBoundary.tsx` | 5KB | P3 | Alert |
| `LogTerminal.tsx` | 6KB | P3 | Card (wrapper only) |

### shadcn Components to Install

Based on the migration mapping, install these shadcn components:

1. **button** - Core interaction
2. **card** - Content containers
3. **dialog** - Modal dialogs
4. **badge** - Status indicators
5. **input** - Form inputs
6. **select** - Dropdowns
7. **tabs** - View switching
8. **tooltip** - Helper text
9. **collapsible** - Expandable sections
10. **scroll-area** - Custom scrollbars
11. **dropdown-menu** - Action menus
12. **progress** - Progress bars
13. **skeleton** - Loading states
14. **alert** - Notifications
15. **separator** - Dividers

---

## Assumptions

1. **Tailwind v4 Compatibility**: shadcn/ui is compatible with Tailwind CSS v4 as used in this project
2. **React 19 Compatibility**: shadcn/ui works with React 19
3. **Vite Compatibility**: shadcn/ui initialization works with Vite 7
4. **No Server Components**: This is a client-side React app; no Next.js RSC considerations needed
5. **CSS Variables**: shadcn/ui will be configured to use the existing CSS variables from `index.css`
6. **xterm.js Preserved**: The terminal emulator (xterm.js) styling is kept but wrapped in shadcn containers
7. **Custom Visualizations**: DAG view and progress ring SVG components are kept but updated to use design tokens
