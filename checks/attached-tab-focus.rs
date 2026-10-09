use std::{env, fs, path::Path};
#[allow(dead_code)]
mod tmux;
use tmux::{Terminal, sleep};

fn highlighted(row: &str, rgb: &[u16; 3]) -> Vec<usize> {
    let mut active = false;
    let mut selected = Vec::new();
    for segment in row.split("\x1b[") {
        let text = if let Some((codes, text)) = segment.split_once('m') {
            if codes.bytes().all(|b| b.is_ascii_digit() || b == b';') {
                let mut codes = codes
                    .split(';')
                    .map(|code| code.parse::<u16>().unwrap_or(0));
                while let Some(code) = codes.next() {
                    match code {
                        0 | 30..=37 | 39 | 90..=97 => active = false,
                        38 | 48 | 58 => {
                            // Consume color components without treating them as attributes.
                            let color = match codes.next() {
                                Some(2) => codes.by_ref().take(3).collect::<Vec<_>>(),
                                Some(5) => {
                                    codes.next();
                                    Vec::new()
                                }
                                _ => Vec::new(),
                            };
                            if code == 38 {
                                active = color == *rgb;
                            }
                        }
                        _ => {}
                    }
                }
                text
            } else {
                segment
            }
        } else {
            segment
        };
        if active {
            selected.extend((1..=3).filter(|index| {
                text.contains(&format!("[{index} ")) || text.contains(&format!("[{index}]"))
            }));
        }
    }
    selected
}

fn selected(terminal: &Terminal, window: usize, rgb: &[u16; 3]) -> Vec<usize> {
    highlighted(
        terminal
            .capture(&format!("clients:{window}"), true)
            .lines()
            .next()
            .unwrap_or_default(),
        rgb,
    )
}

fn wait_tabs(terminal: &Terminal, window: usize) {
    terminal.wait(
        || {
            let text = terminal.capture(&format!("clients:{window}"), false);
            let top = text.lines().next().unwrap_or_default();
            (1..=3).all(|index| top.contains(&format!("[{index}")))
        },
        "client never showed all three tabs",
        &format!("clients:{window}"),
    );
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let terminal = Terminal::new(Path::new(&args[1]), "nova-tabfocus-proof");
    let layout = terminal.root.path.join("layout.kdl");
    fs::write(
        &layout,
        r#"layout {
    default_tab_template {
        pane size=1 borderless=true {
            plugin location="zellij:nova-bar" { role "view"; }
        }
        children
    }
    tab name="poe2" {
        pane command="__SHELL__" { args "-c" "printf ONE; sleep 9999"; }
    }
    tab name="eon café" {
        pane command="__SHELL__" { args "-c" "printf TWO; sleep 9999"; }
    }
    tab name="界面 space" {
        pane command="__SHELL__" { args "-c" "printf THREE; sleep 9999"; }
    }
}
"#
        .replace("__SHELL__", &args[2]),
    )
    .unwrap();
    let config = fs::read_to_string(&terminal.config).unwrap();
    let active = config
        .lines()
        .find(|line| line.trim_start().starts_with("tab_active "))
        .unwrap();
    let hex = &active.split_once("fg=#").unwrap().1[..6];
    let rgb = std::array::from_fn(|i| u16::from_str_radix(&hex[2 * i..2 * i + 2], 16).unwrap());
    let start = terminal.launch(&["-n", layout.to_str().unwrap(), "-s", &terminal.session])
        + "; printf '\\nEXIT:%s\\n' $?; sleep 30";
    terminal.tmux(&[
        "new-session",
        "-d",
        "-s",
        "clients",
        "-n",
        "first",
        "-x",
        "120",
        "-y",
        "40",
        &start,
    ]);
    wait_tabs(&terminal, 0);
    let attach = terminal.launch(&["attach", &terminal.session]);
    terminal.tmux(&[
        "new-window",
        "-t",
        "clients",
        "-n",
        "second",
        &(attach.clone() + "; printf '\\nEXIT:%s\\n' $?; sleep 30"),
    ]);
    wait_tabs(&terminal, 1);
    let expected = " [1 poe2]  [2 eon café]  [3 界面 space]";
    for width in [80, 120, 180] {
        terminal.resize("clients:0", width, 40);
        for index in [1, 2, 3, 2, 1] {
            terminal.tmux(&["send-keys", "-t", "clients:0", &format!("M-{index}")]);
            terminal.wait(
                || {
                    selected(&terminal, 0, &rgb) == [index]
                        && terminal.capture("clients:0", false).starts_with(expected)
                },
                "focus changed tab text or positions",
                "clients:0",
            );
        }
        for (index, column) in [(2, 13), (3, 27), (1, 3)] {
            terminal.tmux(&[
                "send-keys",
                "-t",
                "clients:0",
                "-l",
                &format!("\x1b[<0;{column};1M\x1b[<0;{column};1m"),
            ]);
            terminal.wait(
                || selected(&terminal, 0, &rgb) == [index],
                "tab mouse target moved after focus changes",
                "clients:0",
            );
            assert!(terminal.capture("clients:0", false).starts_with(expected));
        }
    }
    terminal.tmux(&["send-keys", "-t", "clients:1", "M-2"]);
    sleep(1.0);
    assert_eq!(selected(&terminal, 1, &rgb), [2]);
    terminal.tmux(&["send-keys", "-t", "clients:0", "M-2"]);
    sleep(0.5);
    terminal.tmux(&["send-keys", "-t", "clients:0", "M-1"]);
    for delay in [0.1, 0.3, 0.5, 1.0] {
        sleep(delay);
        assert_eq!(selected(&terminal, 0, &rgb), [1]);
        assert!(terminal.capture("clients:0", false).contains("ONE"));
    }
    assert_eq!(selected(&terminal, 1, &rgb), [2]);
    assert!(terminal.capture("clients:1", false).contains("TWO"));
    terminal.tmux(&["send-keys", "-t", "clients:1", "C-q"]);
    terminal.wait(
        || terminal.capture("clients:1", false).contains("EXIT:0"),
        "second client did not exit cleanly",
        "clients:1",
    );
    terminal.tmux(&["new-window", "-t", "clients", "-n", "reattached", &attach]);
    wait_tabs(&terminal, 2);
    terminal.tmux(&["send-keys", "-t", "clients:2", "M-2"]);
    sleep(1.0);
    assert_eq!(selected(&terminal, 2, &rgb), [2]);
    assert!(terminal.capture("clients:2", false).contains("TWO"));
    println!(
        "three outlined tabs kept text and mouse targets stable at 80/120/180 columns; two clients kept independent highlights across attach and reattach"
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tab_highlights_follow_foreground_colour_and_reset() {
        let rgb = &[17, 34, 51];
        assert_eq!(
            highlighted("\x1b[38;2;17;34;51m[1 one]\x1b[0m 2 two", rgb),
            [1]
        );
        assert_eq!(highlighted("1 one \x1b[1;38;2;17;34;51m[2 two]", rgb), [2]);
        assert!(highlighted("\x1b[48;2;17;34;51m[1 one] 2 two", rgb).is_empty());
        for change in ["22;38;2;160;166;175", "39", "31", "38;5;166"] {
            assert_eq!(
                highlighted(
                    &format!("\x1b[38;2;17;34;51m[1 one]\x1b[{change}m [2] two"),
                    rgb
                ),
                [1],
                "foreground change {change}"
            );
        }
        for change in ["1", "48;2;0;39;38", "48;5;0"] {
            assert_eq!(
                highlighted(&format!("\x1b[38;2;17;34;51m\x1b[{change}m[1 one]"), rgb),
                [1],
                "non-foreground change {change}"
            );
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn failed_commands_do_not_expose_environment_values() {
        let mut terminal = Terminal::new(Path::new("/nonexistent"), "diagnostic-proof");
        terminal.environment.clear();
        terminal
            .environment
            .insert("PATH".into(), env::var_os("PATH").unwrap());
        terminal
            .environment
            .insert("CHECK_TEST_SECRET".into(), "private-sentinel".into());
        let failure = std::panic::catch_unwind(|| {
            terminal.run(env::current_exe().unwrap(), &["--not-a-check-option"], None)
        })
        .unwrap_err();
        let message = failure.downcast_ref::<String>().unwrap();
        assert!(
            !message.contains("private-sentinel"),
            "inherited values leaked"
        );
        assert!(
            message.contains("--not-a-check-option"),
            "command context missing"
        );
    }
}
