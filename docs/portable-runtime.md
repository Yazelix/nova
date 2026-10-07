# Portable runtime contract

This is the implementation target for `yazelix-nix-free-installer-nx4-a48.1`.
Published Nova installations currently require Nix; this document does not
announce a portable release.
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
alone loses its dependencies. The portable archive delegates Git to the host;
the existing Nix delivery retains packaged Git.

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

## Linux archive factory

`nix build .#yazelix-no-rio-archive` produces the development Edge archive
`yazelix-nova-<version>-x86_64-linux.tar.gz`. Nix is a build requirement;
the extracted runtime does not use Nix. The output exists only on
`x86_64-linux` and preserves the managed no-Rio composition and its embedded
Zellij plugins. Publication, indexed metadata, licensing notices and an
installer belong to `.4` and `.5`.
The archive is a build output and has no executable flake app entry.

The factory uses existing `patchelf` and `readelf` tools with a narrow
Nova packaging script. Each private library retains its original component
directory, so two incompatible libraries with the same SONAME remain distinct.
ELFs use the host interpreter and root-relative RPATH. Static executables
retain their native execution path. The archive normalizes entry order,
timestamps, ownership and gzip headers.

Packaged data bindings cover terminfo, file signatures,
ImageMagick delegates, fontconfig/fonts, Fish and TLS trust/configuration.
Git is a host prerequisite; its helper and Perl/Python/Gettext stack is excluded.
A generated owned-input list guards private libraries and script interpreters
before dispatch. Required ELF symbol versions select the glibc floor, currently
2.42; this payload does not support glibc 2.39 or musl.

`checks/linux-archive.sh` runs inside a clean Linux environment with tmux and
coreutils and host Git, against an extracted root. It rejects a mounted Nix store,
checks managed tools, host Git configuration and hooks, Lazygit, previews and HTTPS, and drives
a fresh `enter` session through the startup picker, Helix and a managed shell.
It also opens and exits managed Yazi, returning to the same shell.
The shared `runtime-root` check covers moved roots and missing/escaping inputs.
Container checks prove userspace compatibility; a separate VM supplies kernel
evidence. Neither is manual dogfood or a public Linux support announcement.

Stock ImageMagick's default `label:` font lookup also fails in a clean Fedora
image with only its minimal font set. Explicit DejaVu text rendering and SVG
font selection exercise the packaged fonts without changing that child policy.

The 2026-10-07 development payload measured 396,593,484 compressed bytes
(378.22 MiB) and 1,300,709,957 regular-file bytes
(1.21 GiB), across 6,937 regular files and 160 internal
symlinks. Filesystem allocation depends on the extraction target. Zellij's
byte-identical entrypoint copies use aliases, saving 140,411,480 uncompressed
bytes. The largest individual files are Carapace (70.6 MB), Zellij (70.2 MB),
Nushell (60.9 MB), Helix (44.0 MB) and Atuin (37.4 MB).

Unprivileged, read-only installations passed the complete scripted workspace
probe in Fedora 43 (glibc 2.42) and Ubuntu 26.04 (glibc 2.43), without the Nix
store or development checkout. The installed root moved between paths with
spaces and an apostrophe; an external entrypoint symlink retained root ownership.
Linux 6.12.93 has a separate VM probe; older kernels remain unverified. Exact-commit Linux/Darwin
gates and manual fresh-session dogfood remain acceptance requirements.

A single-core VM exposed a startup race: Yazi read a newly created Zellij PTY
at 0×0 and exited before the resize arrived. The existing managed-Yazi launcher
uses packaged `stty` to wait for both dimensions to become positive before
starting its child. Nonterminal probes keep their existing behavior; a pane
that never becomes ready fails after a bounded wait. No Zellij patch is required.

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
fallback. Nix and Home Manager retain their pinned delivery bindings.

The launcher passes its canonical portable root to managed children through
the internal `YAZELIX_RUNTIME_ROOT` value. An inherited value cannot retarget
the front door. Helpers and generated configuration consume the owner's
resolved bindings; they do not guess another root. Packaging supplies those
bindings from its existing composition, including script interpreters, managed
PATH, bar requests, plugin references and configuration templates. Component
translation stays at its existing boundary. This contract requires no new
general path registry, public root override or configuration schema.
The shared child-launch boundary applies the managed PATH to portable children,
including internal scripts used by diagnostics, before their interpreter runs.

`runtime/yzx/package.rs` owns executable-root discovery, contained-path checks
and managed PATH. Native helpers share that module; shell wrappers call its
private `yzx-package` adapter. Required managed commands must be executable
files. Root-relative Zellij, bar and Nu templates materialize into user state
on each fresh invocation. Yazi and LazyGit consume their resolved opener/editor
environment, and Helix receives explicit runtime, Steel and bridge bindings.
Agent identity markers match `/yzx-agent` in either delivery layout.
Watcher updates stage their ownership record before replacing the managed link;
failed staging preserves the existing link and record for retry.
Portable tutor examples use Nushell external-command quoting for their paths.
The bar adapter runs portable widget commands through the managed shell, which
expands the propagated root after Zjstatus parses the command. Root characters
therefore do not become command syntax; Nix widget commands remain unchanged.

The internal Nix `runtimeRootFixture` compiles the front door with
`--cfg yzx_portable` and installs a real `bin/yzx`. It exercises the root
contract while retaining native Nix dependencies. Its env bootstrap uses the
Nix-provided `env` because `/usr/bin/env` is absent inside the Nix sandbox;
managed PATH selects the fixture's packaged Bash/sh. `.2` and `.3` own native
loader relocation and the final host interpreter boundary. This fixture is
not a published archive or an installation method.

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
Nushell/Bash/Zsh/Fish initialization, shell integrations, preview tools,
Lazygit, tutor, anima and Nova helpers. Pruning a closure reference requires
evidence that the selected feature set still works.

For Linux, the host supplies the ELF interpreter and matching glibc family.
The initial mechanism uses `/lib64/ld-linux-x86-64.so.2` on x86_64, relative
ELF library search, and declared host `libgcc_s.so.1`. Packaging records exact
allowed system-library SONAMEs and checks their required symbol versions.
The measured libgcc requirements span `GCC_3.0` through `GCC_4.3.0`, including
`GCC_3.3`, `GCC_3.3.1`, `GCC_3.4`, `GCC_4.0.0` and `GCC_4.2.0`.
Other native dependencies, including OpenSSL, remain private to `R`; the
candidate bundler's wider host exclusion list is not Nova's policy.

Script helpers use host `/usr/bin/env` with a packaged interpreter selected by
the managed PATH, as `.7` tested for Bash. Declare `/usr/bin/env` with `-S`
argument-splitting support as a host prerequisite; check the packaged
interpreter before script execution.
The portable archive requires host Git on inherited PATH. Packaged Lazygit,
Yazi Git indicators and shell prompts use it with the user's Git configuration,
helpers and hooks. Nova does not set `GIT_EXEC_PATH`, `GIT_TEMPLATE_DIR` or
`GIT_SSL_CAINFO`. `yzx doctor` diagnoses missing Git; Git-dependent features
require it, while help and identity remain available. The existing Nix delivery
retains packaged Git. Interactive shells and Nova script interpreters remain
packaged in both deliveries.

The host also supplies a capable terminal/PTY, writable user storage, Unix
sockets and loopback networking. `.2` must check the full payload's use of
Linux process information, DNS/NSS, trust roots, locale and timezone data,
then record any required host facilities. No minimum kernel or blanket
distribution support follows from `.7`.
Clean-system probes cover `/proc` process information, libc DNS/NSS lookup,
private TLS trust, the C locale and host timezone data for UTC and
`America/Sao_Paulo`. Host resolver/NSS configuration, user records and timezone
data remain system inputs; this archive does not supply a replacement host OS.

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
The private package helper records the target when it creates a watcher link.
It replaces a subsequent link only when that recorded target matches, or when
the existing link matches the prior Nix-owned watcher pattern. Regular files
and unknown links are refused. Ownership records and link updates live in
Steel's user state, outside the package root.

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
