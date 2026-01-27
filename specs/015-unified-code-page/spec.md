# Feature Specification: Unified Code Page

**Feature Branch**: `015-unified-code-page`  
**Created**: 2026-01-24  
**Status**: Draft  
**Input**: User description: "build a unified code page on the UI"

## Overview

The current Chakravarti UI has 4 separate navigation pages (Specs, Tasks, Plan, Runner) that together form a linear development workflow. Users must navigate between these pages repeatedly during the code generation process, leading to context switching and a fragmented experience.

This feature consolidates these 4 pages into a single "Code" page with a tabbed or stepped interface, reducing navigation complexity from 9 sidebar items to 5 while keeping all functionality accessible within a unified context.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - View Code Workflow in Single Page (Priority: P1)

As a developer, I want to see all stages of the code generation workflow (Spec → Tasks → Plan → Run) in a single unified page so that I can work through the entire process without switching contexts.

**Why this priority**: This is the core value proposition - eliminating context switching and reducing navigation complexity. Without this, the feature has no value.

**Independent Test**: Can be fully tested by navigating to the Code page and verifying that all 4 sub-views (Spec, Tasks, Plan, Run) are accessible via tabs without leaving the page.

**Acceptance Scenarios**:

1. **Given** a user is on any page, **When** they click the "Code" navigation item, **Then** they see a unified page with tabs for Spec, Tasks, Plan, and Run
2. **Given** a user is on the Code page with Spec tab active, **When** they click on the Tasks tab, **Then** the Tasks view replaces the Spec view without a full page reload
3. **Given** a user is on the Code page, **When** they look at the sidebar, **Then** they see only 5 navigation items (Dashboard, Agents, Code, Test, QA) instead of the previous 9

---

### User Story 2 - Persist Active Tab State (Priority: P2)

As a developer, I want the Code page to remember which tab I was on so that when I navigate away and return, I'm back where I left off.

**Why this priority**: Improves user experience by reducing friction, but the page is still usable without this (users can manually click the tab they want).

**Independent Test**: Can be tested by selecting a tab, navigating to Dashboard, returning to Code page, and verifying the same tab is still active.

**Acceptance Scenarios**:

1. **Given** a user is on the Code page with Plan tab active, **When** they navigate to Dashboard then back to Code, **Then** the Plan tab is still active
2. **Given** a user has never visited the Code page, **When** they navigate to Code for the first time, **Then** the Spec tab is active by default (first stage of workflow)

---

### User Story 3 - Visual Workflow Progress Indicator (Priority: P3)

As a developer, I want to see a visual indicator of my progress through the workflow so that I know which stages are complete and which are pending.

**Why this priority**: Enhances UX and provides helpful context, but the core navigation works without it. This is a polish feature.

**Independent Test**: Can be tested by completing actions in Spec tab, then checking if visual indicators update on the tab bar.

**Acceptance Scenarios**:

1. **Given** a spec has been saved, **When** viewing the Code page tabs, **Then** the Spec tab shows a completion indicator (checkmark or different styling)
2. **Given** tasks have been generated, **When** viewing the Code page tabs, **Then** the Tasks tab shows a completion indicator
3. **Given** no plan exists yet, **When** viewing the Code page tabs, **Then** the Plan and Run tabs appear as "pending" (neutral styling)

---

### Edge Cases

- What happens when a user deep-links directly to a specific tab (e.g., `/code?tab=plan`)? The specified tab should become active.
- How does the page handle when a spec hasn't been selected yet? Show the Spec tab with the spec selector/list view.
- What happens if the user resizes the browser to mobile width? Tabs should remain accessible (consider horizontal scroll or dropdown on narrow screens).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST consolidate Specs, Tasks, Plan, and Runner views into a single "Code" page
- **FR-002**: System MUST display a tab bar with 4 tabs: Spec, Tasks, Plan, Run
- **FR-003**: System MUST allow instant switching between tabs without full page reloads
- **FR-004**: System MUST update the sidebar navigation to show 5 items: Dashboard, Agents, Code, Test, QA
- **FR-005**: System MUST remove the individual Specs, Tasks, Plan, and Runner navigation entries from the sidebar
- **FR-006**: System MUST preserve all existing functionality of SpecEditor, TaskEditor, PlanEditor, and ExecutionRunner components
- **FR-007**: System MUST default to the Spec tab when no tab preference exists
- **FR-008**: System MUST persist the active tab selection during the session (not lost on navigation)
- **FR-009**: System MUST update the page header title to reflect the current tab (e.g., "Code - Specifications", "Code - Tasks")
- **FR-010**: Tab switching MUST be keyboard accessible (arrow keys to navigate between tabs)

### Key Entities

- **CodePage**: The new unified container component that houses all 4 sub-views
- **Tab State**: Tracks which tab is currently active (spec | tasks | plan | run)
- **Workflow Stage**: Conceptual entity representing the current stage of the development workflow

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can access all spec, task, plan, and execution functionality from a single page
- **SC-002**: Navigation sidebar shows 5 items instead of 9, reducing visual clutter by 44%
- **SC-003**: Switching between workflow stages takes less than 100ms (no full page reload)
- **SC-004**: All existing component functionality (editing, saving, executing) works identically to before
- **SC-005**: Tab state persists during navigation within the same session
- **SC-006**: Page passes accessibility audit for keyboard navigation between tabs

## Assumptions

- The existing SpecEditor, TaskEditor, PlanEditor, and ExecutionRunner components are well-encapsulated and can be rendered as tab content without modification
- Users prefer a consolidated view over separate pages (validated by user's request)
- The "Code" label clearly communicates that this is where code generation workflow lives
- Tab-based navigation is a familiar UX pattern for users

## Out of Scope

- Mobile-first responsive redesign (tabs will work on mobile but no special treatment)
- Drag-and-drop tab reordering
- Collapsible sidebar
- Combining Test and QA into the Code page (these remain separate)
