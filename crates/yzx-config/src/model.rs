use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use crate::{
    catalog::*,
    common::*,
    file_actions::build_file_actions,
    helix_config::{HELIX_RECOMMENDED_PATHS, HELIX_REVEAL_PATH, build_helix_fields},
    paths::ConfigPaths,
    root_config::{bar_widgets, default_config, default_config_path_value, validate_root_config},
    starship_inventory::{
        PACKAGED_STARSHIP_DEFAULT_CONFIG_TOML, StarshipInventory, validate_starship_field,
    },
    yazi_config::build_yazi_fields,
    zellij_sidecar::{
        active_zellij_runtime_config_path, packaged_zellij_defaults, packaged_zellij_theme_choices,
        parse_zellij_sidecar, read_zellij_sidecar,
    },
};
use ratconfig::toml_adapter::{get_toml_path, parse_toml_value};
use ratconfig::{
    ConfigUiApplyStatus, ConfigUiCapability, ConfigUiChoice, ConfigUiDiagnostic,
    ConfigUiDiagnosticScope, ConfigUiFieldId, ConfigUiFieldSnapshot, ConfigUiFieldSpec,
    ConfigUiListColumn, ConfigUiListTable, ConfigUiModel, ConfigUiOverride, ConfigUiResolvedValue,
    ConfigUiSource, ConfigUiTextEncoding, ConfigUiTheme, ConfigUiThemeMapping,
    ConfigUiThemeSwitcher, ConfigUiTomlDocumentSpec, build_toml_document_fields,
};
use serde_json::Value as JsonValue;

pub(crate) fn build_model(paths: &ConfigPaths) -> Result<ConfigUiModel> {
    let (config_active, root_document_valid, mut diagnostics) = load_root_config(&paths.root)?;
    let config_default = default_config()?;
    let (starship_active, starship_document_valid, starship_diagnostics) =
        load_toml_source(&paths.starship, "starship.toml", SOURCE_STARSHIP)?;
    diagnostics.extend(starship_diagnostics);
    let starship_inventory = StarshipInventory::parse()?;
    let starship_owner_default = parse_toml_value(PACKAGED_STARSHIP_DEFAULT_CONFIG_TOML)
        .map_err(|error| boxed_debug("invalid packaged Starship defaults", error))?;
    let starship_yazelix_default = parse_toml_value(DEFAULT_STARSHIP_CONFIG_TOML)
        .map_err(|error| boxed_debug("invalid default Starship config", error))?;
    let (zellij_active, zellij_invalid, zellij_diagnostics) =
        parse_zellij_sidecar(&read_zellij_sidecar(&paths.zellij)?);
    diagnostics.extend(zellij_diagnostics);
    let zellij_default = packaged_zellij_defaults();
    let zellij_runtime_active = active_zellij_runtime_config_path().is_some();
    let mut zellij_themes = packaged_zellij_theme_choices();
    for custom in ["theme_dark", "theme_light"]
        .into_iter()
        .filter_map(|path| zellij_active.get(path).and_then(JsonValue::as_str))
    {
        if !zellij_themes.iter().any(|theme| theme == custom) {
            zellij_themes.push(custom.to_string());
        }
    }
    let configured_light = ratconfig::get_json_path(&config_active, APPEARANCE_MODE_PATH)
        .or_else(|| ratconfig::get_json_path(&config_default, APPEARANCE_MODE_PATH))
        .and_then(JsonValue::as_str)
        == Some("light");
    let session_mode = nonempty_env("YZX_APPEARANCE_MODE");
    let (light, fixed_theme) = resolved_appearance(
        configured_light,
        session_mode.as_deref().and_then(|mode| mode.to_str()),
        nonempty_env("YZX_APPEARANCE_LIVE").as_deref() == Some(std::ffi::OsStr::new("1")),
    );
    let yazi = build_yazi_fields(paths, light)?;
    let (helix, helix_diagnostics) = build_helix_fields(paths)?;
    diagnostics.extend(helix_diagnostics);
    let file_actions = build_file_actions(paths);

    let mut fields: Vec<_> = CONFIG_FIELDS
        .iter()
        .filter(|spec| {
            paths.helix_included
                || !matches!(
                    spec.field.path,
                    FOREST_SIDE_PATH | KEYBINDINGS_SIDEBAR_FOCUS_PATH
                )
        })
        .map(|spec| build_root_config_field(&config_active, &config_default, spec))
        .collect::<Result<_>>()?;
    fields.push(build_bar_widgets_field(&config_active, &config_default)?);
    if root_document_valid {
        fields.extend(build_custom_popup_fields(&paths.root)?);
    }
    fields.extend(helix.fields);
    fields.extend(
        KEY_BINDINGS
            .iter()
            .filter(|binding| paths.helix_included || binding[4] != "helix/config.toml")
            .map(build_key_binding_field),
    );
    for spec in starship_inventory.fields() {
        fields.push(build_starship_config_field(
            spec,
            &starship_active,
            &starship_owner_default,
            &starship_yazelix_default,
        ));
    }
    for spec in ZELLIJ_FIELDS {
        let current = zellij_active.get(spec.path);
        let default = zellij_default.get(spec.path).expect("packaged default");
        let mut field = build_config_field(
            SOURCE_ZELLIJ,
            TAB_ZELLIJ,
            spec,
            current,
            Some(default),
            zellij_apply_status(spec.path, zellij_runtime_active),
            false,
        );
        if matches!(spec.path, "theme_dark" | "theme_light") {
            field.capability = choice_capability(zellij_themes.clone());
            field.display_label = if spec.path == "theme_dark" {
                "Dark theme"
            } else {
                "Light theme"
            }
            .to_string();
        }
        if let Some(input) = zellij_invalid.get(spec.path) {
            field.snapshot.intent = ConfigUiOverride::Invalid {
                input: input.clone(),
            };
            field.snapshot.effective = None;
            field.can_unset = true;
        }
        fields.push(field);
    }
    let yazi_dir = paths.yazi_config.parent().expect("Yazi config directory");
    let advanced_dir = paths.nu_config.parent().expect("Nushell config directory");
    let mut sources = vec![
        build_config_source(paths, SOURCE_CONFIG, "config.toml", &paths.root),
        build_config_source(paths, SOURCE_RIO, "rio/config.toml", &paths.rio),
        build_config_source(paths, SOURCE_ZELLIJ, "zellij/config.kdl", &paths.zellij),
        build_config_source(paths, SOURCE_STARSHIP, "starship.toml", &paths.starship),
        build_config_source(
            paths,
            SOURCE_HELIX_CONFIG,
            "helix/config.toml",
            &paths.helix_config,
        ),
        build_config_source(
            paths,
            SOURCE_HELIX_LANGUAGES,
            "helix/languages.toml",
            &paths.helix_languages,
        ),
        build_config_source(paths, SOURCE_HELIX, "helix", &paths.helix_dir),
        build_config_source(
            paths,
            SOURCE_YAZI_CONFIG,
            "yazi/yazi.toml",
            &paths.yazi_config,
        ),
        build_config_source(
            paths,
            SOURCE_YAZI_THEME,
            "yazi/theme.toml",
            &paths.yazi_theme,
        ),
        build_config_source(paths, SOURCE_YAZI, "yazi", yazi_dir),
        build_config_source(paths, SOURCE_ADVANCED, "advanced files", advanced_dir),
        ConfigUiSource {
            id: SOURCE_KEYS.to_string(),
            label: "key bindings".to_string(),
            path: PathBuf::from("packaged-key-bindings"),
            exists: true,
            owner_label: Some("Yazelix".to_string()),
            read_only: true,
        },
    ];
    if !paths.rio_included {
        sources.retain(|source| source.id != SOURCE_RIO);
    }
    fields.extend(yazi);
    if !root_document_valid {
        block_source_fields(
            &mut fields,
            SOURCE_CONFIG,
            "Repair config.toml before editing individual fields.",
        );
    }
    if !starship_document_valid {
        block_source_fields(
            &mut fields,
            SOURCE_STARSHIP,
            "Repair starship.toml before editing individual fields.",
        );
    }
    apply_source_policy(&mut fields, &sources);
    let recommended_fields = Some(
        fields
            .iter()
            .filter(|field| match field.source_id.as_str() {
                SOURCE_CONFIG => ROOT_CONFIG_RECOMMENDED_PATHS.contains(&field.path.as_str()),
                SOURCE_ZELLIJ => ZELLIJ_RECOMMENDED_PATHS.contains(&field.path.as_str()),
                SOURCE_STARSHIP => STARSHIP_RECOMMENDED_PATHS.contains(&field.path.as_str()),
                SOURCE_HELIX_CONFIG => HELIX_RECOMMENDED_PATHS.contains(&field.path.as_str()),
                SOURCE_HELIX_LANGUAGES => false,
                SOURCE_YAZI_CONFIG | SOURCE_YAZI_THEME => {
                    crate::yazi_config::YAZI_RECOMMENDED_FIELDS
                        .contains(&(field.source_id.as_str(), field.path.as_str()))
                }
                _ => true,
            })
            .map(|field| ConfigUiFieldId::new(&field.source_id, &field.path))
            .collect(),
    );

    Ok(ConfigUiModel {
        sources,
        tabs: [
            TAB_CONFIG,
            TAB_POPUPS,
            TAB_RIO,
            TAB_ZELLIJ,
            TAB_STARSHIP,
            TAB_HELIX,
            TAB_YAZI,
            TAB_KEYS,
            TAB_ADVANCED,
        ]
        .into_iter()
        .filter(|tab| paths.rio_included || *tab != TAB_RIO)
        .map(str::to_string)
        .collect(),
        operational_tab: Some(TAB_ADVANCED.to_string()),
        tab_list_tables: BTreeMap::from([
            (TAB_HELIX.to_string(), helix.list_table),
            (
                TAB_KEYS.to_string(),
                ConfigUiListTable {
                    columns: KEY_COLUMNS
                        .iter()
                        .map(|(title, width)| ConfigUiListColumn {
                            title: (*title).to_string(),
                            width: *width,
                        })
                        .collect(),
                },
            ),
        ]),
        fields,
        recommended_fields,
        file_actions,
        sidecars: Vec::new(),
        native_config_statuses: Vec::new(),
        diagnostics,
        theme_switcher: Some(ConfigUiThemeSwitcher {
            field: ConfigUiFieldId::new(SOURCE_CONFIG, APPEARANCE_MODE_PATH),
            mappings: vec![
                ConfigUiThemeMapping {
                    value: JsonValue::String("dark".to_string()),
                    theme: fixed_theme.unwrap_or(ConfigUiTheme::Dark),
                },
                ConfigUiThemeMapping {
                    value: JsonValue::String("light".to_string()),
                    theme: fixed_theme.unwrap_or(ConfigUiTheme::Light),
                },
            ],
        }),
    })
}

pub(crate) fn resolved_appearance(
    configured_light: bool,
    session_mode: Option<&str>,
    live: bool,
) -> (bool, Option<ConfigUiTheme>) {
    if !live {
        match session_mode {
            Some("dark") => return (false, Some(ConfigUiTheme::Dark)),
            Some("light") => return (true, Some(ConfigUiTheme::Light)),
            _ => {}
        }
    }
    (configured_light, None)
}

fn load_root_config(path: &Path) -> Result<(JsonValue, bool, Vec<ConfigUiDiagnostic>)> {
    let (active, document_valid, mut diagnostics) =
        load_toml_source(path, "config.toml", SOURCE_CONFIG)?;
    if !document_valid {
        return Ok((active, false, diagnostics));
    }

    diagnostics.extend(root_field_diagnostics(&active));
    let source_error = validate_root_config(&active)
        .err()
        .map(|source| source.to_string())
        .filter(|message| {
            !diagnostics.iter().any(|diagnostic| {
                diagnostic
                    .detail_lines
                    .iter()
                    .any(|detail| detail == message)
            })
        });
    let document_valid = source_error.is_none();
    if let Some(message) = source_error {
        diagnostics.push(invalid_source_diagnostic(
            "config.toml",
            SOURCE_CONFIG,
            message,
        ));
    }
    Ok((active, document_valid, diagnostics))
}

fn load_toml_source(
    path: &Path,
    display_path: &str,
    source_id: &str,
) -> Result<(JsonValue, bool, Vec<ConfigUiDiagnostic>)> {
    if !path_entry_exists(path)? {
        return Ok((JsonValue::Object(Default::default()), true, Vec::new()));
    }
    let raw = fs::read_to_string(path)?;
    match parse_toml_value(&raw) {
        Ok(active) => Ok((active, true, Vec::new())),
        Err(source) => Ok((
            JsonValue::Object(Default::default()),
            false,
            vec![invalid_source_diagnostic(
                display_path,
                source_id,
                format!("invalid TOML: {source:?}"),
            )],
        )),
    }
}

fn root_field_diagnostics(active: &JsonValue) -> Vec<ConfigUiDiagnostic> {
    let mut diagnostics = CONFIG_FIELDS
        .iter()
        .map(|spec| &spec.field)
        .filter_map(|spec| {
            let value = get_toml_path(active, spec.path)?;
            crate::root_config::validate_config_value(spec.path, value)
                .err()
                .map(|source| ConfigUiDiagnostic {
                    path: spec.path.to_string(),
                    status: "invalid".to_string(),
                    headline: format!("invalid config value for `{}`", spec.path),
                    blocking: true,
                    scope: ConfigUiDiagnosticScope::Field(ConfigUiFieldId::new(
                        SOURCE_CONFIG,
                        spec.path,
                    )),
                    detail_lines: vec![source.to_string()],
                })
        })
        .collect::<Vec<_>>();
    if let Some(value) = get_toml_path(active, BAR_WIDGETS_PATH)
        && let Err(source) = bar_widgets(value)
    {
        diagnostics.push(ConfigUiDiagnostic {
            path: BAR_WIDGETS_PATH.to_string(),
            status: "invalid".to_string(),
            headline: format!("invalid config value for `{BAR_WIDGETS_PATH}`"),
            blocking: true,
            scope: ConfigUiDiagnosticScope::Field(ConfigUiFieldId::new(
                SOURCE_CONFIG,
                BAR_WIDGETS_PATH,
            )),
            detail_lines: vec![source.to_string()],
        });
    }
    diagnostics
}

fn build_key_binding_field(
    [group, chord, action, owner, source]: &[&str; 5],
) -> ratconfig::ConfigUiField {
    ratconfig::ConfigUiField {
        source_id: SOURCE_KEYS.to_string(),
        path: chord.to_string(),
        tab: TAB_KEYS.to_string(),
        display_label: format!("{group}: {chord} - {action}"),
        section_label: String::new(),
        list_cells: [*group, *chord, *action, *owner]
            .into_iter()
            .map(str::to_string)
            .collect(),
        type_label: Some("string".to_string()),
        snapshot: ConfigUiFieldSnapshot {
            intent: ConfigUiOverride::Explicit(JsonValue::String(format!("{owner} / {source}"))),
            effective: Some(ConfigUiResolvedValue {
                value: JsonValue::String(format!("{owner} / {source}")),
                origin: Some("Yazelix packaged keymap".to_string()),
            }),
            baseline: None,
            external_manager: None,
        },
        description: format!("Group: {group}. Owner: {owner}. Source: {source}. Editable: no."),
        validation: KEY_READ_ONLY_REASON.to_string(),
        rebuild_required: false,
        apply_status: apply_status("read-only", "read-only", KEY_READ_ONLY_REASON),
        capability: ConfigUiCapability::ReadOnly {
            reason: KEY_READ_ONLY_REASON.to_string(),
            file_action_id: None,
        },
        can_unset: false,
    }
}
fn build_config_source(paths: &ConfigPaths, id: &str, label: &str, path: &Path) -> ConfigUiSource {
    let home_manager_owned = paths.is_home_manager_owned(path);
    ConfigUiSource {
        id: id.to_string(),
        label: label.to_string(),
        path: path.to_path_buf(),
        exists: path.exists(),
        owner_label: Some(if home_manager_owned {
            "Home Manager".to_string()
        } else {
            "User".to_string()
        }),
        read_only: home_manager_owned || path_read_only(path),
    }
}
fn build_root_config_field(
    active: &JsonValue,
    defaults: &JsonValue,
    spec: &ConfigFieldSpec,
) -> Result<ratconfig::ConfigUiField> {
    let default = default_config_path_value(defaults, spec.field.path)?;
    let current = get_toml_path(active, spec.field.path);
    let invalid = current.is_some_and(|value| {
        crate::root_config::validate_config_value(spec.field.path, value).is_err()
    });
    Ok(build_config_field(
        SOURCE_CONFIG,
        root_config_tab(spec.field.path),
        &spec.field,
        current,
        Some(&default),
        apply_status(spec.apply_summary, "runtime", spec.apply_detail),
        invalid,
    ))
}
fn build_custom_popup_fields(path: &Path) -> Result<Vec<ratconfig::ConfigUiField>> {
    let raw = if path_entry_exists(path)? {
        fs::read_to_string(path)?
    } else {
        String::new()
    };
    let mut fields = build_toml_document_fields(ConfigUiTomlDocumentSpec {
        source_id: SOURCE_CONFIG,
        tab: TAB_POPUPS,
        section_label: "custom popups",
        current_toml: &raw,
        baseline_toml: None,
        validation: "",
        rebuild_required: false,
        apply_status: apply_status(
            "next launch",
            "runtime",
            "Saved custom popup settings apply to newly launched Yazelix sessions.",
        ),
    })
    .map_err(|source| error(source.to_string()))?
    .fields;
    fields.retain(|field| field.path.starts_with("popups."));
    remove_toml_parent_fields(&mut fields, None);
    for field in &mut fields {
        field.list_cells.clear();
        if let ConfigUiOverride::Explicit(value) = &field.snapshot.intent {
            field.snapshot.effective = Some(ConfigUiResolvedValue {
                value: value.clone(),
                origin: Some("User config.toml".to_string()),
            });
        }
        if let Some((capability, can_unset)) = custom_popup_controls(&field.path) {
            field.capability = capability;
            field.can_unset = can_unset;
        }
    }
    Ok(fields)
}
fn custom_popup_controls(path: &str) -> Option<(ConfigUiCapability, bool)> {
    let mut segments = path.split('.');
    if segments.next() != Some("popups") || segments.next().is_none() {
        return None;
    }
    let field = segments.next()?;
    if segments.next().is_some() {
        return None;
    }
    let capability = match field {
        "command" | "title" | "keybinding" => ConfigUiCapability::FreeText {
            encoding: ConfigUiTextEncoding::String,
        },
        "args" => ConfigUiCapability::FreeText {
            encoding: ConfigUiTextEncoding::Json,
        },
        "keep_alive" => ConfigUiCapability::Toggle {
            off: ConfigUiChoice::new(JsonValue::Bool(false)),
            on: ConfigUiChoice::new(JsonValue::Bool(true)),
        },
        _ => return None,
    };
    Some((capability, !matches!(field, "command" | "keybinding")))
}
fn root_config_tab(path: &str) -> &'static str {
    if matches!(
        path,
        AGENT_COMMAND_PATH
            | AGENT_ARGS_PATH
            | POPUP_SIDE_MARGIN_PATH
            | POPUP_VERTICAL_MARGIN_PATH
            | KEYBINDINGS_CONFIG_PATH
            | KEYBINDINGS_AGENT_PATH
            | KEYBINDINGS_GIT_PATH
            | KEYBINDINGS_MENU_PATH
            | KEYBINDINGS_SCREEN_PATH
    ) {
        TAB_POPUPS
    } else {
        TAB_CONFIG
    }
}
fn build_config_field(
    source_id: &'static str,
    tab: &'static str,
    spec: &FieldSpec,
    current: Option<&JsonValue>,
    default: Option<&JsonValue>,
    apply_status: ConfigUiApplyStatus,
    invalid: bool,
) -> ratconfig::ConfigUiField {
    let mut field = ConfigUiFieldSpec {
        can_unset: current.is_some(),
        ..ConfigUiFieldSpec::new(
            source_id,
            spec.path,
            tab,
            spec.description,
            field_capability(spec, string_values(spec.allowed_values)),
            spec.validation,
            apply_status,
        )
    }
    .build(spec.kind, current, default);
    set_snapshot_origins(&mut field, source_id);
    if invalid {
        field.snapshot.intent = ConfigUiOverride::Invalid {
            input: current.map_or_else(String::new, ToString::to_string),
        };
        field.snapshot.effective = None;
    }
    field
}

fn build_starship_config_field(
    spec: &ratconfig::ConfigUiSchemaField,
    active: &JsonValue,
    owner_defaults: &JsonValue,
    yazelix_defaults: &JsonValue,
) -> ratconfig::ConfigUiField {
    let current = get_toml_path(active, &spec.path);
    let yazelix_default = get_toml_path(yazelix_defaults, &spec.path);
    let default = yazelix_default.or_else(|| get_toml_path(owner_defaults, &spec.path));
    let capability = match spec.kind.as_str() {
        "boolean" => ConfigUiCapability::Toggle {
            off: ConfigUiChoice::new(JsonValue::Bool(false)),
            on: ConfigUiChoice::new(JsonValue::Bool(true)),
        },
        "string" => ConfigUiCapability::FreeText {
            encoding: ConfigUiTextEncoding::String,
        },
        _ => ConfigUiCapability::ReadOnly {
            reason: "Edit this Starship value in starship.toml.".to_string(),
            file_action_id: Some(ACTION_STARSHIP_CONFIG.to_string()),
        },
    };
    let editable = !matches!(capability, ConfigUiCapability::ReadOnly { .. });
    let invalid =
        editable && current.is_some_and(|value| validate_starship_field(spec, value).is_err());
    let mut field = ConfigUiFieldSpec {
        section_label: spec
            .path
            .split_once('.')
            .map_or("General", |(section, _)| section)
            .to_string(),
        can_unset: editable && current.is_some(),
        ..ConfigUiFieldSpec::new(
            SOURCE_STARSHIP,
            &spec.path,
            TAB_STARSHIP,
            format!("Setting published by Starship for `{}`.", spec.path),
            capability,
            format!("a Starship {}", spec.kind),
            apply_status(
                "new prompts",
                "starship",
                "Saved values apply to newly rendered managed Nu prompts.",
            ),
        )
    }
    .build(&spec.kind, current, default);
    set_starship_origins(&mut field, yazelix_default.is_some());
    if invalid {
        field.snapshot.intent = ConfigUiOverride::Invalid {
            input: current.map_or_else(String::new, ToString::to_string),
        };
        field.snapshot.effective = None;
    }
    field
}
fn build_bar_widgets_field(
    active: &JsonValue,
    defaults: &JsonValue,
) -> Result<ratconfig::ConfigUiField> {
    let current = get_toml_path(active, BAR_WIDGETS_PATH);
    let default = default_config_path_value(defaults, BAR_WIDGETS_PATH)?;
    let invalid = current.is_some_and(|value| bar_widgets(value).is_err());
    let mut field = ConfigUiFieldSpec {
        can_unset: current.is_some(),
        ..ConfigUiFieldSpec::new(
            SOURCE_CONFIG,
            BAR_WIDGETS_PATH,
            TAB_CONFIG,
            "Top bar widgets, left to right.",
            multi_choice_capability(string_values(BAR_WIDGET_VALUES), true),
            "known widget ids",
            apply_status(
                "next launch",
                "bar",
                "Saved widget order applies to newly launched Yazelix sessions.",
            ),
        )
    }
    .build("string_list", current, Some(&default));
    set_snapshot_origins(&mut field, SOURCE_CONFIG);
    if invalid {
        field.snapshot.intent = ConfigUiOverride::Invalid {
            input: current.map_or_else(String::new, ToString::to_string),
        };
        field.snapshot.effective = None;
    }
    Ok(field)
}

fn field_capability(spec: &FieldSpec, values: Vec<String>) -> ConfigUiCapability {
    match spec.kind {
        "boolean" => ConfigUiCapability::Toggle {
            off: ConfigUiChoice::new(JsonValue::Bool(false)),
            on: ConfigUiChoice::new(JsonValue::Bool(true)),
        },
        "string" if !values.is_empty() => choice_capability(values),
        "string" => ConfigUiCapability::FreeText {
            encoding: ConfigUiTextEncoding::String,
        },
        "key chord or false" => ConfigUiCapability::OptionalString {
            disabled: ConfigUiChoice {
                value: JsonValue::Bool(false),
                label: Some("Unmapped".to_string()),
            },
        },
        "string_list" if !values.is_empty() => multi_choice_capability(values, true),
        "string_list" | "integer" | "float" => ConfigUiCapability::FreeText {
            encoding: ConfigUiTextEncoding::Json,
        },
        _ => ConfigUiCapability::ReadOnly {
            reason: format!("Unsupported owner type {}.", spec.kind),
            file_action_id: None,
        },
    }
}

fn choice_capability(values: impl IntoIterator<Item = String>) -> ConfigUiCapability {
    ConfigUiCapability::Choice {
        choices: values
            .into_iter()
            .map(|value| ConfigUiChoice::new(JsonValue::String(value)))
            .collect(),
    }
}

fn multi_choice_capability(
    values: impl IntoIterator<Item = String>,
    ordered: bool,
) -> ConfigUiCapability {
    ConfigUiCapability::MultiChoice {
        choices: values
            .into_iter()
            .map(|value| ConfigUiChoice::new(JsonValue::String(value)))
            .collect(),
        ordered,
    }
}

fn field_origins(source_id: &str) -> (&'static str, &'static str) {
    match source_id {
        SOURCE_CONFIG => ("User config.toml", "Yazelix packaged default"),
        SOURCE_ZELLIJ => ("User zellij/config.kdl", "Packaged Zellij defaults"),
        SOURCE_STARSHIP => ("User starship.toml", "Yazelix packaged default"),
        SOURCE_YAZI_CONFIG => ("User yazi/yazi.toml", "Packaged Yazi config"),
        SOURCE_YAZI_THEME => ("User yazi/theme.toml", "Yazi default theme"),
        _ => ("User configuration", "Yazelix packaged default"),
    }
}

fn set_snapshot_origins(field: &mut ratconfig::ConfigUiField, source_id: &str) {
    let (effective_origin, baseline_origin) = field_origins(source_id);
    if let Some(baseline) = &mut field.snapshot.baseline {
        baseline.origin = Some(baseline_origin.to_string());
    }
    let origin = if matches!(field.snapshot.intent, ConfigUiOverride::Absent)
        && field.snapshot.baseline.is_some()
    {
        baseline_origin
    } else {
        effective_origin
    };
    if let Some(effective) = &mut field.snapshot.effective {
        effective.origin = Some(origin.to_string());
    }
}

fn set_starship_origins(field: &mut ratconfig::ConfigUiField, yazelix_default: bool) {
    let baseline_origin = if yazelix_default {
        "Yazelix packaged default"
    } else {
        "Packaged Starship default"
    };
    if let Some(baseline) = &mut field.snapshot.baseline {
        baseline.origin = Some(baseline_origin.to_string());
    }
    if let Some(effective) = &mut field.snapshot.effective {
        effective.origin = Some(
            if matches!(field.snapshot.intent, ConfigUiOverride::Absent) {
                baseline_origin
            } else {
                "User starship.toml"
            }
            .to_string(),
        );
    }
}

fn block_source_fields(fields: &mut [ratconfig::ConfigUiField], source_id: &str, reason: &str) {
    for field in fields
        .iter_mut()
        .filter(|field| field.source_id == source_id)
    {
        field.capability = ConfigUiCapability::ReadOnly {
            reason: reason.to_string(),
            file_action_id: source_file_action(source_id).map(str::to_string),
        };
        field.can_unset = false;
    }
}

fn apply_source_policy(fields: &mut [ratconfig::ConfigUiField], sources: &[ConfigUiSource]) {
    for field in fields {
        let source = sources
            .iter()
            .find(|source| source.id == field.source_id)
            .expect("every field source is declared");
        let home_manager_owned = source.owner_label.as_deref() == Some("Home Manager");
        let integration_owned =
            field.source_id == SOURCE_HELIX_CONFIG && field.path == HELIX_REVEAL_PATH;
        if home_manager_owned {
            if !integration_owned
                && matches!(
                    field.snapshot.intent,
                    ConfigUiOverride::Explicit(_) | ConfigUiOverride::Invalid { .. }
                )
                && let Some(effective) = &mut field.snapshot.effective
            {
                effective.origin = Some("Home Manager".to_string());
            }
            field.snapshot.external_manager = Some("Home Manager".to_string());
        }
        if source.read_only {
            if (home_manager_owned && !integration_owned)
                || !matches!(field.capability, ConfigUiCapability::ReadOnly { .. })
            {
                field.capability = ConfigUiCapability::ReadOnly {
                    reason: if home_manager_owned {
                        "Managed by Home Manager."
                    } else {
                        "Source is read-only."
                    }
                    .to_string(),
                    file_action_id: source_file_action(&field.source_id).map(str::to_string),
                };
            }
            field.can_unset = false;
        }
        if let ConfigUiCapability::ReadOnly { file_action_id, .. } = &mut field.capability
            && file_action_id.is_none()
        {
            *file_action_id = source_file_action(&field.source_id).map(str::to_string);
        }
    }
}

fn source_file_action(source_id: &str) -> Option<&'static str> {
    match source_id {
        SOURCE_CONFIG => Some(ACTION_ROOT_CONFIG),
        SOURCE_RIO => Some(ACTION_RIO_CONFIG),
        SOURCE_STARSHIP => Some(ACTION_STARSHIP_CONFIG),
        SOURCE_HELIX_CONFIG => Some(ACTION_HELIX_CONFIG),
        SOURCE_HELIX_LANGUAGES => Some(ACTION_HELIX_LANGUAGES),
        SOURCE_ZELLIJ => Some(ACTION_ZELLIJ_CONFIG),
        SOURCE_YAZI_CONFIG => Some(ACTION_YAZI_CONFIG),
        SOURCE_YAZI_THEME => Some(ACTION_YAZI_THEME),
        _ => None,
    }
}
fn apply_status(summary: &str, label: &str, detail: &str) -> ConfigUiApplyStatus {
    ConfigUiApplyStatus {
        summary: summary.to_string(),
        label: label.to_string(),
        detail: detail.to_string(),
        pending: false,
    }
}

pub(crate) fn zellij_apply_status(path: &str, runtime_active: bool) -> ConfigUiApplyStatus {
    match (path, runtime_active) {
        ("theme_dark" | "theme_light", true) => apply_status(
            "now/next mode",
            "zellij",
            "The active mode applies live inside a managed session; the other applies on the next mode change or launch.",
        ),
        (
            "pane_frames" | "copy_on_select" | "copy_clipboard" | "ui.pane_frames.rounded_corners",
            true,
        ) => apply_status(
            "now",
            "zellij",
            "Saved values update the active managed Zellij session.",
        ),
        (
            "theme_dark"
            | "theme_light"
            | "pane_frames"
            | "mouse_mode"
            | "scroll_buffer_size"
            | "copy_on_select"
            | "copy_clipboard"
            | "styled_underlines"
            | "show_startup_tips"
            | "ui.pane_frames.rounded_corners",
            _,
        ) => apply_status(
            "next session",
            "zellij",
            "Saved values apply to the next managed Zellij session.",
        ),
        _ => unreachable!("Zellij field {path} has no apply timing"),
    }
}
