use kinestra::{Recorder, Result, Size};
use std::{
    env, fs,
    io::{self, Write},
    path::Path,
    process::{Command, ExitCode, Stdio},
    thread,
    time::{Duration, Instant},
};

const TIMEOUT: Duration = Duration::from_secs(20);

fn panes(zellij: &Path, session: &str) -> Result<String> {
    let output = Command::new(zellij)
        .args(["-s", session, "action", "list-panes", "--all", "--json"])
        .output()?;
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        if error.trim() == "There is no active session!" {
            return Ok("[]".into());
        }
        return Err(io::Error::other(error.into_owned()).into());
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn jq(json: &str, filter: &str) -> Result<bool> {
    let mut child = Command::new("jq")
        .args(["-e", filter])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()?;
    child.stdin.take().unwrap().write_all(json.as_bytes())?;
    Ok(child.wait()?.success())
}

fn wait_for_panes(
    recorder: &mut Recorder,
    zellij: &Path,
    session: &str,
    filter: &str,
) -> Result<()> {
    let deadline = Instant::now() + TIMEOUT;
    loop {
        let state = panes(zellij, session)?;
        if jq(&state, filter)? {
            return Ok(());
        }
        if Instant::now() >= deadline {
            if let Ok(output) = Command::new(zellij)
                .args(["-s", session, "action", "dump-screen"])
                .output()
            {
                eprintln!("Last screen:\n{}", String::from_utf8_lossy(&output.stdout));
            }
            return Err(
                io::Error::other(format!("pane state never matched {filter}: {state}")).into(),
            );
        }
        recorder.sleep(Duration::from_millis(100))?;
    }
}

fn wait_for_screen(
    recorder: &mut Recorder,
    zellij: &Path,
    session: &str,
    needle: &str,
) -> Result<()> {
    let deadline = Instant::now() + TIMEOUT;
    loop {
        let output = Command::new(zellij)
            .args(["-s", session, "action", "dump-screen"])
            .output()?;
        let screen = String::from_utf8_lossy(&output.stdout).into_owned();
        if output.status.success() && screen.contains(needle) {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(io::Error::other(format!(
                "screen never contained {needle}; last screen:\n{screen}"
            ))
            .into());
        }
        recorder.sleep(Duration::from_millis(100))?;
    }
}

fn wait_for_session_exit(zellij: &Path, session: &str) -> Result<()> {
    let deadline = Instant::now() + TIMEOUT;
    loop {
        let output = Command::new(zellij)
            .args(["list-sessions", "--no-formatting", "--short"])
            .output()?;
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            if error.trim() == "No active zellij sessions found." {
                return Ok(());
            }
            return Err(io::Error::other(error.into_owned()).into());
        }
        if !String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|name| name.trim() == session)
        {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(io::Error::other(format!("session {session} did not exit")).into());
        }
        thread::sleep(Duration::from_millis(100));
    }
}

fn write_chars(zellij: &Path, session: &str, chars: &str) -> Result<()> {
    let status = Command::new(zellij)
        .args(["-s", session, "action", "write-chars", chars])
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!("could not write characters to {session}")).into())
    }
}

fn new_tab(zellij: &Path, session: &str, layout: &Path, cwd: &Path) -> Result<()> {
    let status = Command::new(zellij)
        .args(["-s", session, "action", "new-tab", "--layout"])
        .arg(layout)
        .arg("--cwd")
        .arg(cwd)
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!("could not create a tab in {session}")).into())
    }
}

fn launch(
    recorder: &mut Recorder,
    yzx: &Path,
    session: &str,
    cwd: &Path,
    home: &Path,
) -> Result<()> {
    recorder.launch(
        "nova-picker-cancel",
        Command::new("xterm")
            .current_dir(cwd)
            .env("HOME", home)
            .env("YAZELIX_CONFIG_HOME", home.join(".config/yazelix"))
            .env("YAZELIX_STATE_DIR", home.join(".local/share/yazelix"))
            .env("_ZO_DATA_DIR", home.join(".local/share/zoxide"))
            .env("FZF_DEFAULT_OPTS", "--not-a-real-fzf-option")
            .env("FZF_DEFAULT_OPTS_FILE", home.join("hostile-fzf-options"))
            .args(["-class", "nova-picker-cancel", "-e"])
            .arg(yzx)
            .args(["enter", "--session", session]),
    )?;
    recorder.sleep(Duration::from_secs(1))
}

fn record(recorder: &mut Recorder) -> Result<()> {
    let yzx = env::var_os("YZX_BIN").expect("Nix supplies YZX_BIN");
    let zellij = env::var_os("ZELLIJ_BIN").expect("Nix supplies ZELLIJ_BIN");
    let yzx = Path::new(&yzx);
    let zellij = Path::new(&zellij);
    let sessions = ["nova-picker-cancel-later", "nova-picker-cancel-only"];
    let cleanup_exe = env::current_exe()?;
    for session in sessions {
        let _ = Command::new(zellij)
            .args(["kill-session", session])
            .status();
        let mut cleanup = Command::new(&cleanup_exe);
        cleanup.args(["--cleanup-session", session]);
        recorder.on_exit(cleanup);
    }

    let home = recorder.work().join("account");
    let picker_dir = recorder.work().join("picker");
    let nested_dir = picker_dir.join("nested");
    let quick_dir = recorder.work().join("quick-target");
    let vanished_dir = recorder.work().join("vanished-target");
    fs::create_dir_all(home.join(".config/yazelix"))?;
    fs::create_dir_all(&nested_dir)?;
    fs::create_dir(&quick_dir)?;
    fs::create_dir(&vanished_dir)?;
    fs::write(
        home.join(".config/yazelix/config.toml"),
        "[welcome]\nenabled = false\n",
    )?;
    fs::write(
        home.join("hostile-fzf-options"),
        "--not-a-real-fzf-option\n",
    )?;
    fs::write(picker_dir.join("target.txt"), "picker cancellation proof\n")?;
    fs::write(nested_dir.join("inside.txt"), "nested picker proof\n")?;
    fs::write(quick_dir.join("quick.txt"), "quick picker proof\n")?;

    recorder.display(Size::new(1200, 720)?, None)?;
    launch(recorder, yzx, sessions[0], &picker_dir, &home)?;
    wait_for_panes(
        recorder,
        zellij,
        sessions[0],
        r#"any(.[]; .title == "yazi_picker" and .is_focused and .tab_name == "picker")"#,
    )?;
    wait_for_screen(recorder, zellij, sessions[0], "target.txt")?;
    wait_for_screen(recorder, zellij, sessions[0], "Tab/Z Search")?;
    wait_for_screen(recorder, zellij, sessions[0], "Shift+Tab Spot")?;
    // Rio's Kitty keyboard path sends physical Shift+Tab as CSI 9;2u.
    let status = Command::new(zellij)
        .args(["-s", sessions[0], "action", "write"])
        .args(["27", "91", "57", "59", "50", "117"])
        .status()?;
    if !status.success() {
        return Err(io::Error::other("could not send Kitty Shift+Tab").into());
    }
    wait_for_screen(recorder, zellij, sessions[0], "Size:")?;
    recorder.key("Tab", Duration::from_millis(100))?;
    wait_for_screen(recorder, zellij, sessions[0], "Tab/Z Search")?;
    recorder.key("shift+Tab", Duration::from_millis(100))?;
    wait_for_screen(recorder, zellij, sessions[0], "Size:")?;
    recorder.key("Tab", Duration::from_millis(100))?;
    wait_for_screen(recorder, zellij, sessions[0], "Tab/Z Search")?;
    recorder.key("Home", Duration::from_millis(100))?;
    recorder.key("Right", Duration::from_millis(100))?;
    wait_for_screen(recorder, zellij, sessions[0], "inside.txt")?;
    recorder.key("Return", Duration::from_millis(100))?;
    wait_for_panes(
        recorder,
        zellij,
        sessions[0],
        r#"any(.[]; .tab_position == 0 and .title == "editor" and .is_focused) and all(.[]; .title != "yazi_picker")"#,
    )?;
    wait_for_screen(recorder, zellij, sessions[0], "nested picker proof")?;

    let zoxide_status = Command::new("zoxide")
        .env("_ZO_DATA_DIR", home.join(".local/share/zoxide"))
        .args(["add", "--score", "100"])
        .arg(&quick_dir)
        .status()?;
    if !zoxide_status.success() {
        return Err(io::Error::other("could not seed isolated zoxide history").into());
    }

    let layout = yzx
        .parent()
        .and_then(Path::parent)
        .expect("yzx is installed under bin")
        .join("share/yazelix/layout.kdl");
    new_tab(zellij, sessions[0], &layout, &picker_dir)?;
    wait_for_panes(
        recorder,
        zellij,
        sessions[0],
        r#"([.[].tab_position] | unique | length) == 2 and any(.[]; .tab_position == 1 and .title == "yazi_picker" and .is_focused and .tab_name == "picker")"#,
    )?;
    wait_for_screen(recorder, zellij, sessions[0], "Ctrl+O Open in editor")?;
    write_chars(zellij, sessions[0], "quick-target")?;
    wait_for_screen(recorder, zellij, sessions[0], "Go to folder > quick-target")?;
    recorder.key("alt+Return", Duration::from_millis(100))?;
    recorder.key("ctrl+o", Duration::from_millis(100))?;
    wait_for_panes(
        recorder,
        zellij,
        sessions[0],
        r#"([.[].tab_position] | unique | length) == 2 and any(.[]; .tab_position == 1 and .title == "editor" and .is_focused) and all(.[]; .title != "yazi_picker")"#,
    )?;
    wait_for_screen(recorder, zellij, sessions[0], "quick.txt")?;

    new_tab(zellij, sessions[0], &layout, &picker_dir)?;
    wait_for_panes(
        recorder,
        zellij,
        sessions[0],
        r#"([.[].tab_position] | unique | length) == 3 and any(.[]; .tab_position == 2 and .title == "yazi_picker" and .is_focused)"#,
    )?;
    wait_for_screen(recorder, zellij, sessions[0], "Enter Go here")?;
    write_chars(zellij, sessions[0], "quick-target")?;
    wait_for_screen(recorder, zellij, sessions[0], "Go to folder > quick-target")?;
    recorder.key("Return", Duration::from_millis(100))?;
    wait_for_screen(recorder, zellij, sessions[0], "quick.txt")?;
    recorder.key("Z", Duration::from_millis(100))?;
    wait_for_screen(recorder, zellij, sessions[0], "Enter Go here")?;
    recorder.key("Tab", Duration::from_millis(100))?;
    wait_for_screen(recorder, zellij, sessions[0], "Tab/Z Search")?;
    recorder.key("q", Duration::from_millis(200))?;
    wait_for_panes(
        recorder,
        zellij,
        sessions[0],
        r#"([.[].tab_position] | unique | length) == 2 and any(.[]; .tab_position == 1 and .title == "editor" and .is_focused)"#,
    )?;

    let zoxide_status = Command::new("zoxide")
        .env("_ZO_DATA_DIR", home.join(".local/share/zoxide"))
        .args(["add", "--score", "100"])
        .arg(&vanished_dir)
        .status()?;
    if !zoxide_status.success() {
        return Err(io::Error::other("could not seed vanished zoxide target").into());
    }
    new_tab(zellij, sessions[0], &layout, &picker_dir)?;
    wait_for_screen(recorder, zellij, sessions[0], "Ctrl+O Open in editor")?;
    fs::remove_dir(&vanished_dir)?;
    write_chars(zellij, sessions[0], "vanished-target")?;
    wait_for_screen(
        recorder,
        zellij,
        sessions[0],
        "Go to folder > vanished-target",
    )?;
    recorder.key("ctrl+o", Duration::from_millis(100))?;
    wait_for_panes(
        recorder,
        zellij,
        sessions[0],
        r#"([.[].tab_position] | unique | length) == 3 and any(.[]; .tab_position == 2 and .title == "yazi_picker" and .is_focused)"#,
    )?;
    wait_for_screen(recorder, zellij, sessions[0], "target does not")?;
    wait_for_screen(recorder, zellij, sessions[0], "Tab/Z Search")?;
    recorder.key("q", Duration::from_millis(100))?;
    wait_for_panes(
        recorder,
        zellij,
        sessions[0],
        r#"([.[].tab_position] | unique | length) == 2 and any(.[]; .tab_position == 1 and .title == "editor" and .is_focused)"#,
    )?;

    Command::new(zellij)
        .args(["kill-session", sessions[0]])
        .status()?;
    recorder.stop_app()?;

    launch(recorder, yzx, sessions[1], &home, &home)?;
    wait_for_panes(
        recorder,
        zellij,
        sessions[1],
        r#"any(.[]; .title == "yazi_picker" and .is_focused and .tab_name == "account")"#,
    )?;
    wait_for_screen(recorder, zellij, sessions[1], "Enter Go here")?;
    recorder.key("Escape", Duration::from_millis(100))?;
    wait_for_screen(recorder, zellij, sessions[1], "Tab/Z Search")?;
    // Expected app exit must not go through Recorder::key's app-health check.
    let status = Command::new(zellij)
        .args(["-s", sessions[1], "action", "send-keys", "Esc"])
        .status()?;
    if !status.success() {
        return Err(io::Error::other("could not cancel the final startup tab").into());
    }
    wait_for_session_exit(zellij, sessions[1])?;
    recorder.stop_app()
}

fn main() -> ExitCode {
    if env::args_os().nth(1).as_deref() == Some(std::ffi::OsStr::new("--cleanup-session")) {
        if let (Some(zellij), Some(session)) = (env::var_os("ZELLIJ_BIN"), env::args_os().nth(2)) {
            let _ = Command::new(zellij)
                .arg("kill-session")
                .arg(session)
                .status();
        }
        return ExitCode::SUCCESS;
    }
    kinestra::run(record)
}
