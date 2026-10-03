import json
import os
import re
import shlex
import subprocess
import sys
import tempfile
import time
from pathlib import Path

package = Path(sys.argv[1])
shell = sys.argv[2]
root = Path(tempfile.mkdtemp(prefix="nova-hints."))
socket = root.name
session = "nova-hints-proof"
binary = str(package / "bin/yzx-zellij")
packaged_config = (package / "share/yazelix/config.kdl").read_text()
menu = re.search(r'menu \{\s*command "([^"]+)"', packaged_config).group(1)
plugin = re.search(r'yazelix_pane_orchestrator location="([^"]+)"', packaged_config).group(1)
env = {k: v for k, v in os.environ.items() if not k.startswith(("ZELLIJ", "YZX_", "YAZELIX_"))}
env.update(HOME=str(root / "home"), XDG_CACHE_HOME=str(root / "cache"), XDG_DATA_HOME=str(root / "data"), ZELLIJ_SOCKET_DIR=str(root / "sockets"), TERM="xterm-256color")
for name in ("home", "cache/yzx-zellij", "sockets"):
    (root / name).mkdir(parents=True, exist_ok=True)
permissions = "\n".join(" " + p for p in ("ReadApplicationState", "ChangeApplicationState", "OpenTerminalsOrPlugins", "RunCommands", "WriteToStdin", "ReadCliPipes", "MessageAndLaunchOtherPlugins", "ReadSessionEnvironmentVariables"))
(root / "cache/yzx-zellij/permissions.kdl").write_text(json.dumps(plugin.removeprefix("file:")) + " {\n" + permissions + "\n}\n")
config = root / "config.kdl"
config.write_text('''default_shell "__SHELL__"
show_startup_tips false
show_release_notes false
pane_frames true
stacked_pane_list false
plugins { yazelix_pane_orchestrator location="__PLUGIN__" { screen_saver_enabled false; }; radar location="zellij:radar" { role "view"; naming "off"; }; }
load_plugins { yazelix_pane_orchestrator; }
keybinds clear-defaults=true {
    normal { bind "Ctrl q" { Quit; }; bind "Ctrl p" { SwitchToMode "Pane"; }; }
    pane { bind "Ctrl p" { SwitchToMode "Normal"; }; bind "Ctrl y" { CloseFocus; }; }
    shared_except "locked" { bind "Ctrl t" { GoToTab 2; }; bind "Ctrl r" { GoToTab 1; }; bind "Alt Shift B" { MessagePlugin "yazelix_pane_orchestrator" { name "toggle_bottom_hints"; }; }; }
}
'''.replace("__SHELL__", shell).replace("__PLUGIN__", plugin))
ui = '''
    top_bar size=1
    pane split_direction="vertical" {
        pane name="sidebar" size=32 { plugin location="radar"; }
        pane split_direction="vertical" {
            pane name="work-one" focus=true command="__SHELL__" { args "-c" "printf WORK_ONE; sleep 9999"; }
            pane name="work-two" command="__SHELL__" { args "-c" "printf WORK_TWO; sleep 9999"; }
        }
    }
    bottom_hints name="bottom_hints" size=1
'''.replace("__SHELL__", shell)
layout = root / "layout.kdl"
canonical = (package / "share/yazelix/layout.kdl").read_text()
header = canonical[:canonical.index("    default_tab_template")].removeprefix("layout {")
layout.write_text('layout {\n' + header + 'tab name="proof" {\n' + ui + '\n}\n}\n')
swap = (package / "share/yazelix/layout.swap.kdl").read_text()
(root / "layout.swap.kdl").write_text(swap)
config.write_text(config.read_text() + '\nlayout_dir ' + json.dumps(str(root)) + '\ndefault_layout ' + json.dumps(str(layout)) + '\n')


def run(args, **kwargs):
    result = subprocess.run(args, env=env, capture_output=True, text=True, timeout=20, **kwargs)
    if result.returncode:
        raise RuntimeError(f"{args}: {result.stderr}\n{result.stdout}")
    return result.stdout


def tmux(*args):
    return run(["tmux", "-f", "/dev/null", "-L", socket, *args])


def action(*args):
    return run([binary, "-c", str(config), "-s", session, "action", *args])


def panes():
    return json.loads(action("list-panes", "--all", "--json"))


def wait_for(check):
    for _ in range(100):
        try:
            data = panes()
            if check(data):
                return data
        except (RuntimeError, json.JSONDecodeError):
            pass
        time.sleep(0.1)
    raise AssertionError(tmux("capture-pane", "-t", "proof", "-p") + "\n" + json.dumps(data))


def pipe(name):
    response = action("pipe", "--plugin", "yazelix_pane_orchestrator", "--name", name, "--", "toggle").strip()
    assert response == "ok", (name, response)


def hints(data):
    return next(p for p in data if p["title"] == "bottom_hints")


def verify(hidden, width=120, height=40, focus=None):
    data = wait_for(lambda p: hints(p)["is_suppressed"] == hidden and (hidden or (hints(p)["pane_rows"], hints(p)["pane_columns"], hints(p)["pane_y"]) == (1, width, height - 1)))
    if not hidden:
        bar = hints(data)
        assert (bar["pane_y"], bar["pane_rows"], bar["pane_columns"]) == (height - 1, 1, width), bar
    else:
        work = [p for p in data if p["tab_position"] == 0 and not p["is_suppressed"] and not p["is_floating"] and not p["is_plugin"]]
        assert max(p["pane_y"] + p["pane_rows"] for p in work) == height, work
    if focus is not None:
        focused = [p for p in data if p["tab_position"] == 0 and p["is_focused"] and not p["is_suppressed"]]
        assert max(focused, key=lambda p: p["is_floating"])["id"] == focus, focused
    return data


try:
    tmux("new-session", "-d", "-s", "proof", "-x", "120", "-y", "40",
         shlex.join([binary, "-c", str(config), "-n", str(layout), "-s", session]) + "; sleep 30")
    # Avoid opening a CLI connection while the first interactive client is starting.
    for _ in range(100):
        if "WORK_ONE" in tmux("capture-pane", "-t", "proof:0", "-p"):
            break
        time.sleep(.1)
    else:
        raise AssertionError(tmux("capture-pane", "-t", "proof:0", "-p"))
    data = wait_for(lambda p: any(x["title"] == "bottom_hints" for x in p))
    work = next(p["id"] for p in data if p["title"] == "work-one")
    time.sleep(.5)
    env["ZELLIJ_PANE_ID"] = str(work)
    wait_for(lambda p: any(x["title"] == "work-two" for x in p))
    for family in ("single_open", "single_closed", "columns_open", "columns_closed"):
        action("apply-tiled-swap-layout", family)
        time.sleep(0.2)
        for _ in range(2):
            pipe("toggle_bottom_hints")
            verify(True, focus=work)
            tmux("send-keys", "-t", "proof:0", "M-B")
            shown = verify(False, focus=work)
            assert next(p["pane_columns"] for p in shown if p["title"] == "sidebar") == (1 if family.endswith("closed") else 32)
        pipe("toggle_bottom_hints")
        verify(True, focus=work)
        pipe("toggle_sidebar")
        verify(True)
        target = action("pipe", "--plugin", "yazelix_pane_orchestrator", "--name", "content_layout_target", "--", "toggle").strip()
        assert target.endswith("_no_hints"), target
        action("apply-tiled-swap-layout", target)
        verify(True)
        pipe("toggle_bottom_hints")
        verify(False)
    sidebar = next(p["id"] for p in panes() if p["title"] == "sidebar")
    action("focus-pane-id", "plugin_" + str(sidebar))
    pipe("toggle_bottom_hints")
    verify(True, focus=sidebar)
    action("new-pane", "--no-focus", "--name", "extra", "--", shell, "-c", "sleep 9999")
    data = wait_for(lambda p: any(x["title"] == "extra" for x in p))
    extra = next(p["id"] for p in data if p["title"] == "extra")
    verify(True)
    action("close-pane", "--pane-id", str(extra))
    wait_for(lambda p: all(x["id"] != extra or x["is_plugin"] for x in p))
    verify(True)
    pipe("toggle_bottom_hints")
    verify(False, focus=sidebar)
    tmux("send-keys", "-t", "proof:0", "M-B", "M-B")
    time.sleep(.5)
    verify(False, focus=sidebar)
    # The menu and shortcut address the same plugin command.
    env["YZX_ZELLIJ"] = binary
    run([menu], input="bottom-hints\n")
    verify(True)
    action("new-tab", "--layout", str(layout))
    data = wait_for(lambda p: any(x["title"] == "bottom_hints" and x["is_suppressed"] and x["tab_position"] == 1 for x in p))
    action("go-to-tab", "1")
    verify(True)
    tmux("resize-window", "-t", "proof:0", "-x", "80", "-y", "24")
    action("new-pane", "--floating", "--", shell, "-c", "printf POPUP; sleep 9999")
    data = wait_for(lambda p: any(x["is_floating"] and x["is_focused"] for x in p))
    popup = next(p["id"] for p in data if p["tab_position"] == 0 and p["is_floating"] and p["is_focused"])
    # Keep the current Zellij mode while toggling with a floating pane focused.
    tmux("send-keys", "-t", "proof:0", "C-p", "M-B")
    verify(False, 80, 24, popup)
    tmux("send-keys", "-t", "proof:0", "M-B")
    verify(True, 80, 24, popup)
    tmux("send-keys", "-t", "proof:0", "C-p")
    tmux("send-keys", "-t", "proof:0", "M-B")
    verify(False, 80, 24, popup)
    tmux("send-keys", "-t", "proof:0", "C-p", "M-B")
    verify(True, 80, 24, popup)
    tmux("send-keys", "-t", "proof:0", "C-y")
    wait_for(lambda p: all(x["is_plugin"] or x["id"] != popup for x in p))
    tmux("new-window", "-t", "proof", "-n", "attach", shlex.join([binary, "-c", str(config), "attach", session]))
    for _ in range(100):
        if "WORK_" in tmux("capture-pane", "-t", "proof:1", "-p"):
            break
        time.sleep(.1)
    else:
        raise AssertionError("attached client did not render the workspace")
    tmux("resize-window", "-t", "proof:1", "-x", "80", "-y", "24")
    verify(True, 80, 24)
    tmux("send-keys", "-t", "proof:1", "C-t")
    time.sleep(.3)
    tmux("send-keys", "-t", "proof:1", "M-B")
    wait_for(lambda p: all(not x["is_suppressed"] for x in p if x["title"] == "bottom_hints"))
    verify(False, 80, 24)
    run([menu], input="bottom-hints\n")
    wait_for(lambda p: all(x["is_suppressed"] for x in p if x["title"] == "bottom_hints"))
    verify(True, 80, 24)
    tmux("kill-window", "-t", "proof:0")
    time.sleep(.3)
    pipe("toggle_bottom_hints")
    wait_for(lambda p: all(not x["is_suppressed"] for x in p if x["title"] == "bottom_hints"))
    tmux("send-keys", "-t", "proof:1", "C-r")
    verify(False, 80, 24)
    pipe("toggle_bottom_hints")
    verify(True, 80, 24)
    bar = hints(panes())
    action("rename-pane", "--pane-id", "plugin_" + str(bar["id"]), "not-hints")
    before = wait_for(lambda p: any(x["title"] == "not-hints" for x in p))
    time.sleep(.2)
    response = action("pipe", "--plugin", "yazelix_pane_orchestrator", "--name", "toggle_bottom_hints", "--", "toggle").strip()
    assert response == "missing", response
    geometry = lambda data: [(p["id"], p["is_plugin"], p["is_suppressed"], p["pane_x"], p["pane_y"], p["pane_rows"], p["pane_columns"]) for p in data]
    assert geometry(before) == geometry(panes()), "missing hint pane changed the layout"
    # Mirrored sessions omit other clients from TabUpdate; still toggle once.
    run([binary, "kill-session", session])
    subprocess.run(["tmux", "-f", "/dev/null", "-L", socket, "kill-server"], env=env, capture_output=True)
    session = "nova-hints-mirror-proof"
    mirror_config = root / "mirror.kdl"
    mirror_config.write_text(config.read_text() + "\nmirror_session true\n")
    config = mirror_config
    for window, command in [(0, ["-n", str(layout), "-s", session]), (1, ["attach", session])]:
        launch = shlex.join([binary, "-c", str(config), *command])
        if window == 0:
            tmux("new-session", "-d", "-s", "proof", "-x", "120", "-y", "40", launch)
        else:
            tmux("new-window", "-t", "proof", "-n", "mirror", launch)
        tmux("resize-window", "-t", f"proof:{window}", "-x", "120", "-y", "40")
        for _ in range(100):
            if "WORK_" in tmux("capture-pane", "-t", f"proof:{window}", "-p"):
                break
            time.sleep(.1)
        else:
            raise AssertionError("mirrored client did not render")
    time.sleep(.3)
    pipe("toggle_bottom_hints")
    verify(True)
    tmux("send-keys", "-t", "proof:1", "M-B")
    verify(False)
    print("bottom hints: layouts, shortcut/CLI/menu, rapid toggles, pane lifecycle, session-wide visibility, geometry, focus, input mode, multiple and mirrored clients, leader departure, attach, and missing-pane safety passed")
finally:
    subprocess.run([binary, "kill-session", session], env=env, capture_output=True)
    subprocess.run(["tmux", "-f", "/dev/null", "-L", socket, "kill-server"], env=env, capture_output=True)
