# Feature Specification: Test and QA Commands

**Feature Branch**: `014-test-qa-commands`  
**Created**: 2026-01-21  
**Status**: Draft  
**Input**: User description: "Add ckrv test and ckrv qa commands for verification using role-based agents"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Run Existing Tests (Priority: P1)

A developer wants to run the project's existing test suite in an isolated Docker sandbox to verify their changes don't break anything.

**Why this priority**: Running tests is the foundation of verification. Without this, no other testing workflow is possible.

**Independent Test**: Can be tested by running `ckrv test run` on any project with tests and verifying results are reported in Markdown.

**Acceptance Scenarios**:

1. **Given** a project with existing tests, **When** user runs `ckrv test run`, **Then** tests execute in Docker sandbox and results are displayed in Markdown format
2. **Given** a project with a Makefile, **When** user runs `ckrv test run`, **Then** system executes `make test` in sandbox
3. **Given** a Rust project without Makefile, **When** user runs `ckrv test run`, **Then** system auto-detects and runs `cargo test`

---

### User Story 2 - Plan and Write New Tests (Priority: P2)

A developer wants the test writer agent to analyze their changes and create new tests for uncovered code.

**Why this priority**: After running existing tests, developers need new tests for their changes to ensure adequate coverage.

**Independent Test**: Can be tested by making changes, running `ckrv test plan` to see proposed tests, then `ckrv test write` to generate them.

**Acceptance Scenarios**:

1. **Given** changes compared to `main` branch, **When** user runs `ckrv test plan`, **Then** system generates a test plan identifying what needs tests
2. **Given** a test plan exists, **When** user runs `ckrv test write`, **Then** test writer agent creates new tests following project conventions
3. **Given** no test writer agent configured, **When** user runs `ckrv test write`, **Then** system shows helpful error message

---

### User Story 3 - QA Code Review (Priority: P2)

A developer wants the QA agent to review their code changes for quality issues and potential bugs.

**Why this priority**: Equal to test writing - catches different types of issues (logic bugs, missing error handling, code smells).

**Independent Test**: Can be tested by making changes and running `ckrv qa review` to receive Markdown report.

**Acceptance Scenarios**:

1. **Given** changes compared to `main` branch, **When** user runs `ckrv qa review`, **Then** QA agent analyzes changes and outputs Markdown report
2. **Given** QA agent finds issues, **When** review completes, **Then** report lists issues with file locations and suggested fixes
3. **Given** no issues found, **When** review completes, **Then** report confirms code quality is acceptable

---

### User Story 4 - Generate Full QA Report (Priority: P3)

A developer wants a comprehensive QA report combining code quality analysis, bug detection, and security concerns.

**Why this priority**: Comprehensive reporting is valuable but builds on basic review functionality.

**Independent Test**: Can be tested by running `ckrv qa report` and verifying output contains all analysis sections.

**Acceptance Scenarios**:

1. **Given** changes to review, **When** user runs `ckrv qa report`, **Then** system generates comprehensive Markdown report
2. **Given** report generated, **When** user views report, **Then** it includes sections for quality, bugs, edge cases, and security

---

### Edge Cases

- What happens when no test writer agent is configured? → Show clear error with setup instructions
- What happens when no QA agent is configured? → Show clear error with setup instructions  
- What happens when there are no changes vs `main`? → Report "No changes to analyze"
- What happens when `main` branch doesn't exist? → Fall back to default branch or show error
- How does system handle test failures during `ckrv test run`? → Return exit code 1 for CI, include failures in Markdown output
- What happens when project has no tests? → Report "No existing tests found" and suggest running `ckrv test plan`

## Requirements *(mandatory)*

### Functional Requirements

**Test Runner (`ckrv test run`)**

- **FR-001**: System MUST execute tests in a Docker sandbox
- **FR-002**: System MUST auto-detect test framework (cargo test, npm test, pytest, go test, etc.)
- **FR-003**: System MUST fall back to `make test` if Makefile exists and no framework detected
- **FR-004**: System MUST output results in Markdown format
- **FR-005**: System MUST return exit code 0 on success, 1 on failure (for CI integration)

**Test Planner (`ckrv test plan`)**

- **FR-006**: System MUST compare current branch against `main` branch
- **FR-007**: System MUST identify files changed and their test coverage
- **FR-008**: System MUST generate a test plan document listing what needs tests

**Test Writer (`ckrv test write`)**

- **FR-009**: System MUST use the designated test writer agent (`is_test_writer=true`)
- **FR-010**: System MUST execute agent in Docker sandbox
- **FR-011**: System MUST generate tests following project conventions
- **FR-012**: System MUST run generated tests to verify they pass

**QA Review (`ckrv qa review`, `ckrv qa bugs`, `ckrv qa report`)**

- **FR-013**: System MUST use the designated QA agent (`is_qa_agent=true`)
- **FR-014**: System MUST analyze diff of changes vs `main` branch
- **FR-015**: System MUST identify code quality issues (complexity, duplication, naming)
- **FR-016**: System MUST identify potential bugs and missing error handling
- **FR-017**: System MUST output analysis as Markdown report

**Agent Configuration**

- **FR-018**: System MUST fail gracefully with helpful error if required agent not configured
- **FR-019**: System MUST support agents configured in Agent Manager (UI)

### Key Entities

- **TestPlan**: Document describing what tests need to be written, test locations, test strategies
- **QAReport**: Markdown document containing code quality analysis, issues found, recommendations
- **TestResult**: Outcome of test execution including pass/fail status, duration, output

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Developers can run project tests in isolated sandbox with single command
- **SC-002**: Test execution results are human-readable Markdown within 30 seconds of completion
- **SC-003**: Test writer agent generates tests that pass on first run at least 80% of the time
- **SC-004**: QA agent identifies real issues (validated by developer agreement) in at least 70% of reviews
- **SC-005**: CI pipelines can use exit codes (0/1) to gate deployments
- **SC-006**: All agent execution occurs in Docker sandbox (no local system side effects)

## Assumptions

- User has Docker installed and running
- User has configured at least one agent in Agent Manager
- Project uses a common test framework or has a Makefile with `test` target
- Git repository has a `main` branch (or another default branch)
- Changes to analyze are committed (not just staged or unstaged)
