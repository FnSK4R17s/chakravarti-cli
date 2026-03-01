---
last_commit: f92f604
last_updated: 2026-03-01
related_files:
  - src/lib.rs
  - src/loader.rs
  - src/validator.rs
---

# ckrv-spec

Specification loading and validation for Chakravarti.

## Overview

This crate handles loading, parsing, and validating YAML specification files. It integrates with the Spec-Kit workflow system.

## Key Types

| Type | Purpose |
|------|---------|
| `SpecLoader` | **Trait** for loading spec files |
| `YamlSpecLoader` | Default YAML implementation |
| `ValidationResult` | Validation outcome struct |
| `ValidationError` | Specific validation error |
| `SpecError` | Loading errors |

## Module Structure

```
src/
├── lib.rs         # Public exports
├── loader.rs      # SpecLoader trait + YamlSpecLoader
├── validator.rs   # validate() function
├── template.rs    # Spec templates
└── error.rs       # Error types
```

## Usage

### Loading Specs

```rust
use ckrv_spec::{SpecLoader, SpecError};
use ckrv_spec::loader::YamlSpecLoader;

let loader = YamlSpecLoader;

// Load a single spec
let spec = loader.load(Path::new("specs/012-feature/spec.yaml"))?;

// List all specs in a directory
let specs = loader.list(Path::new("specs/"))?;
```

### Validating Specs

```rust
use ckrv_spec::validator::{validate, ValidationResult};

let result = validate(&spec);

if result.valid {
    println!("Spec is valid");
} else {
    for err in &result.errors {
        eprintln!("{}: {}", err.field, err.message);
    }
}
```

## Traits

### SpecLoader

```rust
pub trait SpecLoader: Send + Sync {
    /// Load a spec from a file path.
    fn load(&self, path: &Path) -> Result<Spec, SpecError>;
    
    /// List all spec files in a directory.
    fn list(&self, dir: &Path) -> Result<Vec<PathBuf>, SpecError>;
}
```

## Validation

The `validate()` function checks:
- `id` is required and alphanumeric (with underscores/hyphens)
- `overview` is required and non-empty

```rust
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<String>,
}

pub struct ValidationError {
    pub field: String,
    pub message: String,
}
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `ckrv-core` | `Spec` type definition |
| `serde_yaml` | YAML parsing |
| `thiserror` | Error handling |
