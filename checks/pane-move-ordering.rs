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

/// Tiled work panes as columns from left to right, each top to bottom, written
/// `0|1 2` for a single-pane left column beside a two-pane stack.
fn panes(
    recorder: &mut Recorder,
    zellij: &std::ffi::OsStr,
) -> Result<(String, u32, bool, Vec<String>)> {
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
                ".[] | select((.is_plugin or .is_floating) | not) | [.id,.is_focused,.pane_x,.pane_y,.pane_rows] | @tsv",
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
                fields[4].parse::<usize>().unwrap(),
            )
        })
        .collect::<Vec<_>>();
    work.sort_by_key(|pane| (pane.2, pane.3));
    let mut order = String::new();
    for (index, pane) in work.iter().enumerate() {
        if index > 0 {
            order.push(if work[index - 1].2 == pane.2 {
                ' '
            } else {
                '|'
            });
        }
        order.push_str(&pane.0.to_string());
    }
    let focused = work.iter().find(|pane| pane.1).expect("focused work pane");
    let focused_expanded = focused.4 > 1;

    let ui = recorder.output(
        Command::new(program("JQ_BIN"))
            .args([
                "-r",
                ".[] | select(.is_plugin and ((.title == \"sidebar\") or (.pane_rows == 1))) | [.title,.pane_x,.pane_y,.pane_columns,.pane_rows] | @tsv",
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
    order: &str,
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

fn expect_content_split(
    recorder: &mut Recorder,
    zellij: &std::ffi::OsStr,
    split: bool,
    expected_sidebar_width: usize,
) -> Result<()> {
    let deadline = Instant::now() + MOVE_TIMEOUT;
    loop {
        let json = recorder.output(Command::new(zellij).args([
            "-s",
            SESSION,
            "action",
            "list-panes",
            "--all",
            "--json",
        ]))?;
        let path = recorder.work().join("layout-panes.json");
        fs::write(&path, json)?;
        let xs = recorder.output(
            Command::new(program("JQ_BIN"))
                .args(["-r", ".[] | select(.is_plugin | not) | .pane_x"])
                .arg(&path),
        )?;
        let unique_xs = xs.lines().collect::<std::collections::HashSet<_>>().len();
        let (_, _, _, ui) = panes(recorder, zellij)?;
        if (unique_xs > 1) == split && sidebar_width(&ui) == Some(expected_sidebar_width) {
            return Ok(());
        }
        assert!(
            Instant::now() < deadline,
            "split={split}, x={xs:?}, ui={ui:?}"
        );
        recorder.sleep(Duration::from_millis(50))?;
    }
}

fn expect_sidebar_width(
    recorder: &mut Recorder,
    zellij: &std::ffi::OsStr,
    order: &str,
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

fn wait_for_orchestrator(recorder: &mut Recorder, zellij: &std::ffi::OsStr) -> Result<()> {
    let deadline = Instant::now() + TIMEOUT;
    loop {
        let state = recorder.output(Command::new(zellij).args([
            "-s",
            SESSION,
            "action",
            "pipe",
            "--name",
            "maintainer_debug_editor_state",
            "--",
            "debug",
        ]))?;
        if state.contains("\"active_tab_position\":0") {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(std::io::Error::other(format!(
                "pane orchestrator did not observe the active tab: {state}"
            ))
            .into());
        }
        recorder.sleep(Duration::from_millis(100))?;
    }
}

fn record(recorder: &mut Recorder) -> Result<()> {
    let yzx = program("YZX_BIN");
    let zellij = program("ZELLIJ_BIN");
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
        Command::new(&yzx)
            .args(["launch", "--session", SESSION])
            .env("HOME", &home)
            .env("YAZELIX_CONFIG_HOME", &config)
            .env("YAZELIX_STATE_DIR", &state)
            .env("XDG_DATA_HOME", &data)
            .env("VK_ADD_DRIVER_FILES", program("VK_ADD_DRIVER_FILES")),
    )?;
    recorder.sleep(Duration::from_secs(1))?;
    wait_for_session(recorder, &zellij)?;
    wait_for_orchestrator(recorder, &zellij)?;
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
    assert_eq!(initial, "0 1 2");
    assert_eq!(focused, 0);
    assert!(focused_expanded, "focused pane must start expanded");
    assert_eq!(ui.len(), 3, "expected top bar, sidebar and key hints");
    let open_sidebar_width = sidebar_width(&ui).expect("sidebar geometry");
    assert!(open_sidebar_width > 2, "sidebar must begin expanded");
    recorder.key("alt+bracketright", Duration::from_millis(300))?;
    expect_content_split(recorder, &zellij, true, open_sidebar_width)?;
    recorder.key("alt+bracketleft", Duration::from_millis(300))?;
    expect_content_split(recorder, &zellij, false, open_sidebar_width)?;
    recorder.key("alt+shift+h", Duration::from_millis(300))?;
    expect_sidebar_width(recorder, &zellij, "0 1 2", 0, 1)?;
    recorder.key("alt+bracketright", Duration::from_millis(300))?;
    expect_content_split(recorder, &zellij, true, 1)?;
    recorder.key("alt+bracketleft", Duration::from_millis(300))?;
    expect_content_split(recorder, &zellij, false, 1)?;
    recorder.key("alt+shift+h", Duration::from_millis(300))?;
    expect(recorder, &zellij, "0 1 2", 0, &ui)?;

    recorder.key("alt+j", Duration::from_millis(300))?;
    expect(recorder, &zellij, "0 1 2", 1, &ui)?;
    recorder.key("ctrl+alt+k", Duration::from_secs(1))?;
    recorder.key("ctrl+alt+j", Duration::from_millis(300))?;
    expect(recorder, &zellij, "0 1 2", 1, &ui)?;

    recorder.exec(Command::new(&zellij).args([
        "-s",
        SESSION,
        "action",
        "focus-pane-id",
        "terminal_0",
    ]))?;
    recorder.key("ctrl+alt+k", Duration::from_millis(300))?;
    expect(recorder, &zellij, "1 2 0", 0, &ui)?;
    recorder.key("ctrl+alt+j", Duration::from_millis(300))?;
    expect(recorder, &zellij, "0 1 2", 0, &ui)?;

    for expected in ["1 0 2", "1 2 0", "0 1 2"] {
        recorder.key("ctrl+alt+j", Duration::from_millis(200))?;
        expect(recorder, &zellij, expected, 0, &ui)?;
    }
    for expected in ["1 2 0", "1 0 2", "0 1 2"] {
        recorder.key("ctrl+alt+k", Duration::from_millis(200))?;
        expect(recorder, &zellij, expected, 0, &ui)?;
    }
    for _ in 0..10 {
        recorder.key("ctrl+alt+k", Duration::ZERO)?;
        recorder.key("ctrl+alt+j", Duration::from_millis(300))?;
        expect(recorder, &zellij, "0 1 2", 0, &ui)?;
    }
    recorder.exec(Command::new(&zellij).args([
        "-s",
        SESSION,
        "action",
        "close-pane",
        "--pane-id",
        "terminal_2",
    ]))?;
    expect(recorder, &zellij, "0 1", 0, &ui)?;
    recorder.key("ctrl+alt+k", Duration::from_millis(300))?;
    expect(recorder, &zellij, "1 0", 0, &ui)?;
    recorder.key("ctrl+alt+j", Duration::from_millis(300))?;
    expect(recorder, &zellij, "0 1", 0, &ui)?;

    // Cross-column moves in the split arrangement.
    recorder.key("alt+m", Duration::from_millis(350))?;
    let (stacked, _, _, _) = panes(recorder, &zellij)?;
    assert_eq!(stacked.split(' ').count(), 3, "expected a third work pane");
    recorder.key("alt+bracketright", Duration::from_millis(300))?;
    expect_content_split(recorder, &zellij, true, open_sidebar_width)?;
    let (split, _, _, _) = panes(recorder, &zellij)?;
    let ids = split
        .split(['|', ' '])
        .map(|id| id.parse::<u32>().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(split, format!("{}|{} {}", ids[0], ids[1], ids[2]));
    let [a, b, c] = [ids[0], ids[1], ids[2]];
    focus(recorder, &zellij, a)?;
    recorder.key("ctrl+alt+h", Duration::from_millis(300))?;
    expect(recorder, &zellij, &format!("{a}|{b} {c}"), a, &ui)?;
    recorder.key("ctrl+alt+l", Duration::from_millis(300))?;
    expect(recorder, &zellij, &format!("{b}|{a} {c}"), a, &ui)?;
    recorder.key("ctrl+alt+l", Duration::from_millis(300))?;
    expect(recorder, &zellij, &format!("{b}|{a} {c}"), a, &ui)?;
    focus(recorder, &zellij, c)?;
    recorder.key("ctrl+alt+h", Duration::from_millis(300))?;
    expect(recorder, &zellij, &format!("{c}|{b} {a}"), c, &ui)?;
    for _ in 0..5 {
        recorder.key("ctrl+alt+l", Duration::ZERO)?;
        recorder.key("ctrl+alt+h", Duration::from_millis(300))?;
        expect(recorder, &zellij, &format!("{c}|{b} {a}"), c, &ui)?;
    }
    recorder.key("ctrl+alt+j", Duration::from_millis(300))?;
    expect(recorder, &zellij, &format!("{c}|{b} {a}"), c, &ui)?;
    focus(recorder, &zellij, a)?;
    recorder.key("ctrl+alt+k", Duration::from_millis(300))?;
    expect(recorder, &zellij, &format!("{c}|{a} {b}"), a, &ui)?;

    recorder.key("alt+shift+m", Duration::from_millis(300))?;
    wait_for_menu_popup(recorder, &zellij)?;
    recorder.key("ctrl+alt+h", Duration::from_millis(300))?;
    recorder.key("alt+shift+m", Duration::from_millis(300))?;
    expect(recorder, &zellij, &format!("{c}|{a} {b}"), a, &ui)?;

    // Tab movement needs Rio's Kitty keyboard protocol to keep Shift distinct.
    expect_tabs(recorder, &zellij, "0")?;
    recorder.key("alt+shift+t", Duration::from_millis(300))?;
    expect_tabs(recorder, &zellij, "0 1")?;
    recorder.key("ctrl+alt+shift+h", Duration::from_millis(300))?;
    expect_tabs(recorder, &zellij, "1 0")?;
    recorder.key("ctrl+alt+shift+l", Duration::from_millis(300))?;
    expect_tabs(recorder, &zellij, "0 1")?;
    Ok(())
}

fn expect_tabs(recorder: &mut Recorder, zellij: &std::ffi::OsStr, order: &str) -> Result<()> {
    let deadline = Instant::now() + MOVE_TIMEOUT;
    loop {
        let json = recorder.output(Command::new(zellij).args([
            "-s",
            SESSION,
            "action",
            "list-tabs",
            "--json",
        ]))?;
        let path = recorder.work().join("tabs.json");
        fs::write(&path, json)?;
        let actual = recorder.output(
            Command::new(program("JQ_BIN"))
                .args([
                    "-r",
                    "sort_by(.position) | map(.tab_id | tostring) | join(\" \")",
                ])
                .arg(&path),
        )?;
        if actual.trim() == order {
            return Ok(());
        }
        assert!(
            Instant::now() < deadline,
            "tab order {actual:?}, expected {order}"
        );
        recorder.sleep(Duration::from_millis(50))?;
    }
}

fn focus(recorder: &mut Recorder, zellij: &std::ffi::OsStr, pane: u32) -> Result<()> {
    recorder.exec(Command::new(zellij).args([
        "-s",
        SESSION,
        "action",
        "focus-pane-id",
        &format!("terminal_{pane}"),
    ]))?;
    recorder.sleep(Duration::from_millis(300))
}

fn wait_for_menu_popup(recorder: &mut Recorder, zellij: &std::ffi::OsStr) -> Result<()> {
    let deadline = Instant::now() + TIMEOUT;
    loop {
        let json = recorder.output(Command::new(zellij).args([
            "-s",
            SESSION,
            "action",
            "list-panes",
            "--all",
            "--json",
        ]))?;
        let path = recorder.work().join("popup-panes.json");
        fs::write(&path, json)?;
        let focused = recorder.output(
            Command::new(program("JQ_BIN"))
                .args([
                    "-r",
                    ".[] | select(.title == \"menu_popup\" and .is_focused and (.is_suppressed | not)) | .id",
                ])
                .arg(&path),
        )?;
        if !focused.trim().is_empty() {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(std::io::Error::other("menu popup did not take focus").into());
        }
        recorder.sleep(Duration::from_millis(100))?;
    }
}

fn main() -> ExitCode {
    kinestra::run(record)
}
