# Architecture

Yazelix Nova is a small Nix/Lix flake with one front door: **`yzx`**.

This repo owns the glue that makes Rio, Nova Zellij, Yazi, and
Helix feel like one runtime. It is not a general terminal distro, a broad Home
Manager config system, or a main-Yazelix compatibility layer.

## Runtime chain

```text
yzx launch  →  Rio  →  yzx-welcome  →  Nova Zellij  →  Radar + work panes
yzx enter   →  yzx-welcome  →  Nova Zellij  →  same layout
yzx run     →  prepared Yazelix environment  →  exact child argv/status
yzx yazi-config materialize  →  private materializer  →  effective Yazi config path
```

Bare `yzx` prints help. `launch` is the only Rio route.
`enter` is the headless/SSH route and requires an interactive host terminal, not
a display server. `yazelix-no-helix` retains the Rio route and delegates
editing to an installed host command.
`no-yazi` variants keep the managed Yazi launcher and integration while
resolving a matching host `yazi`/`ya` pair. The three omission suffixes compose
into eight explicit package and app outputs.

The [portable runtime contract](docs/portable-runtime.md) defines the future
managed no-Rio artifact and its root owner. Current installation uses Nix.

## Platforms

[Installation](docs/installation.md#capability-matrix) owns the supported
systems, package variants, and per-platform evidence.

## Commands

| Command | Role |
| --- | --- |
| `yzx` / `help` | Concise help; no implicit launch |
| `--version` | Package-owned exact Nova version |
| `launch` | Rio then managed session |
| `enter` | Managed session in current terminal |
| `run` | Structured command in the prepared runtime environment |
| `yzx-zellij` | Exact packaged Zellij CLI; not managed workspace startup |
| `config` | Ratconfig UI |
| `yazi-config materialize` | Explicit-path Yazi config materialization for automation |
| `menu` | Curated command palette |
| `tutor` | Guided lessons / native tutor hints |
| `anima` | Terminal animations / welcome styles |
| `reveal` | Path in the active tab's persistent Yazi popup |
| `status` / `status --json` | Human and schema-versioned runtime status |
| `doctor` | Owned setup checks |
| `env` | Managed shell only |
| `help` | Help |

---

## Owners

One owner per concern. Paths are the durable map.

### Package and install

| Path | Owns |
| --- | --- |
| `flake.nix` | Eight fixed Rio/managed-Helix/managed-Yazi compositions, inputs, helpers, desktop entry, HM export |
| `home-manager/module.nix` | `programs.yazelix.enable` / package; optional config files; no default generation |

### Front door and helpers

| Path | Owns |
| --- | --- |
| `runtime/yzx/` | CLI, public Yazi materializer grammar/delegation, startup env, host-Yazi pair resolution, launch/enter handoff |
| `runtime/yzx-menu.rs` | Menu palette |
| `runtime/yzx-agent.rs` | Initial agent title, custom-command exec, provider bootstrap (`codex resume` → `grok` → `opencode` → `pi` → `claude --resume`), and Codex Radar consent, health checks, and repair notice |
| `runtime/yzx-yazi.rs` | Managed Yazi process/env launch, active session appearance lookup, editor resolve, fast startup search/Yazi toggle lifecycle, and workspace-popup roles |
| `runtime/yzx-nu.rs` | Managed Nu layering; runtime-effective Starship config request |
| `runtime/yzx-zellij-config.rs` | Packaged + guarded Zellij scalar sidecar merge |
| `runtime/yzx/zellij.rs` | Plugin sidecar inject; launch materialize/patches |
| `crates/yzx-open/` | Editor open, Helix bridge, successful startup-picker orchestrator handoff, target-at-launch popup reveal, bounded diagnostics |
| `crates/yzx-yazi-config/` | Managed Yazi config-home materialization, native TOML layering, and runtime-only flavor projection |
| `crates/yzx-tutor/` | Tutor CLI and lessons |
| `runtime/yzx-helix.sh` (`yzx-hx`) | Effective Helix config + Steel wiring |
| `checks/` | Build-time contract guards |

### Config UI

`crates/yzx-config/` is the Ratconfig host. It supplies one stable source/path
identity per field, reviewed recommendation allowlists, sparse persistence,
exact native-file actions, Home Manager read-only guidance, and source
diagnostics; Ratconfig owns views, filtering, search, and editing. It resolves
override intent separately from baseline and effective values and completes
reloads by field identity. Rio and Helix keep their own schemas. It also
serves hidden package-internal reads for launch and custom-popup KDL rendering,
including the internal `agent.popup.kdl` path that routes custom agent argv
through the packaged launcher.
[Configuration](docs/configuration.md) describes the user-visible behavior.
`KEY_BINDINGS` is the human key reference; `defaults/zellij/config.kdl` is the
runtime owner.

#### Nova root schema

Packaged `defaults/config.toml` owns every root default; the optional user file
stores only explicit overrides. `CONFIG_FIELDS` and `root_config.rs` own the
bounded catalog, validation, and sparse persistence. The validator derives fixed
leaves from that catalog, rejects unknown paths before runtime or Ratconfig use,
and delegates only `popups.<id>` to `custom_popups.rs`.
`ROOT_CONFIG_RECOMMENDED_PATHS` selects the Overview recommendations.
[Configuration](docs/configuration.md#main-settings) lists every field.

### Packaged layout and tools

| Path | Owns |
| --- | --- |
| `defaults/rio/` | Complete seed-once Rio window, font, cursor, effects, and adaptive-theme configuration; Nova reserves only top-level `force-theme` for integrated appearance |
| `defaults/zellij/config.kdl` | Zellij keys, plugin loads, popup wiring, Kitty protocol; leaves application-local `Alt r` routing to Helix and Yazi |
| `defaults/zellij/layout*.kdl` | Radar + stacked panes, open/closed swap |
| `defaults/nu/` | Packaged Nu: carapace, zoxide, and Starship invocation |
| `defaults/yazi/` | Picker and popup initialization, opens via `yzx-open`, `Alt Enter` tab-workspace selection, and `Alt r` popup hide |
| `defaults/helix/config.toml` | Packaged defaults; `Alt r` reveal and `Ctrl r` reload; runtime composition adds the Forest binding |

### Packaged components (not owned here)

| Child | Domain |
| --- | --- |
| Nova Rio | Terminal and native configuration schema |
| Nova Zellij distribution | Unreleased upstream Zellij main with two carried patches and embedded plugins |
| Nova Helix | Editor fork |
| Yazelix Forest | Managed Helix file tree and its native interaction behavior |
| zj-radar | Session and agent-attention rail, command pipe, and producer CLI |
| Zellij Popup (`yzpp`) | Popup lifecycle |
| Zellij Pane Orchestrator | Pane focus, Radar visibility, workspace state, and popup request routing |
| Nova Bar | Top bar rendering, widgets, palettes, and integrated KDL |
| zjstatus | WebAssembly renderer used by Nova Bar; host-theme and workspace-pipe fixes |
| Ratconfig | Config UI toolkit |
| Anima | Welcome / screen animations |
| Yazi Bistro | Complete pinned Yazi flavors, provenance, licenses, dark/light classification, and the packaged light default |
| auto-layout.yazi | Adaptive Yazi column layout |

This repo packages them and applies product policy only.

### Installed closure topology

`flake.nix` owns the package graph. The full package includes the terminal,
workspace, managed editor, and managed Yazi. Variant outputs remove managed
Helix and/or the bundled Yazi executable without changing the
remaining integration contracts. Nova's top-level outputs are thin command,
desktop-entry, and asset joins whose references pull in their selected runtime
graph.

### Shell dispatch

`yzx-shell` reads `shell.program` via `yzx-config`, then runs packaged `nu`
(through `yzx-nu`) or plain `bash` / `zsh` / `fish`.

Nix generates locked Atuin init files for every shell; Atuin owns its history
database and native config, and Carapace owns Nushell external completion. Nova
owns package selection, startup composition, and the default-on policy
([Runtime Notes](docs/runtime-notes.md#shell-history-nushell-and-starship)).

---

## Module and repo boundaries

| Boundary | When |
| --- | --- |
| Local module | Large code, still ships with `yzx`, no independent user |
| Local crate | Cargo/binary/test isolation, still product glue |
| Separate repo | Independent users + release cadence + stable artifact/API + low `yzx` path/config coupling + low duplicate-owner risk |

**Extraction counts only when this repo deletes or relinquishes a real owner.**

| Threshold | Needs |
| --- | --- |
| Big extraction | Independent users, stable API/artifact, release cadence, real owner deletion here |
| Edge trim | Child already owns a generic validated concept; deleting local glue must not leave a pure adapter between duplicate owners |

---

## Config layering

Packaged first, unless a surface opts into native replacement.

```text
~/.config/yazelix/
  config.toml              # optional sparse semantic overrides
  rio/config.toml          # complete native Rio config; seeded once with themes/
  zellij/config.kdl        # guarded scalar sidecar
  zellij/plugins.kdl       # extra plugins only
  starship.toml            # optional sparse prompt overrides
  nu/{env,config}.nu       # after packaged Nu
  helix/*                  # lazy; created on tab use
  yazi/{yazi.toml,theme.toml,package.toml,starship.toml,init.lua,keymap.toml,plugins/,flavors/}
```

[Configuration](docs/configuration.md#config-root) owns the config root and
state directory overrides.

| Surface | Layering |
| --- | --- |
| Root TOML | Packaged semantic defaults → sparse explicit user overrides |
| Rio | Packaged complete config and themes → one-time copy to native files; `NRIO_CONFIG_HOME` selects them; every field is user-owned except Nova's top-level `force-theme` projection |
| Nu | Packaged → optional host `mise activate nu` → optional user Nu |
| Starship | Packaged generated schema + `print-config --default` → Ratconfig discovery and bounded scalar editing → sparse user overrides over Nova's `character.format` marker → runtime-effective TOML |
| Helix | Packaged Yazelix default → read-only Ratconfig baseline/observed rows → recursive sparse user override → reserved `keys.normal.A-r` restoration; dynamic languages and Steel stay native-file surfaces |
| Yazi | Packaged TOML → recursive user tables + replacing scalars/arrays → managed opener/Git fetchers → runtime-only root-mode flavor projection; optional complete Starship config replacement |
| Zellij | Packaged → guarded scalar sidecar → runtime materialize under state dir |
| Host `~/.config/{helix,yazi,starship}` | Not loaded by default |

[Runtime Notes](docs/runtime-notes.md) owns the
[Zellij sidecar](docs/runtime-notes.md#zellij-sidecars) and
[Helix](docs/runtime-notes.md#helix) mechanisms.

### Git editor boundary

- `yzx-editor` resolves `editor.command`, disables the Helix bridge, waits for
  the executable, and restores Zellij's default background when it exits. It
  never calls `yzx-open` or a Zellij pane action.
- Config file actions and Git clients invoke `yzx-editor`; the config UI drops
  the session's editor snapshot so each action resolves the current setting.
- Managed sessions export `yzx-editor` through `EDITOR`, `VISUAL`, and
  `GIT_EDITOR`. `YZX_EDITOR` remains the effective editor for managed Yazi opens.
- `yzx-git` appends a LazyGit `os.edit*` overlay while retaining global and
  repository configuration.

---

## Startup boundary

Owned by `runtime/yzx/` (Nix substitutes paths; Rust owns wiring and `exec`).

1. `YAZELIX_STATE_DIR` + optional `YAZELIX_HELIX_BRIDGE_SESSION_ID` (when `yzx-hx`)
2. Effective `YZX_EDITOR` / `YAZELIX_EDITOR`; standard editor variables route through `yzx-editor`
3. Config home: `YAZELIX_CONFIG_HOME` → `XDG_CONFIG_HOME/yazelix` → `~/.config/yazelix`
4. Root settings → env and launch args (`YZX_OPEN_LOG`, welcome, Zellij theme mode, popup chords/custom KDL, bar tray)
5. Rio native config home
6. Zellij materialize (sidecar + patches) + status-bar cache path. Bundled
   plugins resolve through `zellij:` URLs.

Pre-`exec` failures → Yazelix diagnostics.  
After `exec` → Rio / Zellij / child tool.

`status` and `doctor` reuse configuration resolution and validation without
materializing runtime files or initializing Rio
(`DIAGNOSTICS-READ-ONLY-001`). The scalar Zellij helper returns merged text;
runtime preparation owns its final write. Existing launch/run commands retain
initialization. `doctor` reports missing runtime files and warns if managed
Helix TOML overrides reserved `Alt r`; `status --json` retains schema version 1.

---

## Session isolation

| Mechanism | Purpose |
| --- | --- |
| `YAZELIX_HELIX_BRIDGE_SESSION_ID` | Opaque per top-level `yzx` launch with bridge-enabled `yzx-hx` |
| `ZELLIJ_SESSION_NAME` | Compared when a bridge registry recorded it |
| `YAZELIX_ZELLIJ_SESSION_NAME` | Yazi saves real session here before blanking `ZELLIJ_SESSION_NAME` for image previews; open/reveal restore it for Zellij control |
| `ZELLIJ_PANE_ID` → live tab membership | `yzx-open` reuses only a Helix registry whose pane is in the same `tab_id` |
| Startup picker pane id → stable tab id | The pane orchestrator closes that exact picker pane after same-tab editor readiness, or its exact tab on cancellation |
| Helper-derived ids outside `yzx` | `yzx-hx` / `yzx-yazi` / `yzx-open` standalone must not hit a live window bridge |

Host editors (`nvim`, `/usr/bin/hx`, …) skip the Helix bridge entirely.

---

## Runtime contracts

**Index only:** behavior · owner · check · gap.  
Detail lives in Owners, checks, and the notes below.

`C9*` / `C11*` are splits of former mega-rows. Letter suffixes keep `C10` /
`C12` / `C13` stable. Checks may still share `checks/yzx-contracts.rs`.

### Front door and desktop

| ID | Contract | Owner | Check | Gap |
| --- | --- | --- | --- | --- |
| C1 | Front-door CLI, headless `enter`, and pre-exec diagnostics | `runtime/yzx/`, menu/tutor/config/open, screen, flake | launcher unit, `yzx-contracts`, `managed_session` (Linux + Darwin), helix/key parity, `nix build .#yazelix` | GUI launch |
| C8 | Desktop entry starts `yzx`; Rio app id matches `yzx.desktop` | `flake.nix`, `runtime/yzx/` | `yzx-contracts`, `nix build .#yazelix` | Desktop launch |

### Terminal, layout, shell, editor bridge

| ID | Contract | Owner | Check | Gap |
| --- | --- | --- | --- | --- |
| C2 | Complete Rio config is seeded once and selected through `NRIO_CONFIG_HOME`; Nova owns only top-level `force-theme`, using native reload when writable and coherent next-session fallback when read-only | `defaults/rio/config.toml`, runtime, `yzx-config` | launcher/config unit tests, `yzx-contracts` | Visual dogfood |
| C3 | One Radar controller with per-tab views across layout swaps | `defaults/zellij/config.kdl`, `defaults/zellij/layout*.kdl`, zj-radar | `zellij-layout`, `yzx-contracts`, isolated startup benchmark | Fresh-session state-retention dogfood |
| C4 | Packaged keys + guarded Zellij sidecar | `defaults/zellij/config.kdl`, `yzx-zellij-config` | `yzx-contracts` | Full keys |
| C5 | Managed Nu layering | `yzx-nu`, `defaults/nu/` | `yzx-contracts` | — |
| C6 | Managed Yazi layering, public noninteractive materialization, fast zoxide-first startup with exact picker lifecycle, `yzx-open`, and zoxide | `defaults/yazi/`, `runtime/yzx-yazi.rs`, `runtime/yzx/`, `crates/yzx-yazi-config/`, `crates/yzx-open/`, pane orchestrator | host-Yazi contracts + materialization + open tests + isolated startup-picker check | Picker/Yazi UI |
| C7 | Helix bridge window/tab isolation (`session` + `tab_id`) | `yzx-open`, flake | `yzx-open` tests | Multi-window |
| C10 | Top bar tray, home-marker tabs, home-scoped new tabs, usage `tu` + cache | layout, config, runtime, tokenusage | layout + contracts | Visual bar |
| C12 | Welcome defaults and random pool | Anima, runtime, root config | screen tests + contracts | Animation |

### Popups (`C9*`)

| ID | Contract | Owner | Check | Gap |
| --- | --- | --- | --- | --- |
| C9a | Kitty protocol + `yzpp` packaged/loaded | `defaults/zellij/config.kdl`, flake | `yzx-contracts` | Visual |
| C9b | Role popups + popups tab remaps + margins + refresh hooks | config, runtime, `yzx-config` | contracts + keybinding tests | Visual |
| C9c | Custom `[popups.<id>]` argv + unique titles | `yzx-config`, runtime | custom popup tests + contracts | Visual |
| C9d | Agent hide keep-alive + custom command or provider bootstrap + supported stock Radar setup | `yzx-agent`, config | `yzx-contracts` | Provider UX |
| C9e | Git LazyGit + editor env + close-on-toggle | config, runtime | `yzx-contracts` | Visual |

### Config UI (`C11*`)

| ID | Contract | Owner | Check | Gap |
| --- | --- | --- | --- | --- |
| C11a | Root semantic schema + sparse persistence | `yzx-config`, `defaults/config.toml` | config tests + contracts | UI |
| C11b | Popups; Rio exact-file action; owner-complete Starship/Yazi inventories; curated Zellij with exact-file fallback and session patching; Nushell selection plus Advanced file actions | Owner catalogs + Ratconfig + `yzx-config` | config tests + contracts | Session live scalars |
| C11c | Helix observed native rows + exact files + `yzx-hx` merge / `Alt r` / Steel | `yzx-config`, Helix, `yzx-hx` | `helix-contracts` + config tests | Owner catalog unavailable |
| C11d | Keys read-only + Advanced open-file | `yzx-config` | Keys/Advanced tests, key parity | UI |

### Install

| ID | Contract | Owner | Check | Gap |
| --- | --- | --- | --- | --- |
| C13 | Eight fixed Rio/managed-Helix/managed-Yazi package combinations + narrow Home Manager enable/package/optional files | `home-manager/`, flake, `yzx-config` | Rio/no-Helix/host-Yazi contracts + `checks.home_manager` | Full HM switch |

### Notes

**C1:** Bare `yzx` → help; `launch` is explicit. Menu is a curated allowlist,
`run` reuses the prepared environment, and reveal is tab-local. `enter` reaches
the managed Zellij/Yazi workspace and selected editor without starting Rio or using display
variables; terminal-specific graphics and clipboard behavior remain host-owned.
Diagnostics stop before Rio/Zellij handoff.

**C2:** See [Managed Appearance](docs/runtime-notes.md#managed-appearance).

**C9:** Protocol/packaging (a), shared role wiring (b), user custom (c),
agent hide + bootstrap (d), Git close-on-toggle + editor env (e). Agent
cwd-mismatch restart is owned by `yzpp` and consumed via flake pin.

**C11:** Root schema (a), native tabs + session Zellij active-file patch (b),
Helix merge/Steel/`Alt r` (c), Keys/Advanced (d).

---

## Tradeoffs

**Pros:** small public surface; concrete semantic config; one Nix-composed
runtime; Rio isolated; Rust where process/files matter; layout checks in build.

**Cons:** `flake.nix` is heavy; unreleased upstream Zellij main plus carried patches; Yazi
integration surface; user layering is intentionally incomplete.

**Current bet:** one owner per contract beats minimal file count. Nix is the
awkward but reproducible composition layer while children evolve.
