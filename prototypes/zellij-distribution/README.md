# Nova Zellij distribution

Edge builds this small distribution from the merged [Zellij PR #5630](https://github.com/zellij-org/zellij/pull/5630)
at upstream commit `81f56e1aed4e17b822af5cb382a8f524e35f3eae`. It embeds
Nova Bar, Radar, popup, pane orchestrator, and `nova-zjhints` into `yzx-zellij`.
The plugins use `zellij:` URLs and need no user permission cache entries.

`nova-zjhints` is [zjhints v0.5.0](https://github.com/myah-mitchell/zjhints/releases/tag/v0.5.0)
at `709d56292217920a5b7e2702302cd66b60ed6477` with the isolated
[`grouped modifiers patch`](zjhints-group-modifiers.patch). Its opt-in
`group_modifiers true` option joins hints with the same modifier chord. The
default layout uses compact `C`, `C-A`, `A` and `A-S` headers with subdued red
text on separate charcoal backgrounds. Red keys and muted gray descriptions
share the plain background, with one-cell gaps between hints. Whole-hint
fitting keeps core workspace and managed Nova actions ahead of secondary native
hints. Labels include `sess.`, `mv tab` and `full`. Three groups share spare
width through balanced gaps; four or more retain small separators.
The row has a one-column leading inset. Consecutive numbered keys display as
`1–9`, and the default layout omits the mode badge. Nova's two `Alt [` and
`Alt ]` content bindings share one `[] layout` hint.

Build the exact Edge package with `nix build .#yazelix-edge --no-link`. The
standalone components are `.#nova-zellij-distribution` and `.#nova-zjhints`.
The zjhints toolchain pins Fenix separately because it requires Rust 1.96;
Nova Bar's older toolchain remains independent.
The grouped-modifier patch can be dropped once upstream zjhints supplies the
same behavior. When a Zellij release includes PR #5630, Nova can evaluate
replacing this merged-main pin with that release. Until then, test both the
patched plugin and the pinned Zellij API when updating either source.

Zellij's plugin keymap event strips `KeybindPipe` targets. Nova's `nova_hints`
alias supplies `pipe_hint_<id>` chords from its managed configuration; the
existing keybinding patch remaps or omits each annotation with its binding.
The plugin advertises an annotation only for a matching active pure-pipe key,
and uses existing label, grouping and fitting options. This seam can disappear
when native events expose pipe identities and upstream zjhints discovers them.
The default `[] layout` hint still requires both Alt bracket chords to carry
pipe actions; it cannot distinguish another plugin target on that pair.
Other unannotated plugin messages, stock status bar mouse controls and clipboard
feedback are omitted.

Keep `collapse_when_empty false` in Nova's bottom-hints template. zjhints' tile
fork `cb9e59fd3481b6d7becf432db5ae65559ad4dfc4` encodes `SetSelfCollapsed`
as command 230 with payload field 173; native `81f56e1` assigns those IDs to
`ApplyFloatingSwapLayout`. The pane orchestrator owns session-wide visibility.
Recheck both protocol definitions before changing either pin or enabling
collapse. This incompatibility does not establish the cause of reported pane
loss; evidence and the workaround live in Bead `yazelix-optional-bottom-status-bar-og5l-g71`.
