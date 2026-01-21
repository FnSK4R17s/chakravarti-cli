---
last_commit: c1bb442
last_updated: 2026-01-21
related_files:
  - src/lib.rs
  - src/loader.rs
  - src/validator.rs
---

# ckrv-spec

Specification loading and validation for Chakravarti.

## Overview

This crate handles loading, parsing, and validating specification files. It integrates with the Spec-Kit workflow system.

## Key Types

| Type | Purpose |
|------|---------|
| `SpecLoader` | Loads spec files from disk |
| `ValidationResult` | Validation outcome |
| `ValidationError` | Specific validation error |
| `SpecError` | Loading errors |

## Usage

```rust
use ckrv_spec::{SpecLoader, ValidationResult};

let loader = SpecLoader::new();

// Load and validate a spec
let spec = loader.load("specs/012-feature/spec.md")?;
let validation = loader.validate(&spec)?;

match validation {
    ValidationResult::Valid => println!("Spec is valid"),
    ValidationResult::Invalid(errors) => {
        for err in errors {
            eprintln!("Validation error: {}", err);
        }
    }
}
```

## Module Structure

```
src/
├── loader.rs      # Spec file loading
├── template.rs    # Spec templates
├── validator.rs   # Validation logic
└── error.rs       # Error types
```

## Validation Rules

The validator checks:
- Required sections present
- User stories have acceptance criteria
- Requirements are testable
- Success criteria are measurable
- No implementation details in spec

## Dependencies

| Crate | Purpose |
|-------|---------|
| `pulldown-cmark` | Markdown parsing |
| `thiserror` | Error handling |
