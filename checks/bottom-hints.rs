use serde::Deserialize;
use std::{collections::HashSet, env, fs, path::Path};
mod tmux;
use tmux::{Terminal, quote, sleep};

#[derive(Debug, Deserialize)]
struct Pane {
    id: u32,
    title: String,
    tab_position: usize,
    is_plugin: bool,
    is_floating: bool,
    is_suppressed: bool,
    is_focused: bool,
    exited: bool,
    pane_x: usize,
    pane_y: usize,
    pane_rows: usize,
    pane_columns: usize,
    pane_content_columns: usize,
}

#[derive(Clone, Copy)]
struct View {
    width: usize,
    height: usize,
    focus: Option<u32>,
    frameless: Option<u32>,
    columns: Option<bool>,
}

impl Default for View {
    fn default() -> Self {
        Self {
            width: 120,
            height: 40,
            focus: None,
            frameless: None,
            columns: None,
        }
    }
}

struct Check {
    terminal: Terminal,
    required_work: HashSet<u32>,
}

fn quoted(text: &str) -> String {
    serde_json::to_string(text).unwrap()
}

fn location(config: &str, alias: &str) -> String {
    config
        .split_once(&format!("{alias} location=\""))
        .unwrap()
        .1
        .split('"')
        .next()
        .unwrap()
        .into()
}

fn hints(panes: &[Pane]) -> &Pane {
    panes
        .iter()
        .find(|pane| pane.title == "bottom_hints")
        .unwrap()
}

fn work_order(panes: &[Pane], tab: usize) -> Vec<u32> {
    let mut work = panes
        .iter()
        .filter(|p| p.tab_position == tab && !p.is_plugin && !p.is_floating && !p.is_suppressed)
        .collect::<Vec<_>>();
    work.sort_by_key(|p| (p.pane_x, p.pane_y));
    work.iter().map(|p| p.id).collect()
}

impl Check {
    fn action(&self, args: &[&str]) -> String {
        self.terminal.action(args)
    }
    fn tmux(&self, args: &[&str]) -> String {
        self.terminal.tmux(args)
    }
    fn panes(&self) -> Vec<Pane> {
        serde_json::from_str(&self.action(&["list-panes", "--all", "--json"])).unwrap()
    }

    #[track_caller]
    fn wait_for(&self, mut check: impl FnMut(&[Pane]) -> bool) -> Vec<Pane> {
        let mut last = Vec::new();
        let mut error = String::new();
        for _ in 0..100 {
            let output = self
                .terminal
                .action_command(&["list-panes", "--all", "--json"])
                .output()
                .unwrap();
            if output.status.success() {
                match serde_json::from_slice::<Vec<Pane>>(&output.stdout) {
                    Ok(data) => {
                        last = data;
                        if check(&last) {
                            return last;
                        }
                    }
                    Err(err) => error = err.to_string(),
                }
            } else {
                error = String::from_utf8_lossy(&output.stderr).into_owned();
            }
            sleep(0.1);
        }
        panic!(
            "{}\n{last:?}\n{error}",
            self.terminal.capture("proof", false)
        );
    }

    fn pipe_response(&self, name: &str) -> String {
        self.action(&[
            "pipe",
            "--plugin",
            "yazelix_pane_orchestrator",
            "--name",
            name,
            "--",
            "toggle",
        ])
    }

    fn pipe(&self, name: &str) {
        let before = if name == "toggle_bottom_hints" {
            self.panes()
        } else {
            Vec::new()
        };
        let response = self.pipe_response(name);
        assert_eq!(response.trim(), "ok", "{name}");
        if !before.is_empty() {
            let after = self.panes();
            for tab in before
                .iter()
                .map(|p| p.tab_position)
                .collect::<HashSet<_>>()
            {
                assert_eq!(
                    work_order(&after, tab),
                    work_order(&before, tab),
                    "hint toggle reordered tab {tab}"
                );
            }
        }
    }

    #[track_caller]
    fn verify(&self, hidden: bool, view: View) -> Vec<Pane> {
        self.wait_for(|panes| {
            let bars = panes
                .iter()
                .filter(|p| p.title == "bottom_hints")
                .collect::<Vec<_>>();
            if bars.len()
                != panes
                    .iter()
                    .map(|p| p.tab_position)
                    .collect::<HashSet<_>>()
                    .len()
                || bars.is_empty()
            {
                return false;
            }
            let bar = bars[0];
            let work = panes
                .iter()
                .filter(|p| p.tab_position == 0 && !p.is_plugin && !p.is_floating)
                .collect::<Vec<_>>();
            if !self
                .required_work
                .is_subset(&work.iter().map(|p| p.id).collect())
            {
                return false;
            }
            if view.columns.is_some_and(|columns| {
                (work.iter().map(|p| p.pane_x).collect::<HashSet<_>>().len() > 1) != columns
            }) {
                return false;
            }
            if bar.is_suppressed != hidden
                || !work.iter().all(|p| {
                    !p.is_suppressed
                        && !p.exited
                        && p.pane_columns.checked_sub(if view.frameless == Some(p.id) {
                            0
                        } else {
                            2
                        }) == Some(p.pane_content_columns)
                })
            {
                return false;
            }
            let bottom = view.height - usize::from(!hidden);
            if work.iter().map(|p| p.pane_y + p.pane_rows).max() != Some(bottom) {
                return false;
            }
            if !hidden
                && (bar.pane_y, bar.pane_rows, bar.pane_columns) != (view.height - 1, 1, view.width)
            {
                return false;
            }
            let Some(sidebar) = panes
                .iter()
                .find(|p| p.tab_position == 0 && p.title == "sidebar")
            else {
                return false;
            };
            if sidebar.pane_y + sidebar.pane_rows != bottom
                || work.iter().any(|p| p.pane_x < sidebar.pane_columns)
            {
                return false;
            }
            let left = if sidebar.pane_columns > 2 { 33 } else { 1 };
            for pane in panes
                .iter()
                .filter(|p| p.tab_position == 0 && p.title == "managed-popup" && !p.is_suppressed)
            {
                if pane.exited
                    || (pane.pane_x, pane.pane_y, pane.pane_columns, pane.pane_rows)
                        != (
                            left,
                            1,
                            view.width - left - 1,
                            view.height - 1 - usize::from(!hidden),
                        )
                {
                    return false;
                }
            }
            view.focus.is_none_or(|focus| {
                panes
                    .iter()
                    .filter(|p| p.tab_position == 0 && p.is_focused && !p.is_suppressed)
                    // Prefer floating panes, preserving the first pane when clients report ties.
                    .min_by_key(|p| !p.is_floating)
                    .is_some_and(|p| p.id == focus)
            })
        })
    }
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let package = Path::new(&args[1]);
    let shell = &args[2];
    let mut c = Check {
        terminal: Terminal::new(package, "nova-hints-proof"),
        required_work: HashSet::new(),
    };
    let root = c.terminal.root.path.clone();
    let config = root.join("config.kdl");
    c.terminal.config = config.clone();
    let packaged_config = fs::read_to_string(package.join("share/yazelix/config.kdl")).unwrap();
    let menu = packaged_config
        .split_once("menu {")
        .unwrap()
        .1
        .split_once("command \"")
        .unwrap()
        .1
        .split('"')
        .next()
        .unwrap()
        .to_owned();
    let plugin = location(&packaged_config, "yazelix_pane_orchestrator");
    let popup_plugin = location(&packaged_config, "yzpp");
    let hint_tail = packaged_config
        .split_once("nova_hints location=")
        .unwrap()
        .1;
    let hint_plugin = format!(
        "nova_hints location={}}}",
        hint_tail.split_once('}').unwrap().0
    );
    let permissions = [
        "ReadApplicationState",
        "ChangeApplicationState",
        "OpenTerminalsOrPlugins",
        "RunCommands",
        "WriteToStdin",
        "ReadCliPipes",
        "MessageAndLaunchOtherPlugins",
        "ReadSessionEnvironmentVariables",
    ]
    .map(|p| format!(" {p}\n"))
    .concat();
    fs::create_dir_all(root.join("cache/yzx-zellij")).unwrap();
    fs::write(
        root.join("cache/yzx-zellij/permissions.kdl"),
        [
            &plugin,
            &popup_plugin,
            &location(&packaged_config, "nova_hints"),
        ]
        .map(|url| {
            format!(
                "{} {{\n{permissions}\n}}\n",
                quoted(url.trim_start_matches("file:"))
            )
        })
        .concat(),
    )
    .unwrap();
    let fixture = r#"default_shell "__SHELL__"
show_startup_tips false
show_release_notes false
pane_frame_style "full"
stacked_pane_list false
plugins {
    __HINT_PLUGIN__
    yazelix_pane_orchestrator location="__PLUGIN__" { screen_saver_enabled false; }
    radar location="zellij:radar" { role "view"; naming "off"; }
    yzpp location="__POPUP_PLUGIN__" {
        left_margin_pane_title "sidebar"
        popup_defaults { side_margin 1; left_margin 33; vertical_margin 0; }
        popups { proof { command "__SHELL__"; arg_1 "-c"; arg_2 "printf POPUP; echo $$ > __POPUP_PID__; sleep 9999"; pane_title "managed-popup"; }; }
    }
}
load_plugins { yazelix_pane_orchestrator; yzpp; }
keybinds clear-defaults=true {
    normal { bind "Ctrl q" { Quit; }; bind "Ctrl p" { SwitchToMode "Pane"; }; }
    pane { bind "Ctrl p" { SwitchToMode "Normal"; }; bind "Ctrl y" { CloseFocus; }; }
    shared_except "locked" { bind "Ctrl t" { GoToTab 2; }; bind "Ctrl r" { GoToTab 1; }; bind "Alt g" { MessagePlugin "yzpp" { name "toggle"; payload "proof"; }; }; bind "Alt Shift B" { MessagePlugin "yazelix_pane_orchestrator" { name "toggle_bottom_hints"; }; }; }
}
"#.replace("__HINT_PLUGIN__", &hint_plugin).replace("__SHELL__", shell).replace("__PLUGIN__", &plugin).replace("__POPUP_PLUGIN__", &popup_plugin).replace("__POPUP_PID__", root.join("popup.pid").to_str().unwrap());
    let ui = r#"
    top_bar size=1
    pane split_direction="vertical" {
        pane name="sidebar" size=32 { plugin location="radar"; }
        pane split_direction="vertical" {
            pane name="work-one" focus=true command="__SHELL__" { args "-c" "printf WORK_ONE; sleep 9999"; }
            pane name="work-two" command="__SHELL__" { args "-c" "printf WORK_TWO; sleep 9999"; }
            pane name="work-three" command="__SHELL__" { args "-c" "printf WORK_THREE; sleep 9999"; }
        }
    }
    bottom_hints name="bottom_hints" size=1
"#.replace("__SHELL__", shell);
    let layout = root.join("layout.kdl");
    let layout_str = layout.to_str().unwrap();
    let canonical = fs::read_to_string(package.join("share/yazelix/layout.kdl")).unwrap();
    let header = canonical
        .split_once("    default_tab_template")
        .unwrap()
        .0
        .strip_prefix("layout {")
        .unwrap();
    fs::write(
        &layout,
        format!("layout {{\n{header}tab name=\"proof\" {{\n{ui}\n}}\n}}\n"),
    )
    .unwrap();
    fs::copy(
        package.join("share/yazelix/layout.swap.kdl"),
        root.join("layout.swap.kdl"),
    )
    .unwrap();
    let layout_options = format!(
        "\nlayout_dir {}\ndefault_layout {}\n",
        quoted(root.to_str().unwrap()),
        quoted(layout_str)
    );
    let fixture = fixture + &layout_options;
    fs::write(
        &config,
        fixture
            .split_once("keybinds clear-defaults=true {")
            .unwrap()
            .0
            .to_owned()
            + "keybinds {"
            + packaged_config.split_once("keybinds {").unwrap().1
            + &layout_options,
    )
    .unwrap();
    let render_session = format!("{}-rendering", c.terminal.session);
    let launch = c
        .terminal
        .launch(&["-n", layout_str, "-s", &render_session])
        + "; sleep 30";
    c.tmux(&[
        "new-session",
        "-d",
        "-s",
        "rendering",
        "-x",
        "120",
        "-y",
        "40",
        &launch,
    ]);
    for width in [120, 180] {
        c.terminal.resize("rendering:0", width, 40);
        sleep(0.3);
        let expected: &[&str] = if width == 120 {
            &["M menu"]
        } else {
            &[
                "M menu",
                "K config",
                "J git",
                "L agent",
                "H sidebar",
                "B hints",
                "F full",
            ]
        };
        let mut row = String::new();
        c.terminal.wait(
            || {
                row = c.tmux(&[
                    "capture-pane",
                    "-t",
                    "rendering:0",
                    "-p",
                    "-S",
                    "39",
                    "-E",
                    "39",
                ]);
                expected.iter().all(|hint| row.contains(hint))
            },
            "managed hints missing",
            "rendering:0",
        );
        assert!(!row.contains("..."), "partial hint clipped: {row}");
        if width == 180 {
            assert!(
                [" C ", " C-A ", " A ", " A-S "]
                    .iter()
                    .all(|header| row.matches(header).count() == 1),
                "{row}"
            );
        }
    }
    for width in [80, 120, 180, 200] {
        c.terminal.resize("rendering:0", width, 40);
        for (key, mode, exit) in [
            ("C-p", "PANE", "Escape"),
            ("C-n", "RESIZE", "Escape"),
            ("C-M-t", "TAB", "Escape"),
            ("C-M-s", "SCROLL", "Escape"),
            ("C-M-o", "SESSION", "Escape"),
            ("C-M-g", "LOCKED", "C-M-g"),
        ] {
            c.tmux(&["send-keys", "-t", "rendering:0", key]);
            c.terminal.wait(
                || {
                    let screen = c.terminal.capture("rendering:0", false);
                    let row = screen.lines().last().unwrap_or_default();
                    row.starts_with(mode)
                        && row.contains(match mode {
                            "LOCKED" => "unlock",
                            "TAB" | "PANE" => "ESC normal",
                            _ => "back",
                        })
                        && (["LOCKED", "TAB", "PANE"].contains(&mode)
                            || row.contains("ESC / ENTER"))
                        && (mode != "TAB"
                            || (row.contains("hjkl focus")
                                && row.contains(" | ")
                                && ["new", "close", "toggle", "1–9", "ENTER", "←", "C-A t"]
                                    .iter()
                                    .all(|hint| !row.contains(hint))))
                        && (mode != "PANE"
                            || (["hjkl focus", "n new", "x close", " | "]
                                .iter()
                                .all(|hint| row.contains(hint))
                                && [
                                    "embed",
                                    "pin",
                                    "toggle focus",
                                    "focus last",
                                    "ENTER",
                                    "←",
                                    "C p",
                                ]
                                .iter()
                                .all(|hint| !row.contains(hint))
                                && (width < 120
                                    || [
                                        "f full",
                                        "w float",
                                        "r split right",
                                        "d split down",
                                        "c rename",
                                    ]
                                    .iter()
                                    .all(|hint| row.contains(hint)))))
                },
                &format!("minor-mode hints missing: {mode} at {width} columns"),
                "rendering:0",
            );
            c.tmux(&["send-keys", "-t", "rendering:0", exit]);
            c.terminal.wait(
                || {
                    let screen = c.terminal.capture("rendering:0", false);
                    screen.lines().last().unwrap_or_default().contains("p pane")
                },
                "displayed exit did not restore Normal hints",
                "rendering:0",
            );
        }
    }
    c.terminal
        .run(&c.terminal.binary, &["kill-session", &render_session], None);
    c.tmux(&["kill-session", "-t", "rendering"]);
    fs::write(&config, &fixture).unwrap();
    // Seed startup only; visible swap layouts retain the ordinary hint role.
    fs::write(
        &layout,
        fs::read_to_string(&layout).unwrap().replace(
            &ui,
            &ui.replace(
                "name=\"bottom_hints\"",
                "name=\"bottom_hints_start_hidden\"",
            ),
        ),
    )
    .unwrap();
    let launch = c
        .terminal
        .launch(&["-n", layout_str, "-s", &c.terminal.session])
        + "; sleep 30";
    c.tmux(&[
        "new-session",
        "-d",
        "-s",
        "proof",
        "-x",
        "120",
        "-y",
        "40",
        &launch,
    ]);
    c.terminal.wait(
        || c.terminal.capture("proof:0", false).contains("WORK_ONE"),
        "first interactive client did not render",
        "proof:0",
    );
    let data = c.wait_for(|p| p.iter().any(|p| p.title == "bottom_hints"));
    let work = data.iter().find(|p| p.title == "work-one").unwrap().id;
    sleep(0.5);
    c.terminal
        .environment
        .insert("ZELLIJ_PANE_ID".into(), work.to_string().into());
    let data = c.wait_for(|p| p.iter().any(|p| p.title == "work-two"));
    c.required_work = data.iter().filter(|p| !p.is_plugin).map(|p| p.id).collect();
    let second_work = data.iter().find(|p| p.title == "work-two").unwrap().id;
    let third_work = data.iter().find(|p| p.title == "work-three").unwrap().id;
    let base = View::default();
    let focused_work = View {
        focus: Some(work),
        ..base
    };
    c.verify(true, focused_work);
    let order = work_order(&c.panes(), 0);
    c.pipe("toggle_bottom_hints");
    assert_eq!(
        work_order(&c.verify(false, focused_work), 0),
        order,
        "startup restoration reordered work panes"
    );
    assert!(
        c.terminal.capture("proof:0", false).contains("B hints"),
        "configured pipe hint missing"
    );
    c.action(&["new-tab", "--layout", layout_str]);
    c.wait_for(|p| {
        p.iter().filter(|p| p.title == "bottom_hints").count() == 2
            && p.iter()
                .filter(|p| p.title == "bottom_hints")
                .all(|p| !p.is_suppressed)
    });
    c.action(&["close-tab"]);
    c.wait_for(|p| p.iter().all(|p| p.tab_position == 0));
    c.verify(false, focused_work);
    for family in [
        "single_open",
        "single_closed",
        "columns_open",
        "columns_closed",
    ] {
        c.action(&["apply-tiled-swap-layout", family]);
        sleep(0.2);
        if family.starts_with("single") {
            c.action(&["move-pane", "--pane-id", &work.to_string(), "down"]);
            c.action(&["focus-pane-id", &second_work.to_string()]);
            c.action(&["focus-pane-id", &work.to_string()]);
        }
        let order = work_order(&c.panes(), 0);
        for _ in 0..6 {
            c.pipe("toggle_bottom_hints");
            assert_eq!(
                work_order(&c.verify(true, focused_work), 0),
                order,
                "hiding hints reordered work panes"
            );
            c.tmux(&["send-keys", "-t", "proof:0", "M-B"]);
            let shown = c.verify(false, focused_work);
            assert_eq!(
                work_order(&shown, 0),
                order,
                "restoring hints reordered work panes"
            );
            assert_eq!(
                shown
                    .iter()
                    .find(|p| p.title == "sidebar")
                    .unwrap()
                    .pane_columns,
                if family.ends_with("closed") { 1 } else { 32 }
            );
        }
        c.pipe("toggle_bottom_hints");
        c.verify(true, focused_work);
        c.pipe("toggle_sidebar");
        c.verify(true, base);
        let target = c.pipe_response("content_layout_target");
        assert!(target.trim().ends_with("_no_hints"), "{target}");
        c.action(&["apply-tiled-swap-layout", target.trim()]);
        c.verify(true, base);
        c.pipe("toggle_bottom_hints");
        c.verify(false, base);
    }
    c.action(&[
        "set-pane-borderless",
        "--pane-id",
        &work.to_string(),
        "--borderless",
    ]);
    for _ in 0..2 {
        c.pipe("toggle_bottom_hints");
        let frameless_work = View {
            frameless: Some(work),
            ..base
        };
        c.verify(true, frameless_work);
        c.pipe("toggle_bottom_hints");
        c.verify(false, frameless_work);
    }
    c.action(&["set-pane-borderless", "--pane-id", &work.to_string()]);
    let sidebar = c.panes().iter().find(|p| p.title == "sidebar").unwrap().id;
    c.action(&["focus-pane-id", &format!("plugin_{sidebar}")]);
    c.pipe("toggle_bottom_hints");
    let order = work_order(&c.panes(), 0);
    c.action(&[
        "new-pane",
        "--no-focus",
        "--name",
        "extra",
        "--",
        shell,
        "-c",
        "sleep 9999",
    ]);
    let data = c.wait_for(|p| p.iter().any(|p| p.title == "extra"));
    let extra = data.iter().find(|p| p.title == "extra").unwrap().id;
    assert_eq!(
        work_order(&data, 0),
        [order, vec![extra]].concat(),
        "new pane displaced existing work panes"
    );
    c.verify(true, base);
    c.action(&["move-pane", "--pane-id", &extra.to_string(), "up"]);
    c.pipe("toggle_bottom_hints");
    let focused_sidebar = View {
        focus: Some(sidebar),
        ..base
    };
    c.verify(false, focused_sidebar);
    c.pipe("toggle_bottom_hints");
    c.verify(true, focused_sidebar);
    c.action(&["close-pane", "--pane-id", &extra.to_string()]);
    c.wait_for(|p| p.iter().all(|p| p.id != extra || p.is_plugin));
    c.verify(true, base);
    c.pipe("toggle_bottom_hints");
    c.verify(false, focused_sidebar);
    c.tmux(&["send-keys", "-t", "proof:0", "M-B", "M-B"]);
    sleep(0.5);
    c.verify(false, focused_sidebar);
    c.terminal.environment.insert(
        "YZX_ZELLIJ".into(),
        c.terminal.binary.clone().into_os_string(),
    );
    c.terminal.run(&menu, &[], Some("bottom-hints\n"));
    c.verify(true, base);
    for _ in 0..3 {
        c.action(&["new-tab", "--layout", layout_str]);
        c.wait_for(|p| {
            p.iter()
                .any(|p| p.title == "bottom_hints" && p.is_suppressed && p.tab_position == 1)
        });
        c.action(&["close-tab"]);
        c.wait_for(|p| p.iter().all(|p| p.tab_position == 0));
        c.verify(true, base);
        c.pipe("toggle_bottom_hints");
        c.verify(false, base);
        c.pipe("toggle_bottom_hints");
        c.verify(true, base);
    }
    c.action(&["new-tab", "--layout", layout_str]);
    c.wait_for(|p| {
        p.iter()
            .any(|p| p.title == "bottom_hints" && p.is_suppressed && p.tab_position == 1)
    });
    c.action(&["go-to-tab", "1"]);
    c.verify(true, base);
    c.terminal.resize("proof:0", 80, 24);
    c.action(&["apply-tiled-swap-layout", "columns_open_no_hints"]);
    c.tmux(&["send-keys", "-t", "proof:0", "M-g"]);
    let data = c.wait_for(|p| p.iter().any(|p| p.is_floating && p.is_focused));
    let popup = data
        .iter()
        .find(|p| p.tab_position == 0 && p.is_floating && p.is_focused)
        .unwrap()
        .id;
    let popup_view = View {
        width: 80,
        height: 24,
        focus: Some(popup),
        columns: Some(true),
        ..base
    };
    c.verify(true, popup_view);
    let popup_pid = fs::read_to_string(root.join("popup.pid")).unwrap();
    c.action(&["apply-tiled-swap-layout", "single_open_no_hints"]);
    let stacked_popup = View {
        columns: Some(false),
        ..popup_view
    };
    c.verify(true, stacked_popup);
    for hidden in [false, true] {
        c.pipe("toggle_bottom_hints");
        c.verify(hidden, stacked_popup);
    }
    c.action(&["apply-tiled-swap-layout", "columns_open_no_hints"]);
    c.verify(true, popup_view);
    for hidden in [false, true] {
        c.action(&["hide-floating-panes"]);
        c.pipe("toggle_bottom_hints");
        c.action(&["show-floating-panes"]);
        c.verify(hidden, popup_view);
    }
    for columns in [1, 32] {
        c.action(&["hide-floating-panes"]);
        c.pipe("toggle_sidebar");
        c.wait_for(|p| {
            p.iter()
                .any(|p| p.title == "sidebar" && p.tab_position == 0 && p.pane_columns == columns)
        });
        c.action(&["show-floating-panes"]);
        c.verify(true, popup_view);
    }
    let idle = root.join("idle-popup.output");
    sleep(0.6);
    c.tmux(&[
        "pipe-pane",
        "-t",
        "proof:0",
        &format!("cat > {}", quote(idle.to_str().unwrap())),
    ]);
    sleep(0.6);
    c.tmux(&["pipe-pane", "-t", "proof:0"]);
    assert!(
        fs::metadata(idle).unwrap().len() < 20000,
        "hidden hints caused an idle popup redraw loop"
    );
    c.tmux(&["send-keys", "-t", "proof:0", "C-p", "M-B"]);
    c.verify(false, popup_view);
    c.tmux(&["send-keys", "-t", "proof:0", "M-B"]);
    c.verify(true, popup_view);
    c.tmux(&["send-keys", "-t", "proof:0", "C-p"]);
    c.tmux(&["send-keys", "-t", "proof:0", "M-B"]);
    c.verify(false, popup_view);
    c.tmux(&["send-keys", "-t", "proof:0", "C-p", "M-B"]);
    c.verify(true, popup_view);
    c.pipe("toggle_sidebar");
    c.verify(true, popup_view);
    c.pipe("toggle_sidebar");
    c.verify(true, popup_view);
    c.terminal.resize("proof:0", 100, 30);
    c.verify(
        true,
        View {
            width: 100,
            height: 30,
            ..popup_view
        },
    );
    c.terminal.resize("proof:0", 80, 24);
    c.verify(true, popup_view);
    assert_eq!(
        fs::read_to_string(root.join("popup.pid")).unwrap(),
        popup_pid,
        "popup process restarted during reflow"
    );
    c.tmux(&["send-keys", "-t", "proof:0", "C-y"]);
    c.wait_for(|p| p.iter().all(|p| p.is_plugin || p.id != popup));
    let small = View {
        width: 80,
        height: 24,
        ..base
    };
    c.verify(
        true,
        View {
            columns: Some(true),
            ..small
        },
    );
    c.pipe("toggle_bottom_hints");
    c.verify(
        false,
        View {
            columns: Some(true),
            ..small
        },
    );
    let attach = c.terminal.launch(&["attach", &c.terminal.session]);
    c.tmux(&["new-window", "-t", "proof", "-n", "attach", &attach]);
    c.terminal.wait(
        || c.terminal.capture("proof:1", false).contains("WORK_"),
        "attached client did not render",
        "proof:1",
    );
    c.terminal.resize("proof:1", 80, 24);
    c.verify(false, small);
    c.pipe("toggle_bottom_hints");
    c.verify(true, small);
    c.tmux(&["send-keys", "-t", "proof:1", "C-t"]);
    sleep(0.3);
    c.tmux(&["send-keys", "-t", "proof:1", "M-B"]);
    c.wait_for(|p| {
        p.iter()
            .filter(|p| p.title == "bottom_hints")
            .all(|p| !p.is_suppressed)
    });
    c.verify(false, small);
    c.terminal.run(&menu, &[], Some("bottom-hints\n"));
    c.wait_for(|p| {
        p.iter()
            .filter(|p| p.title == "bottom_hints")
            .all(|p| p.is_suppressed)
    });
    c.verify(true, small);
    c.tmux(&["kill-window", "-t", "proof:0"]);
    sleep(0.3);
    c.pipe("toggle_bottom_hints");
    c.wait_for(|p| {
        p.iter()
            .filter(|p| p.title == "bottom_hints")
            .all(|p| !p.is_suppressed)
    });
    c.tmux(&["send-keys", "-t", "proof:1", "C-r"]);
    c.verify(false, small);
    c.pipe("toggle_bottom_hints");
    c.verify(true, small);
    let bar = hints(&c.panes()).id;
    c.action(&[
        "rename-pane",
        "--pane-id",
        &format!("plugin_{bar}"),
        "not-hints",
    ]);
    let before = c.wait_for(|p| p.iter().any(|p| p.title == "not-hints"));
    sleep(0.2);
    assert_eq!(c.pipe_response("toggle_bottom_hints").trim(), "missing");
    let geometry = |data: &[Pane]| {
        data.iter()
            .map(|p| {
                (
                    p.id,
                    p.is_plugin,
                    p.is_suppressed,
                    p.pane_x,
                    p.pane_y,
                    p.pane_rows,
                    p.pane_columns,
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(
        geometry(&before),
        geometry(&c.panes()),
        "missing hint pane changed the layout"
    );
    c.action(&[
        "rename-pane",
        "--pane-id",
        &format!("plugin_{bar}"),
        "bottom_hints",
    ]);
    c.action(&["close-pane", "--pane-id", &second_work.to_string()]);
    c.action(&["close-pane", "--pane-id", &third_work.to_string()]);
    c.required_work = HashSet::from([work]);
    c.wait_for(|p| p.iter().all(|p| p.is_plugin || p.id != second_work));
    c.action(&["apply-tiled-swap-layout", "single_open_no_hints"]);
    c.verify(true, small);
    for _ in 0..3 {
        c.pipe("toggle_bottom_hints");
        c.verify(false, small);
        c.pipe("toggle_bottom_hints");
        c.verify(true, small);
    }
    // Mirrored sessions omit peers from TabUpdate; broadcasts still toggle once.
    c.terminal.stop();
    c.terminal.session = "nova-hints-mirror-proof".into();
    c.terminal.config = root.join("mirror.kdl");
    fs::write(
        &c.terminal.config,
        fs::read_to_string(&config).unwrap() + "\nmirror_session true\n",
    )
    .unwrap();
    for window in 0..2 {
        let launch = if window == 0 {
            c.terminal
                .launch(&["-n", layout_str, "-s", &c.terminal.session])
        } else {
            c.terminal.launch(&["attach", &c.terminal.session])
        };
        if window == 0 {
            c.tmux(&[
                "new-session",
                "-d",
                "-s",
                "proof",
                "-x",
                "120",
                "-y",
                "40",
                &launch,
            ]);
        } else {
            c.tmux(&["new-window", "-t", "proof", "-n", "mirror", &launch]);
        }
        let target = format!("proof:{window}");
        c.terminal.resize(&target, 120, 40);
        c.terminal.wait(
            || c.terminal.capture(&target, false).contains("WORK_"),
            "mirrored client did not render",
            &target,
        );
    }
    sleep(0.3);
    c.required_work = c
        .panes()
        .iter()
        .filter(|p| !p.is_plugin)
        .map(|p| p.id)
        .collect();
    c.verify(true, base);
    c.pipe("toggle_bottom_hints");
    c.verify(false, base);
    c.pipe("toggle_bottom_hints");
    c.verify(true, base);
    c.action(&["new-tab", "--layout", layout_str]);
    c.wait_for(|p| {
        p.iter()
            .any(|p| p.title == "bottom_hints" && p.is_suppressed && p.tab_position == 1)
    });
    c.action(&["close-tab"]);
    c.wait_for(|p| p.iter().all(|p| p.tab_position == 0));
    c.verify(true, base);
    // Both choices survive replacing the original tab before its marker settles.
    for hidden in [false, true] {
        c.tmux(&["send-keys", "-t", "proof:1", "M-B"]);
        c.verify(hidden, base);
        for _ in 0..3 {
            c.action(&["new-tab", "--layout", layout_str]);
            c.wait_for(|_| {
                serde_json::from_str::<serde_json::Value>(&c.action(&["list-tabs", "--json"]))
                    .unwrap()
                    .as_array()
                    .unwrap()
                    .len()
                    == 2
            });
            c.action(&["go-to-tab", "1"]);
            c.action(&["close-tab"]);
            let data = c.wait_for(|p| {
                p.iter()
                    .map(|p| p.tab_position)
                    .collect::<HashSet<_>>()
                    .len()
                    == 1
                    && p.iter()
                        .filter(|p| {
                            matches!(
                                p.title.as_str(),
                                "bottom_hints" | "bottom_hints_start_hidden"
                            )
                        })
                        .count()
                        == 1
                    && p.iter()
                        .any(|p| p.title == "bottom_hints" && p.is_suppressed == hidden)
            });
            c.required_work = data.iter().filter(|p| !p.is_plugin).map(|p| p.id).collect();
            c.verify(hidden, base);
            c.terminal.environment.insert(
                "ZELLIJ_PANE_ID".into(),
                c.required_work.iter().next().unwrap().to_string().into(),
            );
        }
    }
    println!(
        "bottom hints: compact managed-hint priorities and fitting, repeated toggles, manual pane order, new-pane placement, pane frames and identities, single/stacked/split panes, shortcut/CLI/menu, pane and closed-tab lifecycle, session visibility, geometry, focus, input mode, multiple/mirrored clients, attach, and missing-pane safety passed"
    );
}
