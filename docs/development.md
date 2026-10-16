# Development

## Check map

Run the focused checks for each owner path you changed. Fast checks build no
Nova package; focused checks build the package they test. A named check runs
as `nix build .#checks.x86_64-linux.<name> --no-link`; on Apple Silicon use
`aarch64-darwin`, except for checks marked Linux.

| Owner path | Fast | Focused |
| --- | --- | --- |
| `runtime/yzx/`, `runtime/yzx-agent.rs`, `runtime/yzx-package.rs` | `yzx_launcher_unit` | `runtime_root`, `managed_session`, `contracts` |
| `runtime/yzx-yazi.rs` | — | `yzx_yazi_materialization` |
| `runtime/yzx-menu.rs`, `runtime/yzx-nu.rs`, `runtime/yzx-shell.sh`, `runtime/yzx-env-supervisor.sh` | — | `contracts`, `runtime_root` |
| `runtime/yzx-zellij-config.rs` | `zellij_sidecar_guard_parity` | `contracts` |
| `runtime/yzx-helix.sh`, `runtime/yzx-helix-init.scm`, `defaults/helix/` | — | `helix_contracts`, `helix_file_watcher_runtime` (Linux) |
| `crates/yzx-config/`, `defaults/config.toml` | — (tests need Nix-provided Starship inputs) | `nix build .#yazelix` runs its tests; `key_reference_parity`, `contracts` |
| `crates/yzx-open/` | crate tests | `contracts` |
| `crates/yzx-tutor/` | crate tests | `key_reference_parity` |
| `crates/yzx-yazi-config/`, `defaults/yazi/` | crate tests | `yzx_yazi_materialization`, `contracts`, `startup_picker_cancellation` (Linux) |
| `crates/zellij-distribution/` | `zellij_theme_inventory_parity` | `runtime_root`, `managed_session` |
| `defaults/zellij/` | — | `contracts`, `managed_session`, `zjstatus_native_tabs`, `bottom_hints`, `attached_tab_focus`, `pane_move_ordering` (last three Linux) |
| `defaults/rio/` | — | `rio_contracts` |
| `defaults/nu/` | — | `contracts`, `runtime_root` |
| `checks/` | `cargo test --locked --manifest-path checks/Cargo.toml` and `cargo fmt --manifest-path checks/Cargo.toml --check` | the changed check |
| `flake.nix` (package variants) | `nix flake check --no-build` | `desktop_channels`, `no_rio_contracts`, `no_helix_contracts`, `host_yazi_contracts` |
| `home-manager/` | — | `home_manager` |
| `packaging/` | — | `archive_glibc_baseline` (Linux); release assets are proven by Portable Release |
| `.github/scripts/` | `python3 -B -m unittest discover -s .github/scripts -p 'test_*.py'` | — |
| `.github/workflows/` | no local check | the dispatched workflow itself |

Crate tests run with `cargo test --locked --manifest-path crates/<crate>/Cargo.toml`.
The full tier is `nix flake check`, `nix flake show --all-systems`, and
`nix build .#<package> --no-link` for each affected package variant. Shared
changes still need the [platform gates](../AGENTS.md#linux-and-darwin-gates).

## CI

Normal CI runs Linux checks and the Darwin no-Helix evaluation guard on
development-branch pushes (including `edge`), pull requests, and manual dispatch.
Pushes to promotion-only `main` and `stable` reuse the exact commit's successful
`linux` check from `edge`; they do not start another CI run. Required checks and
their trusted GitHub Actions source remain enforced. A different SHA or a failed,
cancelled, or skipped check cannot supply the required proof.
Manual dispatch remains available when proof is missing or a rerun is justified.
For example, to check the current Main revision:

```sh
gh workflow run ci.yml --repo Yazelix/nova --ref main
```

Verify the run's exact head SHA before using its result for promotion.
The Home Manager evaluation guard overrides nixpkgs with the reviewed revision
in `checks/newer-nixpkgs.txt` on Linux and Darwin, exercising the consumer
`follows` path with the existing configuration and package-override cases.
Ordinary CI evaluates this override; heavy compatibility builds run in Version Gate.
The focused package-contract check protects the installed `yzx` namespace,
public help, managed command discovery, and child argument/output/exit status.
`Publish Nix Cache` publishes Linux and Darwin capability variants, the Main and
Edge Linux full-package launcher outputs, and representative Home Manager closures
from `main` and manual dispatch. Both jobs push the requested output closures
synchronously, including substituted outputs. Missing credentials or failed
uploads fail the job. `Version Gate` is manual. `linux-profile-smoke` installs all
four profile shapes and runs release contracts and installed commands.
`linux-compatibility` builds the complete Home Manager check with the reviewed
nixpkgs override in a separate job, giving this independent dependency graph its
own timeout. Both jobs are required. `Darwin Package Smoke` owns the
locked-input Darwin packages, Home Manager, Rio, no-Helix, and host-Yazi checks.
Version Gate's `darwin-smoke-proof` verifies that completed native work for the
exact candidate before starting any expensive release job. Missing proof fails
immediately; it does not wait for or dispatch another build. Complete the smoke
workflow on the candidate's channel first, then dispatch Version Gate:

```sh
gh workflow run darwin_package_smoke.yml --repo Yazelix/nova --ref edge
gh workflow run version_gate.yml --repo Yazelix/nova --ref edge
```

Run these separately: verify the first run's exact head SHA and successful native
steps before dispatching the second. Keep the channel at that candidate throughout.
Long native checks follow the repository's recorded handoff procedure.
The proof verifier uses the canonical repository and workflow identity/path,
all paginated exact-SHA runs without filtering failures, and the most recently
updated run's current attempt. An older run's later rerun can supersede a newer
run. Equal latest update times or incomplete API results fail closed. The native
macOS job and both build steps must succeed; a green idle scheduled run with
skipped builds is invalid. A final refresh rejects a run changed during inspection.
Logs and the job summary identify the accepted run URL, SHA, attempt and steps.
Only this proof job adds `actions: read`; API errors retain GitHub's diagnostic,
and 60-second request deadlines fail the check. Ordinary CI runs the verifier's
standard-library contract tests.
Three dependent Darwin jobs exercise the same reviewed override:

1. `darwin-core` builds `yazelix-no-rio-no-helix-no-yazi` and publishes its closure.
2. `darwin-editor` restores the core closure, builds `yazelix-no-rio`, and publishes it.
3. `darwin-integration` restores both closures and builds the complete
   `checks.aarch64-darwin.home_manager`, including Rio and all configuration and
   installed-command assertions. It does not publish another closure.

Each stage checks out the same candidate SHA and reads `checks/newer-nixpkgs.txt`.
Prerequisite paths come from successful job outputs. `nix-store --realise` restores
those exact closures through the configured caches, including Cachix and
`cache.nixos.org`; Cachix omits paths already available upstream. Local and remote
builds are disabled during restoration. Missing paths or failed restoration,
build, or publication fail the stage. Rerun a failed job after
addressing its failure; completed prerequisites remain in the shared cache.

Cache writes use the existing `CACHIX_AUTH_TOKEN` only in the two prerequisite
jobs. Stages reject dispatches outside the canonical repository's `edge`, `main`,
and `stable` branches before checkout or credential use. Missing publication
credentials fail explicitly; fork and tag dispatches cannot pass this gate.
There are no compatibility push/PR triggers, store archives, retention pins,
or additional cache services. Only the two requested runtime closures are
published; common store paths are deduplicated under the existing cache policy.
Logs include exact paths, closure sizes, elapsed build time and runner disk use.
Closure size is an upper bound on added cache storage, not measured cache growth.

Version Gate permits one active and one pending run. Five native jobs have
90-minute limits; the proof job and `release-gate` each have five minutes, bounding
configured runner time at 460 minutes per invocation. The separate smoke producer
retains its 90-minute cap and can supply proof across channel promotions of the
same SHA. These are configured bounds, not measured savings. Linux keeps two Nix
build jobs per runner, Darwin one, with two cores each. Stage limits retain their
90-minute caps until native timings support tighter bounds. The aggregate requires
all six prerequisites to succeed, even
after a dependency fails; failed, cancelled, or skipped work cannot satisfy it.
Manual workflow checks do not appear in GitHub's protection status summary.
The aggregate publishes its result as the `release-gate` commit status for the
exact candidate. Only this canonical-repository job has `statuses: write`;
failed publication fails the gate. Branch protection keeps requiring that status.
Review the comparison pin deliberately and verify all native stages when
changing it. This proves the reviewed pin, not every future nixpkgs revision.
Retain compatibility failures and measured limits before expanding the CI budget.
`Darwin Package Smoke` runs locked-input Darwin verification weekly on Monday when
`main` has commits in the last 7 days, and on manual dispatch always, while
idle weeks skip the macOS build. The native producer asserts that Darwin packages
contain no Linux desktop entry. The flake advertises the optional Yazelix
Cachix cache, while source builds remain valid without it. Use Version Gate
before publishing a release.

Linux CI and Darwin Package Smoke build `helix_grammar_sources`, Nova Helix's
`HELIX-GRAMMAR-SOURCES-001` check. It rejects Codeberg and SourceHut fetches during
Nix evaluation and compiles their pinned grammars from bundled, hash-verified
sources. Grammar revisions and snapshot maintenance belong to Nova Helix.
The Linux-only `startup_picker_cancellation` check reuses Anima's pinned Kinestra
input to exercise empty-history Yazi, automatic and reopened search, startup
Tab/Z/Shift-Tab search and Spot keys, ignored startup Alt-Enter, successful
handoff, exact later-tab cancellation, and sole-tab session exit on a private X
display. Shared acceptance still requires Darwin Package Smoke on the exact
revision; macOS interaction remains unverified.
Physical keys use the recorder; the final session-exit key avoids its live-app
health check.
The shared managed-Yazi keymap gives the persistent popup one Tab/Z search and
both Shift-Tab encodings for Spot.
The Linux-only `pane_move_ordering` check uses the same private display to verify
circular work-pane movement, rapid opposite moves and fixed UI panes through
packaged Rio and Zellij.

Autonomous GUI verification uses the pinned Kinestra recorder's private display,
including installed-launcher probes. Launch applications and send input through
the recorder so desktop focus and the user's typing cannot affect the test.
Do not fall back to host `gtk-launch`, desktop focus changes, or host keyboard
automation when isolation fails. Visible desktop testing requires an explicit
user request for that interaction.

The Linux-only `helix_file_watcher_runtime` check opens a fresh managed
`yzx run hx` in a private X11 terminal with isolated config, Steel state and
tmux socket. It observes an initial external write before replacing the file,
then requires both the atomic replacement and a later write to appear in Helix.
This checks default watcher activation and reloads; Rio and picker startup have
separate integration checks. Failures retain the last pane capture and screenshot.

```sh
nix build .#checks.x86_64-linux.helix_file_watcher_runtime --no-link --print-build-logs
```

## Edge, main, and stable

All development commits land on `edge`, including fixes, reverts,
documentation, and Beads updates. CI runs there, and users who select `edge`
accept the active experimental dogfood channel.

Keep `stable ⊆ main ⊆ edge` on one linear history. A promotion includes every
accumulated commit up to the candidate, including changes unrelated to the fix
that motivated it. Review that whole range. If intervening work is unaccepted,
accept it, revert it on `edge`, or defer promotion. Do not cherry-pick fixes,
commit directly to the promotion channels, or skip a channel.

`main` is promotion-only accepted development. After an `edge` revision is
accepted and verified, advance `main` to that exact revision without merging or
cherry-picking. The successful exact-SHA Edge CI check satisfies Main's required
`linux` check; cache publishing runs on `main`. Users who select Main accept more
frequent updates than `stable`:

```sh
git fetch origin edge main
git merge-base --is-ancestor origin/main <sha>
git merge-base --is-ancestor <sha> origin/edge
git log --oneline origin/main..<sha>
git diff origin/main <sha>
git push origin <sha>:main
```

The protected `stable` branch accepts fast-forward promotions from `main`. Its
required checks are `linux`, `publish_x86_64_linux`,
`publish_aarch64_darwin`, and `release-gate`, sourced from GitHub Actions.
`main` requires `linux`. Both branches enforce checks for administrators,
require linear history, and reject force-pushes and branch deletion.

Before promotion, verify that the candidate descends from the current `stable`,
belongs to `main`, passes all required checks on that exact SHA, and has no
known P0 or P1 regression. User-visible runtime interaction changes also need a
fresh-session dogfood pass. Ordinary promotions wait at least seven days after
the last Stable promotion, including an urgent one. Routine documentation and
planning changes wait for that window. Explicitly requested urgent fixes may
skip the wait, with the same verification and acceptance of the whole candidate.
There is no automatic weekly release and no release when the gate fails.

Record the candidate range, exact proof, actual promotion time, and any urgency
reason in the owning Bead. Use the actual previous promotion time, not the source
commit date. GitHub enforces status and linear-history requirements; checking
predecessor membership, cadence, acceptance, and dogfood remains the maintainer's
responsibility. Run the push only after all checks pass:

```sh
git fetch origin main stable
git merge-base --is-ancestor origin/stable <sha>
git merge-base --is-ancestor <sha> origin/main
git log --oneline origin/stable..<sha>
git diff origin/stable <sha>
git push origin <sha>:stable
```

Skip promotion when no candidate meets the contract. To roll back, commit the
revert on `edge`, verify it, promote it to `main`, and then promote it through
the same stable path. Do not move `stable` backward.

### Stable releases

Every Stable promotion is a release. Edge and Main are rolling channels: they
carry no tags, and once a candidate bumps the version they report it with their
channel label, such as `1.4.0 Edge` while Stable reports `1.3.0`. `yzx --version`
omits the Git revision.

Prepare the release in the Edge candidate, so the promoted revision already
carries it:

- Set `novaVersion` in `flake.nix`. Bump the minor version when the promotion
  contains user-visible features, configuration fields, keymap or behavior
  changes, and the patch version when it contains only fixes, including urgent
  fixes. Major versions are reserved for an explicitly chosen breaking release.
- Rename `## Unreleased` in `CHANGELOG.md` to `## X.Y.Z` and start a new empty
  `## Unreleased` above it.

After pushing the candidate to `stable`, tag that exact revision and publish its
GitHub release with the version's changelog section and the sponsorship footer:

```sh
git tag -a nova-vX.Y.Z <sha> -m "Yazelix Nova X.Y.Z"
git push origin nova-vX.Y.Z
gh release create nova-vX.Y.Z --repo Yazelix/nova --verify-tag --title "Yazelix Nova X.Y.Z" --notes-file <notes.md>
```

A plain version tag publishes no portable assets: Portable Release classifies
the tag and skips unless its annotation is a portable approval
([portable runtime](portable-runtime.md#nova-release-assets-001-reviewed-tag-and-publication)).
Each version has one release owner, either the maintainer's plain release or
Portable Release publishing that version's assets, never both. Record the
version, tag and release URL in the promotion Bead.

### Release notes

End each GitHub release body with this sponsorship footer, once:

```markdown
If Yazelix saves you time, you can support continued development on [GitHub Sponsors](https://github.com/sponsors/luccahuguet).
```

For existing releases, save the original body and metadata, append only the footer,
and verify the published body, tags, titles, flags and assets through the GitHub API.

## Dependency updates

Lucca owns a monthly dependency review and may delegate it to an agent. Review
the current Edge pins during the first maintenance session each month; an
available update does not require adoption. Prefer suitable official releases
for third-party tools. Use reviewed commits where the dependency's integration
branch or release model requires them. Versions remain owned by `flake.nix`,
`flake.lock`, package definitions and child manifests.

Nova Bar, Anima, Popup and the pane orchestrator share Nova Bar's Fenix input
and its followed root nixpkgs. Review all four consumers when either input
changes, including native WASI builds and Popup interactions. The pane
orchestrator, Popup, Radar and zjstatus share flake-utils. Nova's `follows`
declarations own this selection; each child retains its build recipe.
Helix keeps its independent overlay so Rio updates do not select its development
tools. Other toolchain alignment requires an explicit maintenance benefit and
compatibility decision. Compare affected derivations across supported systems;
equal repository names or Rust versions alone do not establish equivalent builds.

Review related changes together: Yazi and `ya` must stay paired with compatible
schemas, presets and plugins. Review nixpkgs deliberately because child inputs
that follow it share the changed package graph. Consume verified first-party
fixes and features, such as a Radar fork revision, when Nova needs them without
waiting for the monthly review. Compare forks with their intended integration
branch; a newer commit elsewhere does not make the accepted pin stale.

Relevant security fixes and serious user regressions receive prompt review.
Major or API-breaking updates may wait for dedicated compatibility work.
Choose updates for a concrete benefit and keep unrelated changes separable;
the review does not authorize a broad lock refresh or automatic merge.

Start with release notes, source comparisons and focused local checks. Selected
updates follow the existing [CI](#ci), [local verification](#local-development)
and platform gates in `AGENTS.md`: shared package changes need exact-revision
Linux CI, Darwin Package Smoke and installed-artifact proof, plus fresh-session
dogfood for interaction changes. Discovery alone needs no native builds. Keep
existing build limits; diagnose a failed candidate, narrow the update or defer
it with its reason and revisit condition in the owning maintenance Bead. One
review may record several deferrals without creating an issue for each version.
Record the review date, inspected revision and outcome even when no update is
selected. Schedule material implementation work separately under the issue rules.

Monthly review balances staleness against maintainer effort and native build
cost. Weekly broad updates spend that budget too often; release-only review can
leave dependencies unattended. Adoption on Edge and [channel promotion](#edge-main-and-stable)
remain separate decisions; the Stable interval does not schedule dependency bumps.

The **Dependency Updates** workflow reports available changes in its Actions run
summary on the first of each month at 12:23 UTC and on manual dispatch. It checks
out Edge once and identifies that inventory's exact revision. Open
[Dependency Updates](https://github.com/Yazelix/nova/actions/workflows/dependency_updates.yml),
select a run and read its summary. To request a report:

```sh
gh workflow run dependency_updates.yml --repo Yazelix/nova --ref edge
```

GitHub runs schedules from the default branch, Main. Scheduling becomes active
when the workflow reaches Main through normal promotion; an Edge-only workflow
does not establish an active schedule. GitHub may delay scheduled runs, so a
missing report does not replace the maintainer's monthly review.

The report groups Yazi/ya integration pins and covers direct flake inputs plus
Nova's explicit Yazi and CodexBar packages. GitHub supplies full releases and
exact branch comparisons.
Comparison branches absent from commit-pinned URLs live in
`.github/scripts/dependency_report.py`; current versions stay in their manifests.
New pinned sources without comparison policy require review. The two explicit
package readers accept their existing literal Nix declarations and report an
incomplete inventory if those declarations become computed or change shape.
Nixpkgs tool versions and child internals remain with their owners.

Available updates, pins ahead of branches and divergent history are review
findings. Missing policy, malformed data and failed API requests mark the report
incomplete and fail the run. Nothing updates pins, builds packages or decides
compatibility. One canonical-repository Linux job has a ten-minute limit,
read-only access and one active run. API requests have eight-second socket
timeouts, a 4 MiB response limit and no retries. There are no custom secrets,
cache writes, uploaded artifacts or issue notifications. Expand the budget only
when measured coverage or failures justify it; simplify or remove the checker
if its reports do not assist review.

Run the offline report contract without credentials or network access:

```sh
python3 -B -m unittest discover -s .github/scripts -p 'test_dependency_report.py'
```

## Local development

Use local sibling repositories while hacking runtime inputs:

```sh
nix run --override-input rio ../nova-rio
nix run --override-input yazelixHelix ../nova-helix
nix run --override-input yazelixForest ../yazelix-forest
nix run --override-input zjRadar ../zj-radar
nix run --override-input yazelixZellijPopup ../zellij-popup
nix run --override-input novaBar ../nova-bar
nix run --override-input yazelixZellijPaneOrchestrator ../zellij-pane-orchestrator
nix run --override-input yaziBistro ../yazi-bistro
```

For coupled child changes, use the same complete override set for builds,
checks, and every dogfood profile refresh. The profile does not retain overrides.
Publish matching child revisions and lock them before verifying without overrides.
Check popup open/hide and sidebar toggle together in a fresh installed session.

The [Nova Zellij distribution](../crates/zellij-distribution/README.md)
builds `yzx-zellij` from unreleased upstream Zellij main with Nova's carried
patches and embeds the packaged plugins through the API merged in PR #5630.
Each channel uses the revision it was promoted with.
The Zellij API revision is pinned in its `Cargo.toml` and `Cargo.lock`;
`zellijSource` supplies the matching theme inventory for a flake check.

The [check map](#check-map) lists the focused local checks.

`TEST-LANGUAGE-001`: maintained product checks use Rust; Python is confined to
GitHub automation in `.github/scripts`. The native attached-client and bottom-hint
checks share a headless tmux driver with isolated state, bounded polling and process
cleanup. Tab checks force a redirected mouse release, match highlights to pane
contents, and wait for both attached clients to resize. Their interactive scenarios
run on `x86_64-linux`; the Rio TOML assertion
also runs in the Darwin package gate. Parsing uses the same serde, serde_json and
toml versions as the config crate. Linux CI runs both native interaction checks.
The same driver runs `managed_session` in Linux CI and Darwin Package Smoke: a
fresh headless `yzx enter` must render the bar, Radar, hints and Yazi picker,
toggle Radar and the menu popup through their plugins, and pass in-session
`yzx status --json` and `yzx doctor` while the Zellij server stays alive.
Failures print the screen and the Zellij log tail.

Run the release compatibility check locally on the matching native platform
(replace `x86_64-linux` with `aarch64-darwin` on Apple Silicon):

```nu
nix build --no-link --no-write-lock-file --print-out-paths --print-build-logs --override-input nixpkgs (open --raw checks/newer-nixpkgs.txt | str trim) .#checks.x86_64-linux.home_manager
```

Runtime package changes should also pass a temporary profile install:

```sh
nix profile add --refresh /absolute/path/to/yazelix --profile /tmp/yzx-profile
```

Detailed launch, config, editor, and shell contracts live in
[Runtime Notes](runtime-notes.md)

The Rio configuration contract can be exercised against a candidate native binary
with `YZX_TEST_RIO` set to its absolute path: run the `yzx-config` crate test
`rio_native_controls_preserve_validate_reset_and_respect_ownership` with
`--include-ignored`. It covers real native validation, preserved comments, reset,
invalid-document fallback, Home Manager protection, and Rio-free models. Ordinary
crate checks omit this external-tool test; the packaged `rio_contracts` check also
validates the seeded config through the exact pinned Rio executable.

## LOC scorecard

Counts raw physical lines in tracked project files. **Code and configuration**
includes every counted file except Markdown, plain-text inventories, and the
license; those form **documentation and text**. Beads state (`.beads/`),
lockfiles (`*.lock`), and binary assets are excluded. New owned files count
automatically once committed.

```sh
git ls-files | grep -Ev '^\.beads/|\.lock$|^assets/' | grep -Ev '\.(md|txt)$|^LICENSE$' | xargs wc -l | tail -n 1
git ls-files | grep -Ev '^\.beads/|\.lock$|^assets/' | grep -E '\.(md|txt)$|^LICENSE$' | xargs wc -l | tail -n 1
```

| Category | Current |
| --- | ---: |
| Code and configuration | 35,292 |
| Documentation and text | 6,239 |
| Total | 41,531 |
