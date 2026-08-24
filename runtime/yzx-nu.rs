use std::{
    env,
    ffi::OsString,
    fs,
    io::{self, ErrorKind},
    os::unix::process::CommandExt,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
    time::{SystemTime, UNIX_EPOCH},
};

const NU: &str = "@nu@";
const PACKAGED_NU: &str = "@packagedNu@";
const PATH_PREFIX: &str = "@pathPrefix@";
const YZX_CONFIG: &str = "@yzxConfig@";
const ATUIN_INIT: &str = r#"if not (
    (scope commands | any {|command| $command.name == "_atuin_search_cmd" }) or
    ($env.config.keybindings? | default [] | any {|binding| ($binding.name? | default "") == "atuin" })
) {
    try {
        if "ATUIN_NOBIND" in $env {
            source "@atuinNoBindInit@"
        } else {
            source "@atuinInit@"
        }
    } catch {|error|
        print --stderr $"yzx-nu: managed Atuin init failed: ($error.msg)"
    }
}
"#;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("yzx-nu: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> io::Result<()> {
    let config_home = config_home()?;
    let user_nu = config_home.join("nu");
    let user_starship = config_home.join("starship.toml");
    let packaged_nu = PathBuf::from(PACKAGED_NU);
    let runtime = state_dir();
    let runtime_nu = runtime.join("nu");
    fs::create_dir_all(&runtime_nu)?;
    let starship_config = runtime.join("starship.toml");
    let status = Command::new(YZX_CONFIG)
        .arg("--write-effective-starship-config")
        .arg(&user_starship)
        .arg(&starship_config)
        .status()?;
    if !status.success() {
        return Err(io::Error::other(format!(
            "Starship config materializer exited with {status}"
        )));
    }

    let mise_init = host_mise_init();
    let atuin_init = atuin_enabled()?.then_some(ATUIN_INIT);
    let env_config = runtime_nu.join("env.nu");
    write_layered_config(
        &env_config,
        "source-env",
        &packaged_nu.join("env.nu"),
        &user_nu.join("env.nu"),
        None,
        None,
    )?;
    let config = runtime_nu.join("config.nu");
    write_layered_config(
        &config,
        "source",
        &packaged_nu.join("config.nu"),
        &user_nu.join("config.nu"),
        mise_init.as_deref(),
        atuin_init,
    )?;

    let error = Command::new(NU)
        .arg("--experimental-options=native-clip")
        .arg("--env-config")
        .arg(env_config)
        .arg("--config")
        .arg(config)
        .args(env::args_os().skip(1))
        .env("PATH", runtime_path())
        .env("STARSHIP_CONFIG", starship_config)
        .exec();
    Err(error)
}

fn config_home() -> io::Result<PathBuf> {
    nonempty_env("YAZELIX_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| nonempty_env("XDG_CONFIG_HOME").map(|path| PathBuf::from(path).join("yazelix")))
        .or_else(|| nonempty_env("HOME").map(|path| PathBuf::from(path).join(".config/yazelix")))
        .ok_or_else(|| io::Error::new(ErrorKind::NotFound, "HOME is required"))
}

fn state_dir() -> PathBuf {
    nonempty_env("YAZELIX_STATE_DIR")
        .map(PathBuf::from)
        .or_else(|| nonempty_env("XDG_DATA_HOME").map(|path| PathBuf::from(path).join("yazelix")))
        .or_else(|| {
            nonempty_env("HOME").map(|path| PathBuf::from(path).join(".local/share/yazelix"))
        })
        .unwrap_or_else(|| env::temp_dir().join("yazelix"))
}

fn write_layered_config(
    path: &Path,
    command: &str,
    packaged: &Path,
    user: &Path,
    after_packaged: Option<&str>,
    after_user: Option<&str>,
) -> io::Result<()> {
    let mut contents = format!("{command} {}\n", nu_quote(packaged));
    if let Some(snippet) = after_packaged {
        contents.push_str(snippet);
        if !snippet.ends_with('\n') {
            contents.push('\n');
        }
    }
    if user.is_file() {
        contents.push_str(&format!("{command} {}\n", nu_quote(user)));
    }
    if let Some(snippet) = after_user {
        contents.push_str(snippet);
    }
    atomic_write(path, contents)
}

fn atuin_enabled() -> io::Result<bool> {
    let output = Command::new(YZX_CONFIG)
        .args(["--get", "shell.atuin"])
        .output()?;
    if !output.status.success() {
        return Err(io::Error::other(format!(
            "shell.atuin lookup exited with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    match String::from_utf8_lossy(&output.stdout).trim() {
        "true" => Ok(true),
        "false" => Ok(false),
        value => Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("shell.atuin lookup returned {value:?}"),
        )),
    }
}

fn host_mise_init() -> Option<String> {
    let output = Command::new("mise")
        .arg("activate")
        .arg("nu")
        .env("PATH", runtime_path())
        .output()
        .ok()?;
    if output.status.success() {
        String::from_utf8(output.stdout)
            .ok()
            .filter(|text| !text.is_empty())
    } else {
        None
    }
}

fn nu_quote(path: &Path) -> String {
    let mut quoted = String::from("\"");
    for ch in path.as_os_str().to_string_lossy().chars() {
        match ch {
            '\\' => quoted.push_str("\\\\"),
            '"' => quoted.push_str("\\\""),
            _ => quoted.push(ch),
        }
    }
    quoted.push('"');
    quoted
}

fn atomic_write(path: &Path, contents: String) -> io::Result<()> {
    let tmp = path.with_extension(format!("tmp.{}.{}", std::process::id(), unix_nanos()));
    fs::write(&tmp, contents)?;
    fs::rename(tmp, path)
}

fn unix_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default()
}

fn runtime_path() -> OsString {
    match nonempty_env("PATH") {
        Some(path) => {
            let mut merged = OsString::from(PATH_PREFIX);
            merged.push(":");
            merged.push(path);
            merged
        }
        _ => PATH_PREFIX.into(),
    }
}

fn nonempty_env(name: &str) -> Option<OsString> {
    env::var_os(name).filter(|value| !value.is_empty())
}
