---
last_commit: c1bb442
last_updated: 2026-01-21
related_files:
  - src/lib.rs
---

# ckrv-verify

Code verification for Chakravarti.

## Overview

This crate provides code verification capabilities including linting, testing, and type checking. It runs verification steps in the sandbox.

## Key Types

| Type | Purpose |
|------|---------|
| `Verifier` | Main verification interface |
| `LintResult` | Linting outcome |
| `TestResult` | Test execution outcome |
| `TypeCheckResult` | Type checking outcome |

## Verification Steps

1. **Linting**: Code style and best practices
2. **Type Checking**: Static type analysis
3. **Testing**: Unit and integration tests

## Usage

```rust
use ckrv_verify::{Verifier, VerifyOptions};

let verifier = Verifier::new(sandbox);

let result = verifier.run(VerifyOptions {
    lint: true,
    test: true,
    typecheck: true,
    fix: false,
})?;

if result.all_passed() {
    println!("Verification successful");
}
```

## Module Structure

```
src/
├── verifier.rs   # Main verification logic
├── lint.rs       # Linting implementation
├── test.rs       # Test runner
├── typecheck.rs  # Type checking
└── error.rs      # Error types
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `ckrv-sandbox` | Command execution |
