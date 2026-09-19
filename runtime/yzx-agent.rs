use std::{
    env,
    ffi::{OsStr, OsString},
    fs, io,
    io::{BufRead, IsTerminal, Write},
    os::unix::{fs::PermissionsExt, process::CommandExt},
    path::{Path, PathBuf},
    process::{Command, Stdio, exit},
};

const PROVIDERS: &[(&str, &[&str])] = &[
    ("codex", &["resume"]),
    ("grok", &[]),
    ("opencode", &[]),
    ("pi", &[]),
    ("claude", &["--resume"]),
];

fn main() {
    exit(run());
}

fn run() -> i32 {
    emit_initial_title();
    let state_dir = state_dir();
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    if let Some((command, args)) = args.split_first() {
        return launch(command, args, &state_dir);
    }
    let provider_file = state_dir.join("agent/provider");

    if let Some(id) = read_provider(&provider_file) {
        return launch_configured(&id, &provider_file, &state_dir);
    }

    for (provider, provider_args) in PROVIDERS.iter().copied() {
        if command_available(provider) {
            let _ = write_provider(&provider_file, provider);
            return launch(OsStr::new(provider), provider_args, &state_dir);
        }
    }

    0
}

fn launch<T: AsRef<OsStr>>(command: &OsStr, args: &[T], state_dir: &Path) -> i32 {
    if Path::new(command).file_name() == Some(OsStr::new("codex")) && radar_enabled() {
        offer_codex_radar_setup(command, state_dir);
    }
    exec_command(command, args)
}

fn radar_enabled() -> bool {
    env::var_os("YZX_RADAR_ENABLED").as_deref() != Some(OsStr::new("false"))
}

fn emit_initial_title() {
    let mut stdout = io::stdout().lock();
    let _ = stdout.write_all(b"\x1b]0;agent popup\x07");
    let _ = stdout.flush();
}

fn exec_command<T: AsRef<OsStr>>(command: &OsStr, args: &[T]) -> i32 {
    let error = Command::new(command).args(args).exec();
    eprintln!(
        "Yazelix Nova agent popup\n\nFailed to launch `{}`: {error}",
        command.to_string_lossy()
    );
    pause_if_tty();
    127
}

fn launch_configured(id: &str, provider_file: &Path, state_dir: &Path) -> i32 {
    let Some((provider, provider_args)) = PROVIDERS
        .iter()
        .copied()
        .find(|(provider, _)| *provider == id)
    else {
        eprintln!(
            "Yazelix Nova agent popup\n\nConfigured agent provider `{id}` is unknown.\nRemove {} to let Yazelix choose again.",
            provider_file.display()
        );
        pause_if_tty();
        return 127;
    };

    if !command_available(provider) {
        eprintln!(
            "Yazelix Nova agent popup\n\nConfigured agent provider `{id}` is not available on PATH.\nInstall it or remove {} to let Yazelix choose again.",
            provider_file.display()
        );
        pause_if_tty();
        return 127;
    }

    launch(OsStr::new(provider), provider_args, state_dir)
}

fn offer_codex_radar_setup(codex: &OsStr, state_dir: &Path) {
    let marker = state_dir.join("agent/radar-codex-setup-offered");
    if !command_available("zj-radar") {
        return;
    }
    let interactive = io::stdin().is_terminal() && io::stderr().is_terminal();

    match radar_check(codex) {
        Ok(RadarHealth::Healthy) => {
            remember_radar_disposition(&marker, "enabled");
            return;
        }
        Ok(RadarHealth::Unhealthy) => {}
        Ok(RadarHealth::Unknown) => {
            eprintln!(
                "Yazelix Nova: could not interpret Radar's Codex hook check; Codex will still start."
            );
            return;
        }
        Err(error) => {
            eprintln!(
                "Yazelix Nova: failed to check Radar's Codex hooks: {error}; Codex will still start."
            );
            return;
        }
    }

    match read_provider(&marker).as_deref() {
        Some("enabled") => {
            if interactive {
                notify_missing_radar_hooks();
            }
            return;
        }
        Some("declined") => return,
        _ => {}
    }

    if !interactive {
        return;
    }
    eprint!(
        "Radar needs Codex hooks to show agent activity.\nYou can enable this later: zj-radar setup codex\n\nEnable Codex activity in Radar? [Y/n] "
    );
    let _ = io::stderr().flush();
    let Some(install) = read_offer_consent(io::stdin().lock()) else {
        return;
    };
    if !install {
        remember_radar_disposition(&marker, "declined");
        return;
    }
    match radar_command(codex, "--yes").status() {
        Ok(status) if status.success() => match radar_check(codex) {
            Ok(RadarHealth::Healthy) => remember_radar_disposition(&marker, "enabled"),
            Ok(_) => eprintln!(
                "Yazelix Nova: Radar setup did not establish healthy Codex hooks; Codex will still start."
            ),
            Err(error) => eprintln!(
                "Yazelix Nova: failed to verify Radar setup: {error}; Codex will still start."
            ),
        },
        Ok(status) => {
            eprintln!("Yazelix Nova: Radar setup failed with {status}; Codex will still start.")
        }
        Err(error) => {
            eprintln!("Yazelix Nova: failed to run Radar setup: {error}; Codex will still start.")
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum RadarHealth {
    Healthy,
    Unhealthy,
    Unknown,
}

fn radar_check(codex: &OsStr) -> io::Result<RadarHealth> {
    let output = radar_command(codex, "--check").output()?;
    Ok(classify_radar_report(
        output.status.success(),
        &String::from_utf8_lossy(&output.stdout),
    ))
}

fn classify_radar_report(success: bool, report: &str) -> RadarHealth {
    let mut codex_report = false;
    let mut codex_ok = false;
    let mut radar_ok = false;
    let mut hooks_ok = false;
    let mut feature_ok = false;
    let mut unhealthy = false;
    for line in report.lines() {
        if line == "codex:" {
            codex_report = true;
            continue;
        }
        if !codex_report {
            continue;
        }
        let line = line.trim_start();
        unhealthy |= line.starts_with("warn ") || line.starts_with("missing ");
        codex_ok |= line.starts_with("ok codex binary:");
        radar_ok |= line.starts_with("ok zj-radar binary:");
        hooks_ok |= line.starts_with("ok hooks.json:");
        feature_ok |= line.starts_with("ok hooks feature:");
    }
    if !codex_report {
        RadarHealth::Unknown
    } else if unhealthy {
        RadarHealth::Unhealthy
    } else if success && codex_ok && radar_ok && hooks_ok && feature_ok {
        RadarHealth::Healthy
    } else {
        RadarHealth::Unknown
    }
}

fn read_offer_consent(mut input: impl BufRead) -> Option<bool> {
    let mut answer = String::new();
    if input.read_line(&mut answer).ok()? == 0 {
        return None;
    }
    match answer.trim().to_ascii_lowercase().as_str() {
        "" | "y" | "yes" => Some(true),
        "n" | "no" => Some(false),
        _ => None,
    }
}

fn radar_command(codex: &OsStr, flag: &str) -> Command {
    let mut command = Command::new("zj-radar");
    command.args(["setup", "codex", flag]);
    if let Some(parent) = Path::new(codex)
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
    {
        let mut path = env::var_os("PATH").unwrap_or_default();
        if !path.is_empty() {
            path.push(":");
        }
        path.push(parent);
        command.env("PATH", path);
    }
    command
}

fn remember_radar_disposition(marker: &Path, disposition: &str) {
    if let Err(error) = write_provider(marker, disposition) {
        eprintln!(
            "Yazelix Nova: could not remember the Radar setup choice at {}: {error}",
            marker.display()
        );
    }
}

fn notify_missing_radar_hooks() {
    let (Some(zellij), Some(session)) = (
        nonempty_env("YZX_ZELLIJ"),
        nonempty_env("ZELLIJ_SESSION_NAME"),
    ) else {
        return;
    };
    let status = notification_command(&zellij, &session)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .status();
    match status {
        Ok(status) if status.success() => {}
        Ok(status) => eprintln!(
            "Yazelix Nova: Radar warning could not be shown ({status}); Codex will still start."
        ),
        Err(error) => eprintln!(
            "Yazelix Nova: Radar warning could not be shown: {error}; Codex will still start."
        ),
    }
}

fn notification_command(zellij: &OsStr, session: &OsStr) -> Command {
    let mut command = Command::new(zellij);
    command
        .args([
            "action",
            "pipe",
            "--name",
            "zjstatus",
            "--",
            "zjstatus::notify::⚠ Radar cannot see Codex activity · run zj-radar setup codex",
        ])
        .env("ZELLIJ_SESSION_NAME", session);
    command
}

fn read_provider(path: &Path) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|text| text.trim().to_string())
        .filter(|id| !id.is_empty())
}

fn write_provider(path: &Path, id: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, format!("{id}\n"))
}

fn command_available(command: &str) -> bool {
    let Some(path) = env::var_os("PATH").filter(|path| !path.is_empty()) else {
        return false;
    };

    env::split_paths(&path).any(|entry| is_executable(&entry.join(command)))
}

fn is_executable(path: &Path) -> bool {
    fs::metadata(path)
        .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

fn state_dir() -> PathBuf {
    nonempty_env("YAZELIX_STATE_DIR")
        .map(PathBuf::from)
        .or_else(|| nonempty_env("XDG_DATA_HOME").map(|path| PathBuf::from(path).join("yazelix")))
        .or_else(|| {
            nonempty_env("HOME").map(|path| PathBuf::from(path).join(".local/share/yazelix"))
        })
        .unwrap_or_else(|| PathBuf::from("/tmp/yazelix"))
}

fn nonempty_env(name: &str) -> Option<OsString> {
    env::var_os(name).filter(|value| !value.is_empty())
}

fn pause_if_tty() {
    if io::stdin().is_terminal() {
        eprint!("\nPress Enter to close this popup...");
        let mut line = String::new();
        let _ = io::stdin().read_line(&mut line);
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsStr;

    use super::{RadarHealth, classify_radar_report, notification_command, read_offer_consent};

    #[test]
    fn radar_health_uses_the_report_not_just_the_exit_code() {
        let healthy = "codex:\n  ok codex binary: found on PATH\n  ok zj-radar binary: found on PATH\n  ok hooks feature: enabled or unset in config.toml\n  ok hooks.json: all zj-radar Codex hooks installed\n  note hook trust: review with /hooks\n";
        assert_eq!(classify_radar_report(true, healthy), RadarHealth::Healthy);
        assert_eq!(
            classify_radar_report(
                true,
                &healthy.replace("  ok hooks.json:", "  warn hooks.json:")
            ),
            RadarHealth::Unhealthy
        );
        assert_eq!(
            classify_radar_report(
                true,
                &healthy.replace("  ok hooks feature:", "  warn hooks feature:")
            ),
            RadarHealth::Unhealthy
        );
        assert_eq!(
            classify_radar_report(
                false,
                &healthy.replace("  ok hooks.json:", "  missing hooks.json:")
            ),
            RadarHealth::Unhealthy
        );
        assert_eq!(
            classify_radar_report(
                true,
                &format!("{healthy}  warn hook enablement: disabled\n")
            ),
            RadarHealth::Unhealthy
        );
        assert_eq!(classify_radar_report(true, ""), RadarHealth::Unknown);
        assert_eq!(classify_radar_report(false, healthy), RadarHealth::Unknown);
        assert_eq!(
            classify_radar_report(
                true,
                &healthy.replace("  ok codex binary:", "  note codex binary:")
            ),
            RadarHealth::Unknown
        );

        let pipe = notification_command(OsStr::new("zellij"), OsStr::new("nova-test"));
        assert_eq!(
            pipe.get_args()
                .map(OsStr::to_str)
                .collect::<Option<Vec<_>>>()
                .unwrap(),
            [
                "action",
                "pipe",
                "--name",
                "zjstatus",
                "--",
                "zjstatus::notify::⚠ Radar cannot see Codex activity · run zj-radar setup codex",
            ]
        );
        assert_eq!(
            pipe.get_envs()
                .find(|(key, _)| *key == OsStr::new("ZELLIJ_SESSION_NAME"))
                .and_then(|(_, value)| value),
            Some(OsStr::new("nova-test"))
        );
    }

    #[test]
    fn radar_setup_offer_accepts_default_and_explicit_answers() {
        assert_eq!(read_offer_consent("".as_bytes()), None);
        assert_eq!(read_offer_consent("\n".as_bytes()), Some(true));
        assert_eq!(read_offer_consent("yes\n".as_bytes()), Some(true));
        assert_eq!(read_offer_consent("no\n".as_bytes()), Some(false));
        assert_eq!(read_offer_consent("later\n".as_bytes()), None);
    }
}
