# Nova Zellij distribution

Edge builds this small distribution from the merged [Zellij PR #5630](https://github.com/zellij-org/zellij/pull/5630)
at upstream commit `81f56e1aed4e17b822af5cb382a8f524e35f3eae`. It embeds
Nova Bar, Radar, popup, pane orchestrator, and `nova-zjhints` into `yzx-zellij`.
The plugins use `zellij:` URLs and need no user permission cache entries.

`nova-zjhints` is [zjhints v0.5.0](https://github.com/myah-mitchell/zjhints/releases/tag/v0.5.0)
at `709d56292217920a5b7e2702302cd66b60ed6477` with the isolated
[`grouped modifiers patch`](zjhints-group-modifiers.patch). Its opt-in
`group_modifiers true` option joins hints with the same modifier chord. The
default layout sets `modifier_format` to a deeper green (`#4ca630`); layouts
without that option use the Zellij theme color. Each modifier ends in `+`;
each key sits on charcoal with cream text. Adjacent bindings touch within a
group; one column separates its modifier from the keys. Three groups share
the available row width through balanced gaps.
The row has a one-column leading inset and fills the plugin width. Consecutive
numbered keys display as `1–9`. The default layout omits the mode badge. Nova's
two `Alt [` and `Alt ]` content bindings share one `[] layout` hint.

Build the exact Edge package with `nix build .#yazelix-edge --no-link`. The
standalone components are `.#nova-zellij-distribution` and `.#nova-zjhints`.
The zjhints toolchain pins Fenix separately because it requires Rust 1.96;
Nova Bar's older toolchain remains independent.
The grouped-modifier patch can be dropped once upstream zjhints supplies the
same behavior. When a Zellij release includes PR #5630, Nova can evaluate
replacing this merged-main pin with that release. Until then, test both the
patched plugin and the pinned Zellij API when updating either source.

Zellij's plugin keymap event strips `KeybindPipe` targets. `nova-zjhints`
shows `[] layout` while both default Alt bracket chords still carry pipe
actions; it cannot distinguish another plugin target on that same pair. Other
plugin messages, stock status bar mouse controls, and clipboard feedback are
omitted.
