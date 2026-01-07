# Feature Specification: Persistent Run History

**Feature Branch**: `008-persistent-run-history`  
**Created**: 2026-01-07  
**Status**: Draft  
**Input**: User description: "Execution Runner UI improvements with persistent run history - UI consistency with other pages, show completed runs after merge, persistent storage for run data using YAML files"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - View Execution History (Priority: P1)

As a developer using the Execution Runner, I want to see a history of past runs for a specification so that I can track progress, review completed work, and understand what has been executed previously.

**Why this priority**: This is the core value proposition - without persistent history, users lose all context when switching tabs or refreshing the page. This directly addresses the user's pain point about lost run information.

**Independent Test**: Can be fully tested by running an execution, switching tabs (or refreshing), returning to the Runner page, and verifying the run history is still visible with accurate status information.

**Acceptance Scenarios**:

1. **Given** a user has run execution for spec "build-todo-list", **When** the user navigates away and returns to the Runner page, **Then** they see the run in their history with correct status (running/completed/failed)
2. **Given** an execution completed with all batches merged, **When** the user views the spec in the Runner, **Then** the history shows "Completed ✓" with timestamp and batch summary
3. **Given** multiple runs have been executed for a spec, **When** the user selects that spec, **Then** they see all runs listed in reverse chronological order (newest first)

---

### User Story 2 - UI Consistency with Other Pages (Priority: P2)

As a developer, I want the Execution Runner page layout to match the visual style and structure of other pages (Planner, Tasks) so that the application feels cohesive and professional.

**Why this priority**: UI consistency improves user experience and reduces cognitive load, but the app is still functional without it. This is an enhancement over the core feature.

**Independent Test**: Can be tested by visual comparison of the Runner page against Planner/Tasks pages, checking header styling, spacing, panel layout, and empty states.

**Acceptance Scenarios**:

1. **Given** a user navigates to the Runner page, **When** the page loads, **Then** the header, navigation, and panel structure match other pages in the application
2. **Given** no specification is selected, **When** viewing the Runner page, **Then** the empty state message and styling are consistent with empty states on other pages
3. **Given** a specification is selected, **When** viewing batch cards and controls, **Then** the component styling (colors, shadows, spacing) follows the design system used elsewhere

---

### User Story 3 - Run Completion Summary (Priority: P2)

As a developer, I want to see a clear summary when all batches complete and code is merged so that I know the execution was successful and what was accomplished.

**Why this priority**: Provides closure and confirmation to users after potentially long-running executions. Tied closely to the history feature (P1).

**Independent Test**: Can be tested by completing a full execution run and verifying the summary panel appears with accurate information about merged branches and completed tasks.

**Acceptance Scenarios**:

1. **Given** an execution with 5 batches completes successfully, **When** all batches are merged, **Then** the UI displays a completion summary showing "5/5 batches completed, all code merged"
2. **Given** an execution completes, **When** viewing the summary, **Then** it shows total elapsed time, number of tasks completed, and branches merged
3. **Given** a partial failure (3/5 batches completed), **When** viewing the summary, **Then** it clearly indicates which batches failed and which succeeded

---

### User Story 4 - Resume or Retry Failed Runs (Priority: P3)

As a developer, I want to resume an incomplete run or retry failed batches so that I don't have to restart everything from scratch after a failure.

**Why this priority**: Quality-of-life improvement that builds on history tracking. Requires P1 to be implemented first.

**Independent Test**: Can be tested by intentionally failing a batch, then using the retry functionality to re-run only the failed batch.

**Acceptance Scenarios**:

1. **Given** a run with 2 failed batches, **When** the user clicks "Retry Failed", **Then** only the failed batches are re-executed
2. **Given** an incomplete run (user stopped it), **When** the user clicks "Resume", **Then** execution continues from where it left off

---

### Edge Cases

- What happens when the YAML history file becomes corrupted or unreadable? → System gracefully degrades to showing no history with a warning message, and allows starting fresh runs
- What happens when disk space is low and YAML cannot be written? → System shows an error toast but continues execution (history is best-effort, not critical path)
- What happens when the user deletes the spec directory while a run is in progress? → System detects missing directory and terminates the run with an appropriate error
- What happens when multiple browser tabs run the same spec simultaneously? → System shows a warning that another run is in progress and prevents duplicate execution
- What happens when history grows very large (hundreds of runs)? → System paginates or limits displayed history to most recent 50 runs, with option to view older

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST persist run execution data to a YAML file within the spec directory (e.g., `.specs/001-build-todo-list/runs.yaml`)
- **FR-002**: System MUST store for each run: unique run ID, start time, end time, status, batch statuses, and any error messages
- **FR-003**: System MUST load and display run history when a specification is selected in the Runner page
- **FR-004**: System MUST update the run YAML file in real-time as batch statuses change during execution
- **FR-005**: System MUST display a completion summary panel when all batches in a run complete (successfully or with failures)
- **FR-006**: System MUST match the visual styling of the Runner page to other application pages (header, panels, empty states, buttons)
- **FR-007**: System MUST display run history in reverse chronological order (newest first)
- **FR-008**: System MUST handle missing or corrupted history files gracefully without crashing
- **FR-009**: System MUST prevent duplicate runs of the same spec from starting simultaneously

### Key Entities

- **Run**: Represents a single execution attempt for a specification. Contains run ID, spec name, start/end timestamps, overall status, and list of batch results.
- **BatchResult**: Represents the outcome of a single batch within a run. Contains batch ID, name, status (pending/running/completed/failed), start/end times, branch name, and error message if failed.
- **RunHistory**: Collection of all runs for a specification, persisted to `runs.yaml` within the spec directory.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can view run history after refreshing the page or switching tabs (100% of runs are persisted and retrievable)
- **SC-002**: Run history loads within 1 second for specs with up to 100 historical runs
- **SC-003**: Users can identify completed vs failed runs at a glance (visual distinction is immediately clear)
- **SC-004**: 95% of users report the Runner page feels consistent with other pages in usability testing
- **SC-005**: Users can see a clear completion summary within 2 seconds of execution finishing
- **SC-006**: Storage overhead is minimal (< 50KB per run for typical executions)

## Assumptions

- YAML files are an acceptable persistence mechanism (no database required for this use case)
- The `.specs/` directory structure already exists and is well-maintained by the CLI
- Users have sufficient disk space for reasonable history (< 10MB for typical usage)
- Run history is per-specification, not global across all specs
- The existing batch and execution data structures can be serialized to YAML without modification
