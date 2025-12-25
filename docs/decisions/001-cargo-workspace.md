# ADR 001: Cargo Workspace Structure

## Status

Accepted

## Context

We need to structure the Chakravarti CLI codebase for maintainability, testability, and clear separation of concerns. The project involves multiple subsystems:

- CLI interface
- Core orchestration logic
- Specification parsing
- LLM provider integration
- Git operations
- Sandbox execution
- Verification
- Metrics tracking

## Decision

We will use a Cargo workspace with multiple crates, each responsible for a distinct domain:

```
chakravarti-cli/
├── Cargo.toml          # Workspace root
└── crates/
    ├── ckrv-cli/       # Binary crate - CLI interface
    ├── ckrv-core/      # Library - Core types, orchestrator
    ├── ckrv-spec/      # Library - Spec parsing
    ├── ckrv-model/     # Library - LLM providers
    ├── ckrv-git/       # Library - Git operations
    ├── ckrv-sandbox/   # Library - Execution isolation
    ├── ckrv-verify/    # Library - Test/lint verification
    └── ckrv-metrics/   # Library - Cost/time tracking
```

### Naming Convention

- All crates use `ckrv-` prefix for namespace consistency
- Short, descriptive names reflecting primary responsibility
- Snake_case for crate names (Rust convention)

### Dependency Direction

```
ckrv-cli
  └── ckrv-core
        ├── ckrv-spec
        ├── ckrv-model
        ├── ckrv-git
        ├── ckrv-sandbox
        ├── ckrv-verify
        └── ckrv-metrics
```

Each library crate is independent and can be tested in isolation.

## Consequences

### Positive

- **Clear boundaries**: Each crate has a single responsibility
- **Independent testing**: Unit tests for each crate in isolation
- **Parallel compilation**: Cargo can compile crates in parallel
- **Selective dependencies**: Only include what you need
- **Easier refactoring**: Changes are localized to specific crates

### Negative

- **More boilerplate**: Each crate needs its own Cargo.toml
- **Version coordination**: All crates must be versioned together
- **Build complexity**: Workspace features need coordination

### Neutral

- Learning curve for new contributors understanding crate relationships
- Integration tests span multiple crates

## Alternatives Considered

### Single Crate with Modules

Simpler structure but leads to tight coupling and longer compile times.

### Separate Repositories

Maximum independence but complicates development workflow and version coordination.

## References

- [Cargo Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html)
- [The Rust Programming Language - Workspaces](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html)
