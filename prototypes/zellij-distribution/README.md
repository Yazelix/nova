# Nova Zellij distribution

Edge builds this small distribution from the merged [Zellij PR #5630](https://github.com/zellij-org/zellij/pull/5630)
at upstream commit `333d7898a840314e41d3cc9d2a264a1b9732bb7b`. It embeds
Nova Bar, Radar, popup, pane orchestrator, and `nova-zjhints` into `yzx-zellij`.
The plugins use `zellij:` URLs and need no user permission cache entries.

Upstream [PR #5687](https://github.com/zellij-org/zellij/pull/5687) preserves
relative pane order when reapplying tiled layouts. The native
[`background startup patch`](zellij-background-startup.patch) retains startup
callbacks until the first client connects, then replays them.
Drop it when upstream preserves pending events across client registration.
The isolated [mouse-selection patch](zellij-mouse-selection.patch) lets a fresh
press start a gesture when a tab switch redirected the preceding release.
Drop it when upstream accepts fresh presses during stale selection. The attached
client check forces that sequence before checking ordinary rapid clicks.
The bottom-hint check covers manual rearrangements, stacked and split work areas,
background tabs, floating popups and new-pane placement.

`nova-zjhints` is [zjhints v0.5.0](https://github.com/myah-mitchell/zjhints/releases/tag/v0.5.0)
at `709d56292217920a5b7e2702302cd66b60ed6477` with the isolated
[`grouped modifiers patch`](zjhints-group-modifiers.patch). Its opt-in
`group_modifiers true` option joins hints with the same modifier chord. The
default layout uses compact `{C}`, `{C-A}`, `{A}` and `{A-S}` headers with subdued red
text on separate charcoal backgrounds. Red keys and muted gray descriptions
share the plain background, with one-cell gaps between hints. Whole-hint
fitting keeps core workspace and managed Nova actions ahead of secondary native
hints. Labels include `sess.`, `move` and `full`. Three groups share spare
width through balanced gaps; four or more retain small separators.
Normal hints and minor-mode labels have a one-column leading inset. Consecutive
numbered keys display as `1–9`. Minor modes show a brace-outlined label and one direction-key family; Normal
has no mode prefix. Tab mode uses HJKL, dim pipe separators and an Esc return
hint, omitting numbered-tab, new-tab, close and last-tab-toggle hints.
Pane hints use the same presentation, with one key per main action; secondary
focus, embed, pin and fullscreen-without-UI hints are hidden; bindings remain.
Resize uses `hjkl grow`, `HJKL shrink` and `ESC normal` with the same pipes,
omitting generic resize hints while preserving their bindings.
Scroll uses `jk scroll`, `hl page`, search, scrollback editing and Esc with
the same pipes, omitting duplicate and secondary hints; all bindings remain.
Search shows next/previous matches, letter-based scrolling and separate
case/whole-word/wrap controls; option hints yield to navigation and exit on
narrow rows. Search Input shows Enter search and Esc cancel back to Scroll.
Rename Tab and Rename Pane show Enter save and Esc cancel with the same pipes.
Enter accepts the typed name and returns to Normal; Esc restores the original
name and returns to Tab or Pane. Ctrl-C remains bound, with Enter preferred in the hint.
Session shows detach, manager, config, plugins and Esc with the same pipes;
secondary plugin and nested-session hints are hidden while their bindings remain.
Locked uses the same inset and pipe, with a `{C-A} g unlock` badge.
Brace outlines have no interior padding and use the existing mode/header templates.
Mode-scoped `hint_spacer_<mode>` and `direction_keys_<mode>` keep these choices local.
Compound modes use `enter_search`, `rename_tab` and `rename_pane` consistently
for prefixes and hint settings.
`preferred_key_<mode>_<hint>` advertises a bound key or comma-separated group;
all requested keys must be available, otherwise the actual bindings are shown.
Single-key parsing takes priority, preserving literal comma keys and aliases.
Return controls survive before secondary hints as the row
narrows. Named keys use separators (`ESC / ENTER`, `PgUp / PgDn`), and inline
word-like modifiers use hyphens (`C-A`). Compact character runs, configured
chord aliases and symbol aliases retain their existing presentation.
Nova's two `Alt [` and
`Alt ]` content bindings share one `[] layout` hint.

The isolated [lightweight event patch](zjhints-lightweight-events.patch) keeps
`InitialKeybinds` as the catalog owner across mode updates and native reloads.
Nova's direct pane sets the existing `pipe_name` to an empty string: it publishes
no compatibility messages, omits unused session events and becomes ready even
when base-mode hints are empty. Unset and custom pipe names retain publication
and startup retries. Focus changes redraw nested hints when the host is descended.
The patch can disappear when upstream supplies these catalog/direct-pane paths.
An output cache was rejected after two traced trials produced no cache hits.

Build the exact Edge package with `nix build .#yazelix-edge --no-link`. The
standalone components are `.#nova-zellij-distribution` and `.#nova-zjhints`.
The zjhints package runs its native renderer tests before installing the Wasm.
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
as command 230 with payload field 173; native Zellij assigns those IDs to
`ApplyFloatingSwapLayout`. The pane orchestrator owns session-wide visibility.
Recheck both protocol definitions before changing either pin or enabling
collapse. This incompatibility does not establish the cause of reported pane
loss; evidence and the workaround live in Bead `yazelix-optional-bottom-status-bar-og5l-g71`.
