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

The experiment passed on glibc 2.39. The original interactive Bash depended
on ncurses requiring `GLIBC_2.42`; copying that payload unchanged required 2.42.
The archive rebuilds the affected runtime providers for the lower baseline.
Interactive Bash remains in scope.

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
`yazelix-nova-<version>-x86_64-unknown-linux-gnu-no-rio.tar.gz`. Nix is a build requirement;
the extracted runtime does not use Nix. The output exists only on
`x86_64-linux` and preserves the managed no-Rio composition and its embedded
Zellij plugins. The archive carries package identity and a hashed file index.
Publication, complete licensing notices and an installer belong to `.4` and `.5`.
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
before dispatch. Required ELF symbol versions select the glibc floor. Assembly
rejects any requirement above 2.39; musl remains unsupported.

The archive uses the current coreutils, ncurses, libssh and librist recipes with
GCC's native sysroot set to pinned glibc 2.39 headers/startup/link inputs.
Those inputs are build-only; the host supplies libc at runtime. All other tool
versions and the ordinary Nix build remain unchanged. One archive composition
selects the rebuilt commands and shared-library providers; the existing bundler
applies those provider choices while preserving component directories. This
avoids rebuilding Nixpkgs' compiler and build-tool graph. Whole-payload ELF
checks and clean-system interaction checks protect that boundary.
Auxiliary tool and delegate collection preserves commands already selected by
the runtime composition.

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

The earlier glibc-2.42 payload measured 396,593,484 compressed bytes
(378.22 MiB) and 1,300,709,957 regular-file bytes
(1.21 GiB), across 6,937 regular files and 160 internal
symlinks. Filesystem allocation depends on the extraction target. Zellij's
byte-identical entrypoint copies use aliases, saving 140,411,480 uncompressed
bytes. The largest individual files are Carapace (70.6 MB), Zellij (70.2 MB),
Nushell (60.9 MB), Helix (44.0 MB) and Atuin (37.4 MB).

That payload's unprivileged, read-only installations passed the complete scripted
workspace probe in Fedora 43 (glibc 2.42) and Ubuntu 26.04 (glibc 2.43), without the Nix
store or development checkout. The installed root moved between paths with
spaces and an apostrophe; an external entrypoint symlink retained root ownership.
Linux 6.12.93 has a separate VM probe; older kernels remain unverified. Exact-commit Linux/Darwin
gates and manual fresh-session dogfood remain acceptance requirements.

The glibc-2.39 development payload measures 396,646,946 compressed bytes and
1,301,148,493 regular-file bytes, with 6,937 files and 160 internal symlinks.
All 729 ELF files require at most glibc 2.39; private library search is root-relative.
Unprivileged, read-only roots pass the complete scripted workspace probe in
Ubuntu 24.04/glibc 2.39, Fedora 43/glibc 2.42 and Ubuntu 26.04/glibc 2.43,
without Nix, a mounted store or the checkout. This is userspace proof on the
host's Linux 7.1.5 kernel; the earlier 6.12.93 VM result covers the older payload.

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

The Linux archive factory owns the interpreter, allowed host SONAMEs and
libgcc symbol-version list. Its relocator matches whole host SONAMEs and rejects
undeclared libgcc requirements. The metadata records glibc 2.39, host Git and
`/usr/bin/env` with argument splitting. Python is a build/maintainer tool only.

Version and channel come from the existing packaged runtime identity. A clean
Git flake supplies the source revision; dirty and `path:` development builds
record `revision: null`. Such builds remain useful development artifacts but
fail release verification. A base commit must not identify modified source.

The file index covers payload regular files and symlinks with unique normalized
root-relative paths. Regular-file records contain SHA-256 and permission mode;
symlink records contain the literal target. Verification checks both declared
entries and unexpected payload files. Only `share/yazelix/package.json` and
`share/yazelix/files.json` are excluded to avoid a digest cycle. `.4`'s immutable
archive digest covers both; installers verify it before trusting the index.
Packaging rejects absolute index paths or `..` path components, special filesystem
entries and setuid/setgid payload files.

`files.json` uses schema version 1 and a `files` object keyed by path. Each record
has `type: file` with a SHA-256 and integer Unix permission mode, or
`type: symlink` with its literal `target`. Directories are not file records.
The factory preserves existing file modes and adds only the two metadata files.
Extraction must retain indexed modes; a restrictive umask or an extractor that
adds write permissions changes the indexed tree. For GNU tar, extract into an
existing directory and run the maintainer verifier:

```sh
tar --no-same-owner --same-permissions -xzf /path/to/archive.tar.gz -C /path/to/root
python3 -B packaging/archive_metadata.py /path/to/root --release
```

The verifier checks identity agreement, the index digest, integer schema versions
and permission modes, file bytes, literal symlink targets, missing files,
unexpected files and unsafe entries.
Unreadable directories fail the inventory; an incomplete scan cannot pass
verification or produce package metadata.
Omit `--release` when checking an uncommitted development build.

The existing bar identity can remain a derived view of the same package
producer. Version, channel and revision must agree with the artifact manifest;
helpers must not maintain a second identity source. `.4` also carries `.6`'s
source/license inventory and resolves notice/source obligations for the exact
published contents. The inventory alone is not redistribution clearance.

The maintainer helper `.github/scripts/grammar_sources.py` creates a grammar
source bundle for this review. It consumes the recursive Nix derivation graph,
the exact archive derivation, that candidate's pinned Helix source, and its Nova
revision. Only reachable grammar derivations are selected; missing sources,
unreadable directories, stale pins and duplicate grammar identities fail before
output.
Shared source trees are stored once. `SOURCE-MANIFEST.json` records each grammar's
origin and derivation, hashes every regular source file, and retains link targets.
The bundle preserves original notices, headers, README and package declarations.
Links to selected content are mapped to its archive placement, retaining their
original targets in the manifest. This also handles links that leave and re-enter
the original source. Foreign and dangling links are inactive metadata, so they
cannot expose the extraction host.
The archive is reproducible and an existing destination is never replaced.
Its `redistribution_review: required` field keeps it distinct from cleared release
notices or a complete corresponding-source distribution. Native/static dependency
coverage and the applicable source-delivery obligations still require review.

The offline helper `.github/scripts/runtime_notices.py` preserves Rust and Go
notice material from the same recursive graph and archive derivation:

```nu
let archive_drv = (nix eval --raw .#yazelix-no-rio-archive.drvPath)
nix derivation show --recursive $archive_drv | save --force archive-build-graph.json
python3 -B .github/scripts/runtime_notices.py archive-build-graph.json $archive_drv language-runtime-notices.tar.gz
```

Its pinned input archives must already exist in the Nix store. Rust standard
library packages supply their top-level notices; the matching Linux compiler
archive supplies `COPYRIGHT-library.html` and license texts. Go's source archive
supplies its original notices, including vendored notices and `PATENTS`.
The packet records provider derivations, source-archive hashes and file hashes.
Missing inputs or required notices, a Go source/version mismatch and unsafe
notice entries prevent output. Generated permissions are fixed so the packet
reproduces across umasks; existing output is never replaced.

These are reachable build-graph candidates, including build-time candidates;
they are not a census of shipped components. The packet retains
`redistribution_review: required`. It does not replace native/static/embedded
source review, applicable corresponding source, build scripts or relink material.
Source delivery must cover the exact selected programs and libraries, Nova's
packaging changes, pinned recipes, patches and vendor inputs. A successful
library replacement probe alone cannot establish complete source delivery.

Hosted capture uses `--realise` to realise and GC-root those selected original
notice archives before reading them, rebuilding missing inputs through the
recorded graph's producing derivations. Offline capture leaves the store unchanged.

## NPR-SOURCES-001: component origins and source capture

The Linux archive records its assembly selection in
`share/yazelix/archive-components.json`: command packages, copied data and
embedded plugins, plus complete upstream and Nova recipe source roots.
The exact Nova source root belongs to the final archive derivation, so source
delivery edits do not invalidate the native payload's cached relocation.
The declaration reuses the assembly's command list. Preview/delegate tools,
font inputs and relocated private libraries add their selected store origins to
`share/yazelix/archive-origins.txt`. Both records use store basenames and are
covered by the package's file index.

The maintainer source helper consumes these records and the exact archive's
recursive derivation graph. It resolves producing derivations, follows selected
`buildInputs`/`propagatedBuildInputs`, preserves complete `src`/`srcs`, patches,
Cargo vendor trees and Go modules, and follows source references in generated
composition recipes. It excludes unrelated and native build-tool derivations.
The graph remains in the source manifest as recipe evidence. Embedded Zellij
plugins are declared explicitly, including zjstatus and the Nova Bar widget;
full upstream source roots supplement filtered compile inputs.

After extracting the archive with GNU tar preserving its permissions:

```sh
let archive_drv = (nix eval --raw .#yazelix-no-rio-archive.drvPath)
nix derivation show --recursive $archive_drv | save --force archive-build-graph.json
python3 -B .github/scripts/runtime_sources.py archive-build-graph.json $archive_drv /path/to/installed /path/to/new-sources.tar.gz --release
```

The helper verifies the installed file index and revision agreement with the
build graph. `--release` requires a clean immutable source revision; omit it only
for development preparation. Input realization and indirect GC-root creation
finish before capture begins, including linked Nix-store vendor sources.
Generated inputs use their producing derivation and selected output from the
recorded graph, so missing custom vendor trees can build without a substitute.
Roots use the output filename plus `.inputs-<batch>`;
retain them while that source artifact needs verification. Existing output is
never replaced. The shared bundler verifies every archived regular file and
selected link against its manifest, then finalizes the destination atomically.
Hard-linked inputs, such as files deduplicated by Nix store optimisation, are
archived as full regular-file copies.
Changed inputs, incomplete capture and checksum failures leave no final archive.

The source manifest retains `redistribution_review: required`. Origin selection
is not license clearance: applicable notices, static/embedded coverage and
source/relink obligations must be reviewed for the exact published contents.
The separate Rust/Go notice helper remains a conservative review supplement.

## Release asset preparation

The offline preparer consumes an installed candidate, its original runtime
archive, the complete component source kit and its language-runtime notice
packet. It prepares review files without publishing:

```nu
python3 -B .github/scripts/release_assets.py /path/to/installed /path/to/runtime.tar.gz /path/to/component-sources.tar.gz /path/to/language-runtime-notices.tar.gz nova-v1.3.0 /path/to/new-assets
```

The tag must match the packaged version. Dirty candidates fail. The preparer
copies inputs into a private temporary directory before verification, compares
every runtime archive member with the verified installation, and verifies the
complete source and notice packets. Their package identity, archive derivation
and language-runtime providers must agree. Existing destinations are refused.

`nova-release.json` schema 1 maps the supported target to its archive name,
size, SHA-256, intended immutable download URL, variant, host compatibility,
entrypoints and file index. The source record lists ordered 1,900-MiB parts,
individual hashes, and the reconstructed archive's size and hash. Parts preserve
the complete source archive and stay below GitHub's strict 2-GiB asset limit.
The directory also contains `SHA256SUMS`, the notice packet and
`SOURCE-DELIVERY.md`. Temporary preparation needs space for both whole source
inputs and their delivery parts. A prepared URL does not imply publication.

Redistribution review remains pending in the manifest; GitHub owns release exposure.
Preparation supplies no GitHub OIDC provenance or immutable-release guarantee.
`SOURCE-DELIVERY.md` gives the exact-revision Nix archive build command and maps
the preserved Nova recipes, component materials and build flags. It describes
testing a dynamic replacement in a private runtime copy; static and embedded
changes require rebuilding their consuming program. Nix is a build requirement,
not a portable-runtime requirement. These instructions do not claim an offline
source rebuild or complete replacement/relink clearance.
Native/static/embedded notices, corresponding-source/build instructions and
applicable replacement/relink obligations remain publication gates. The current
399,098,483-byte runtime and 3,358,139,438-byte source kit provide a review
baseline; growth above 10% in either requires explanation before approval.

## NOVA-RELEASE-ASSETS-001: reviewed tag and publication

`Portable Release` prepares the existing Linux no-Rio flake artifact, installs
those exact bytes and runs `checks/linux-archive.sh` in an unprivileged, read-only
Ubuntu 24.04 container without Nix or the checkout. It captures complete sources
and notices through their existing owners. Manual dispatch on Edge/Main/Stable
is a dry run: it uploads `portable-assets` and `portable-provenance` for one day
and creates no release. Download review assets before that retention expires.

An annotated `nova-v<version>` tag records approval as a JSON tag message:

```json
{
  "contract": "NOVA-RELEASE-ASSETS-001",
  "revision": "<full reviewed commit SHA>",
  "assets": {
    "runtime": "<targets.x86_64-unknown-linux-gnu.archive.sha256>",
    "sources": "<sources.sha256, covering the complete reconstructed archive>",
    "language_runtime_notices": "<language_runtime_notices.sha256>"
  },
  "redistribution_review": "required",
  "installed_artifact_review": "required",
  "review": ""
}
```

Replace the hashes from that revision's `nova-release.json`. Set both reviews
to `approved` only after reviewing the exact files. The review text records the
native/static/embedded notice decision, applicable corresponding-source/build
instructions and replacement/relink evidence, installed proof and any explained
size growth. This template deliberately fails approval. Source manifests retain
their original capture state; `RELEASE-APPROVAL.json` records the later decision
and annotated tag identity. Original license scopes remain authoritative.
The approved review text appears in `SOURCE-DELIVERY.md` and the release body.

Push the authorized annotated tag only after the exact revision belongs to Main
and its Linux CI, both cache publication jobs and Version Gate succeed. A tag
push with this approval triggers publication; choosing this approval format
does not authorize a tag, public release, channel promotion or repository
settings change. The workflow's `classify` job skips plain Stable version tags,
whose annotation is not this JSON; an annotation that names the contract always
goes through the fail-closed preflight.

The workflow has four bounded jobs: tag classification (5 minutes), read-only
preparation (120 minutes), OIDC attestation (20 minutes), and tag-only
publication (45 minutes). GitHub scopes
permissions to jobs, so build processes receive neither release-write nor OIDC
credentials. There are no fork, pull-request, schedule or variant-matrix builds,
new caches, or long-lived provenance signing keys. Global concurrency serializes
publication. The pinned GitHub attestation action signs every asset, including
the manifest, approval and checksums; a separate Sigstore bundle accompanies
them. The bundle is verified cryptographically rather than included in its own
attested checksum inventory.

Publication requires GitHub's immutable-release setting, the annotated approval
and all exact-revision gates. The setting's API requires Administration read
access, which the ordinary workflow token lacks. A private GitHub App installed
only on `Yazelix/nova` supplies Administration read and implicit Metadata read;
it cannot change the setting or write releases. Store its client ID in the
`NOVA_RELEASE_GUARD_CLIENT_ID` Actions variable and its private key in the
`NOVA_RELEASE_GUARD_PRIVATE_KEY` Actions secret.

Only the publication job receives the key. The pinned
`actions/create-github-app-token` action requests a Nova-only token with explicit
Administration read access. Tokens expire after one hour and the action revokes
them when the job finishes. Preparation and attestation use the ordinary workflow
token. Publication checks the immutable setting before creating a draft and
again immediately before publishing; missing credentials, API failures or a
disabled setting stop it. Enabling immutable releases remains a separate
maintainer action before the first portable publication.

The publisher creates a draft, uploads all assets without replacement, checks
their server-reported sizes and SHA-256 digests, repeats approval/gate checks,
then publishes and requires GitHub to report an immutable release. Existing
releases, including interrupted drafts, are refused; inspect an interrupted
draft explicitly instead of retrying an overwrite. Publication does not change
the Latest designation or any Git channel. Public URLs are downloaded again,
checksums and workflow/ref/revision-bound provenance are verified, and the
downloaded installation runs the same fresh no-Nix proof. A failure after
publication is reported; automation never deletes or rewrites a release.

Source capture is not a generated SPDX/CycloneDX SBOM. No existing generator
owns that format here, so this slice adds no SBOM framework. The source manifest,
component origins and build graph remain inspectable coverage evidence.

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
