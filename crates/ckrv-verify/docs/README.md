---
last_commit: 2a2da7f
last_updated: 2026-03-02
related_files:
  - src/lib.rs
  - src/runner.rs
  - src/verdict.rs
  - src/parser.rs
  - src/acceptance.rs
---

# ckrv-verify

Code verification pipeline for Chakravarti.

> [!WARNING]
> **This crate is currently NOT USED.** While declared as a dependency in `ckrv-cli`, it is not imported or called anywhere in the codebase. It was built as infrastructure for automated test verification but has not been integrated. It remains in the workspace for potential future use.

## Overview

This crate handles test execution, result parsing, and acceptance criteria checking. It runs shell commands in worktrees and parses output from various test frameworks (Cargo, npm, Python, Go).

## Key Types

| Type | Module | Purpose |
|------|--------|---------|
| `Verifier` | runner.rs | Trait for running verification |
| `DefaultVerifier` | runner.rs | Shell-based test execution |
| `VerifyConfig` | runner.rs | Worktree + spec + commands |
| `Verdict` | verdict.rs | Pass/fail result with details |
| `TestResult` | verdict.rs | Single test outcome |
| `TestStatus` | verdict.rs | Passed, Failed, Skipped, Error |
| `TestOutputParser` | parser.rs | Parses test framework output |
| `TestFramework` | parser.rs | Cargo, Npm, Python, Go, Generic |
| `AcceptanceChecker` | acceptance.rs | Checks if criteria are met |
| `AcceptanceResult` | acceptance.rs | Overall acceptance status |
| `CriterionResult` | acceptance.rs | Single criterion outcome |
| `VerifyError` | error.rs | Error types |

## Module Structure

```
src/
├── lib.rs         # Public exports
├── runner.rs      # Verifier trait + DefaultVerifier (6KB)
├── verdict.rs     # Verdict + TestResult types (5KB)
├── parser.rs      # Test output parsing (11KB)
├── acceptance.rs  # Acceptance criteria checking (5KB)
└── error.rs       # Error types
```

## Usage

### Running Tests

```rust
use ckrv_verify::{DefaultVerifier, Verifier, VerifyConfig};
use ckrv_core::Spec;

let spec = Spec::load("specs/001-my-feature")?;
let config = VerifyConfig::new("/path/to/worktree", spec)
    .with_command("cargo test")
    .with_command("cargo clippy -- -D warnings")
    .with_timeout(300);

let verifier = DefaultVerifier::new();
let verdict = verifier.verify(&config)?;

if verdict.passed {
    println!("✓ All {} tests passed in {}ms", 
        verdict.passed_count(), verdict.duration_ms);
} else {
    println!("✗ {} failed, {} passed", 
        verdict.failed_count(), verdict.passed_count());
    for log in &verdict.logs {
        eprintln!("{}", log);
    }
}
```

### Parsing Test Output

```rust
use ckrv_verify::{TestOutputParser, TestFramework};

// Auto-detect framework from command
let framework = TestOutputParser::detect_framework("cargo test");
assert_eq!(framework, TestFramework::Cargo);

// Parse output
let parser = TestOutputParser::new(TestFramework::Cargo);
let results = parser.parse(&output, success, duration_ms);

for result in results {
    println!("{}: {:?}", result.name, result.status);
}
```

### Checking Acceptance Criteria

```rust
use ckrv_verify::{AcceptanceChecker, Verdict};

let checker = AcceptanceChecker::new();
let acceptance = checker.check(&spec, &verdict);

if acceptance.passed {
    println!("All acceptance criteria met");
} else {
    for criterion in &acceptance.criteria {
        if !criterion.met {
            println!("✗ {}: {}", criterion.criterion, criterion.evidence);
        }
    }
}
```

## Supported Test Frameworks

| Framework | Detection | Parsing |
|-----------|-----------|---------|
| Cargo (Rust) | `cargo test` | Parses `test ... ok/FAILED` lines |
| npm/yarn (JS) | `npm test`, `yarn test` | Parses pass/fail counts |
| pytest (Python) | `pytest`, `python -m pytest` | Parses test output |
| go test (Go) | `go test` | Parses `--- PASS/FAIL` lines |
| Generic | (fallback) | Exit code only |

## Traits

### Verifier

```rust
pub trait Verifier: Send + Sync {
    fn verify(&self, config: &VerifyConfig) -> Result<Verdict, VerifyError>;
}
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `ckrv-core` | Spec type |
| `serde` | Serialization |
