use std::{
    env,
    ffi::OsString,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{
    PATH_PREFIX,
    error::{AppError, startup},
};

pub(crate) fn package_root() -> Result<Option<PathBuf>, AppError> {
    crate::package::front_door(crate::PORTABLE_RUNTIME)
        .map_err(|error| startup(error.to_string(), "installed package root", 1))
}

pub(crate) fn package_path(binding: (&str, &str)) -> Result<PathBuf, AppError> {
    crate::package::owned(package_root()?.as_deref(), binding.0, binding.1)
        .map_err(|error| startup(error.to_string(), binding.1, 1))
}

pub(crate) fn apply_package(command: &mut Command) -> Result<(), AppError> {
    let root = package_root()?;
    crate::package::apply(root.as_deref(), command);
    command.env("YZX_PACKAGE_HELPER", package_path(crate::PACKAGE_HELPER)?);
    Ok(())
}

pub(crate) fn config_home() -> Result<PathBuf, AppError> {
    if let Some(path) = nonempty_env("YAZELIX_CONFIG_HOME") {
        return Ok(path.into());
    }
    if let Some(path) = nonempty_env("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(path).join("yazelix"));
    }
    nonempty_env("HOME")
        .map(|path| PathBuf::from(path).join(".config/yazelix"))
        .ok_or_else(|| {
            startup(
                "HOME is required when YAZELIX_CONFIG_HOME and XDG_CONFIG_HOME are unset.",
                "",
                1,
            )
        })
}

pub(crate) fn home_dir() -> Result<PathBuf, AppError> {
    nonempty_env("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| startup("HOME is required to scope home-marker new tabs.", "", 1))
}

pub(crate) fn state_dir() -> PathBuf {
    nonempty_env("YAZELIX_STATE_DIR")
        .map(PathBuf::from)
        .or_else(|| nonempty_env("XDG_DATA_HOME").map(|path| PathBuf::from(path).join("yazelix")))
        .or_else(|| {
            nonempty_env("HOME").map(|path| PathBuf::from(path).join(".local/share/yazelix"))
        })
        .unwrap_or_else(|| PathBuf::from("/tmp/yazelix"))
}

pub(crate) fn enter_terminal_label() -> OsString {
    nonempty_env("YAZELIX_SESSION_TERMINAL")
        .or_else(|| nonempty_env("TERM_PROGRAM"))
        .or_else(|| nonempty_env("TERM"))
        .unwrap_or_else(|| OsString::from("unknown"))
}

pub(crate) fn runtime_path() -> Result<OsString, AppError> {
    if let Some(root) = package_root()? {
        return crate::package::search_path(Some(&root), "")
            .map_err(|error| startup(error.to_string(), "managed PATH", 1));
    }
    let mut merged = env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .map(PathBuf::into_os_string)
        .unwrap_or_default();
    for path in [Some(PATH_PREFIX.into()), nonempty_env("PATH")]
        .into_iter()
        .flatten()
    {
        if !merged.is_empty() {
            merged.push(":");
        }
        merged.push(path);
    }
    Ok(merged)
}

pub(crate) fn nonempty_env(name: &str) -> Option<OsString> {
    env::var_os(name).filter(|value| !value.is_empty())
}

pub(crate) fn parent(path: &Path) -> &Path {
    path.parent().unwrap_or_else(|| Path::new("."))
}

pub(crate) fn zellij_session_label(inside: &'static str, outside: &'static str) -> &'static str {
    if nonempty_env("ZELLIJ_SESSION_NAME").is_some() {
        inside
    } else {
        outside
    }
}
