<!--
SYNC IMPACT REPORT
==================
Version change: none → 1.0.0
Bump rationale: Initial constitution ratification (MAJOR: new governance document)

Added Principles:
  - I. Code Quality Excellence
  - II. Testing Standards
  - III. Reliability First
  - IV. Security by Default
  - V. Deterministic CLI Behavior

Added Sections:
  - Core Principles (5 principles)
  - Development Standards
  - Quality Gates
  - Governance

Templates Updated:
  ✅ plan-template.md - Constitution Check section aligns with 5 principles
  ✅ spec-template.md - Requirements section compatible with testing/security principles
  ✅ tasks-template.md - Phase structure supports TDD and quality gates

Follow-up TODOs: None
-->

# Chakravarti CLI Constitution

## Core Principles

### I. Code Quality Excellence

Every line of code MUST meet these non-negotiable quality standards:

- **Type Safety**: All code MUST be fully typed with no `any` escape hatches unless explicitly justified in a code comment
- **Linting**: Zero linting errors permitted; all warnings MUST be addressed or explicitly suppressed with rationale
- **Self-Documenting**: Function and type names MUST be descriptive; comments reserved for "why" not "what"
- **Single Responsibility**: Each module, function, and class MUST have one clear purpose
- **No Dead Code**: Unused imports, variables, and functions MUST be removed before commit
- **Consistent Formatting**: All code MUST pass automated formatting (Prettier/ESLint) with zero manual overrides

**Rationale**: High code quality reduces debugging time, improves maintainability, and ensures agents can reliably parse and modify the codebase.

### II. Testing Standards

Testing is MANDATORY and follows strict TDD discipline:

- **Test-First Development**: Tests MUST be written before implementation; no exceptions
- **Red-Green-Refactor**: Tests MUST fail initially, pass after implementation, then code may be refactored
- **Coverage Requirements**: 
  - Unit tests: All pure functions and business logic (minimum 80% coverage)
  - Integration tests: All module boundaries and external interfaces
  - Contract tests: All CLI commands with exact input/output specifications
- **Test Isolation**: Each test MUST be independent; no shared mutable state between tests
- **Deterministic Tests**: All tests MUST produce identical results on every run; no flaky tests permitted
- **Fast Feedback**: Unit test suite MUST complete in under 10 seconds

**Rationale**: Comprehensive testing enables confident refactoring, prevents regressions, and provides living documentation of expected behavior.

### III. Reliability First

System reliability takes precedence over feature velocity:

- **Fail Fast**: Errors MUST be detected and reported at the earliest possible point
- **Explicit Error Handling**: All error paths MUST be handled explicitly; no silent failures
- **Graceful Degradation**: Partial failures MUST NOT corrupt system state
- **Idempotency**: Operations MUST be safely repeatable without side effects
- **Recovery Paths**: All operations MUST have clear rollback or recovery mechanisms
- **Observable State**: System state MUST be inspectable at any point via CLI commands

**Rationale**: Unreliable tools erode trust. Users MUST be able to depend on consistent, predictable behavior even under adverse conditions.

### IV. Security by Default

Security is built-in, not bolted-on:

- **No Secrets in Code**: API keys, tokens, and credentials MUST be injected via environment variables only
- **Minimal Permissions**: Operations MUST request only the minimum required permissions
- **Input Validation**: All external input MUST be validated and sanitized before processing
- **No Network by Default**: CLI operations MUST NOT make network calls unless explicitly requested
- **Audit Trail**: All significant operations MUST be logged for forensic analysis
- **Secure Defaults**: Default configurations MUST be the most secure option, not the most convenient
- **Isolated Execution**: Agent execution MUST be isolated via git worktrees; main branch MUST never be touched

**Rationale**: As specified in DESIGN.md, "All execution is local or on-prem. Containers isolate filesystem + network. Tool allow-list enforced. No hidden network calls."

### V. Deterministic CLI Behavior

CLI commands MUST produce predictable, reproducible results:

- **Same Input → Same Output**: Given identical inputs and state, commands MUST produce identical outputs
- **Explicit State Dependencies**: All dependencies on external state MUST be documented and controllable
- **No Hidden Side Effects**: All side effects MUST be explicitly declared in command documentation
- **Machine-Readable Output**: All commands MUST support structured output (JSON) alongside human-readable format
- **Exit Codes**: Commands MUST return meaningful exit codes (0=success, non-zero=failure with specific meaning)
- **Stderr for Errors**: Errors and diagnostics MUST go to stderr; only command output goes to stdout
- **Version-Locked Behavior**: Behavior changes require version bumps; no silent breaking changes

**Rationale**: As stated in DESIGN.md: "Prefer determinism over creativity. Prefer specs over prompts. Prefer diffs over chat."

## Development Standards

### Code Organization

- **Library-First Architecture**: Core functionality MUST be implemented as importable libraries before CLI wrappers
- **Vertical Slicing**: Features MUST be organized by domain capability, not technical layer
- **Dependency Direction**: Dependencies MUST flow inward (CLI → services → core); no circular dependencies
- **Explicit Imports**: All imports MUST be explicit; no barrel files or re-exports that obscure dependency graphs

### Documentation Requirements

- **README per Module**: Each module MUST have a README explaining purpose, usage, and examples
- **CLI Help**: All commands MUST have comprehensive `--help` output with examples
- **API Contracts**: All inter-module interfaces MUST be documented with TypeScript types or equivalent
- **Decision Records**: Significant architectural decisions MUST be recorded in `/docs/decisions/`

## Quality Gates

All code MUST pass these gates before merge:

| Gate | Requirement | Enforcement |
|------|-------------|-------------|
| Type Check | Zero type errors | `tsc --noEmit` |
| Lint | Zero errors, zero warnings | `eslint . --max-warnings 0` |
| Format | Consistent formatting | `prettier --check .` |
| Unit Tests | All pass, ≥80% coverage | `vitest run --coverage` |
| Integration Tests | All pass | `vitest run --integration` |
| Build | Clean build with no warnings | `npm run build` |
| Audit | No high/critical vulnerabilities | `npm audit --audit-level=high` |

## Governance

### Amendment Process

1. Propose changes via pull request to this constitution
2. Changes MUST include rationale and impact analysis
3. All template files MUST be updated for consistency
4. Version MUST be incremented according to semantic versioning:
   - **MAJOR**: Principle removals, fundamental governance changes
   - **MINOR**: New principles, expanded guidance, new sections
   - **PATCH**: Clarifications, typo fixes, non-semantic refinements

### Compliance

- All PRs MUST be verified against these principles before merge
- Constitution violations MUST be justified in the Complexity Tracking table of plan.md
- Automated checks SHOULD enforce as many principles as technically feasible
- Manual review MUST verify principles that cannot be automated

### Runtime Guidance

For development guidance specific to coding agents, refer to `.specify/memory/agent-guidance.md` (when created).

**Version**: 1.0.0 | **Ratified**: 2025-12-12 | **Last Amended**: 2025-12-12
