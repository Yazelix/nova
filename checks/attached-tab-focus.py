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
root = Path(tempfile.mkdtemp(prefix="nova-tabfocus."))
socket = "nova-tabfocus-" + root.name.rsplit(".", 1)[-1]
session = "nova-tabfocus-proof"
config = package / "share/yazelix/config.kdl"
layout = root / "layout.kdl"
layout.write_text('''layout {
    default_tab_template {
        pane size=1 borderless=true {
            plugin location="zellij:nova-bar" {
                role "view"
            }
        }
        children
    }
    tab name="one" {
        pane command="__SHELL__" {
            args "-c" "printf ONE; sleep 9999"
        }
    }
    tab name="two" {
        pane command="__SHELL__" {
            args "-c" "printf TWO; sleep 9999"
        }
    }
}
'''.replace("__SHELL__", shell))
active_hex = re.search(r'(?m)^\s*tab_active\s+"[^"]*bg=#([0-9a-fA-F]{6})', config.read_text()).group(1)
active_rgb = "48;2;" + ";".join(str(int(active_hex[index:index + 2], 16)) for index in (0, 2, 4))
env = {key: value for key, value in os.environ.items() if not key.startswith(("ZELLIJ", "YZX_", "YAZELIX_"))}
env.update(
    HOME=str(root / "home"),
    XDG_CACHE_HOME=str(root / "cache"),
    XDG_DATA_HOME=str(root / "data"),
    ZELLIJ_SOCKET_DIR=str(root / "sockets"),
    TERM="xterm-256color",
)
(root / "home").mkdir()
(root / "sockets").mkdir()
binary = str(package / "bin/yzx-zellij")


def tmux(*args):
    result = subprocess.run(
        ["tmux", "-f", "/dev/null", "-L", socket, *args],
        env=env,
        capture_output=True,
        text=True,
    )
    if result.returncode:
        raise RuntimeError(f"tmux {args}: {result.stderr}")
    return result.stdout


def top(window):
    return tmux("capture-pane", "-t", f"clients:{window}", "-p", "-e").splitlines()[0]


def highlighted(window):
    active = False
    selected = []
    for part in re.split(r"(\x1b\[[0-9;]*m)", top(window)):
        if part.startswith("\x1b["):
            if part == "\x1b[0m":
                active = False
            elif active_rgb in part:
                active = True
        else:
            selected.extend(index for index in (1, 2) if f"[{index}]" in part and active)
    return selected


def pane_text(window):
    return tmux("capture-pane", "-t", f"clients:{window}", "-p")


try:
    tmux(
        "new-session", "-d", "-s", "clients", "-n", "first", "-x", "120", "-y", "40",
        shlex.join([binary, "-c", str(config), "-n", str(layout), "-s", session]) + " ; printf '\\nEXIT:%s\\n' $?; sleep 30",
    )
    for _ in range(100):
        if "[1]" in top(0) and "[2]" in top(0):
            break
        time.sleep(0.1)
    else:
        raise RuntimeError("first client never showed both tabs: " + repr(tmux("capture-pane", "-t", "clients:0", "-p", "-e")))
    tmux("new-window", "-t", "clients", "-n", "second", shlex.join([binary, "-c", str(config), "attach", session]) + " ; printf '\\nEXIT:%s\\n' $?; sleep 30")
    for _ in range(100):
        if "[1]" in top(1) and "[2]" in top(1):
            break
        time.sleep(0.1)
    else:
        raise RuntimeError("second client never showed both tabs: " + repr(tmux("capture-pane", "-t", "clients:1", "-p", "-e")))
    tmux("send-keys", "-t", "clients:1", "M-2")
    time.sleep(1)
    assert highlighted(1) == [2], f"second client: {top(1)!r}"
    tmux("send-keys", "-t", "clients:0", "M-2")
    time.sleep(0.5)
    tmux("send-keys", "-t", "clients:0", "M-1")
    samples = []
    for delay in (0.1, 0.3, 0.5, 1.0):
        time.sleep(delay)
        samples.append((highlighted(0), "ONE" in pane_text(0)))
    assert all(selected == [1] and pane_one for selected, pane_one in samples), samples
    assert highlighted(1) == [2] and "TWO" in pane_text(1)
    tmux("send-keys", "-t", "clients:1", "C-q")
    for _ in range(100):
        if "EXIT:0" in pane_text(1):
            break
        time.sleep(0.1)
    else:
        raise RuntimeError("second client did not exit cleanly")
    tmux("new-window", "-t", "clients", "-n", "reattached", shlex.join([binary, "-c", str(config), "attach", session]))
    for _ in range(100):
        if "[1]" in top(2) and "[2]" in top(2):
            break
        time.sleep(0.1)
    else:
        raise RuntimeError("reattached client never showed both tabs")
    tmux("send-keys", "-t", "clients:2", "M-2")
    time.sleep(1)
    assert highlighted(2) == [2] and "TWO" in pane_text(2)
    print("two clients kept independent tab highlights across attach and reattach")
finally:
    subprocess.run([binary, "kill-session", session], env=env, capture_output=True)
    subprocess.run(["tmux", "-f", "/dev/null", "-L", socket, "kill-server"], env=env, capture_output=True)
