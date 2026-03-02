//! JSON Schema generation from clap command metadata.
//!
//! This module converts clap argument definitions into JSON Schema format
//! for MCP tool input validation.

// ============================================================
// IMPORTS
// ============================================================

use ckrv_cli::{ArgumentMetadata, CommandMetadata, OptionMetadata};
use serde_json::{json, Value};

// ============================================================
// SCHEMA BUILDING
// ============================================================

/// Build a JSON Schema from command metadata.
///
/// # Arguments
///
/// * `cmd` - Command metadata containing arguments and options to convert.
#[must_use]
pub fn build_json_schema(cmd: &CommandMetadata) -> Value {
    let mut properties = serde_json::Map::new();
    let mut required = Vec::new();

    // Add positional arguments
    for arg in &cmd.arguments {
        let schema = build_argument_schema(arg);
        properties.insert(arg.id.clone(), schema);

        if arg.required {
            required.push(Value::String(arg.id.clone()));
        }
    }

    // Add options
    for opt in &cmd.options {
        let schema = build_option_schema(opt);
        properties.insert(opt.id.clone(), schema);

        // Options with takes_value but no default are effectively required
        // For safety, we don't mark any options as required
    }

    json!({
        "type": "object",
        "properties": Value::Object(properties),
        "required": required
    })
}

/// Build schema for a positional argument
fn build_argument_schema(arg: &ArgumentMetadata) -> Value {
    let json_type = type_hint_to_json_type(&arg.type_hint);

    let mut schema = serde_json::Map::new();
    schema.insert("type".to_string(), Value::String(json_type));

    if !arg.help.is_empty() {
        schema.insert("description".to_string(), Value::String(arg.help.clone()));
    }

    Value::Object(schema)
}

/// Build schema for an option
fn build_option_schema(opt: &OptionMetadata) -> Value {
    let json_type = if opt.takes_value {
        type_hint_to_json_type(&opt.value_type)
    } else {
        "boolean".to_string()
    };

    let mut schema = serde_json::Map::new();
    schema.insert("type".to_string(), Value::String(json_type));

    if !opt.help.is_empty() {
        schema.insert("description".to_string(), Value::String(opt.help.clone()));
    }

    if let Some(default) = &opt.default {
        schema.insert("default".to_string(), Value::String(default.clone()));
    }

    Value::Object(schema)
}

/// Convert clap type hint to JSON Schema type
fn type_hint_to_json_type(type_hint: &str) -> String {
    match type_hint.to_uppercase().as_str() {
        "FLAG" | "BOOL" | "BOOLEAN" => "boolean".to_string(),
        "NUMBER" | "INTEGER" | "INT" | "I32" | "U32" | "I64" | "U64" => "integer".to_string(),
        "FLOAT" | "F32" | "F64" => "number".to_string(),
        // Default everything else to string (PATH, URL, STRING, etc.)
        _ => "string".to_string(),
    }
}

// ============================================================
// TESTS
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_hint_to_json_type() {
        assert_eq!(type_hint_to_json_type("STRING"), "string");
        assert_eq!(type_hint_to_json_type("PATH"), "string");
        assert_eq!(type_hint_to_json_type("FLAG"), "boolean");
        assert_eq!(type_hint_to_json_type("INTEGER"), "integer");
        assert_eq!(type_hint_to_json_type("NUMBER"), "integer");
        assert_eq!(type_hint_to_json_type("FLOAT"), "number");
    }

    #[test]
    fn test_build_argument_schema() {
        let arg = ArgumentMetadata {
            id: "description".to_string(),
            help: "Feature description".to_string(),
            required: true,
            type_hint: "STRING".to_string(),
        };

        let schema = build_argument_schema(&arg);

        assert_eq!(schema["type"], "string");
        assert_eq!(schema["description"], "Feature description");
    }

    #[test]
    fn test_build_option_schema_flag() {
        let opt = OptionMetadata {
            id: "force".to_string(),
            long: Some("force".to_string()),
            short: Some('f'),
            help: "Force operation".to_string(),
            takes_value: false,
            value_type: "FLAG".to_string(),
            default: None,
        };

        let schema = build_option_schema(&opt);

        assert_eq!(schema["type"], "boolean");
        assert_eq!(schema["description"], "Force operation");
    }

    #[test]
    fn test_build_json_schema() {
        let cmd = CommandMetadata {
            path: vec!["ckrv".to_string(), "spec".to_string(), "new".to_string()],
            name: "new".to_string(),
            description: "Create a new spec".to_string(),
            long_description: None,
            after_help: None,
            arguments: vec![ArgumentMetadata {
                id: "description".to_string(),
                help: "Feature description".to_string(),
                required: true,
                type_hint: "STRING".to_string(),
            }],
            options: vec![],
            hidden: false,
            subcommands: vec![],
        };

        let schema = build_json_schema(&cmd);

        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["description"].is_object());
        assert_eq!(schema["required"][0], "description");
    }
}
