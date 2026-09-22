#!/usr/bin/env python3
"""Build an isolated Zellij distribution from Nova's exact packaged plugins."""

import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parents[2]
PROTOTYPE = Path(__file__).resolve().parent


def main():
    package = Path(
        subprocess.check_output(
            ["nix", "build", ".#yazelix", "--no-link", "--print-out-paths"],
            cwd=ROOT,
            text=True,
        ).strip()
    )
    target = PROTOTYPE / "target"
    target.mkdir(exist_ok=True)
    work = Path(tempfile.mkdtemp(prefix="runtime-", dir=target))
    config_home = work / "config"
    config_home.mkdir()
    (config_home / "config.toml").write_bytes((ROOT / "defaults/config.toml").read_bytes())
    state = work / "state"
    runtime_env = os.environ.copy()
    runtime_env.update(HOME=str(work), YAZELIX_CONFIG_HOME=str(config_home), YAZELIX_STATE_DIR=str(state))
    runtime_env.pop("ZELLIJ_SESSION_NAME", None)
    runtime_env.pop("YAZELIX_ZELLIJ_SESSION_NAME", None)
    subprocess.run([str(package / "bin/yzx"), "run", "true"], env=runtime_env, check=True)
    config = (state / "zellij/config.kdl").read_text()

    aliases = dict(
        re.findall(
            r'^\s*(yzpp|yazelix_pane_orchestrator|radar|radar_controller) location="file:([^"]+)"',
            config,
            re.MULTILINE,
        )
    )
    if set(aliases) != {"yzpp", "yazelix_pane_orchestrator", "radar", "radar_controller"}:
        raise RuntimeError("Nova's packaged plugin aliases changed")
    if aliases["radar"] != aliases["radar_controller"]:
        raise RuntimeError("Radar view and controller use different Wasm artifacts")
    bar = re.findall(r'^\s*"file:([^"]*zjstatus\.wasm)"\s*\{', config, re.MULTILINE)
    if len(bar) != 1:
        raise RuntimeError("Nova's packaged bar controller changed")

    plugins = {
        "YZX_PROTO_YZPP_WASM": ("yzpp", aliases["yzpp"]),
        "YZX_PROTO_ORCHESTRATOR_WASM": (
            "yazelix_pane_orchestrator",
            aliases["yazelix_pane_orchestrator"],
        ),
        "YZX_PROTO_RADAR_WASM": ("radar", aliases["radar"]),
        "YZX_PROTO_BAR_WASM": ("nova-bar", bar[0]),
    }
    build_env = os.environ.copy()
    build_env["CARGO_TARGET_DIR"] = str(target)
    for key, (name, path) in plugins.items():
        if Path(path).read_bytes()[:4] != b"\0asm":
            raise RuntimeError(f"{name} is not a Wasm module: {path}")
        config = config.replace(f"file:{path}", f"zellij:{name}")
        build_env[key] = path

    layout_dirs = re.findall(r'^layout_dir "([^"]+)"$', config, re.MULTILINE)
    if len(layout_dirs) != 1:
        raise RuntimeError("Nova's managed layout directory changed")
    layout_dir = Path(layout_dirs[0])
    isolated_layout = work / "layout"
    isolated_layout.mkdir()
    layout = (layout_dir / "layout.kdl").read_text()
    if f"file:{bar[0]}" not in layout:
        raise RuntimeError("Nova's packaged bar view changed")
    (isolated_layout / "layout.kdl").write_text(layout.replace(f"file:{bar[0]}", "zellij:nova-bar"))
    shutil.copy2(layout_dir / "layout.swap.kdl", isolated_layout / "layout.swap.kdl")
    config = config.replace(str(layout_dir), str(isolated_layout))

    rendered = state / "zellij/config.prototype.kdl"
    rendered.write_text(config)
    (state / "zellij/permissions.kdl").unlink(missing_ok=True)
    subprocess.run(["cargo", "build", "--release", "--locked", "--manifest-path", str(PROTOTYPE / "Cargo.toml")], env=build_env, check=True)
    install = work / "bin"
    install.mkdir()
    binary = install / "yzx-zellij-prototype"
    shutil.copy2(target / "release/yzx-zellij-prototype", binary)
    version = subprocess.check_output([str(binary), "--version"], text=True).strip()
    if not version.startswith("yzx-zellij-prototype 0.1.0 (zellij "):
        raise RuntimeError(f"unexpected prototype identity: {version}")
    subprocess.run([str(binary), "--config", str(rendered), "setup", "--check"], env=runtime_env, check=True)
    print(f"binary: {binary}\nconfig: {rendered}\nversion: {version}")


if __name__ == "__main__":
    main()
