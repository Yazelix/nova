# Zellij distribution prototype

This isolated executable tests [Zellij PR #5630](https://github.com/zellij-org/zellij/pull/5630)
at commit `252454d2c53b56e18aea06da3cf79b174ce1d7c0`. It embeds Nova's four
packaged Wasm modules and renders isolated copies of Nova's config and layouts
with `zellij:` plugin locations. It uses the `yzx-zellij-prototype` distribution
name so its config, cache, and sessions do not collide with installed Nova.
The production `yzx` launcher and user config are unchanged.

From the Nova repository root, with Nix and Cargo available:

```text
python3 prototypes/zellij-distribution/build.py
```

The command builds the current Nova package, prepares a throwaway Nova runtime
under the prototype's ignored `target/`, compiles the pinned Zellij distribution,
copies the binary into that runtime, checks its version, and runs `setup --check`
against the translated config. It removes Nova's generated plugin permission
grants so terminal testing exercises the distribution's embedded-plugin policy.
It prints the installed binary and config paths for further testing. The
prototype does not replace Nova's dynamically generated Zellij config or
reproduce the fork's status-bar modifier hints behavior.
