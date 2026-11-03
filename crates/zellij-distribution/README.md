# Nova Zellij distribution

Nova builds this small distribution from unreleased upstream Zellij main at
commit `77debdeb2c48d7e3529c55dbed16c78e16ee9cf9`, the merge of
[PR #5706](https://github.com/zellij-org/zellij/pull/5706), which replays cached
startup events to background plugins that start before a client connects.
Upstream [PR #5630](https://github.com/zellij-org/zellij/pull/5630) merged the plugin API
Nova uses. The distribution embeds Nova Bar, Radar, popup, pane orchestrator, and `nova-zjhints` into `yzx-zellij`.
The plugins use `zellij:` URLs and need no user permission cache entries.

The native wrapper owns the effective release profile: fat LTO and one codegen
unit match the pinned Zellij workspace's optimization settings. Cargo ignores
profiles in dependencies. Nix strips the installed executable. It keeps
upstream's default features except `window`, Zellij's standalone GUI terminal.

Upstream [PR #5687](https://github.com/zellij-org/zellij/pull/5687) preserves
relative pane order when reapplying tiled layouts. Nova builds Zellij without
patches.
When stdin is not a terminal, upstream `action pipe -- <payload>` also streams
stdin and ends with an empty message from the same CLI pipe. The popup and pane
orchestrator plugins ignore that message instead of patching Zellij.
The bottom-hint check covers manual rearrangements, stacked and split work areas,
background tabs, floating popups and new-pane placement.

`nova-zjhints` is [zjhints v0.5.1](https://github.com/myah-mitchell/zjhints/releases/tag/v0.5.1)
at `1488a64ce2268f141ca538987c01dd261f7681a5` with seven ordered patches,
applied in order:
[named-key separators](zjhints-01-named-key-separators.patch),
[search option labels](zjhints-02-search-option-labels.patch),
[mode-scoped settings](zjhints-03-mode-scoped-settings.patch),
[grouped modifiers](zjhints-04-group-modifiers.patch),
[pipe hints](zjhints-05-pipe-hints.patch),
[direct pane](zjhints-06-direct-pane.patch) and
[descended focus redraw](zjhints-07-descended-focus-redraw.patch).
The opt-in `group_modifiers true` option joins hints with the same modifier chord. The
default layout uses compact `{C}`, `{C-A}`, `{A}` and `{A-S}` headers with subdued red
text on separate charcoal backgrounds. Red keys and muted gray descriptions
share the plain background, with one-cell gaps between hints. Whole-hint
fitting keeps core workspace and managed Nova actions ahead of secondary native
hints. Labels include `sess.`, `move` and `full`. Three groups share spare
width through balanced gaps; four or more retain small separators.
Normal hints and minor-mode labels have a one-column leading inset. Upstream
`keys_go_to_tab "1–9"` shows the numbered tab keys as one run; it is a fixed
string and does not follow remapped `Alt 1-9`. Minor modes show a
brace-outlined label and one direction-key family; Normal has no mode prefix.
Tab mode uses HJKL, dim pipe separators and an Esc return hint, omitting
numbered-tab, new-tab, close and last-tab-toggle hints.
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
Hint settings name compound modes as upstream's `label_<mode>_<id>` does
(`entersearch`, `renametab`, `renamepane`); `mode_format_<mode>` keeps
`enter_search`, `rename_tab` and `rename_pane`.
`preferred_key_<mode>_<hint>` advertises a bound key or comma-separated group;
all requested keys must be available, otherwise the actual bindings are shown.
Single-key parsing takes priority, preserving literal comma keys and aliases.
Return controls survive before secondary hints as the row
narrows. Named keys use separators (`ESC / ENTER`, `PgUp / PgDn`), and inline
word-like modifiers use hyphens (`C-A`). Compact character runs, configured
chord aliases and symbol aliases retain their existing presentation.
Nova's two `Alt [` and
`Alt ]` content bindings are `pipe_hint_layout_prev` and `pipe_hint_layout_next`
with one shared `layout` label, so they merge into one `[] layout` hint.

Upstream v0.5.1 keeps `InitialKeybinds` as the catalog owner across mode
updates and native reloads. Nova's direct pane sets the existing `pipe_name` to
an empty string: it publishes no compatibility messages, omits unused session
events and settles after its first render, even when base-mode hints are empty.
Unset and custom pipe names retain publication and startup retries. Focus
changes redraw nested hints only while the host is descended. v0.5.1 fetches a
nested session's keybindings with a bounded blocking call of about one second
and no longer dims the hints of a descended session.
An output cache was rejected after two traced trials produced no cache hits.

Build the exact Edge package with `nix build .#yazelix-edge --no-link`. The
standalone components are `.#nova-zellij-distribution` and `.#nova-zjhints`.
The zjhints package runs its native renderer tests before installing the Wasm.
The zjhints toolchain pins Fenix separately because it requires Rust 1.96;
Nova Bar's older toolchain remains independent.
Each patch can be dropped once upstream zjhints supplies the same behavior.
When a Zellij release includes PR #5630, Nova can evaluate
replacing this merged-main pin with that release. Until then, test both the
patched plugin and the pinned Zellij API when updating either source.

Zellij's plugin keymap event strips `KeybindPipe` targets. Nova's `nova_hints`
alias supplies `pipe_hint_<id>` chords from its managed configuration; the
existing keybinding patch remaps or omits each annotation with its binding.
The plugin advertises an annotation only for a matching active pure-pipe key,
and uses existing label, grouping and fitting options. This seam can disappear
when native events expose pipe identities and upstream zjhints discovers them.
The default `[] layout` hint shows each Alt bracket chord that still carries a
pipe action; it cannot distinguish another plugin target on that pair. If only
one stays a pure pipe, the bar shows `[ layout`.
Other unannotated plugin messages, stock status bar mouse controls and clipboard
feedback are omitted.

Keep `collapse_when_empty false` in Nova's bottom-hints template. zjhints
v0.5.1 builds against `myah-mitchell/zellij` `fork-v0.46.3` (`9cda980b`), which
encodes `SetSelfCollapsed` as command 237 with payload field 180; Nova's Zellij
`77debdeb` assigns 237 to `OpenContextMenu` and encodes `SetSelfCollapsed` as
256 with field 199. The pane orchestrator owns session-wide visibility.
Recheck both protocol definitions before changing either pin or enabling
collapse. This incompatibility does not establish the cause of reported pane
loss; evidence and the workaround live in Bead `yazelix-optional-bottom-status-bar-og5l-g71`.
