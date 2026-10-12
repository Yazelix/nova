use kinestra::{Recorder, Result, Size};
use std::{
    env, fs,
    path::Path,
    process::{Command, ExitCode},
    time::{Duration, Instant},
};

fn tmux(socket: &Path, args: &[&str]) -> Command {
    let mut command = Command::new("tmux");
    command
        .args(["-f", "/dev/null", "-S"])
        .arg(socket)
        .args(args);
    command
}

fn wait_for_text(r: &mut Recorder, socket: &Path, marker: &str) -> Result<()> {
    let start = Instant::now();
    loop {
        let screen = r.output(&mut tmux(
            socket,
            &["capture-pane", "-p", "-t", "watcher:0.0"],
        ))?;
        fs::write(r.work().join("last-screen.txt"), &screen)?;
        if screen.contains(marker) {
            println!("{marker}: {:.2}s", start.elapsed().as_secs_f32());
            return Ok(());
        }
        if start.elapsed() > Duration::from_secs(20) {
            r.snapshot(&r.work().join("timeout.png"))?;
            return Err(
                std::io::Error::other(format!("Helix did not show {marker}:\n{screen}")).into(),
            );
        }
        r.sleep(Duration::from_millis(100))?;
    }
}

fn record(r: &mut Recorder) -> Result<()> {
    let root = r.work().to_owned();
    let home = root.join("home");
    let config = root.join("config");
    let state = root.join("state");
    let socket = root.join("tmux.sock");
    assert!(socket.as_os_str().as_encoded_bytes().len() < 108);
    for path in [
        &home,
        &config,
        &state,
        &root.join("cache"),
        &root.join("steel"),
    ] {
        fs::create_dir_all(path)?;
    }
    fs::write(config.join("config.toml"), "[forest]\nenabled = false\n")?;
    let file = root.join("watch.txt");
    fs::write(&file, "WATCHER_INITIAL\n")?;

    r.display(Size::new(1200, 800)?, None)?;
    let isolated = r.command("true");
    let display = isolated
        .get_envs()
        .find(|(name, _)| *name == "DISPLAY")
        .unwrap()
        .1;
    assert!(display.is_some());
    assert_ne!(display.map(ToOwned::to_owned), env::var_os("DISPLAY"));
    assert!(
        isolated
            .get_envs()
            .any(|(name, value)| name == "WAYLAND_DISPLAY" && value.is_none())
    );

    r.on_exit(tmux(&socket, &["kill-server"]));
    r.exec(
        tmux(
            &socket,
            &[
                "new-session",
                "-d",
                "-s",
                "watcher",
                "-x",
                "120",
                "-y",
                "35",
            ],
        )
        .arg(env::var_os("YZX_BIN").expect("Nix supplies YZX_BIN"))
        .args(["run", "hx"])
        .arg(&file)
        .env("HOME", &home)
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env("XDG_DATA_HOME", root.join("data"))
        .env("XDG_CACHE_HOME", root.join("cache"))
        .env("YAZELIX_CONFIG_HOME", &config)
        .env("YAZELIX_STATE_DIR", &state)
        .env("STEEL_HOME", root.join("steel"))
        .env_remove("ZELLIJ")
        .env_remove("ZELLIJ_SESSION_NAME")
        .env_remove("TMUX"),
    )?;
    r.launch(
        "nova-watcher",
        Command::new("xterm")
            .args([
                "-class",
                "nova-watcher",
                "-e",
                "tmux",
                "-f",
                "/dev/null",
                "-S",
            ])
            .arg(&socket)
            .args(["attach-session", "-t", "watcher"]),
    )?;
    wait_for_text(r, &socket, "WATCHER_INITIAL")?;

    // Observe a write before replacing the inode, proving the watcher is already active.
    fs::write(&file, "WATCHER_FIRST_WRITE\n")?;
    wait_for_text(r, &socket, "WATCHER_FIRST_WRITE")?;
    let replacement = root.join("replacement");
    fs::write(&replacement, "WATCHER_ATOMIC_REPLACEMENT\n")?;
    fs::rename(replacement, &file)?;
    wait_for_text(r, &socket, "WATCHER_ATOMIC_REPLACEMENT")?;
    fs::write(&file, "WATCHER_LATER_WRITE\n")?;
    wait_for_text(r, &socket, "WATCHER_LATER_WRITE")?;
    Ok(())
}

fn main() -> ExitCode {
    kinestra::run(record)
}
