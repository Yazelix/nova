use std::{collections::BTreeSet, env, fs, process::ExitCode};

fn main() -> ExitCode {
    let args = env::args().collect::<Vec<_>>();
    let [_, layout_path, swap_path] = args.as_slice() else {
        eprintln!("usage: zellij-layout <layout.kdl> <layout.swap.kdl>");
        return ExitCode::FAILURE;
    };

    let layout = read(layout_path);
    let templates = layout
        .lines()
        .filter_map(tab_template)
        .collect::<BTreeSet<_>>();
    let mut ok = true;
    for (block, needle, message) in [
        (
            "tab_template name=\"ui\"",
            "children",
            "missing content delimiter in swap UI template",
        ),
        (
            "default_tab_template",
            "plugin location=\"radar\"",
            "missing Radar sidebar in default tab template",
        ),
        (
            "new_tab_template",
            "plugin location=\"radar\"",
            "missing Radar sidebar in new tab template",
        ),
        (
            "tab {",
            "pane name=\"yazi_picker\" command=",
            "missing tiled Yazi picker in startup tab",
        ),
        (
            "new_tab_template",
            "pane name=\"yazi_picker\" command=",
            "missing tiled Yazi picker in new tab template",
        ),
    ] {
        if !block_contains(&layout, block, needle) {
            eprintln!("{layout_path}: {message}");
            ok = false;
        }
    }
    if layout.matches(r#"plugin location="radar""#).count() != 2
        || layout
            .matches(r#"pane name="sidebar" size=32 borderless=false {"#)
            .count()
            != 2
    {
        eprintln!(
            "{layout_path}: startup and new-tab templates must each own one framed sidebar pane"
        );
        ok = false;
    }
    if layout
        .matches(r#"pane name="yazi_picker" command="#)
        .count()
        != 2
        || layout.matches(r#"args "--yzx-startup-picker""#).count() != 2
        || layout.matches("focus=true close_on_exit=true").count() != 2
        || layout.contains("floating_panes {")
        || layout.contains(r#"pane name="editor" command="#)
        || layout.contains(r#"args "--yzx-managed-pane""#)
    {
        eprintln!(
            "{layout_path}: startup and new-tab templates must each open one focused tiled Yazi picker without a prestarted editor"
        );
        ok = false;
    }
    if !layout_order_is_valid(&layout) {
        eprintln!(
            "{layout_path}: startup tab must follow default_tab_template and precede new_tab_template"
        );
        ok = false;
    }
    if !layout
        .lines()
        .any(|line| line.trim() == r#"new_tab_template cwd="$HOME" {"#)
    {
        eprintln!("{layout_path}: new tabs must open in the user's home directory");
        ok = false;
    }
    if !bar_layout_is_valid(&layout) {
        eprintln!(
            "{layout_path}: top bars must use the rendered Nova bar and bottom bars must use nova-zjhints"
        );
        ok = false;
    }

    let swap = read(swap_path);
    if swap.matches(r#"plugin location="radar""#).count() != 4
        || swap
            .matches(r#"pane name="sidebar" size=32 borderless=false {"#)
            .count()
            != 2
        || swap
            .matches(r#"pane name="sidebar" size=1 borderless=false {"#)
            .count()
            != 2
    {
        eprintln!("{swap_path}: the open and collapsed Radar sidebar states must use pane frames");
        ok = false;
    }
    let variants = [
        "single_open",
        "single_closed",
        "columns_open",
        "columns_closed",
    ];
    let positions = variants
        .iter()
        .map(|name| swap.find(&format!("swap_tiled_layout name=\"{name}\"")))
        .collect::<Option<Vec<_>>>();
    if !positions.is_some_and(|positions| positions.windows(2).all(|pair| pair[0] < pair[1])) {
        eprintln!(
            "{swap_path}: stacked and columns modes must each provide open and collapsed sidebar variants, with stacked open first"
        );
        ok = false;
    }
    for (name, minimum) in [
        ("single_open", 4),
        ("single_closed", 4),
        ("columns_open", 5),
        ("columns_closed", 5),
    ] {
        for hidden in [false, true] {
            let variant = format!(
                "swap_tiled_layout name=\"{name}{}\"",
                if hidden { "_no_hints" } else { "" }
            );
            let ui = if hidden { "ui_no_hints" } else { "ui" };
            if !block_contains(
                &swap,
                &variant,
                &format!("{ui} min_panes={}", minimum - usize::from(hidden)),
            ) {
                eprintln!("{swap_path}: {variant} must match the visible pane count");
                ok = false;
            }
        }
    }
    if !block_contains(
        &layout,
        "tab_template name=\"ui\"",
        "bottom_hints name=\"bottom_hints\"",
    ) || block_contains(&layout, "tab_template name=\"ui_no_hints\"", "bottom_hints")
        || !block_contains(&layout, "tab_template name=\"ui_no_hints\"", "children")
    {
        eprintln!(
            "{layout_path}: visible and hidden hint layouts must share content and keep a named bottom hint pane only when visible"
        );
        ok = false;
    }
    let mut depth = 0i32;

    for (index, line) in swap.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("swap_tiled_layout ") {
            depth = 1;
            continue;
        }
        if depth == 1 && trimmed.ends_with('{') {
            let name = trimmed.split_whitespace().next().unwrap_or_default();
            if !templates.contains(name) {
                eprintln!("{swap_path}:{}: missing tab_template {name}", index + 1);
                ok = false;
            }
        }
        if depth > 0 {
            depth += line.matches('{').count() as i32 - line.matches('}').count() as i32;
        }
    }

    ExitCode::from((!ok) as u8)
}

fn read(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("could not read {}: {}", path, error))
}

fn tab_template(line: &str) -> Option<String> {
    line.trim()
        .strip_prefix("tab_template name=\"")?
        .split('"')
        .next()
        .map(str::to_owned)
}

fn block_contains(text: &str, block_name: &str, needle: &str) -> bool {
    let mut depth = 0i32;

    for line in text.lines() {
        let trimmed = line.trim();
        if depth == 0 && trimmed.starts_with(block_name) && trimmed.ends_with('{') {
            depth = line.matches('{').count() as i32 - line.matches('}').count() as i32;
            if trimmed.contains(needle) {
                return true;
            }
            continue;
        }

        if depth > 0 {
            if trimmed.contains(needle) {
                return true;
            }
            depth += line.matches('{').count() as i32 - line.matches('}').count() as i32;
        }
    }

    false
}

fn layout_order_is_valid(layout: &str) -> bool {
    let mut default = None;
    let mut tab = None;
    let mut new = None;
    let mut depth = 0i32;

    for (index, line) in layout.lines().enumerate() {
        let trimmed = line.trim();
        if depth == 1 {
            if trimmed.starts_with("default_tab_template") {
                default = Some(index);
            } else if trimmed == "tab" || trimmed.starts_with("tab ") {
                tab = Some(index);
            } else if trimmed.starts_with("new_tab_template") {
                new = Some(index);
            }
        }
        depth += line.matches('{').count() as i32 - line.matches('}').count() as i32;
    }

    matches!((default, tab, new), (Some(default), Some(tab), Some(new)) if default < tab && tab < new)
}

fn bar_layout_is_valid(layout: &str) -> bool {
    let bars = layout
        .matches(r#"plugin location="zellij:nova-bar""#)
        .count();
    let hint_bars = layout.matches(r#"plugin location="nova_hints""#).count();
    let views = layout.matches(r#"role "view""#).count();
    bars == 1
        && hint_bars == 1
        && views == bars
        && !layout.contains("format_right")
        && !layout.contains("command_cpu")
        && !layout.contains(r#"YZX " // {datetime}"#)
        && !layout.contains("NOVA ")
        && !layout.contains("{mode}")
        && !layout.contains("mode_normal")
        && !layout.contains(r#"plugin location="tab-bar""#)
}
