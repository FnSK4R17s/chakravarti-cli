# Feature Specification: Execution State Synchronization Fix

## Overview

Fix critical bugs in the Execution Runner where runs appear to run endlessly (timer stuck at 0s) and batch completion is not being tracked (0/14 batches completed despite batches showing as complete).

## Problem Statement

The Execution Runner UI shows:
- **0/14 BATCHES COMPLETED** even though individual batches show "✓ Task Complete"
- **0s ELAPSED TIME** - timer never starts despite execution being in progress
- Batches stuck in "Merging branch..." or "Spawning batch..." state indefinitely

### Root Cause

There is a **message format mismatch** between the backend (Rust) and frontend (TypeScript):

1. **Starting Status**: Backend sends `{ type_: "start", message: "..." }` but frontend expects `{ type: "status", status: "running" }` to transition from 'starting' to 'running'

2. **Batch Completion**: Backend sends `{ type_: "batch_complete", message: "Batch X completed on branch Y" }` but frontend regex expects patterns like:
   - `"Mission completed: <batch_name>"`
   - `"Successfully merged batch '<batch_name>'"`

3. **Overall Completion**: Backend sends `{ type_: "success", message: "All batches completed..." }` but frontend expects `{ type: "status", status: "completed" }`

## User Stories

### US1: Timer and Progress Tracking
**As a** developer running batch executions  
**I want** the elapsed timer to start when execution begins  
**So that** I can track how long the execution has been running

### US2: Batch Completion Tracking
**As a** developer monitoring execution progress  
**I want** the completion counter (X/14 BATCHES COMPLETED) to update in real-time  
**So that** I can see actual progress through the batches

### US3: Execution Status Accuracy
**As a** developer  
**I want** the execution status (running/completed/failed) to accurately reflect the backend state  
**So that** I know when execution is truly complete or has failed

## Functional Requirements

### FR1: Backend Message Format Standardization
- Backend MUST send `{ type: "status", status: "running" }` when execution starts
- Backend MUST send `{ type: "status", status: "completed" }` when all batches complete
- Backend MUST send `{ type: "status", status: "failed" }` when execution fails
- Backend SHOULD include batch state updates in a consistent format

### FR2: Frontend Message Handling Enhancement
- Frontend MUST handle both legacy log message patterns AND new status messages
- Frontend SHOULD update batch state based on `type: "batch_start"` and `type: "batch_complete"` messages
- Frontend MUST start timer when receiving 'running' status

### FR3: Robust Pattern Matching
- Frontend batch parsing MUST handle multiple message formats:
  - `"Spawning batch: <name>"` → batch running
  - `"Batch <id> completed on branch <branch>"` → batch completed
  - `"Mission completed: <name>"` → batch completed (legacy)
  - `"Successfully merged batch '<name>'"` → batch completed (legacy)

## Non-Functional Requirements

### NFR1: Backward Compatibility
- Must maintain compatibility with existing log message formats
- Must not break existing functionality

### NFR2: Real-time Updates
- UI must update within 100ms of receiving WebSocket messages
- Progress counter must be accurate to current backend state

## Technical Constraints

- Backend: Rust (axum, tokio)
- Frontend: React, TypeScript
- Communication: WebSocket with JSON messages
- Must not require database changes

## Acceptance Criteria

1. [ ] Elapsed timer starts counting when execution begins
2. [ ] Batch completion counter accurately reflects completed batches
3. [ ] Overall execution status transitions correctly through states
4. [ ] All existing functionality continues to work
5. [ ] No console errors related to message parsing
6. [ ] Batch/agent status can be verified via `docker ps` (ground truth)

## Out of Scope

- Changes to execution engine logic → **Tracked in Feature 008** (Docker migration)
- New batch retry functionality
- Performance optimizations beyond this fix

## Clarifications

### Session 2026-01-06

- Q: Should we use `docker ps` to check agent status? → A: Yes, batch execution should ALWAYS be in Docker; agents run only in Docker containers
- Q: Should Docker migration be part of this feature? → A: Split into two features (007 = quick message fix, 008 = Docker migration)

### Architectural Requirement (Clarified)

**Important**: The current implementation runs batches as local subprocesses (`ckrv task`). This violates the architectural requirement that **agents MUST run in Docker containers**.

**Decision**: Split into two features:
1. **Feature 007** (this feature): Quick message sync fix for immediate UX improvement
2. **Feature 008** (new feature): Migrate batch execution to Docker containers with `docker ps` monitoring

### Related Feature

See `specs/008-docker-batch-execution/spec.md` for the Docker migration work.


