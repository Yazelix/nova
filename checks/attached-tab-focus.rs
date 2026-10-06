use std::{env, fs, path::Path};
#[allow(dead_code)]
mod tmux;
use tmux::{Terminal, sleep};

fn highlighted(row: &str, rgb: &str) -> Vec<usize> {
    let mut active = false;
    let mut selected = Vec::new();
    for segment in row.split("\x1b[") {
        let text = if let Some((codes, text)) = segment.split_once('m') {
            if codes.bytes().all(|b| b.is_ascii_digit() || b == b';') {
                if codes == "0" {
                    active = false;
                } else if codes.contains(rgb) {
                    active = true;
                }
                text
            } else {
                segment
            }
        } else {
            segment
        };
        if active {
            selected.extend((1..=2).filter(|index| text.contains(&format!("[{index}]"))));
        }
    }
    selected
}

fn pane_text(terminal: &Terminal, window: usize, ansi: bool) -> String {
    terminal.capture(&format!("clients:{window}"), ansi)
}

fn selected(terminal: &Terminal, window: usize, rgb: &str) -> Vec<usize> {
    highlighted(
        pane_text(terminal, window, true)
            .lines()
            .next()
            .unwrap_or_default(),
        rgb,
    )
}

fn wait_tabs(terminal: &Terminal, window: usize) {
    terminal.wait(
        || {
            let text = pane_text(terminal, window, true);
            let top = text.lines().next().unwrap_or_default();
            top.contains("[1]") && top.contains("[2]")
        },
        "client never showed both tabs",
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
    tab name="one" {
        pane command="__SHELL__" { args "-c" "printf ONE; sleep 9999"; }
    }
    tab name="two" {
        pane command="__SHELL__" { args "-c" "printf TWO; sleep 9999"; }
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
    let hex = &active.split_once("bg=#").unwrap().1[..6];
    let rgb = format!(
        "48;2;{}",
        (0..6)
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap().to_string())
            .collect::<Vec<_>>()
            .join(";")
    );
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
    terminal.tmux(&["send-keys", "-t", "clients:1", "M-2"]);
    sleep(1.0);
    assert_eq!(selected(&terminal, 1, &rgb), [2]);
    terminal.tmux(&["send-keys", "-t", "clients:0", "M-2"]);
    sleep(0.5);
    terminal.tmux(&["send-keys", "-t", "clients:0", "M-1"]);
    for delay in [0.1, 0.3, 0.5, 1.0] {
        sleep(delay);
        assert_eq!(selected(&terminal, 0, &rgb), [1]);
        assert!(pane_text(&terminal, 0, false).contains("ONE"));
    }
    assert_eq!(selected(&terminal, 1, &rgb), [2]);
    assert!(pane_text(&terminal, 1, false).contains("TWO"));
    terminal.tmux(&["send-keys", "-t", "clients:1", "C-q"]);
    terminal.wait(
        || pane_text(&terminal, 1, false).contains("EXIT:0"),
        "second client did not exit cleanly",
        "clients:1",
    );
    terminal.tmux(&["new-window", "-t", "clients", "-n", "reattached", &attach]);
    wait_tabs(&terminal, 2);
    terminal.tmux(&["send-keys", "-t", "clients:2", "M-2"]);
    sleep(1.0);
    assert_eq!(selected(&terminal, 2, &rgb), [2]);
    assert!(pane_text(&terminal, 2, false).contains("TWO"));
    println!("two clients kept independent tab highlights across attach and reattach");
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tab_highlights_follow_background_colour_and_reset() {
        let rgb = "48;2;17;34;51";
        assert_eq!(highlighted("\x1b[48;2;17;34;51m[1]\x1b[0m [2]", rgb), [1]);
        assert_eq!(highlighted("[1] \x1b[1;48;2;17;34;51m[2]", rgb), [2]);
        assert!(highlighted("\x1b[38;2;17;34;51m[1] [2]", rgb).is_empty());
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
