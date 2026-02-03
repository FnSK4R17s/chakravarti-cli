# Feature Specification: Frontend Code Documentation

**Feature Branch**: `018-frontend-docs`  
**Created**: 2026-02-03  
**Status**: Draft  
**Input**: User description: "Documentation for frontend code - add @module headers, Props JSDoc, section comments, and state documentation to all React components and hooks following FRONTEND_CONVENTIONS.md patterns"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - AI Agent Understands Component Context (Priority: P1)

An AI coding agent opens a React component file and immediately understands what the component does, where it fits in the application, and how to modify it correctly without needing to read other files.

**Why this priority**: The primary goal of documentation is enabling AI agents to work effectively with the codebase. Without self-contained context, agents make mistakes and require more iterations.

**Independent Test**: Open any documented component file in isolation; an AI agent should be able to correctly modify a prop or handler without referencing external files.

**Acceptance Scenarios**:

1. **Given** a component file with @module header, **When** an AI agent reads the first 30 lines, **Then** it understands the component's purpose, where it's used, and its key dependencies.
2. **Given** a Props interface with JSDoc, **When** an AI agent needs to add a new prop, **Then** it follows the existing documentation pattern without additional guidance.
3. **Given** a component with section comments, **When** an AI agent needs to add a new effect, **Then** it places it in the correct section automatically.

---

### User Story 2 - New Developer Onboarding (Priority: P2)

A new developer joins the project and can understand the frontend architecture by reading individual component files without asking teammates for explanations.

**Why this priority**: Self-documenting code reduces onboarding time and team interruptions, but this is secondary to AI agent effectiveness.

**Independent Test**: A developer unfamiliar with the project can correctly identify what ExecutionRunner.tsx does by reading only that file.

**Acceptance Scenarios**:

1. **Given** a documented component, **When** a new developer reads it, **Then** they understand its purpose within 5 minutes.
2. **Given** documented Props interfaces, **When** a developer needs to use a component, **Then** they know which props are required vs optional and what each does.

---

### User Story 3 - Consistent Code Style (Priority: P3)

All components follow the same documentation structure, making the codebase predictable and maintainable over time.

**Why this priority**: Consistency is important but less urgent than the immediate value of having any documentation at all.

**Independent Test**: Run the `/docs.frontend` workflow verification script - all files pass module header and section comment checks.

**Acceptance Scenarios**:

1. **Given** the documentation workflow is complete, **When** running verification scripts, **Then** 100% of components have @module headers.
2. **Given** a component with >400 lines, **When** inspected, **Then** it has at least 4 section comments (STATE, EFFECTS, HANDLERS, RENDER).

---

### Edge Cases

- What happens when a component has no props? (Still needs @module header, but Props section is omitted)
- How to document inline sub-components within a file? (Sub-components get a single-line JSDoc only)
- What if a component file exports multiple components? (Each export gets its own JSDoc block)
- How to handle files that are mostly types/interfaces? (Use @module with description noting "type definitions for X")

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Every `.tsx` component file MUST have an `@module` header as the first content (before imports) containing:
  - `@module` with component name
  - `@description` explaining what the component does and why it exists
  - `@context` describing where this fits in the application architecture
  - `@dependencies` listing key imports and their purposes
  - `@example` showing the most common usage pattern (REQUIRED)
- **FR-002**: Every `.ts` hook file MUST have an `@module` header as the first content (before imports) with the same structure as FR-001
- **FR-002a**: Every custom hook MUST have JSDoc with `@param` for each argument and `@returns` describing the return value/tuple
- **FR-003**: Every Props interface MUST have a JSDoc block describing the component's purpose AND an `@example` block showing how to construct the props
- **FR-004**: Every prop in a Props interface MUST have an inline JSDoc comment describing its purpose
- **FR-005**: Optional props MUST include `@default` annotation when a default value exists
- **FR-006**: Components with >400 lines MUST have section separator comments using `// === SECTION_NAME ===` pattern (minimum: STATE, EFFECTS, HANDLERS, RENDER)
- **FR-006a**: Components with 200-400 lines SHOULD have section separator comments (recommended but not required)
- **FR-007**: Components with >5 useState calls MUST group related state with explanatory section comments
- **FR-008**: The `frontend/README.md` file MUST be updated to describe the project (not Vite boilerplate)
- **FR-009**: All documentation MUST follow patterns defined in `FRONTEND_CONVENTIONS.md`
- **FR-010**: Feature MUST be accessible via both CLI (`/docs.frontend` workflow) and manual editing with identical outcomes
- **FR-011**: Inline comments MUST explain WHY (reasoning, business logic) not WHAT (code description)
- **FR-012**: Error handling code (try/catch, error boundaries, fallbacks) MUST have comments explaining the fallback behavior and recovery strategy
- **FR-013**: Naming conventions MUST follow patterns: handlers use `handle*` prefix, boolean variables/props use `is*/has*/should*/can*` prefix

### Key Entities

- **Component File**: A `.tsx` file in `src/components/` that exports a React component
- **Hook File**: A `.ts` file in `src/hooks/` that exports a custom React hook
- **Module Header**: The `@module` JSDoc block at the top of a file containing description, context, and dependencies
- **Section Comment**: A `// ===...===` separator that divides code into logical sections

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of component files (27 files) have valid @module headers (valid = contains @module, @description, @context, @dependencies, and @example)
- **SC-002**: 100% of hook files (12 files) have valid @module headers  
- **SC-003**: 100% of Props interfaces have JSDoc documentation with @example
- **SC-004**: 100% of components >400 lines have at least 4 section comments
- **SC-005**: Frontend README contains project-specific content (not default Vite text)
- **SC-006**: Verification script (`/docs.frontend` Phase 5.6) passes with all ✅ for module headers
- **SC-007**: AI agents can correctly modify documented components on first attempt (measured by reduced clarification requests)
- **SC-008**: Feature is INCOMPLETE until 100% of files pass verification (no partial compliance accepted)

## Assumptions

- FRONTEND_CONVENTIONS.md already exists and defines the documentation patterns to follow
- The `/docs.frontend` workflow exists and will be used to apply documentation
- Component and hook file locations are fixed (`src/components/` and `src/hooks/`)
- All 27 components and 12 hooks are in scope for this feature
- Documentation is additive (existing code logic is not modified)
