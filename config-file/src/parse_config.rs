use std::path::PathBuf;

use serde_yaml::{Mapping, Value};

use crate::{ConfigFile, GenerateConfig, GenerateMode};

/// Parse config file from given string.
/// Returns None if there is a validation error.
pub fn parse_config(source: &str) -> Option<ConfigFile> {
    let parsed: Value = serde_yaml::from_str(&source).ok()?;

    read_config(parsed)
}

fn read_config(config: Value) -> Option<ConfigFile> {
    let schema = 'schema: {
        let schema = config.get("schema");
        let Some(schema) = schema else {
            break 'schema None;
        };
        if let Some(string) = schema.as_str() {
            break 'schema Some(vec![string.to_owned()]);
        }
        if let Some(seq) = schema.as_sequence() {
            let strs: Option<Vec<String>> = seq
                .iter()
                .map(|value| value.as_str().map(|s| s.to_owned()))
                .collect();
            let strs = strs?;
            break 'schema Some(strs);
        }
        None
    };
    let documents = 'documents: {
        let documents = config.get("documents");
        let Some(documents) = documents else {
            break 'documents None;
        };
        if let Some(string) = documents.as_str() {
            break 'documents Some(vec![string.to_owned()]);
        }
        if let Some(seq) = documents.as_sequence() {
            let strs: Option<Vec<String>> = seq
                .iter()
                .map(|value| value.as_str().map(|s| s.to_owned()))
                .collect();
            let strs = strs?;
            break 'documents Some(strs);
        }
        None
    };
    let extensions = config
        .get("extensions")
        .and_then(|e| e.get("nitrogql"))
        .and_then(|e| e.as_mapping());
    let generate = extensions.map(generate_config).unwrap_or_default();
    Some(ConfigFile {
        schema,
        documents,
        generate,
    })
}

/// Reads extensions.generate config.
fn generate_config(extensions: &Mapping) -> GenerateConfig {
    let mut config = GenerateConfig::default();
    let Some(generate) = extensions.get("generate") else {
        return config;
    };

    if let Some(schema_output) = generate
        .get("schema-output")
        .and_then(|path| path.as_str())
        .map(PathBuf::from)
    {
        config.schema_output = Some(schema_output);
    }
    if let Some(mode) = generate
        .get("mode")
        .and_then(|v| v.as_str())
        .and_then(GenerateMode::from_str)
    {
        config.mode = mode;
    }
    if let Some(default_export_for_operation) = generate
        .get("defaultExportForOperation")
        .and_then(|v| v.as_bool())
    {
        config.default_export_for_operation = default_export_for_operation;
    }

    config
}
