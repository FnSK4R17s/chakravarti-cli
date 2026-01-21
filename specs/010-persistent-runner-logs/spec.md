# Feature Specification: Persistent Runner Logs

**Feature Branch**: `010-persistent-runner-logs`  
**Created**: 2026-01-15  
**Status**: Draft  
**Input**: User description: "Build runner logging so I can move to some other tab and come back and see the logs later"

## Clarifications

### Session 2026-01-15

- Q: How long are logs retained before cleanup? → A: Until manually deleted by user; stored in a dedicated folder with `.gitkeep` for persistence
- Q: How to handle high-volume logs (thousands/second)? → A: Display only tail 10 logs in real-time view; full history available via scroll
- Q: Are logs available after browser closed and reopened? → A: Yes, logs from any past execution are accessible; auto-cleaned when all worktrees are merged
- Q: How to handle multiple simultaneous executions? → A: Separate log view per execution; each execution has its own log file
- Q: Scope of "navigating away"? → A: Includes both switching browser tabs AND navigating to other pages within the UI

## User Scenarios & Testing *(mandatory)*

### User Story 1 - View Logs After Navigating Away (Priority: P1)

As a user running executions in the Chakravarti UI, I want to navigate to a different page in the UI or switch to another browser tab, then return to the execution view and see all the logs that were generated while I was away.

**Why this priority**: This is the core value proposition - users should not be forced to watch logs in real-time; they should be able to multitask and review logs at their convenience.

**Independent Test**: Can be fully tested by starting an execution, navigating to another UI page (or switching browser tabs), waiting for the execution to progress, returning to the execution view, and verifying all logs from the away period are visible.

**Acceptance Scenarios**:

1. **Given** an execution is running and generating logs, **When** I navigate to another page in the UI and return after 30 seconds, **Then** I see all logs generated during my absence in chronological order
2. **Given** an execution is running and generating logs, **When** I switch to another browser tab and return after 30 seconds, **Then** I see all logs generated during my absence in chronological order
3. **Given** an execution completed while I was on another page/tab, **When** I return to the execution view, **Then** I see the complete log history including the completion status
4. **Given** an execution failed while I was on another page/tab, **When** I return to the execution view, **Then** I see all error logs and the failure state clearly indicated

---


### User Story 2 - Scroll Through Historical Logs (Priority: P2)

As a user reviewing an execution, I want to scroll through the complete log history to understand what happened at each stage of the execution.

**Why this priority**: Being able to scroll back through logs is essential for debugging and understanding execution flow, but requires log persistence to be implemented first.

**Independent Test**: Can be tested by running an execution that generates 100+ log lines, then scrolling from bottom to top and verifying all lines are accessible.

**Acceptance Scenarios**:

1. **Given** an execution has generated 500 log lines, **When** I scroll to the top of the log view, **Then** I see the first log lines from execution start
2. **Given** I am viewing logs and new logs arrive, **When** I am scrolled up reviewing older logs, **Then** the view does not auto-scroll and interrupt my reading
3. **Given** I am viewing logs and new logs arrive, **When** I am scrolled to the bottom, **Then** the view auto-scrolls to show new logs

---

### User Story 3 - Persist Logs Across Page Refresh (Priority: P3)

As a user who accidentally refreshed the page or experienced a browser crash, I want to see the logs from the execution that was running so I don't lose visibility into what happened.

**Why this priority**: While important for user experience, this extends beyond the core tab-switching use case and requires more comprehensive persistence.

**Independent Test**: Can be tested by starting an execution, refreshing the browser page, and verifying logs are still visible.

**Acceptance Scenarios**:

1. **Given** an execution is in progress, **When** I refresh the browser page, **Then** I see all logs generated before the refresh and continue receiving new logs
2. **Given** an execution completed 5 minutes ago, **When** I navigate to that execution's view, **Then** I see the complete log history

---

### Edge Cases

- ~~What happens when the log volume is extremely high (thousands of lines per second)?~~ → Resolved: Show only tail 10 logs in real-time; full history available via scroll
- ~~How does the system handle logs when the browser was closed entirely and reopened?~~ → Resolved: Logs persist on disk and are available from any past execution
- ~~What happens when multiple executions are running simultaneously?~~ → Resolved: Separate log view per execution; each has its own log file
- ~~How long are logs retained before being cleaned up?~~ → Resolved: Logs retained until manually deleted

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST persist all execution logs to storage (not just in-memory)
- **FR-002**: System MUST deliver missed logs to the UI when a user returns to the execution view
- **FR-003**: System MUST maintain chronological order of all log entries
- **FR-004**: System MUST support scrolling through the complete log history
- **FR-005**: System MUST indicate execution status (running, completed, failed) accurately when user returns
- **FR-006**: System MUST preserve logs across browser page refreshes
- **FR-007**: System MUST handle log streaming reconnection gracefully when returning to the view
- **FR-008**: System MUST store logs in a dedicated folder with `.gitkeep` file; logs retained until manually deleted or auto-cleaned when all worktrees are merged
- **FR-009**: System MUST display only the most recent 10 log entries during real-time streaming; full history accessible via scrolling up
- **FR-010**: System MUST automatically clean up logs when all associated worktrees have been merged
- **FR-011**: System MUST maintain separate log files per execution; each execution has its own isolated log view

### Key Entities

- **ExecutionLog**: A single log entry containing timestamp, log level, message content, and source (which execution step generated it)
- **Execution**: A running or completed execution that owns a collection of logs
- **LogBuffer**: The stored collection of logs for an execution, ordered chronologically

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can leave and return to an execution view within 10 minutes and see 100% of logs generated during their absence
- **SC-002**: Log view loads historical logs within 2 seconds of returning to the execution view
- **SC-003**: Users can scroll through up to 10,000 log lines without UI lag or missing entries
- **SC-004**: Zero log entries are lost due to tab switching, page refresh, or browser reconnection
