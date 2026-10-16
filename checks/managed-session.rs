use std::{
    env, fs,
    path::Path,
    time::{Duration, Instant},
};
#[allow(dead_code)]
mod tmux;
use tmux::{Terminal, quote, sleep};

const SESSION: &str = "nova-session";
// Darwin's sun_path holds 104 bytes, including the terminating NUL.
const SOCKET_LIMIT: usize = 103;

fn zellij_log(terminal: &Terminal) -> String {
    let tmp = terminal.root.path.join("tmp");
    // The distribution logs to <TMPDIR>/<name>-<uid>/<name>-log/<name>.log.
    let name = terminal.binary.file_name().unwrap().to_str().unwrap();
    fs::read_dir(&tmp)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path().join(format!("{name}-log/{name}.log")))
        .find_map(|log| fs::read_to_string(log).ok())
        .map(|log| {
            let lines = log.lines().collect::<Vec<_>>();
            lines[lines.len().saturating_sub(80)..].join("\n")
        })
        .unwrap_or_else(|| format!("no Zellij log under {}", tmp.display()))
}

fn wait(terminal: &Terminal, seconds: u64, context: &str, mut check: impl FnMut(&str) -> bool) {
    let start = Instant::now();
    loop {
        let screen = terminal.capture("clients:0", false);
        if check(&screen) {
            return;
        }
        if screen.contains("EXIT:") || start.elapsed() > Duration::from_secs(seconds) {
            panic!(
                "{context} after {:.1}s\nscreen:\n{screen}\nZellij log tail:\n{}",
                start.elapsed().as_secs_f32(),
                zellij_log(terminal)
            );
        }
        sleep(0.2);
    }
}

fn alive(terminal: &Terminal) -> bool {
    terminal
        .run(
            &terminal.binary,
            &["list-sessions", "--no-formatting"],
            None,
        )
        .lines()
        .any(|line| line.starts_with(&format!("{SESSION} ")) && !line.contains("EXITED"))
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let package = Path::new(&args[1]);
    let shell = &args[2];
    let mut terminal = Terminal::new(package, SESSION);
    let root = terminal.root.path.clone();
    for (key, value) in [
        ("TMPDIR", root.join("tmp")),
        ("XDG_CONFIG_HOME", root.join("xdg-config")),
        ("YAZELIX_CONFIG_HOME", root.join("config")),
        ("YAZELIX_STATE_DIR", root.join("state")),
    ] {
        fs::create_dir_all(&value).unwrap();
        terminal.environment.insert(key.into(), value.into());
    }
    fs::write(
        root.join("config/config.toml"),
        "[welcome]\nenabled = false\n",
    )
    .unwrap();
    let socket = root.join(format!("s/contract_version_1/{SESSION}"));
    assert!(
        socket.as_os_str().len() <= SOCKET_LIMIT,
        "Zellij socket path exceeds {SOCKET_LIMIT} bytes: {}",
        socket.display()
    );
    let yzx = package.join("bin/yzx");
    let yzx_text = quote(yzx.to_str().unwrap());
    terminal.tmux(&[
        "new-session",
        "-d",
        "-s",
        "clients",
        "-x",
        "160",
        "-y",
        "45",
        &format!("{yzx_text} enter --session {SESSION}; printf '\\nEXIT:%s\\n' $?; sleep 30"),
    ]);
    let started = Instant::now();
    // First startup compiles every packaged Wasm plugin.
    wait(
        &terminal,
        120,
        "managed workspace never rendered",
        |screen| {
            let lines = screen.lines().collect::<Vec<_>>();
            lines.first().is_some_and(|bar| bar.contains("NOVA "))
                && screen.contains("RADAR")
                && screen.contains("yazi_picker")
                && lines.last().is_some_and(|hints| hints.contains("M menu"))
        },
    );
    let rendered = started.elapsed();
    sleep(3.0);
    assert!(alive(&terminal), "Zellij server exited after startup");
    terminal.tmux(&["send-keys", "-t", "clients:0", "M-H"]);
    wait(
        &terminal,
        20,
        "pane orchestrator never hid Radar",
        |screen| !screen.contains("RADAR"),
    );
    terminal.tmux(&["send-keys", "-t", "clients:0", "M-H"]);
    wait(
        &terminal,
        20,
        "pane orchestrator never restored Radar",
        |screen| screen.contains("RADAR"),
    );
    terminal.tmux(&["send-keys", "-t", "clients:0", "M-M"]);
    wait(
        &terminal,
        20,
        "popup plugin never opened the menu",
        |screen| screen.contains("menu_popup") && screen.contains("Command Palette"),
    );
    terminal.tmux(&["send-keys", "-t", "clients:0", "M-M"]);
    wait(
        &terminal,
        20,
        "popup plugin never closed the menu",
        |screen| !screen.contains("menu_popup"),
    );
    let probe = root.join("probe");
    fs::create_dir(&probe).unwrap();
    let probe_text = quote(probe.to_str().unwrap());
    terminal.run(
        &terminal.binary,
        &[
            "--session",
            SESSION,
            "run",
            "--close-on-exit",
            "--",
            shell,
            "-c",
            &format!(
                "{yzx_text} status --json > {probe_text}/status; {yzx_text} doctor > {probe_text}/doctor 2>&1; echo $? > {probe_text}/done"
            ),
        ],
        None,
    );
    wait(
        &terminal,
        30,
        "in-session status and doctor never finished",
        |_| fs::read_to_string(probe.join("done")).is_ok_and(|code| code.ends_with('\n')),
    );
    let doctor = fs::read_to_string(probe.join("doctor")).unwrap();
    assert_eq!(
        fs::read_to_string(probe.join("done")).unwrap().trim(),
        "0",
        "yzx doctor failed inside the session:\n{doctor}"
    );
    for line in ["all packaged helpers and plugins found", "inside Zellij"] {
        assert!(
            doctor.contains(line),
            "doctor is missing {line:?}:\n{doctor}"
        );
    }
    let status = fs::read_to_string(probe.join("status")).unwrap();
    let status: serde_json::Value = serde_json::from_str(&status)
        .unwrap_or_else(|error| panic!("status --json is invalid ({error}): {status}"));
    assert_eq!(status["schema_version"], 1, "{status}");
    assert_eq!(status["inside_zellij"], true, "{status}");
    assert_eq!(status["state_dir"], root.join("state").to_str().unwrap());
    assert!(alive(&terminal), "Zellij server exited during the session");
    println!(
        "yzx enter rendered the bar, Radar, hints and Yazi picker in {:.1}s; the orchestrator toggled Radar, the popup plugin toggled the menu, and in-session status and doctor succeeded",
        rendered.as_secs_f32()
    );
}
