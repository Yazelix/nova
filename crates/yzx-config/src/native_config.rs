use std::{fs, path::Path};

use ratconfig::toml_adapter::{set_toml_value_text, unset_toml_value_text};
use serde_json::Value as JsonValue;
use toml::Value as TomlValue;

use crate::{
    catalog::*,
    common::*,
    starship_inventory::{StarshipInventory, validate_starship_field},
};

pub(crate) fn write_starship_config_field(
    path: &Path,
    field_path: &str,
    value: &JsonValue,
) -> Result<()> {
    let inventory = StarshipInventory::parse()?;
    let field = inventory
        .field(field_path)
        .ok_or_else(|| error(format!("unknown Starship config path: {field_path}")))?;
    validate_starship_field(field, value)?;
    let raw = if path.exists() {
        fs::read_to_string(path)?
    } else {
        String::new()
    };
    let text = set_toml_value_text(&raw, field_path, value)
        .map_err(|error| boxed_debug("could not update starship.toml", error))?
        .text;
    atomic_write(path, &text)
}
pub(crate) fn unset_starship_config_field(path: &Path, field_path: &str) -> Result<()> {
    let inventory = StarshipInventory::parse()?;
    let field = inventory
        .field(field_path)
        .ok_or_else(|| error(format!("unknown Starship config path: {field_path}")))?;
    if !matches!(field.kind.as_str(), "boolean" | "string") {
        return Err(error(format!(
            "Starship config path {field_path} has no schema-backed inline editor"
        )));
    }
    if !path.exists() {
        return Ok(());
    }
    let raw = fs::read_to_string(path)?;
    let text = unset_toml_value_text(&raw, field_path)
        .map_err(|error| boxed_debug("could not update starship.toml", error))?
        .text;
    let value: TomlValue = toml::from_str(&text)
        .map_err(|error| boxed_debug("could not read updated starship.toml", error))?;
    if !toml_has_values(&value) {
        fs::remove_file(path)?;
        Ok(())
    } else {
        atomic_write(path, &text)
    }
}
fn toml_has_values(value: &TomlValue) -> bool {
    match value {
        TomlValue::Table(table) => table.values().any(toml_has_values),
        _ => true,
    }
}
pub(crate) fn write_effective_starship_config(user: &Path, output: &Path) -> Result<()> {
    let mut config: TomlValue = toml::from_str(DEFAULT_STARSHIP_CONFIG_TOML)
        .map_err(|error| boxed_debug("invalid default Starship config", error))?;
    if user.is_file() {
        let overrides = toml::from_str(&fs::read_to_string(user)?)
            .map_err(|error| boxed_debug("invalid user Starship config", error))?;
        deep_merge_toml(&mut config, &overrides);
    }
    let text = toml::to_string_pretty(&config)
        .map_err(|error| boxed_debug("could not serialize effective Starship config", error))?;
    atomic_write(output, &text)
}
