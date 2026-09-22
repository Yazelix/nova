# Managed Yazi key model

Status: Option F+ selected for Nova. It retains Option F, keeps `Alt Enter` only
in the persistent popup, and adds a direct startup-search editor path. This
record preserves the alternatives and the reasoning that led to the decision.

## Terms

- **Move Yazi** changes the directory shown in Yazi. It does not change the tab
  folder or open an editor.
- **Tab folder** is the user-facing name for Nova's canonical workspace root for
  the active Zellij tab. Setting it does not rewrite the cwd of Yazi, existing
  shells, or an existing editor process.
- **Open editor** opens or focuses the editor selected by `editor.command`.
  Helix is the default.

Across the six options originally analyzed, `Alt Enter` sets the hovered
directory, or the current directory when no directory is hovered, as the tab
folder without opening or focusing an editor. The accepted refinement below
removes that action from startup.

## Option A: Tab switches browse and search

Startup Yazi opens in recent-folder search. `Tab` switches between search and
browsing, and `Shift Tab` takes over Yazi's Spot action.

| Context | Search `Enter` |
| --- | --- |
| Startup | Set the selected folder as the tab folder, open it in the configured editor, and complete startup. |
| Popup | Move Yazi to the selected folder for inspection. |

`Alt Enter` sets the hovered or current folder as the tab folder in both
contexts.

Benefits:

- Users can discover search through the footer.
- Users reach the editor in one step from startup search.
- One key switches between the two Yazi views.

Costs:

- Users must track a search/browse mode.
- Search `Enter` changes meaning based on the hidden startup or popup role.
- Nova replaces native Tab Spot and moves Spot to `Shift Tab`.
- The popup result required repeated explanation, which indicates a
  learnability problem.

## Option B: Separate browse and open searches

Each invocation chooses the result before search opens.

| Key | Action |
| --- | --- |
| `Shift Z` | Search recent folders and move Yazi to the selection. |
| `Alt Z` | Search recent folders, set the selection as the tab folder, and open it in the configured editor. |
| `Alt Enter` | Set the hovered or current folder as the tab folder without opening an editor. |
| `Enter` | Keep Yazi's normal navigation and file-opening behavior. |
| `Tab` | Keep Yazi's native Spot behavior. |

Startup Yazi enters the `Alt Z` flow. Accepting a recent folder sets the initial
tab folder, opens the editor, and completes startup. `Esc` returns to ordinary
Yazi browsing. `Alt Z` reopens the start-and-open search, while `Shift Z` opens
the browse-only search.

The persistent popup opens in ordinary Yazi browsing mode. Users choose
`Shift Z` to inspect another folder, `Alt Z` to open another workspace, or
`Alt Enter` to commit the folder already under inspection.

Benefits:

- Each shortcut keeps the same result in startup and popup Yazi.
- Users choose browse, open, or tab-folder-only before entering search.
- Tab keeps its native Yazi behavior.
- Startup retains a one-step search-and-open path.

Costs:

- Users must learn both `Shift Z` and `Alt Z`.
- Users must distinguish two quick-search results.
- The similar shortcuts have consequences different enough to punish a
  modifier mistake.

## Option C: Quick search only moves Yazi

Quick search has one task in every context: move Yazi to a recent folder.

| Key | Action |
| --- | --- |
| `Shift Z` | Search recent folders and move Yazi to the selection. |
| `Alt Enter` | Set the hovered or current folder as the tab folder without opening an editor. |
| `Enter` | Keep Yazi's normal navigation and file-opening behavior. |
| `Tab` | Keep Yazi's native Spot behavior. |

Startup Yazi opens in recent-folder search. Accepting a folder moves Yazi there
and returns to browsing. Opening a file with normal Yazi `Enter` performs the
startup tab-folder/editor handoff. `Alt Enter` can set the folder first, but it
leaves the user in Yazi.

The persistent popup uses the same flow. `Shift Z` moves Yazi, normal `Enter`
opens a file, and `Alt Enter` changes the tab folder.

Benefits:

- Quick search has one result to learn and one implementation path.
- Startup and popup search produce the same result.
- Tab and Enter retain their normal Yazi roles.
- Users need no search mode or `Alt Z` shortcut.

Costs:

- Startup requires another action after choosing a recent folder.
- Users cannot search for a folder and open that folder in the editor with one
  command.
- Opening a searched workspace requires moving there, choosing a file, and
  pressing `Enter`, or setting the tab folder with `Alt Enter` before opening a
  file.

## Option D: Tab switches views, actions stay fixed

`Tab` switches between Yazi browsing and recent-folder search. Search remains a
folder-moving tool. `Enter` and `Alt Enter` keep the same target-based behavior
in both views and both Yazi roles.

| Key | Browse view | Search view |
| --- | --- | --- |
| `Tab` | Open recent-folder search. | Return to Yazi browsing. |
| `Enter` | Enter a directory or open a file through normal Yazi behavior. | Move Yazi to the selected directory and return to browsing. |
| `Alt Enter` | Set the hovered or current directory as the tab folder. | Set the selected directory as the tab folder. |
| `Shift Tab` | Spot the hovered file. | Move through search results. |

`Alt Enter` opens no editor and does not move Yazi in either view. After using
it in search, the user can press `Tab` or `Esc` to return to the prior Yazi
location.

Startup Yazi opens in recent-folder search. `Enter` moves Yazi to the selected
folder and returns to browsing. The user then opens a file with normal Yazi
`Enter`, which performs the startup tab-folder/editor handoff. The persistent
popup follows the same flow.

Benefits:

- Quick search has one result: move Yazi.
- `Enter` and `Alt Enter` keep the same meaning across views and roles.
- Users need neither `Shift Z` nor `Alt Z`.
- The footer can teach search through the visible Tab switch.

Costs:

- Users must track the browse/search view.
- Nova replaces native Tab Spot and moves Spot to `Shift Tab`.
- Startup requires another action after choosing a recent folder.
- `Alt Enter` from search changes the tab folder without moving Yazi, so the
  footer and confirmation notice must make that result clear.
- Users cannot search for a folder and open that folder in the editor with one
  command.

## Option E: Give editor opening its own browse key

This variation keeps Option D's Tab-switched, move-only search. Search `Enter`
selects the directory, moves Yazi there, and returns to browsing. A dedicated
browse key such as `o` then opens the selection in the editor, making the second
startup step explicit.

Benefits:

- Search stays move-only and consistent.
- Editor opening is an explicit follow-up rather than extra startup power on
  ordinary `Enter`.

Costs:

- It changes Yazi's browse key model to solve a Nova startup distinction.
- Vanilla Yazi already treats `Enter` and `o` as open actions and uses `l` or
  Right to enter a directory, so redefining the pair would reduce rather than
  improve compatibility.
- It adds another key concept without removing the search/browse distinction.

## Option F: Option D with vanilla-compatible Z

This was the selected model before the popup-only `Alt Enter` refinement below.
It keeps Option D and retains capital `Z`, Yazi's standard zoxide motion key,
as a second way to enter the same search.

| Key | Browse view | Search view |
| --- | --- | --- |
| `Tab` | Open recent-folder search. | Return to Yazi browsing. |
| `Z` | Open the same recent-folder search. | Type a normal `Z` query character. |
| `Enter` | Use normal Yazi open/navigation behavior. | Move Yazi to the selected directory and return to browsing. |
| `Alt Enter` | Set the hovered or current directory as the tab folder. | Set the selected directory as the tab folder without moving Yazi. |
| `Shift Tab` | Spot the hovered file. | Move through search results. |

Startup and popup search are identical and move-only. Startup ordinary open has
one additional lifecycle responsibility: it establishes the initial tab folder,
opens the configured editor, and closes the exact startup picker after the
editor appears. Popup ordinary opens preserve the established tab folder.

Benefits:

- Quick search has one task and one implementation.
- `Enter` and `Alt Enter` have the same search result in startup and popup.
- `Tab` makes the mode switch discoverable and two-way.
- `Z` preserves useful vanilla Yazi muscle memory without adding another mode.
- No `Alt Z` action or destructive modifier distinction remains.

Costs:

- Nova still moves native Tab Spot to `Shift Tab` in the two managed surfaces.
- Startup takes a normal browse/open action after choosing a recent folder.
- `Tab` and `Z` are aliases in browse mode, although only Tab is two-way.

## Comparison

| Concern | A | B | C | D | E | F |
| --- | --- | --- | --- | --- | --- | --- |
| Search concepts | One contextual search | Browse and open searches | One move-only search | One move-only search with Tab switching | D plus explicit editor-open key | D plus vanilla `Z` alias |
| Direct search-to-editor | Startup only | Startup and popup | None | None | None | None |
| Search result consistency | Changes by Yazi role | Changes by invoking key | Moves Yazi in both roles | Moves Yazi in both roles | Moves Yazi in both roles | Moves Yazi in both roles |
| Native Tab Spot | Replaced in managed roles | Preserved | Preserved | Replaced in managed roles | Replaced in managed roles | Replaced in managed roles |
| Vanilla Z motion | Replaced by Tab model | `Shift Z` variant | `Shift Z` variant | Not retained | Not retained | Retained as shared search entry |
| Shortcut load | Low | Highest | Lowest | Low | Medium | Low |
| Startup speed | One step | One step | At least two steps | At least two steps | At least two steps | At least two steps |
| Main risk | Hidden role changes meaning | Modifier mistake | Search discoverability | View confusion | Non-vanilla browse keys | Alias redundancy |

Option F initially won because the shared move-only search is consistent, Tab
teaches the feature, `Z` preserves useful vanilla compatibility, and `Alt Enter`
had one non-editor meaning everywhere. The redundant browse-mode alias was
cheaper than separate search modes or contextual acceptance.

## Accepted refinement: popup-only Alt Enter

Startup ordinary open always sets the initial tab folder, opens the editor, and
closes the picker. A prior workspace-only `Alt Enter` selection was therefore
normally overwritten; its only distinct use was setting the tab folder before
creating another pane from the unfinished picker. That niche did not justify a
prominent startup shortcut or footer entry.

The accepted model keeps Option F's shared move-first search and vanilla `Z`
alias. Startup browse and search ignore `Alt Enter`. The persistent popup
retains `Alt Enter` in browse and search because changing an established tab
folder without opening or moving anything is useful there. This is a deliberate
role difference tied to startup lifecycle, not a second search mode.

## Accepted Option F+: direct startup open

Option F+ retains Option F's search and adds one explicit acceptance shortcut
for the common case where zoxide already knows the intended project.

| Key | Startup search | Popup search |
| --- | --- | --- |
| `Enter` | Move Yazi to the selected directory and return to browsing. | Move Yazi to the selected directory and return to browsing. |
| `Ctrl O` | Set the selected directory as the initial tab folder, open it in the configured editor, and complete startup. | No managed action. |
| `Alt Enter` | No managed action. | Set the selected directory as the tab folder without moving Yazi or opening an editor. |

This restores a one-action startup path without making ordinary `Enter`
role-dependent or adding a second search mode. `Ctrl O` is mnemonic and remains
distinguishable in legacy terminals and over SSH; modified Enter variants would
depend more heavily on enhanced keyboard-protocol support. The cost is one
startup-only shortcut, kept visible in the startup search footer.

## Separate architecture question: replace startup Yazi with the popup

Using the canonical persistent popup for startup is independent from the keymap
choice, so it is not another lettered key model.

The current startup pane remains an ephemeral tiled Yazi picker. Earlier
experiments established that a tab containing only Radar or another plugin pane
does not provide the terminal anchor Zellij needs for normal pane creation, so
opening the popup on top of that is not a valid bootstrap. Adding a permanent
starter shell would make its cwd stale after the tab folder changes. A
layout-declared canonical floating Yazi may be worth a separate experiment, but
it is not required for Option F and is not selected here.

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

The implementation uses `zoxide query --list` as the recent-folder source and
feeds its output to packaged `fzf`. Startup uses `--expect=ctrl-o` and binds
`Alt Enter` to ignore; the popup uses `--expect=alt-enter`. Passing expectations
through zoxide 0.9.9's interactive helper is rejected: the helper strips the
first seven bytes of fzf output, corrupting the expected-key marker. Direct fzf
ownership keeps the acceptance keys distinguishable without duplicating search
behavior.
