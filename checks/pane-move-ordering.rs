use kinestra::{Recorder, Result, Size};
use std::{
    env, fs,
    process::{Command, ExitCode, Stdio},
    time::{Duration, Instant},
};

const SESSION: &str = "nova-pane-move-ordering";
const TIMEOUT: Duration = Duration::from_secs(20);
const MOVE_TIMEOUT: Duration = Duration::from_secs(3);

fn program(name: &str) -> std::ffi::OsString {
    env::var_os(name).unwrap_or_else(|| panic!("Nix supplies {name}"))
}

fn panes(
    recorder: &mut Recorder,
    zellij: &std::ffi::OsStr,
) -> Result<(Vec<u32>, u32, bool, Vec<String>)> {
    let json = recorder.output(Command::new(zellij).args([
        "-s",
        SESSION,
        "action",
        "list-panes",
        "--all",
        "--json",
    ]))?;
    let path = recorder.work().join("panes.json");
    fs::write(&path, json)?;
    let work = recorder.output(
        Command::new(program("JQ_BIN"))
            .args([
                "-r",
                ".[] | select(.is_plugin | not) | [.id,.is_focused,.pane_y,.pane_rows] | @tsv",
            ])
            .arg(&path),
    )?;
    let mut work = work
        .lines()
        .map(|line| {
            let fields = line.split('\t').collect::<Vec<_>>();
            (
                fields[0].parse::<u32>().unwrap(),
                fields[1] == "true",
                fields[2].parse::<usize>().unwrap(),
                fields[3].parse::<usize>().unwrap(),
            )
        })
        .collect::<Vec<_>>();
    work.sort_by_key(|pane| pane.2);
    let order = work.iter().map(|pane| pane.0).collect();
    let focused = work.iter().find(|pane| pane.1).expect("focused work pane");
    let focused_expanded = focused.3 > 1;

    let ui = recorder.output(
        Command::new(program("JQ_BIN"))
            .args([
                "-r",
                ".[] | select(.is_plugin and ((.title == \"sidebar\") or (.title == \"status-bar\") or ((.pane_y == 0) and (.pane_rows == 1)))) | [.title,.pane_x,.pane_y,.pane_columns,.pane_rows] | @tsv",
            ])
            .arg(&path),
    )?;
    let mut ui = ui.lines().map(str::to_owned).collect::<Vec<_>>();
    ui.sort();
    Ok((order, focused.0, focused_expanded, ui))
}

fn expect(
    recorder: &mut Recorder,
    zellij: &std::ffi::OsStr,
    order: &[u32],
    focused_pane: u32,
    ui: &[String],
) -> Result<()> {
    let deadline = Instant::now() + MOVE_TIMEOUT;
    loop {
        let (actual, focused, focused_expanded, actual_ui) = panes(recorder, zellij)?;
        if actual == order && focused == focused_pane && focused_expanded && actual_ui == ui {
            return Ok(());
        }
        if Instant::now() >= deadline {
            assert_eq!(actual, order);
            assert_eq!(focused, focused_pane);
            assert!(focused_expanded, "focused pane must stay expanded");
            assert_eq!(actual_ui, ui);
        }
        recorder.sleep(Duration::from_millis(50))?;
    }
}

fn sidebar_width(ui: &[String]) -> Option<usize> {
    ui.iter()
        .find(|row| row.starts_with("sidebar\t"))
        .and_then(|row| row.split('\t').nth(3))
        .and_then(|columns| columns.parse().ok())
}

fn expect_sidebar_width(
    recorder: &mut Recorder,
    zellij: &std::ffi::OsStr,
    order: &[u32],
    focused_pane: u32,
    expected_width: usize,
) -> Result<()> {
    let deadline = Instant::now() + MOVE_TIMEOUT;
    loop {
        let (actual, focused, focused_expanded, ui) = panes(recorder, zellij)?;
        if actual == order
            && focused == focused_pane
            && focused_expanded
            && sidebar_width(&ui) == Some(expected_width)
        {
            return Ok(());
        }
        if Instant::now() >= deadline {
            assert_eq!(actual, order);
            assert_eq!(focused, focused_pane);
            assert!(focused_expanded, "focused pane must stay expanded");
            assert_eq!(sidebar_width(&ui), Some(expected_width));
        }
        recorder.sleep(Duration::from_millis(50))?;
    }
}

fn wait_for_session(recorder: &mut Recorder, zellij: &std::ffi::OsStr) -> Result<()> {
    let deadline = Instant::now() + TIMEOUT;
    loop {
        let output = Command::new(zellij)
            .args(["-s", SESSION, "action", "list-panes", "--all", "--json"])
            .output()?;
        if output.status.success()
            && String::from_utf8_lossy(&output.stdout).contains("yazi_picker")
        {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(std::io::Error::other("fresh Nova session did not become ready").into());
        }
        recorder.sleep(Duration::from_millis(100))?;
    }
}

fn record(recorder: &mut Recorder) -> Result<()> {
    let yzx = program("YZX_BIN");
    let zellij = program("ZELLIJ_BIN");
    let rio = program("RIO_BIN");
    let home = recorder.work().join("home");
    let config = recorder.work().join("config");
    let state = recorder.work().join("state");
    let data = recorder.work().join("data");
    for path in [&home, &config, &state, &data, &home.join(".config/rio")] {
        fs::create_dir_all(path)?;
    }
    fs::write(config.join("config.toml"), "[welcome]\nenabled = false\n")?;
    fs::write(home.join(".config/rio/config.toml"), "")?;

    let _ = Command::new(&zellij)
        .args(["kill-session", SESSION])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let mut cleanup = Command::new(&zellij);
    cleanup.args(["kill-session", SESSION]);
    recorder.on_exit(cleanup);
    recorder.display(Size::new(1600, 900)?, None)?;
    recorder.launch(
        "yzx",
        Command::new(&rio)
            .args(["--app-id", "yzx", "-e"])
            .arg(&yzx)
            .args(["enter", "--session", SESSION])
            .env("HOME", &home)
            .env("YAZELIX_CONFIG_HOME", &config)
            .env("YAZELIX_STATE_DIR", &state)
            .env("XDG_DATA_HOME", &data)
            .env("VK_ADD_DRIVER_FILES", program("VK_ADD_DRIVER_FILES")),
    )?;
    wait_for_session(recorder, &zellij)?;
    for _ in 0..2 {
        recorder.key("alt+m", Duration::from_millis(350))?;
    }
    recorder.exec(Command::new(&zellij).args([
        "-s",
        SESSION,
        "action",
        "focus-pane-id",
        "terminal_0",
    ]))?;
    recorder.sleep(Duration::from_millis(300))?;

    let (initial, focused, focused_expanded, ui) = panes(recorder, &zellij)?;
    assert_eq!(initial, [0, 1, 2]);
    assert_eq!(focused, 0);
    assert!(focused_expanded, "focused pane must start expanded");
    assert_eq!(ui.len(), 3, "expected top bar, sidebar and status bar");
    let open_sidebar_width = sidebar_width(&ui).expect("sidebar geometry");
    assert!(open_sidebar_width > 2, "sidebar must begin expanded");
    recorder.key("alt+shift+h", Duration::from_millis(300))?;
    expect_sidebar_width(recorder, &zellij, &[0, 1, 2], 0, 1)?;
    recorder.key("alt+shift+h", Duration::from_millis(300))?;
    expect(recorder, &zellij, &[0, 1, 2], 0, &ui)?;

    recorder.key("alt+j", Duration::from_millis(300))?;
    expect(recorder, &zellij, &[0, 1, 2], 1, &ui)?;
    recorder.key("ctrl+alt+k", Duration::from_secs(1))?;
    recorder.key("ctrl+alt+j", Duration::from_millis(300))?;
    expect(recorder, &zellij, &[0, 1, 2], 1, &ui)?;

    recorder.exec(Command::new(&zellij).args([
        "-s",
        SESSION,
        "action",
        "focus-pane-id",
        "terminal_0",
    ]))?;
    recorder.key("ctrl+alt+k", Duration::from_millis(300))?;
    expect(recorder, &zellij, &[1, 2, 0], 0, &ui)?;
    recorder.key("ctrl+alt+j", Duration::from_millis(300))?;
    expect(recorder, &zellij, &[0, 1, 2], 0, &ui)?;

    for expected in [[1, 0, 2], [1, 2, 0], [0, 1, 2]] {
        recorder.key("ctrl+alt+j", Duration::from_millis(200))?;
        expect(recorder, &zellij, &expected, 0, &ui)?;
    }
    for expected in [[1, 2, 0], [1, 0, 2], [0, 1, 2]] {
        recorder.key("ctrl+alt+k", Duration::from_millis(200))?;
        expect(recorder, &zellij, &expected, 0, &ui)?;
    }
    for _ in 0..10 {
        recorder.key("ctrl+alt+k", Duration::ZERO)?;
        recorder.key("ctrl+alt+j", Duration::from_millis(300))?;
        expect(recorder, &zellij, &[0, 1, 2], 0, &ui)?;
    }
    recorder.exec(Command::new(&zellij).args([
        "-s",
        SESSION,
        "action",
        "close-pane",
        "--pane-id",
        "terminal_2",
    ]))?;
    expect(recorder, &zellij, &[0, 1], 0, &ui)?;
    recorder.key("ctrl+alt+k", Duration::from_millis(300))?;
    expect(recorder, &zellij, &[1, 0], 0, &ui)?;
    recorder.key("ctrl+alt+j", Duration::from_millis(300))?;
    expect(recorder, &zellij, &[0, 1], 0, &ui)?;
    Ok(())
}

fn main() -> ExitCode {
    kinestra::run(record)
}
