use std::{
    io::Write,
    process::{Command, Stdio},
};

use ratconfig::toml_adapter::{
    get_toml_path, parse_toml_value, set_toml_value_text, unset_toml_value_text,
};
use ratconfig::{
    ConfigUiApplyStatus, ConfigUiCapability, ConfigUiChoice, ConfigUiDiagnostic, ConfigUiField,
    ConfigUiFieldSpec, ConfigUiTextEncoding,
};
use serde_json::Value;

use crate::{catalog::*, common::*, paths::ConfigPaths};

// RIO-CONFIG-UI-001: Rio owns both the inventory and native validation.
fn inventory(paths: &ConfigPaths, raw: &str) -> Result<Vec<Value>> {
    let command = paths
        .rio_command
        .as_ref()
        .ok_or_else(|| error("Packaged Rio configuration controls are unavailable."))?;
    let mut child = Command::new(command)
        .arg("--config-editor")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let written = child
        .stdin
        .take()
        .expect("piped stdin")
        .write_all(raw.as_bytes());
    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(error(format!(
            "Rio rejected the configuration: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    written?;
    let value = parse_toml_value(std::str::from_utf8(&output.stdout)?)
        .map_err(|source| boxed_debug("invalid Rio editor inventory", source))?;
    if value.get("version").and_then(Value::as_u64) != Some(1) {
        return Err(error("Unsupported Rio editor inventory version."));
    }
    value
        .get("fields")
        .and_then(Value::as_array)
        .cloned()
        .ok_or_else(|| error("Rio editor inventory has no fields."))
}

pub(crate) fn build_rio_fields(
    paths: &ConfigPaths,
) -> (Vec<ConfigUiField>, Vec<ConfigUiDiagnostic>) {
    if !paths.rio_included || paths.rio_command.is_none() {
        return (Vec::new(), Vec::new());
    }
    let result = (|| {
        let raw = read_optional_text(&paths.rio)?;
        let specs = inventory(paths, &raw)?;
        let active =
            parse_toml_value(&raw).map_err(|source| boxed_debug("invalid Rio TOML", source))?;
        specs
            .iter()
            .map(|spec| build_field(spec, &active))
            .collect::<Result<Vec<_>>>()
    })();
    match result {
        Ok(fields) => (fields, Vec::new()),
        Err(source) => (
            Vec::new(),
            vec![invalid_source_diagnostic(
                "rio/config.toml",
                SOURCE_RIO,
                format!("{source} Open the full configuration to repair it."),
            )],
        ),
    }
}

fn build_field(spec: &Value, active: &Value) -> Result<ConfigUiField> {
    let text = |key| {
        spec.get(key)
            .and_then(Value::as_str)
            .ok_or_else(|| error(format!("Rio editor field is missing {key}.")))
    };
    let path = text("path")?;
    let kind = text("kind")?;
    let capability = match kind {
        "boolean" => ConfigUiCapability::Toggle {
            off: ConfigUiChoice::new(Value::Bool(false)),
            on: ConfigUiChoice::new(Value::Bool(true)),
        },
        "choice" => ConfigUiCapability::Choice {
            choices: spec
                .get("choices")
                .and_then(Value::as_array)
                .ok_or_else(|| error("Rio editor choice has no values."))?
                .iter()
                .cloned()
                .map(ConfigUiChoice::new)
                .collect(),
        },
        "string" => ConfigUiCapability::FreeText {
            encoding: ConfigUiTextEncoding::String,
        },
        "number" => ConfigUiCapability::FreeText {
            encoding: ConfigUiTextEncoding::Json,
        },
        _ => return Err(error(format!("Unsupported Rio editor field kind: {kind}"))),
    };
    let current = get_toml_path(active, path);
    let mut field = ConfigUiFieldSpec {
        can_unset: current.is_some(),
        ..ConfigUiFieldSpec::new(SOURCE_RIO, path, TAB_RIO, text("description")?, capability,
            "Rio native TOML parser; font and theme availability are checked by Rio at launch",
            ConfigUiApplyStatus { summary: "next Rio".to_string(), label: "rio".to_string(), detail: "Saved global values are guaranteed on the next Rio launch. Some values also reload live.".to_string(), pending: false })
    }.build(kind, current, spec.get("default"));
    if let Some(baseline) = &mut field.snapshot.baseline {
        baseline.origin = Some("Rio native default".to_string());
    }
    if let Some(effective) = &mut field.snapshot.effective {
        effective.origin = Some(
            if current.is_some() {
                "User rio/config.toml"
            } else {
                "Rio native default"
            }
            .to_string(),
        );
    }
    Ok(field)
}

pub(crate) fn write_rio_field(
    paths: &ConfigPaths,
    path: &str,
    value: Option<&Value>,
) -> Result<()> {
    if !paths.rio_included {
        return Err(error("This package does not include Rio."));
    }
    paths.reject_mutation(&paths.rio, SOURCE_RIO)?;
    let raw = read_optional_text(&paths.rio)?;
    let edited = match value {
        Some(value) => set_toml_value_text(&raw, path, value),
        None => unset_toml_value_text(&raw, path),
    }
    .map_err(|source| boxed_debug("could not update rio/config.toml", source))?
    .text;
    let specs = inventory(paths, &edited)?;
    if !specs
        .iter()
        .any(|spec| spec.get("path").and_then(Value::as_str) == Some(path))
    {
        return Err(error(format!(
            "Rio does not expose an inline editor for {path}."
        )));
    }
    atomic_write(&paths.rio, &edited)
}
