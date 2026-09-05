use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use ratconfig::toml_adapter::set_toml_value_text;
use serde_json::json;

use crate::{catalog::*, common::*};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RioAppearanceProjection {
    Live,
    NextLaunch,
}

pub(crate) struct ConfigPaths {
    pub(crate) rio_included: bool,
    pub(crate) rio_command: Option<PathBuf>,
    pub(crate) helix_included: bool,
    pub(crate) store_root: PathBuf,
    pub(crate) root: PathBuf,
    pub(crate) rio: PathBuf,
    pub(crate) zellij: PathBuf,
    pub(crate) helix_dir: PathBuf,
    pub(crate) helix_config: PathBuf,
    pub(crate) helix_languages: PathBuf,
    pub(crate) helix_module: PathBuf,
    pub(crate) helix_init: PathBuf,
    pub(crate) nu_env: PathBuf,
    pub(crate) nu_config: PathBuf,
    pub(crate) starship: PathBuf,
    pub(crate) yazi_config: PathBuf,
    pub(crate) yazi_init: PathBuf,
    pub(crate) yazi_keymap: PathBuf,
    pub(crate) yazi_package: PathBuf,
    pub(crate) yazi_theme: PathBuf,
    pub(crate) packaged_yazi: PathBuf,
    pub(crate) zellij_plugins: PathBuf,
}
impl ConfigPaths {
    fn home_manager_files(&self) -> [(&Path, &'static str); 15] {
        [
            (&self.root, "settings"),
            (&self.rio, "rio"),
            (&self.zellij, "zellij"),
            (&self.starship, "starship"),
            (&self.helix_config, "helix.config"),
            (&self.helix_languages, "helix.languages"),
            (&self.helix_module, "helix.module"),
            (&self.helix_init, "helix.init"),
            (&self.yazi_config, "yazi.config"),
            (&self.yazi_init, "yazi.init"),
            (&self.yazi_keymap, "yazi.keymap"),
            (&self.yazi_package, "yazi.package"),
            (&self.yazi_theme, "yazi.theme"),
            (&self.nu_env, "nu.env"),
            (&self.nu_config, "nu.config"),
        ]
    }

    pub(crate) fn is_home_manager_owned(&self, path: &Path) -> bool {
        self.home_manager_option(path).is_some()
            && resolved_target(path).is_some_and(|path| path.starts_with(&self.store_root))
    }

    pub(crate) fn home_manager_guidance(&self, path: &Path) -> Option<String> {
        self.is_home_manager_owned(path).then(|| {
            format!(
                "Managed by Home Manager through `programs.yazelix.config.{}`; edit that option and run your normal Home Manager switch.",
                self.home_manager_option(path).expect("mapped path")
            )
        })
    }

    pub(crate) fn reject_mutation(&self, path: &Path, source_id: &str) -> Result<()> {
        if let Some(guidance) = self.home_manager_guidance(path) {
            return Err(error(guidance));
        }
        reject_read_only_source(path, source_id)
    }

    fn home_manager_option(&self, path: &Path) -> Option<&'static str> {
        self.home_manager_files()
            .into_iter()
            .find_map(|(candidate, option)| (candidate == path).then_some(option))
    }
}
pub(crate) fn ensure_config_sources() -> Result<ConfigPaths> {
    ensure_config_sources_at(config_paths()?)
}
pub(crate) fn ensure_config_sources_at(paths: ConfigPaths) -> Result<ConfigPaths> {
    if paths.rio_included {
        initialize_rio_config(&paths.rio)?;
    }
    Ok(paths)
}
pub(crate) fn initialize_rio_config(path: &Path) -> Result<()> {
    if !path_entry_exists(path)? {
        let themes = path
            .parent()
            .expect("Rio config has a parent")
            .join("themes");
        for (name, contents) in [
            ("nova-dark.toml", DEFAULT_RIO_DARK_THEME_TOML),
            ("nova-light.toml", DEFAULT_RIO_LIGHT_THEME_TOML),
        ] {
            let theme = themes.join(name);
            if !path_entry_exists(&theme)? {
                atomic_write(&theme, contents)?;
            }
        }
        atomic_write(path, DEFAULT_RIO_CONFIG_TOML)?;
    }
    Ok(())
}

pub(crate) fn project_rio_appearance(
    paths: &ConfigPaths,
    mode: &str,
) -> Result<RioAppearanceProjection> {
    if !matches!(mode, "dark" | "light") {
        return Err(error(format!("unsupported appearance mode: {mode}")));
    }
    if !paths.rio_included {
        return Ok(RioAppearanceProjection::NextLaunch);
    }
    initialize_rio_config(&paths.rio)?;
    if paths.reject_mutation(&paths.rio, SOURCE_RIO).is_err() {
        return Ok(RioAppearanceProjection::NextLaunch);
    }
    let raw = fs::read_to_string(&paths.rio)?;
    let projected = set_toml_value_text(&raw, "force-theme", &json!(mode))
        .map_err(|source| boxed_debug("could not project appearance into rio/config.toml", source))?
        .text;
    if projected != raw {
        atomic_write(&paths.rio, &projected)?;
    }
    Ok(RioAppearanceProjection::Live)
}
pub(crate) fn config_paths() -> Result<ConfigPaths> {
    let home = config_home()?;
    Ok(ConfigPaths {
        rio_included: nonempty_env("YZX_RIO_INCLUDED").as_deref() != Some(OsStr::new("0")),
        rio_command: nonempty_env("YZX_RIO").map(PathBuf::from),
        helix_included: nonempty_env("YZX_HELIX_INCLUDED").as_deref() != Some(OsStr::new("0")),
        store_root: option_env!("YAZELIX_NIX_STORE_ROOT")
            .map(PathBuf::from)
            .ok_or_else(|| error("yzx-config is missing its packaged Nix store root"))?,
        root: home.join("config.toml"),
        rio: home.join("rio/config.toml"),
        zellij: home.join("zellij/config.kdl"),
        helix_dir: home.join("helix"),
        helix_config: home.join("helix/config.toml"),
        helix_languages: home.join("helix/languages.toml"),
        helix_module: home.join("helix/helix.scm"),
        helix_init: home.join("helix/init.scm"),
        nu_env: home.join("nu/env.nu"),
        nu_config: home.join("nu/config.nu"),
        starship: home.join("starship.toml"),
        yazi_config: home.join("yazi/yazi.toml"),
        yazi_init: home.join("yazi/init.lua"),
        yazi_keymap: home.join("yazi/keymap.toml"),
        yazi_package: home.join("yazi/package.toml"),
        yazi_theme: home.join("yazi/theme.toml"),
        packaged_yazi: option_env!("YAZELIX_PACKAGED_YAZI")
            .map(PathBuf::from)
            .ok_or_else(|| error("yzx-config is missing its packaged Yazi config"))?,
        zellij_plugins: home.join("zellij/plugins.kdl"),
    })
}
fn resolved_target(path: &Path) -> Option<PathBuf> {
    path.canonicalize().ok().or_else(|| {
        let target = fs::read_link(path).ok()?;
        Some(if target.is_absolute() {
            target
        } else {
            path.parent()?.join(target)
        })
    })
}
pub(crate) fn config_home() -> Result<PathBuf> {
    if let Some(path) = nonempty_env("YAZELIX_CONFIG_HOME") {
        return Ok(PathBuf::from(path));
    }
    if let Some(path) = nonempty_env("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(path).join("yazelix"));
    }
    let home = nonempty_env("HOME").ok_or_else(|| error("HOME is required"))?;
    Ok(PathBuf::from(home).join(".config/yazelix"))
}
