# Nova Zellij distribution

Edge builds this small distribution on [Zellij PR #5630](https://github.com/zellij-org/zellij/pull/5630)
at commit `252454d2c53b56e18aea06da3cf79b174ce1d7c0`. The PR is still
unmerged, so Nova relies on its pinned upstream branch. The distribution embeds
Nova Bar, Radar, popup, pane orchestrator, and `nova-zjhints` into `yzx-zellij`.
The plugins use `zellij:` URLs and need no user permission cache entries.

`nova-zjhints` is [zjhints v0.5.0](https://github.com/myah-mitchell/zjhints/releases/tag/v0.5.0)
at `709d56292217920a5b7e2702302cd66b60ed6477` with the isolated
[`grouped modifiers patch`](zjhints-group-modifiers.patch). Its opt-in
`group_modifiers true` option joins hints with the same modifier chord. The
modifier tile is Zellij green and ends in `+`; each key sits on charcoal with
cream text. Adjacent bindings touch, the modifier has a one-column gap before
its keys, and three groups share the available row width through balanced gaps.
The first and last groups align with the surrounding pane frames. Consecutive
numbered keys display as `1–9`. The default layout omits the mode badge.

Build the exact Edge package with `nix build .#yazelix-edge --no-link`. The
standalone components are `.#nova-zellij-distribution` and `.#nova-zjhints`.
The zjhints toolchain pins Fenix separately because it requires Rust 1.96;
Nova Bar's older toolchain remains independent.
The grouped-modifier patch can be dropped once upstream zjhints supplies the
same behavior. The distribution wrapper can switch to released Zellij once
PR #5630 is merged and packaged. Until then, test both the patched plugin and
the pinned Zellij API when updating either source.

zjhints omits bindings backed by Zellij's `KeybindPipe` action, including
Nova's `MessagePlugin` shortcuts. It also omits the stock status bar's mouse
controls and clipboard feedback.
