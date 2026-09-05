use std::{
    env,
    ffi::{OsStr, OsString},
    fs, io,
    path::{Path, PathBuf},
    process::{self, Command},
    time::{SystemTime, UNIX_EPOCH},
};

use ratconfig::ConfigUiFileAction;
use serde_json::Value as JsonValue;

use crate::{
    catalog::*,
    common::*,
    native_config::{unset_starship_config_field, write_starship_config_field},
    paths::{ConfigPaths, RioAppearanceProjection, project_rio_appearance},
    rio_config::write_rio_field,
    root_config::{
        config_field, default_config_value, read_config_field, unset_config_field,
        write_config_field,
    },
    yazi_config::{unset_yazi_field, write_yazi_field},
    zellij_sidecar::{unset_zellij_config_field, write_zellij_config_field},
};

pub(crate) struct FileActionSpec {
    source_id: &'static str,
    action_id: &'static str,
    tab: &'static str,
    label: &'static str,
    description: &'static str,
    path: PathBuf,
    starter: &'static str,
}
pub(crate) fn build_file_actions(paths: &ConfigPaths) -> Vec<ConfigUiFileAction> {
    file_action_specs(paths)
        .into_iter()
        .map(|spec| {
            let disabled_reason = paths
                .reject_mutation(&spec.path, spec.source_id)
                .err()
                .map(|error| error.to_string());
            ConfigUiFileAction {
                source_id: spec.source_id.to_string(),
                action_id: spec.action_id.to_string(),
                tab: spec.tab.to_string(),
                label: spec.label.to_string(),
                description: spec.description.to_string(),
                exists: spec.path.exists(),
                read_only: disabled_reason.is_some(),
                create_if_missing: true,
                disabled_reason,
                path: spec.path,
            }
        })
        .collect()
}
fn file_action_specs(paths: &ConfigPaths) -> impl Iterator<Item = FileActionSpec> + '_ {
    [
        FileActionSpec {
            source_id: SOURCE_CONFIG,
            action_id: ACTION_ROOT_CONFIG,
            tab: TAB_ADVANCED,
            label: "config.toml",
            description: "Open the complete Yazelix config file.",
            path: paths.root.clone(),
            starter: "",
        },
        FileActionSpec {
            source_id: SOURCE_RIO,
            action_id: ACTION_RIO_CONFIG,
            tab: TAB_RIO,
            label: "rio/config.toml",
            description: "Open the complete native Rio configuration.",
            path: paths.rio.clone(),
            starter: DEFAULT_RIO_CONFIG_TOML,
        },
        FileActionSpec {
            source_id: SOURCE_HELIX_CONFIG,
            action_id: ACTION_HELIX_CONFIG,
            tab: TAB_HELIX,
            label: "helix/config.toml",
            description: "Open the managed Helix TOML config file.",
            path: paths.helix_config.clone(),
            starter: HELIX_CONFIG_STARTER,
        },
        FileActionSpec {
            source_id: SOURCE_HELIX_LANGUAGES,
            action_id: ACTION_HELIX_LANGUAGES,
            tab: TAB_HELIX,
            label: "helix/languages.toml",
            description: "Open the managed Helix language override file.",
            path: paths.helix_languages.clone(),
            starter: HELIX_LANGUAGES_STARTER,
        },
        FileActionSpec {
            source_id: SOURCE_HELIX,
            action_id: ACTION_HELIX_MODULE,
            tab: TAB_HELIX,
            label: "helix/helix.scm",
            description: "Open the managed Helix Steel module file.",
            path: paths.helix_module.clone(),
            starter: HELIX_MODULE_STARTER,
        },
        FileActionSpec {
            source_id: SOURCE_HELIX,
            action_id: ACTION_HELIX_INIT,
            tab: TAB_HELIX,
            label: "helix/init.scm",
            description: "Open the managed Helix Steel startup file.",
            path: paths.helix_init.clone(),
            starter: HELIX_INIT_STARTER,
        },
        FileActionSpec {
            source_id: SOURCE_ADVANCED,
            action_id: ACTION_NU_ENV,
            tab: TAB_ADVANCED,
            label: "nu/env.nu",
            description: "Executable Nushell environment source loaded after packaged env.nu.",
            path: paths.nu_env.clone(),
            starter: NU_ENV_STARTER,
        },
        FileActionSpec {
            source_id: SOURCE_ADVANCED,
            action_id: ACTION_NU_CONFIG,
            tab: TAB_ADVANCED,
            label: "nu/config.nu",
            description: "Executable Nushell config source loaded after packaged config.nu and any successful host mise activation.",
            path: paths.nu_config.clone(),
            starter: NU_CONFIG_STARTER,
        },
        FileActionSpec {
            source_id: SOURCE_YAZI_CONFIG,
            action_id: ACTION_YAZI_CONFIG,
            tab: TAB_YAZI,
            label: "yazi/yazi.toml",
            description: "Open the managed native Yazi config file.",
            path: paths.yazi_config.clone(),
            starter: YAZI_CONFIG_STARTER,
        },
        FileActionSpec {
            source_id: SOURCE_YAZI,
            action_id: ACTION_YAZI_INIT,
            tab: TAB_YAZI,
            label: "yazi/init.lua",
            description: "Open the managed Yazi user init.lua file.",
            path: paths.yazi_init.clone(),
            starter: YAZI_INIT_STARTER,
        },
        FileActionSpec {
            source_id: SOURCE_YAZI,
            action_id: ACTION_YAZI_KEYMAP,
            tab: TAB_YAZI,
            label: "yazi/keymap.toml",
            description: "Open the managed Yazi user keymap.toml file.",
            path: paths.yazi_keymap.clone(),
            starter: YAZI_KEYMAP_STARTER,
        },
        FileActionSpec {
            source_id: SOURCE_YAZI,
            action_id: ACTION_YAZI_PACKAGE,
            tab: TAB_YAZI,
            label: "yazi/package.toml",
            description: "Open the managed Yazi package metadata file.",
            path: paths.yazi_package.clone(),
            starter: YAZI_PACKAGE_STARTER,
        },
        FileActionSpec {
            source_id: SOURCE_YAZI_THEME,
            action_id: ACTION_YAZI_THEME,
            tab: TAB_YAZI,
            label: "yazi/theme.toml",
            description: "Open the managed native Yazi theme config file.",
            path: paths.yazi_theme.clone(),
            starter: YAZI_THEME_STARTER,
        },
        FileActionSpec {
            source_id: SOURCE_STARSHIP,
            action_id: ACTION_STARSHIP_CONFIG,
            tab: TAB_STARSHIP,
            label: "starship.toml",
            description: "Open the complete native Starship override file.",
            path: paths.starship.clone(),
            starter: STARSHIP_CONFIG_STARTER,
        },
        FileActionSpec {
            source_id: SOURCE_ZELLIJ,
            action_id: ACTION_ZELLIJ_CONFIG,
            tab: TAB_ZELLIJ,
            label: "zellij/config.kdl",
            description: "Open the native Zellij sidecar for settings outside the curated editor.",
            path: paths.zellij.clone(),
            starter: ZELLIJ_CONFIG_STARTER,
        },
        FileActionSpec {
            source_id: SOURCE_ADVANCED,
            action_id: ACTION_ZELLIJ_PLUGINS,
            tab: TAB_ADVANCED,
            label: "zellij/plugins.kdl",
            description: "Open the managed Zellij plugin sidecar file.",
            path: paths.zellij_plugins.clone(),
            starter: ZELLIJ_PLUGINS_STARTER,
        },
    ]
    .into_iter()
    .filter(|spec| paths.rio_included || spec.source_id != SOURCE_RIO)
}
pub(crate) fn write_source_field(
    paths: &ConfigPaths,
    source_id: &str,
    field_path: &str,
    value: &JsonValue,
) -> Result<()> {
    match source_id {
        SOURCE_RIO => write_rio_field(paths, field_path, Some(value)),
        SOURCE_CONFIG => {
            paths.reject_mutation(&paths.root, source_id)?;
            write_config_field(&paths.root, field_path, value)
        }
        SOURCE_ZELLIJ => {
            paths.reject_mutation(&paths.zellij, source_id)?;
            write_zellij_config_field(&paths.zellij, field_path, value)
        }
        SOURCE_STARSHIP => {
            paths.reject_mutation(&paths.starship, source_id)?;
            write_starship_config_field(&paths.starship, field_path, value)
        }
        SOURCE_YAZI_CONFIG | SOURCE_YAZI_THEME => {
            write_yazi_field(paths, source_id, field_path, value)
        }
        _ => Err(error(format!("unknown config source: {source_id}"))),
    }
}
pub(crate) fn write_source_default(
    paths: &ConfigPaths,
    source_id: &str,
    field_path: &str,
) -> Result<()> {
    match source_id {
        SOURCE_RIO => write_rio_field(paths, field_path, None),
        SOURCE_CONFIG => {
            paths.reject_mutation(&paths.root, source_id)?;
            unset_config_field(&paths.root, field_path)
        }
        SOURCE_STARSHIP => {
            paths.reject_mutation(&paths.starship, source_id)?;
            unset_starship_config_field(&paths.starship, field_path)
        }
        SOURCE_ZELLIJ => {
            paths.reject_mutation(&paths.zellij, source_id)?;
            unset_zellij_config_field(&paths.zellij, field_path)
        }
        SOURCE_YAZI_CONFIG | SOURCE_YAZI_THEME => unset_yazi_field(paths, source_id, field_path),
        _ => Err(error(format!("unknown config source: {source_id}"))),
    }
}

pub(crate) fn write_config_ui(
    paths: &ConfigPaths,
    source_id: &str,
    field_path: &str,
    value: Option<&JsonValue>,
) -> Result<Option<RioAppearanceProjection>> {
    if source_id == SOURCE_CONFIG && field_path == APPEARANCE_MODE_PATH {
        return write_appearance_config(paths, value).map(Some);
    }
    match value {
        Some(value) => write_source_field(paths, source_id, field_path, value),
        None => write_source_default(paths, source_id, field_path),
    }?;
    Ok(None)
}

fn write_appearance_config(
    paths: &ConfigPaths,
    value: Option<&JsonValue>,
) -> Result<RioAppearanceProjection> {
    let Some((command, session)) = live_appearance_context() else {
        match value {
            Some(value) => write_source_field(paths, SOURCE_CONFIG, APPEARANCE_MODE_PATH, value),
            None => write_source_default(paths, SOURCE_CONFIG, APPEARANCE_MODE_PATH),
        }?;
        return Ok(RioAppearanceProjection::NextLaunch);
    };
    let mode = match value.cloned() {
        Some(value) => value,
        None => default_config_value(APPEARANCE_MODE_PATH)?,
    };
    let mode = config_field(APPEARANCE_MODE_PATH)?
        .field
        .json_choice(&mode)?;
    let old_mode = read_config_field(&paths.root, config_field(APPEARANCE_MODE_PATH)?)?;
    let root_existed = path_entry_exists(&paths.root)?;
    let root_before = read_optional_text(&paths.root)?;
    paths.reject_mutation(&paths.root, SOURCE_CONFIG)?;

    if project_rio_appearance(paths, mode)? == RioAppearanceProjection::NextLaunch {
        return Err(error(
            "Rio config became read-only; no appearance setting changed. Start a new session to use the coherent next-session fallback",
        ));
    }
    if let Err(source) = match value {
        Some(value) => write_source_field(paths, SOURCE_CONFIG, APPEARANCE_MODE_PATH, value),
        None => write_source_default(paths, SOURCE_CONFIG, APPEARANCE_MODE_PATH),
    } {
        restore_rio_appearance(paths, &old_mode)?;
        return Err(source);
    }
    if let Err(source) = apply_zellij_appearance_to(mode, &command, &session) {
        rollback_appearance(paths, root_existed, &root_before, &old_mode)?;
        return Err(source);
    }
    Ok(RioAppearanceProjection::Live)
}

fn live_appearance_context() -> Option<(OsString, OsString)> {
    if nonempty_env("YZX_APPEARANCE_LIVE").as_deref() != Some(OsStr::new("1"))
        || nonempty_env("YAZELIX_STATE_DIR").is_none()
    {
        return None;
    }
    Some((nonempty_env("YZX_ZELLIJ")?, managed_zellij_session()?))
}

fn rollback_appearance(
    paths: &ConfigPaths,
    root_existed: bool,
    root_before: &str,
    old_mode: &str,
) -> Result<()> {
    if root_existed {
        atomic_write(&paths.root, root_before)?;
    } else if path_entry_exists(&paths.root)? {
        fs::remove_file(&paths.root)?;
    }
    restore_rio_appearance(paths, old_mode)
}

fn restore_rio_appearance(paths: &ConfigPaths, mode: &str) -> Result<()> {
    if project_rio_appearance(paths, mode)? != RioAppearanceProjection::Live {
        return Err(error(
            "could not restore Rio appearance after the update failed",
        ));
    }
    Ok(())
}

pub(crate) fn apply_zellij_appearance_to(
    mode: &str,
    command: &OsStr,
    session: &OsStr,
) -> Result<()> {
    let action = match mode {
        "dark" => "set-dark-theme",
        "light" => "set-light-theme",
        _ => return Err(error(format!("unsupported appearance mode: {mode}"))),
    };
    let output = Command::new(command)
        .arg("--session")
        .arg(session)
        .args(["action", action])
        .output()
        .map_err(|source| error(format!("failed to run `{action}`: {source}")))?;
    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(error(format!(
            "`{action}` failed with status {}{}",
            output.status,
            if stderr.trim().is_empty() {
                String::new()
            } else {
                format!(": {}", stderr.trim())
            }
        )))
    }
}

pub(crate) fn open_file_action(
    paths: &ConfigPaths,
    source_id: &str,
    action_id: &str,
    path: &Path,
    create_if_missing: bool,
) -> Result<()> {
    let spec = file_action_spec(paths, source_id, action_id, path)?;
    paths.reject_mutation(&spec.path, source_id)?;
    let editor = configured_editor()?;
    prepare_file_action(paths, source_id, action_id, path, create_if_missing)?;
    let status = editor_command(&editor, path).status().map_err(|error| {
        io::Error::other(format!(
            "failed to launch editor `{}`: {error}",
            editor.display()
        ))
    })?;
    if !status.success() {
        return Err(error(format!(
            "editor `{}` exited with status {status}",
            editor.display()
        )));
    }
    Ok(())
}
pub(crate) fn prepare_file_action(
    paths: &ConfigPaths,
    source_id: &str,
    action_id: &str,
    path: &Path,
    create_if_missing: bool,
) -> Result<()> {
    let spec = file_action_spec(paths, source_id, action_id, path)?;
    paths.reject_mutation(&spec.path, source_id)?;
    let is_helix_steel_action = spec.source_id == SOURCE_HELIX
        && matches!(spec.action_id, ACTION_HELIX_MODULE | ACTION_HELIX_INIT);
    if path_entry_exists(&spec.path)? {
        if is_helix_steel_action {
            ensure_helix_steel_pair(paths)?;
        }
        return Ok(());
    }
    if !create_if_missing {
        return Err(error(format!("config file is missing: {}", path.display())));
    }
    atomic_write(&spec.path, spec.starter)?;
    if is_helix_steel_action {
        ensure_helix_steel_pair(paths)?;
    }
    Ok(())
}
fn file_action_spec(
    paths: &ConfigPaths,
    source_id: &str,
    action_id: &str,
    path: &Path,
) -> Result<FileActionSpec> {
    let Some(spec) = file_action_specs(paths)
        .into_iter()
        .find(|spec| spec.source_id == source_id && spec.action_id == action_id)
    else {
        return Err(error(format!("unknown file action: {action_id}")));
    };
    if spec.path != path {
        return Err(error(format!(
            "file action `{action_id}` does not own {}",
            path.display()
        )));
    }
    Ok(spec)
}
fn ensure_helix_steel_pair(paths: &ConfigPaths) -> Result<()> {
    if !path_entry_exists(&paths.helix_module)? {
        atomic_write(&paths.helix_module, HELIX_MODULE_STARTER)?;
    }
    if !path_entry_exists(&paths.helix_init)? {
        atomic_write(&paths.helix_init, HELIX_INIT_STARTER)?;
    }
    Ok(())
}
fn configured_editor() -> Result<PathBuf> {
    ["YAZELIX_EDITOR", "VISUAL", "EDITOR"]
        .into_iter()
        .find_map(nonempty_env)
        .map(PathBuf::from)
        .ok_or_else(|| error("no editor configured; set YAZELIX_EDITOR, VISUAL, or EDITOR"))
}
pub(crate) fn edit_text_externally(field_path: &str, input: &str) -> Result<String> {
    edit_text_with_editor(field_path, input, &configured_editor()?)
}
pub(crate) fn edit_text_with_editor(
    field_path: &str,
    input: &str,
    editor: &Path,
) -> Result<String> {
    let path = external_text_edit_path(field_path);
    let result = (|| -> Result<String> {
        fs::write(&path, input)?;
        let status = editor_command(editor, &path).status().map_err(|error| {
            io::Error::other(format!(
                "failed to launch editor `{}`: {error}",
                editor.display()
            ))
        })?;
        if !status.success() {
            return Err(error(format!(
                "editor `{}` exited with status {status}",
                editor.display()
            )));
        }

        let mut text = fs::read_to_string(&path)?;
        if text.ends_with('\n') {
            text.pop();
            if text.ends_with('\r') {
                text.pop();
            }
        }
        Ok(text)
    })();
    let _ = fs::remove_file(&path);
    result
}
fn editor_command(editor: &Path, path: &Path) -> Command {
    let mut command = Command::new(editor);
    command.arg(path).env("YAZELIX_HELIX_BRIDGE", "0");
    command
}
fn external_text_edit_path(field_path: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let label = field_path
        .chars()
        .take(80)
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_') {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>();
    let label = label.trim_matches(|ch| matches!(ch, '.' | '-'));
    let label = if label.is_empty() { "value" } else { label };
    env::temp_dir().join(format!("yzx-config-{label}-{}-{nonce}.txt", process::id()))
}
