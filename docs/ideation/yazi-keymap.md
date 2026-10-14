# Managed Yazi key model

Status: Option F+ selected and implemented. The resulting keys are documented
under [Editor and file opens](../configuration.md#editor-and-file-opens) and
their mechanism under [Tab Workspace](../runtime-notes.md#tab-workspace). This
record keeps the decision, its constraints, and the rejected alternatives.

**Move Yazi** changes the directory shown in Yazi. **Tab folder** is Nova's
canonical workspace root for the active tab; setting it rewrites no process cwd.
**Open editor** opens or focuses `editor.command`.

## Accepted Option F+

`Tab` and vanilla-compatible `Z` open one move-only recent-folder search in
startup and popup Yazi; `Tab` and `Esc` return to browsing, and `Shift Tab`
takes over Spot.

| Key | Startup search | Popup search |
| --- | --- | --- |
| `Enter` | Move Yazi to the selected directory and return to browsing. | Move Yazi to the selected directory and return to browsing. |
| `Ctrl O` | Set the selected directory as the initial tab folder, open it in the configured editor, and complete startup. | No managed action. |
| `Alt Enter` | No managed action. | Set the selected directory as the tab folder without moving Yazi or opening an editor. |

Startup ordinary open establishes the initial tab folder, opens the editor, and
closes the exact picker. A startup `Alt Enter` was therefore normally
overwritten, so it was removed; the persistent popup keeps it because changing
an established tab folder without moving or opening anything is useful there.
`Ctrl O` restores a one-action startup path without making `Enter`
role-dependent. It is mnemonic and distinguishable in legacy terminals and over
SSH, where modified Enter variants depend on enhanced keyboard protocols.

## Rejected alternatives

- **A, contextual search:** search `Enter` opened the editor at startup but only
  moved Yazi in the popup. The hidden role change needed repeated explanation.
- **B, separate `Shift Z` and `Alt Z` searches:** consistent per key, but two
  similar shortcuts with very different results punish a modifier mistake.
- **C, move-only `Shift Z` without a Tab switch:** fewest shortcuts, but no
  discoverable search entry and no one-step startup open.
- **D, Tab-switched move-only search:** the base of F; it dropped vanilla `Z`.
- **E, D plus a dedicated editor-open browse key:** redefining Yazi's `Enter`/`o`
  pair would reduce vanilla compatibility without removing the search/browse
  distinction.
- **F, D plus vanilla `Z`:** selected before the popup-only `Alt Enter` and
  `Ctrl O` refinements, at the cost of moving native Tab Spot and a redundant
  browse-mode alias.

Replacing the startup picker with the persistent popup is a separate question.
A tab containing only Radar or another plugin pane lacks the terminal anchor
Zellij needs for normal pane creation, and a permanent starter shell would keep
a stale cwd. A layout-declared floating Yazi may merit a separate experiment.

## Implementation constraints

- Keep one quick-search implementation for both `Tab` and `Z`.
- Ignore `Alt Enter` in startup browse and search, and omit it from both startup
  footers.
- Let startup search `Ctrl O` pass the selected directory to the existing
  editor-open boundary and advertise it only in the startup search footer.
- Keep `Alt Enter` tab-folder-only in persistent-popup browse and search.
- Let the startup lifecycle own the initial tab-folder/editor handoff.
- Keep tab-folder mutation in the existing workspace boundary and Yazi movement
  in Yazi's native `cd` mechanism.
- Outside `startup-picker` and `workspace-popup`, preserve native Tab Spot and
  native `Z` zoxide behavior.
- Do not add a setting, dependency, second picker, persistent mode state, or
  `Alt Z` action.
- Prompts and footers must name the result before confirmation.
- Verify the installed interaction on Linux and Darwin because the keymap and
  packaged Yazi configuration are shared surfaces.

The implementation feeds `zoxide query --list` to packaged `fzf`. Startup uses
`--expect=ctrl-o` and binds `Alt Enter` to ignore; the popup uses
`--expect=alt-enter`. Passing expectations through zoxide 0.9.9's interactive
helper is rejected: it strips the first seven bytes of fzf output, corrupting
the expected-key marker.
