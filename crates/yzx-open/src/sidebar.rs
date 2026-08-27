//! Shared calls into Nova's Zellij plugins.

use anyhow::{Context, Result, bail};
use std::{
    env,
    ffi::OsString,
    process::{Command, Output},
};

const ORCHESTRATOR_PLUGIN: &str = "yazelix_pane_orchestrator";
const POPUP_PLUGIN: &str = "yzpp";
const ZELLIJ_SESSION_NAME_ENV: &str = "ZELLIJ_SESSION_NAME";

#[derive(Clone, Debug)]
pub struct Config {
    pub zellij: OsString,
    pub zellij_session_name: Option<OsString>,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            zellij: nonempty_env("YZX_ZELLIJ").unwrap_or_else(|| "zellij".into()),
            zellij_session_name: nonempty_env(ZELLIJ_SESSION_NAME_ENV)
                .or_else(|| nonempty_env("YAZELIX_ZELLIJ_SESSION_NAME")),
        }
    }
}

pub fn orchestrator_query(config: &Config, name: &str) -> Result<String> {
    let response = orchestrator_pipe(config, name, "")?;
    if response.is_empty() {
        bail!("pane orchestrator returned no response for {name}");
    }
    Ok(response)
}

pub fn orchestrator_pipe(config: &Config, name: &str, payload: &str) -> Result<String> {
    plugin_pipe(config, ORCHESTRATOR_PLUGIN, name, payload)
        .with_context(|| format!("could not pipe {name} to pane orchestrator"))
}

pub fn popup_pipe(config: &Config, name: &str, payload: &str) -> Result<String> {
    plugin_pipe(config, POPUP_PLUGIN, name, payload)
        .with_context(|| format!("could not pipe {name} to popup plugin"))
}

fn plugin_pipe(config: &Config, plugin: &str, name: &str, payload: &str) -> Result<String> {
    let mut command = Command::new(&config.zellij);
    if let Some(session_name) = &config.zellij_session_name {
        command.env(ZELLIJ_SESSION_NAME_ENV, session_name);
    }
    let output = command
        .args([
            "action", "pipe", "--plugin", plugin, "--name", name, "--", payload,
        ])
        .output()
        .context("could not run zellij action pipe")?;
    ensure_success(&output, "zellij action pipe failed")?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn ensure_success(output: &Output, context: &str) -> Result<()> {
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let message = [stderr.trim(), stdout.trim()]
        .into_iter()
        .find(|part| !part.is_empty())
        .unwrap_or("no output");
    bail!("{context}: {message}");
}

fn nonempty_env(name: &str) -> Option<OsString> {
    env::var_os(name).filter(|value| !value.is_empty())
}
