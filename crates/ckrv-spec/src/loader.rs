//! Spec file loading.

// ============================================================
// IMPORTS
// ============================================================

use std::path::Path;

use ckrv_core::Spec;

use crate::SpecError;

// ============================================================
// TRAITS
// ============================================================

/// Trait for loading specification files.
pub trait SpecLoader: Send + Sync {
    /// Load a spec from a file path.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    fn load(&self, path: &Path) -> Result<Spec, SpecError>;

    /// List all spec files in a directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be read.
    fn list(&self, dir: &Path) -> Result<Vec<std::path::PathBuf>, SpecError>;
}

// ============================================================
// IMPLEMENTATION
// ============================================================

/// Default YAML spec loader.
///
/// Reads `.yaml` and `.yml` files and deserializes them into [`Spec`] instances.
pub struct YamlSpecLoader;

impl SpecLoader for YamlSpecLoader {
    fn load(&self, path: &Path) -> Result<Spec, SpecError> {
        let content =
            std::fs::read_to_string(path).map_err(|e| SpecError::ReadError(e.to_string()))?;

        let mut spec: Spec =
            serde_yaml::from_str(&content).map_err(|e| SpecError::ParseError(e.to_string()))?;

        spec.source_path = Some(path.to_path_buf());
        Ok(spec)
    }

    fn list(&self, dir: &Path) -> Result<Vec<std::path::PathBuf>, SpecError> {
        let mut specs = Vec::new();

        let entries = std::fs::read_dir(dir).map_err(|e| SpecError::ReadError(e.to_string()))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path
                .extension()
                .is_some_and(|ext| ext == "yaml" || ext == "yml")
            {
                specs.push(path);
            }
        }

        Ok(specs)
    }
}

// ============================================================
// TESTS
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_spec_file(dir: &Path, name: &str, content: &str) -> std::path::PathBuf {
        let path = dir.join(name);
        std::fs::write(&path, content).expect("write spec file");
        path
    }

    #[test]
    fn test_load_valid_yaml_spec() {
        let dir = TempDir::new().expect("temp dir");
        let content = r#"id: test_spec
overview: Test goal

constraints:
  - Constraint 1
"#;
        let path = create_spec_file(dir.path(), "test.yaml", content);

        let loader = YamlSpecLoader;
        let spec = loader.load(&path).expect("load spec");

        assert_eq!(spec.id, "test_spec");
        assert_eq!(spec.overview, Some("Test goal".to_string()));
        assert_eq!(spec.constraints.len(), 1);
    }

    #[test]
    fn test_load_sets_source_path() {
        let dir = TempDir::new().expect("temp dir");
        let content = r#"id: test
overview: Goal
"#;
        let path = create_spec_file(dir.path(), "spec.yaml", content);

        let loader = YamlSpecLoader;
        let spec = loader.load(&path).expect("load spec");

        assert_eq!(spec.source_path, Some(path));
    }

    #[test]
    fn test_load_nonexistent_file_fails() {
        let loader = YamlSpecLoader;
        let result = loader.load(Path::new("/nonexistent/path.yaml"));

        assert!(result.is_err());
        assert!(matches!(result, Err(SpecError::ReadError(_))));
    }

    #[test]
    fn test_load_invalid_yaml_fails() {
        let dir = TempDir::new().expect("temp dir");
        let content = "not: valid: yaml: [[[";
        let path = create_spec_file(dir.path(), "bad.yaml", content);

        let loader = YamlSpecLoader;
        let result = loader.load(&path);

        assert!(result.is_err());
        assert!(matches!(result, Err(SpecError::ParseError(_))));
    }

    #[test]
    fn test_list_finds_yaml_files() {
        let dir = TempDir::new().expect("temp dir");
        create_spec_file(dir.path(), "one.yaml", "id: one\noverview: G");
        create_spec_file(dir.path(), "two.yml", "id: two\noverview: G");
        create_spec_file(dir.path(), "not_a_spec.txt", "text file");

        let loader = YamlSpecLoader;
        let specs = loader.list(dir.path()).expect("list specs");

        assert_eq!(specs.len(), 2);
    }

    #[test]
    fn test_list_empty_directory() {
        let dir = TempDir::new().expect("temp dir");

        let loader = YamlSpecLoader;
        let specs = loader.list(dir.path()).expect("list specs");

        assert!(specs.is_empty());
    }

    #[test]
    fn test_list_nonexistent_directory_fails() {
        let loader = YamlSpecLoader;
        let result = loader.list(Path::new("/nonexistent/dir"));

        assert!(result.is_err());
    }
}
