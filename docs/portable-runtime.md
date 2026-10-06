# Portable runtime contract

This is the implementation target for `yazelix-nix-free-installer-nx4-a48.1`.
Nova currently requires Nix; this document does not announce a portable release.
It covers one payload: managed Helix, managed Yazi, and no Rio. `yzx` remains
the public command, with the existing no-Rio command behavior.

Nix owns composition. Release tooling packages that composition; installers,
Homebrew formulas and AppImage wrappers place or select it. Those consumers
must not reconstruct Nova's tools, defaults or dependency choices.

## Evidence and decisions

The completed `.6` inventory and `.7` Linux experiment used source revision
`dd727ac201d00160870d7494b50cffbaa6862e5b`. The inventory's 632 store paths total
1,654,994,200 uncompressed NAR bytes. That is a conservative closure census,
not a download size or a certified minimum payload. Copying the joined output
alone loses its dependencies; no payload deletion has been approved.

The Linux experiment used the real native `yzx`, noninteractive Bash, Zellij
and OpenSSL, with a substituted Radar helper. It passed in two isolated roots,
including one with spaces, without `/nix/store` or the checkout. Direct host
ELF execution preserved `current_exe`; relative library search plus declared
private-file guards prevented missing OpenSSL from falling back to a host copy.
This proves a bootstrap slice, not a fresh Nova workspace or the full closure.

The experiment passed on glibc 2.39. The inventoried interactive Bash depends
on ncurses requiring `GLIBC_2.42`, so the unchanged payload cannot claim a floor
below 2.42. `.2` must measure and prove the complete artifact's requirements,
or rebuild dependencies for a lower floor. Interactive Bash remains in scope.

These results select direct host-loader execution and root-relative private
libraries for Linux. They reject naive invocation through a copied loader,
which made `current_exe` identify the loader, and the tested versioned
`DT_NEEDED` rewrites to `$ORIGIN`, which failed in glibc. Relative RPATH alone
does not enforce package ownership.

The `.7` prototype used
[nix-bundle-dir at `ffc410b7`](https://github.com/logos-co/nix-bundle-dir/tree/ffc410b7f2bab32b1c2ceb347c56902e7321049c).
It remains a build-time candidate, not an adopted dependency. Its default host
library exclusions and permissive store-string warnings do not establish this
contract. The [custom `nix bundle` interface](https://nix.dev/manual/nix/2.34/command-ref/new-cli/nix3-bundle.html)
is distinct from its default bundler. A build interface can remain useful;
target-side Nix or `/nix/store` virtualization is outside this contract.

## NPR-ROOT-001: root selection

The portable tree contains a real native executable at `R/bin/yzx`. The native
launcher owns root selection: obtain `current_exe`, canonicalize it, then take
the parent of `bin` as `R`. An external package-manager symlink may point to
that executable. Failure to resolve the executable or required package files
must stop the affected operation with a diagnostic and nonzero status.
Neither the working directory, `argv[0]` nor inherited PATH selects the root.

[Rust documents platform differences in executable and symlink discovery](https://doc.rust-lang.org/std/env/fn.current_exe.html).
Native platform proof must cover the canonicalization step. Moving a running
tree or migrating a live session between roots is outside this contract.

The native path owner also supplies the Nix delivery bindings. Current Nix
outputs join symlinks to separate store outputs, so their resolved executable
parent does not identify the joined package root. Packaging selects the
delivery mode explicitly; absent portable metadata must not trigger a store
fallback. `.8` must preserve existing Nix and Home Manager behavior.

The launcher passes its canonical portable root to managed children through
the internal `YAZELIX_RUNTIME_ROOT` value. An inherited value cannot retarget
the front door. Helpers and generated configuration consume the owner's
resolved bindings; they do not guess another root. Packaging supplies those
bindings from its existing composition, including script interpreters, managed
PATH, bar requests, plugin references and configuration templates. Component
translation stays at its existing boundary. This contract requires no new
general path registry, public root override or configuration schema.

## NPR-PAYLOAD-001: owned files and host boundary

| Location below `R` | Consumer |
| --- | --- |
| `bin/yzx` | Public native front door |
| `libexec/yazelix/` | Nova helpers and managed commands |
| `lib/` | Private native libraries, with component subdirectories where needed |
| `share/` | Component runtimes, plugins, grammars, themes and shell assets |
| `share/yazelix/` | Nova defaults, templates and artifact metadata |
| `share/licenses/` | Notices and source-retrieval information for the selected contents |

Additional existing command entrypoints use the same resolved bindings.
Internal tool layout follows the consuming component; packaging need not
flatten conflicting libraries or copy an embedded asset into another owner.
Zellij retains ownership of its embedded Nova Wasm plugins.

Carry `.6`'s managed tools and feature assets: Helix runtime and grammars,
Steel/Forest cogs, watcher library and modules, Yazi plugins and flavors,
Nushell/Bash/Zsh/Fish initialization, shell integrations, preview tools, Git,
Lazygit, tutor, anima and Nova helpers. Pruning a closure reference requires
evidence that the selected feature set still works.

For Linux, the host supplies the ELF interpreter and matching glibc family.
The initial mechanism uses `/lib64/ld-linux-x86-64.so.2` on x86_64, relative
ELF library search, and declared host `libgcc_s.so.1`. Packaging records exact
allowed system-library SONAMEs and checks their required symbol versions.
Other native dependencies, including OpenSSL, remain private to `R`; the
candidate bundler's wider host exclusion list is not Nova's policy.

Script helpers use host `/usr/bin/env` with a packaged interpreter selected by
the managed PATH, as `.7` tested for Bash. Declare `/usr/bin/env` as a host
prerequisite; check the packaged interpreter before script execution.

The host also supplies a capable terminal/PTY, writable user storage, Unix
sockets and loopback networking. `.2` must check the full payload's use of
Linux process information, DNS/NSS, trust roots, locale and timezone data,
then record any required host facilities. No minimum kernel or blanket
distribution support follows from `.7`.

Configured host editors, agents, LSPs, formatters, mise, custom popup commands
and the optional host integrations identified in `.6` remain explicit external
tools. Their absence produces a diagnostic when their feature is invoked.
Managed commands and required assets must not fall back to host PATH, an old
installation or the development checkout.

Packaging rejects escaping or dangling owned symlinks, store-bound script
interpreters, owned executable/config/plugin references outside `R`, and undeclared
dynamic dependencies. Relative symlink targets may contain `..` only when
their complete resolution stays inside `R`. Provenance strings containing
`/nix/store` are not execution dependencies; review actual references rather
than using a raw string count as proof.

The runtime checks required owned files before the affected child/feature can
use an external substitute, including private libraries reached by `dlopen`.
Missing files must identify the expected package path and fail. The kernel or
loader can reject the front door before Nova runs; the future installer owns
host preflight diagnostics. These checks do not promise protection against a
hostile process environment or concurrent mutation of the installed tree.

Darwin keeps the same root and ownership contract. `.3` owns native Mach-O
loading, system-library boundaries, minimum macOS version and signing/trust
proof. Linux loader flags and successful Linux probes establish none of those.

## NPR-METADATA-001: artifact identity

Nix packaging emits `share/yazelix/package.json`. Release and install tooling
consume these fields; they do not infer identity from a directory name.

| Field | Contract |
| --- | --- |
| `schema_version` | Integer `1`; reject an unsupported schema |
| `version` | Nova version from the package producer |
| `channel` | Explicit `edge`, `main` or `stable` selection |
| `revision` | Exact 40-character source Git revision |
| `target` | Initial target `x86_64-unknown-linux-gnu` |
| `variant` | `no-rio`, with managed Helix and Yazi |
| `entrypoints` | Root-relative paths, including `bin/yzx` |
| `compatibility` | Target-specific, measured host requirements |
| `file_index` | Path `share/yazelix/files.json` and its SHA-256 |

The Linux compatibility record names libc family `glibc`, minimum version,
interpreter path, host commands and allowed SONAMEs with symbol requirements.
`.2` supplies actual values after full-payload proof. Darwin requires a defined
record and native proof before publication. Consumers reject an unsupported
target or compatibility record; required constraint changes need a schema
version understood by those consumers.

The file index covers payload regular files and symlinks with unique normalized
root-relative paths. Regular-file records contain SHA-256 and permission mode;
symlink records contain the literal target. Verification checks both declared
entries and unexpected payload files. Only `share/yazelix/package.json` and
`share/yazelix/files.json` are excluded to avoid a digest cycle. `.4`'s immutable
archive digest covers both; installers verify it before trusting the index.
Packaging rejects absolute index paths or `..` path components, special filesystem
entries and setuid/setgid payload files.

The existing bar identity can remain a derived view of the same package
producer. Version, channel and revision must agree with the artifact manifest;
helpers must not maintain a second identity source. `.4` also carries `.6`'s
source/license inventory and resolves notice/source obligations for the exact
published contents. The inventory alone is not redistribution clearance.

## NPR-STATE-001: package and user ownership

Treat `R` as read-only runtime input. Keep user configuration, effective config,
state, cache and sockets outside it using existing directory overrides. Resolve
package inputs from the selected root when materializing effective config;
stale absolute paths from a prior installation must not survive a fresh start.
Packaged sources and Home Manager-owned sources remain read-only in Ratconfig.

Preserve user Steel files and the watcher's refusal to replace an unknown
native library. Across `R1` to `R2`, update a prior Nova-managed watcher link
only with evidence of its ownership; matching the current root alone is
insufficient. An unknown file or link must remain untouched with a diagnostic.
`.8` chooses the smallest mechanism that proves both cases.

## NPR-PROOF-001: acceptance boundaries

| Owner | Required proof |
| --- | --- |
| `.1` | Checked-in contract, evidence limits, consistent links/scorecard and clean diff |
| `.8` | One path owner across native/helpers/config; Nix behavior and user-file protection |
| `.2` | Full Linux archive, host floor and installed fresh-session proof without Nix/store |
| `.3` | Full native Darwin archive and its host/trust boundary |
| `.4` | Immutable asset identity, index, notices and source information |
| `.5` | Safe placement/selection and installation diagnostics |

The implementation and archive checks must exercise these concrete outcomes:

- Install the exact candidate under `/tmp/nova-a` and `/tmp/nova moved/b`, then
  invoke each from an unrelated directory and through an external symlink.
  Hide `/nix/store`, the checkout and the other root; use fresh user state and
  a host PATH without managed tools. Both runs must identify their selected
  artifact and complete the full anima-to-workspace handoff, managed Helix with
  Steel/watcher loading, managed Yazi, supported shells and preview flows.
- Remove a packaged interpreter, helper, config/asset or private library
  (including `dlopen` libraries). The affected operation must fail with the
  owned path even when a compatible host substitute is available.
- Add `libexec/yazelix/escape -> /nix/store/example/bin/tool` or a config plugin
  URL requiring that store path. Packaging must reject it. A contained relative
  symlink must pass; a dangling or indirectly escaping link must fail.
- Start from prior Nova-managed watcher state, then select a different root.
  Its managed link must update; a user-owned file or unknown link must remain
  unchanged and trigger the existing refusal behavior.

Record exact source/artifact identity, native environment, commands and results
with each proof. Build/evaluation and scripted probes do not replace manual
fresh-session dogfood. Shared implementation changes still require Linux CI
and Darwin Package Smoke on the exact revision under the repository gates.

This decision adds no installer, archive variant matrix, Rio/GUI bundle,
package manager, updater or general Nix closure relocator.
