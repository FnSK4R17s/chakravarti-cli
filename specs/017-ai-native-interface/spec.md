# Feature Specification: AI-Native Interface Layer

**Feature Branch**: `017-ai-native-interface`  
**Created**: 2026-01-29  
**Status**: Draft  
**Input**: User description: "Auto-generate SKILL.md from clap and MCP server for AI agent integration"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - AI Agent Discovers CLI Commands (Priority: P1)

An AI coding agent (Claude, Cursor, Copilot) working on a project that uses Chakravarti needs to understand what CLI commands are available. The agent reads the auto-generated SKILL.md file from `.agent/skills/chakravarti-cli/SKILL.md` and immediately understands how to use `ckrv` commands without hallucinating incorrect syntax or options.

**Why this priority**: This is the foundation - without accurate command documentation, AI agents will make mistakes when using the CLI. SKILL.md enables any AI agent to use Chakravarti correctly.

**Independent Test**: Generate SKILL.md, have an AI agent read it, and verify it correctly uses `ckrv spec list` and `ckrv plan` commands.

**Acceptance Scenarios**:

1. **Given** a developer runs `make skill`, **When** the command completes, **Then** a valid SKILL.md file exists at `.agent/skills/chakravarti-cli/SKILL.md` that passes `agentskills validate`
2. **Given** a new CLI command is added to ckrv-cli, **When** `make skill` is run, **Then** the new command appears in SKILL.md without manual editing
3. **Given** an AI agent reads SKILL.md, **When** asked to create a spec, **Then** it uses the correct `ckrv spec new "description"` syntax

---

### User Story 2 - MCP-Compatible Agent Calls Chakravarti Directly (Priority: P2)

A user has Claude Desktop configured with Chakravarti's MCP server. When they ask Claude to "create a spec for user authentication", Claude calls the `ckrv_spec_new` MCP tool directly, without needing to explain CLI syntax. The spec is created in the correct location.

**Why this priority**: MCP provides a richer integration than CLI - tools have structured inputs/outputs, reducing errors. However, this depends on P1 (same command metadata).

**Independent Test**: Start MCP server, connect MCP Inspector, call `tools/list` and verify all CLI commands appear as tools. Call `ckrv_spec_list` and verify it returns spec list.

**Acceptance Scenarios**:

1. **Given** ckrv-mcp server is running, **When** an MCP client sends `initialize` request, **Then** server responds with valid capabilities
2. **Given** ckrv-mcp server is running, **When** client calls `tools/list`, **Then** response includes tools for all non-hidden CLI commands (init, spec, plan, run, etc.)
3. **Given** ckrv-mcp server is running, **When** client calls `ckrv_spec_list` tool, **Then** response contains list of specs in JSON format

---

### User Story 3 - Zero-Maintenance Documentation (Priority: P1)

A maintainer adds a new `ckrv deploy` command to the CLI. After making the code change and running `make skill`, the new command automatically appears in SKILL.md with correct description, arguments, and options - without any manual documentation updates.

**Why this priority**: The entire value proposition is zero-maintenance. If maintainers have to manually update docs, adoption will fail.

**Independent Test**: Add a mock command, run generation, verify it appears. Remove mock command, run generation, verify it disappears.

**Acceptance Scenarios**:

1. **Given** a new command is added using clap derive macros, **When** `make skill` is executed, **Then** the command appears in SKILL.md with name, description, and all options
2. **Given** a command's `#[command(about = "...")]` is modified, **When** `make skill` is executed, **Then** SKILL.md reflects the updated description
3. **Given** a command is marked `#[command(hide = true)]`, **When** `make skill` is executed, **Then** the command does NOT appear in SKILL.md

---

### User Story 4 - Claude Desktop Integration (Priority: P2)

A user installs Chakravarti and wants to use it with Claude Desktop. They run `make install-mcp` which builds the MCP server and prints the exact JSON configuration to add to their Claude Desktop settings.

**Why this priority**: Easy installation is critical for adoption, but depends on MCP server being complete (P2).

**Independent Test**: Run `make install-mcp`, copy the printed JSON to Claude Desktop config, restart Claude, verify Chakravarti tools appear.

**Acceptance Scenarios**:

1. **Given** user runs `make install-mcp`, **When** command completes, **Then** output includes valid JSON for Claude Desktop `mcpServers` config
2. **Given** user adds config to Claude Desktop, **When** Claude starts, **Then** user can ask Claude to "list my chakravarti specs" and get a response

---

### Edge Cases

- What happens when SKILL.md generation is run but no commands exist? → Generate minimal valid SKILL.md with frontmatter only
- What happens when MCP client sends malformed JSON-RPC? → Server returns proper error response, does not crash
- What happens when MCP tool execution fails (e.g., `ckrv` not in PATH)? → Return structured error with helpful message
- What happens when a command has no arguments or options? → Generate command entry with just name and description
- What happens when subcommand names conflict with JSON keys? → Use underscore-separated names (e.g., `ckrv_spec_new`)

## Requirements *(mandatory)*

### Functional Requirements

#### SKILL.md Generation (skill_gen binary)

- **FR-001**: System MUST generate SKILL.md from clap `Command` metadata using `Cli::command()` introspection
- **FR-002**: System MUST include all non-hidden commands in SKILL.md
- **FR-003**: System MUST exclude commands marked with `#[command(hide = true)]`
- **FR-004**: System MUST generate valid Agent Skills frontmatter (name, description, metadata)
- **FR-005**: System MUST include command descriptions from `#[command(about = "...")]` or `///` doc comments
- **FR-006**: System MUST include argument names and help text for each command
- **FR-007**: System MUST include option flags with descriptions in table format
- **FR-008**: System MUST output to stdout for piping to file
- **FR-009**: Generated SKILL.md MUST pass `agentskills validate`

#### MCP Server (ckrv-mcp crate)

- **FR-010**: System MUST implement MCP protocol over stdio transport (JSON-RPC 2.0)
- **FR-011**: System MUST handle `initialize` method and return server capabilities
- **FR-012**: System MUST handle `tools/list` method and return all CLI commands as tools
- **FR-013**: System MUST handle `tools/call` method and execute the specified command
- **FR-014**: System MUST generate tool names from CLI commands (e.g., `ckrv spec new` → `ckrv_spec_new`)
- **FR-015**: System MUST generate JSON Schema for each tool's input based on clap arguments
- **FR-016**: System MUST execute commands by shelling out to `ckrv --json`
- **FR-017**: System MUST parse JSON output from CLI and return as tool result
- **FR-018**: System MUST include proper annotations (readOnlyHint, destructiveHint) based on command nature

#### Build Integration

- **FR-019**: Makefile MUST include `skill` target that generates SKILL.md and validates it
- **FR-020**: Makefile MUST include `mcp` target that builds the MCP server
- **FR-021**: Makefile MUST include `install-mcp` target that prints Claude Desktop config

#### CLI/UI Parity

- **FR-022**: SKILL.md and MCP tools MUST reflect the same commands available via CLI
- **FR-023**: SKILL.md and MCP tools MUST NOT expose UI-only functionality (web dashboard routes)

### Key Entities

- **Tool**: An MCP tool representing a CLI command, with name, description, and input schema
- **SKILL.md**: A markdown file following Agent Skills specification with command documentation
- **Command Metadata**: Information extracted from clap including name, about, arguments, and options

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: SKILL.md generation completes in under 2 seconds
- **SC-002**: Generated SKILL.md passes `agentskills validate` on every run
- **SC-003**: Adding a new CLI command requires zero manual documentation changes for it to appear in SKILL.md
- **SC-004**: MCP server responds to `initialize` within 100ms
- **SC-005**: MCP `tools/call` execution completes within 5 seconds for typical operations
- **SC-006**: 100% of non-hidden CLI commands appear as MCP tools
- **SC-007**: AI agents using SKILL.md can correctly invoke 95% of commands without syntax errors
- **SC-008**: MCP server handles 10 concurrent tool calls without errors

## Assumptions

- clap derive macros are used consistently across all CLI commands
- All commands support `--json` flag for structured output
- `ckrv` binary is available in PATH when MCP server runs
- Agent Skills format remains stable (current specification)
- MCP protocol stdio transport is supported by target clients (Claude Desktop)
- Hidden commands (`#[command(hide = true)]`) are intentionally excluded from external interfaces
