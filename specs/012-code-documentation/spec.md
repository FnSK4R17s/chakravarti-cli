# Feature Specification: Comprehensive Code Documentation

**Feature Branch**: `012-code-documentation`  
**Created**: 2026-01-21  
**Status**: Draft  
**Input**: User description: "create docs for all code"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Discovering Crate Purpose (Priority: P1)

A developer joining the project navigates to any crate directory and immediately understands what the crate does, its key modules, and how to use it by reading the README or inline docs.

**Why this priority**: First impressions matter. New contributors need to quickly orient themselves without reading every source file.

**Independent Test**: Can be tested by having a new developer locate and understand the purpose of `ckrv-core` crate within 5 minutes using only documentation.

**Acceptance Scenarios**:

1. **Given** a developer new to the codebase, **When** they open `crates/ckrv-core/`, **Then** they find a README explaining: purpose, key types/traits, usage examples, and dependencies.
2. **Given** a developer new to the codebase, **When** they run `cargo doc --open`, **Then** every public module has crate-level and module-level doc comments visible in rustdoc.

---

### User Story 2 - Understanding CLI Commands (Priority: P1)

A user wanting to extend the CLI locates documentation explaining each command's purpose, options, and implementation pattern.

**Why this priority**: The CLI is the main user interface. Understanding command structure enables both users and contributors.

**Independent Test**: A developer can implement a new CLI command by following documented patterns without asking for help.

**Acceptance Scenarios**:

1. **Given** a developer wants to add a new command, **When** they read the CLI documentation, **Then** they find a guide explaining command structure, registration, and argument handling patterns.
2. **Given** a user runs `ckrv <command> --help`, **When** they read the output, **Then** all options have clear descriptions and examples.

---

### User Story 3 - Understanding Agent System (Priority: P2)

A developer wanting to add a new agent type (e.g., Cursor, Codex) finds documentation explaining the agent trait interface and integration steps.

**Why this priority**: Agent extensibility is a core feature. Clear docs enable community contributions.

**Independent Test**: Developer can implement a minimal agent adapter by following the agent interface documentation.

**Acceptance Scenarios**:

1. **Given** a developer wants to add agent support, **When** they read the sandbox crate docs, **Then** they understand the `Agent` trait, required methods, and configuration.
2. **Given** the agent system has multiple implementations, **When** viewing docs, **Then** examples show how Claude, Codex, and OpenRouter agents differ.

---

### User Story 4 - Architecture Reference (Priority: P2)

A developer needing to understand the overall system architecture finds a comprehensive architecture document with diagrams and data flow explanations.

**Why this priority**: High-level understanding accelerates deep dives into specific areas.

**Independent Test**: Developer can explain the spec→plan→execute→merge flow after reading architecture docs.

**Acceptance Scenarios**:

1. **Given** a new team member, **When** they read the architecture docs, **Then** they understand how specs, plans, jobs, and worktrees relate.
2. **Given** the architecture docs, **When** viewed, **Then** they include a diagram showing crate dependencies and data flow.

---

### User Story 5 - API Reference for UI Development (Priority: P3)

A frontend developer extending the Web UI finds documentation for all API endpoints, WebSocket events, and data structures.

**Why this priority**: UI development requires stable API contracts. Docs prevent guesswork.

**Independent Test**: Frontend developer can add a new UI feature using only API documentation.

**Acceptance Scenarios**:

1. **Given** a frontend developer, **When** they read the UI API docs, **Then** they find all REST endpoints with request/response examples.
2. **Given** WebSocket functionality, **When** documented, **Then** all event types and payload structures are described.

---

### Edge Cases

- What happens when a crate has no public API (internal only)?
- How are deprecated functions/modules documented?
- What about undocumented third-party integrations?
- What happens when code changes but docs are not updated?

## Documentation Structure

### Folder Hierarchy

```
crates/
├── docs/                    # Top-level cross-crate documentation
│   ├── architecture.md      # System architecture and crate dependencies
│   ├── getting-started.md   # Onboarding guide for new contributors
│   ├── cli-commands.md      # CLI command reference
│   └── agent-guide.md       # Guide for adding new agents
│
├── ckrv-cli/
│   └── docs/                # CLI-specific documentation
│       ├── README.md        # Crate overview and usage
│       └── commands/        # Per-command detailed docs
│
├── ckrv-core/
│   └── docs/                # Core crate documentation
│       ├── README.md        # Crate overview
│       └── concepts.md      # Domain concepts (Spec, Plan, Job, etc.)
│
├── ckrv-sandbox/
│   └── docs/                # Sandbox crate documentation
│       ├── README.md        # Crate overview
│       └── agents.md        # Agent implementations and extensibility
│
└── [other-crates]/
    └── docs/
        └── README.md        # Each crate has at minimum a README
```

### Git Commit Hash Tracking

Every documentation file MUST begin with a frontmatter block containing the last git commit hash when the doc was last updated. This enables:

1. **Staleness Detection**: Coding agents can compare the doc hash against recent commits to identify outdated docs
2. **Change Correlation**: Easy lookup of what code changes prompted doc updates
3. **Automated Freshness Checks**: Scripts can flag docs that haven't been updated after related code changes

**Format**:

```markdown
---
last_commit: abc1234
last_updated: 2026-01-21
related_files:
  - src/lib.rs
  - src/agent/mod.rs
---

# Document Title
...
```

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Every crate MUST have a crate-level doc comment (`//!`) explaining its purpose
- **FR-002**: Every public module MUST have module-level doc comments explaining its role
- **FR-003**: Every public function/method MUST have doc comments with at least: purpose, parameters, return value, and a usage example
- **FR-004**: Every public struct/enum MUST have doc comments explaining its purpose and fields
- **FR-005**: Every crate MUST have a README.md with: overview, key types, usage examples, and crate dependencies
- **FR-006**: A `crates/docs/` folder MUST exist containing top-level cross-crate documentation
- **FR-007**: Each crate MUST have a `docs/` subfolder with at least a README.md
- **FR-008**: Every documentation file MUST begin with a frontmatter block containing the last git commit hash
- **FR-009**: The CLI commands MUST be documented in `crates/docs/cli-commands.md`
- **FR-010**: The agent system MUST have a developer guide at `crates/docs/agent-guide.md`
- **FR-011**: The Web UI API endpoints MUST be documented in `crates/ckrv-ui/docs/api-reference.md`
- **FR-012**: All documentation MUST pass `cargo doc --deny warnings` with no warnings
- **FR-013**: The frontmatter MUST include `last_commit`, `last_updated`, and optionally `related_files`

### Key Entities

- **Top-Level Docs**: `crates/docs/` - Cross-crate architecture, guides, and reference docs
- **Crate Docs**: `crates/<crate>/docs/` - Detailed per-crate documentation
- **Rustdoc Comments**: Inline `//!` and `///` comments in source files
- **Frontmatter**: YAML header with git commit tracking metadata
- **API Reference**: REST endpoint and WebSocket event documentation

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of public crate APIs have rustdoc comments (verified by `cargo doc --deny warnings`)
- **SC-002**: All 10 crates have `docs/README.md` files with overview and usage examples
- **SC-003**: `crates/docs/` contains at least: `architecture.md`, `getting-started.md`, `cli-commands.md`, `agent-guide.md`
- **SC-004**: 100% of documentation files have valid frontmatter with `last_commit` hash
- **SC-005**: New contributors can complete onboarding within 2 hours using only docs
- **SC-006**: Architecture documentation includes at least 2 diagrams (crate dependencies, execution flow)
- **SC-007**: `cargo doc --open` generates navigable documentation for all workspace crates

## Assumptions

- Documentation will follow Rust documentation conventions (rustdoc)
- Diagrams will be created using Mermaid or similar markdown-compatible format
- Existing README.md provides baseline content to expand upon
- Documentation will be maintained alongside code in the same repository
