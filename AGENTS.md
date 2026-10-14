# Agent Guidelines

This file is self-contained. Canonical protocol text was rendered into it;
the source repository is needed only to update or verify the import.
Do not edit this generated file directly. Edit `.agent-protocols.local.md`
or `.agent-protocols.exceptions.json`, then render from the pinned source.

## Protocol import record

- Source: `https://github.com/Yazelix/starcompass`
- Source commit: `4af5a3265acd425d1359f56f805221fc264a9e1a`
- Profiles: `orchestrator`, `release`, `fork`
- Manifest: `.agent-protocols.json` (schema 1)

| Protocol | Version | SHA-256 |
| --- | ---: | --- |
| `AP-SCOPE-001` | 3 | `cae30f9031beea4f831a89445f35807f50618e0f2cb0432f3b94bec40da9aeef` |
| `AP-CONTRACT-001` | 4 | `cdf2e5d69cefd34ceeeaa6b7511b21f921e2be50c17d3c01b85120b20234cff7` |
| `AP-REFERENCE-001` | 3 | `186fa74aa530c9c76ae507685461dff2b1506ecb5087230ded51f02cecb9231a` |
| `AP-MINIMAL-001` | 4 | `612bc9c62a20adf7332724d4663af0a0e591d624de2a76b1c257a9ed428db1f6` |
| `AP-DEPENDENCY-001` | 2 | `ded49add538b82de0a9c522bc8a34720f4ebbb47f39fc2f7ddb25e7aad1700d3` |
| `AP-OWNERSHIP-001` | 2 | `d218a4e0625b659ec366284110bdfce02bd66cb229ab6ba07f2317092ea13053` |
| `AP-TEST-001` | 3 | `58b5837cb679e958192b366bb15d6e34649f5a91ff9f4accdc7edb4ef5cdb873` |
| `AP-PROOF-001` | 6 | `01fc3712fd369189b934b0503173e6a8963be64b7f18869abfaef3cca53866f7` |
| `AP-PLAN-001` | 11 | `fcb7c8d2b99c6641a7cd1244072eda6230f1ccfc3a4aa47c29cf2525fcb1376d` |
| `AP-CI-001` | 3 | `c7cb65a81ce8434d02f2d19306bf93358f3624d0b77886bc91d4de93f7a779ba` |
| `AP-ORCHESTRATOR-001` | 2 | `f78ebc0db6db525aac1d3226703af7e84cba9953c579470763831241a7d0c2ad` |
| `AP-FRONTIER-001` | 2 | `0777c74645955a26dc3829812ff26fa1a00a017c3b4cc1ac6740284f2664566c` |
| `AP-PORTABILITY-001` | 2 | `a2f5025bc7a10436b5c2f8f42c6002ccf1e361f05abbff3dea6507a3426394c0` |
| `AP-EXCEPTION-001` | 2 | `2c4f00299922edac83286821af07ad485e5a630bca6acbbce918d87614abb395` |
| `AP-GIT-001` | 5 | `f2c3254311de57a23a13aa382fe3285e65bb34e535ecd813d1be783ca79d4bf9` |
| `AP-DELIVERY-001` | 3 | `b6b0944ba57587204fd5f8f42daf69f7ff46486281056c99f2b403e8278366ee` |
| `AP-PROMOTION-001` | 2 | `07b53035fd705affe21ca1b0c07a2b766bc91fe45bc83c036c5c335577a08f93` |
| `AP-SESSION-001` | 1 | `f8bdac637027895ec8d1ad4f192e13808d095a809d091b76eadadaf936b2a340` |
| `AP-FORK-001` | 2 | `11847b012a06b859b42c01c5aa3f4a2fb1d079cb25de80f27c3b129ae4f4780e` |
| `AP-FORK-002` | 2 | `4d527d23672d2541e9f8b0ba0d83613ba715bfe6e7870437eccc4bf267d0bb4d` |

### Local exceptions

No local exceptions.

## Canonical protocols

### AP-SCOPE-001 — User-owned scope

The user owns scope. Inspection, audit, diagnosis, explanation, and
recommendation authorize no implementation, external-state, or durable-planning
write. Implement only a chosen outcome.

Do not silently add features, compatibility promises, public surfaces,
migrations, repositories, or planning items. Preserve user-owned inputs and
artifacts; replace or delete an exact target only when the outcome requires it.
Otherwise write a distinct result. State consequential assumptions and stop
when a missing scope choice would materially change the result. “Finish” adds
persistence, not authority. Report out-of-scope findings unless the user chose
a durable destination.

### AP-CONTRACT-001 — Contract-driven changes

Before code shape, state the smallest observable contract: consumer, trigger,
result, and important failures. Of the contracts consistent with the request
and evidence, choose the fewest unsupported guarantees or restrictions; leave
reasonable future behavior unspecified. Give it a stable ID only when later
consumers need one.

Keep docs, help, examples, and configuration aligned with current commands,
paths, flags, defaults, and availability; label planned, partial, or gated
behavior. Choose one source-of-truth owner, the cheapest falsifying check, and
the smallest complete vertical slice. Update the contract first for an
intentional behavior change. Implementation details are not contracts unless a
component must rely on them.

### AP-REFERENCE-001 — Evidence before code shape

Before a consequential code shape, read affected instructions, code, contracts,
tests, and required subsystem references. Record the adopted, rejected, or
unresolved mechanism; separate evidence from inference and revisit it after a
material shape change. Memory, summaries, and reputation are discovery only.

Apply source-license terms to their exact actors, uses, conditions,
beneficiaries, and direction. Choosing a provider does not make a user act for
it; an “including” example remains scoped by its condition. Inspection is
distinct from copying, adaptation, redistribution, dependency selection, and
incorporation, so a restriction on one does not spread to another. If an
interpretation would block required evidence, identify the exact clause and
roles and resolve material ambiguity with the user. Review does not require
reuse.

### AP-MINIMAL-001 — Minimum sufficient implementation

After identifying the contract, evidence, owner, flow, and boundaries, take the
first sufficient option: no change; existing owner, helper, or pattern;
standard library or native platform; accepted dependency that owns the
behavior; minimum correct local code.

Judge the whole lifecycle, including duplicate truth, coordination, coupling,
migration and removal, portability, operations, proof, and agent context. Scope
instructions narrowly; retain non-obvious constraints and reusable
behavior-changing workflows, not generic or duplicated policy. Load details
only when needed.

A patch is not minimal if it preserves a wrong or duplicate owner, treats a
symptom below its shared cause, bypasses a boundary, or raises downstream cost.
Prefer deletion, direct ownership, and fewer files; use patch size only between
equally correct system shapes. Never remove required behavior, trust-boundary
validation, data-loss protection, security, accessibility, or the cheapest
runnable check for non-trivial logic.

When available, use upstream
[Ponytail](https://github.com/DietrichGebert/ponytail/tree/16f29800fd2681bdf24f3eb4ccffe38be3baec6b)
as a fallible code-shape bias after these constraints. Its absence does not
suspend them, and its hooks do not prove compliance.

### AP-DEPENDENCY-001 — Dependency gate

Before adding a dependency, name its capability and contract. Compare owned
code, the standard library, and credible candidates for correctness,
maintenance, platform and license fit, transitive weight, API stability, and
net complexity. Record the choice, meaningful rejections, and replacement cost;
pin it and prove the relied-on behavior. Remove it when its ownership no longer
justifies its cost.

### AP-OWNERSHIP-001 — One owner per invariant

Give every invariant, state transition, and user-visible policy one owner;
consumers must not reconstruct or reinterpret it. Name the owner before adapters
or synchronization and delete duplicates. Put policy at the highest layer with
enough context and enforcement at the lowest correct layer. Make cross-boundary
data explicit and versioned for independently released consumers.

### AP-TEST-001 — Strong and few tests

Protect meaningful contracts, regressions, boundaries, and failures with a few
strong tests. Use TDD for deterministic helpers, parsers, protocol behavior,
and regressions whose expected behavior is known first; use contract-first
integration checks for layout, runtime, architecture, forks, and dogfood.

For consequential agent-instruction, prompt, or skill changes, run isolated
representative tasks without supplying the expected conclusion. Structural
checks prove structure, not agent behavior. Test observable effects; delete
duplicate proof, implementation trivia, and scaffolding. Guard absence only
when absence is a security, licensing, size, ownership, or known-regression
contract.

### AP-PROOF-001 — Explicit proof lifecycle

Proof records revision, command/observation, environment, result/surface,
and phase: proposed, implemented, mechanically verified, dogfooded, accepted,
or promoted. Never widen/reuse after material change.

Use the highest boundary exercising the changed contract. Add broader/lower
suites only for changed owner, failure, trust, distinct delivery, or
promotion—not availability, issue text, or prior phase. Exact downstream proof
subsumes lower checks only for the identity/behavior exercised.

Run a complete suite once per unchanged candidate/environment. Metadata/docs/
planning follow-ups reuse it; commands are evidence, not a checklist. Keep logs
only when reproduction is costly, disputed, performance-sensitive, or lossily
summarized; temporary paths are not proof.

Performance claims need a measured bottleneck, fixed workload/environment,
baseline, oracle, and distribution/bound; retain constraining negatives. Review
changes require another pass; convergence needs a full clean pass.

### AP-PLAN-001 — Durable planning state

Keep later-needed outcomes/constraints in planning or canonical docs. Issues
hold chosen goals/decisions, material defects, or schedulable follow-ups;
methods stay with their owner. Acceptance names outcomes/cheapest falsifiers,
not command catalogs unless the boundary/transition needs them.

A run includes continuations until control returns; reads are unrestricted. A
writing run does exactly one:

1. Owns at most one issue and may create/claim/update/implement/close it.
2. With explicit user authorization, changes a named/accepted planning-only
   batch; claims nothing and changes no implementation.

Bind implementation before writes; return at completion, block, or handoff.
Report unapproved findings; create issues only for material out-of-scope or
schedulable work, deferred outside authorized batches.

Keep fields current; append only needed contracts, decisions, dependencies,
acceptance, material failures/negatives, constraining rejections, and approvals.
Baseline/reference comments record only changed identities, dirty state,
mechanism/stop decisions, and first check. Keep status/prerequisites honest; use
the issue tool, never its storage.

### AP-CI-001 — Bounded continuous integration

Before hosted automation, name its contract and why local proof is
insufficient. Bound triggers, jobs, timeouts, permissions, artifacts, cache,
concurrency; specify fork and secret behavior. Prefer one cheap deterministic
job. Record when to expand, reduce, or remove. Confidence must justify its
financial, latency, security, and maintenance costs, including private minutes
and storage.

### AP-ORCHESTRATOR-001 — Thin orchestrator ownership

An orchestrator owns composition, lifecycle policy, compatibility selection,
and user defaults, not child mechanisms or state. Consume pinned child artifacts
through explicit contracts; keep translation narrow; reject copies, hidden
forks, and duplicate schemas. Expose startup, shutdown, update, and partial
failure policy, and test with substitutes when a child is unavailable.

### AP-FRONTIER-001 — One active integration frontier

When greenfield components integrate, keep one boundary under active
architectural change when practical. Stabilize the others with released
artifacts, fixtures, adapters, or interchangeable projects. Dogfood the active
frontier through a real vertical slice and move it only after contract and
failure evidence. Parallel work may continue behind stable integration
contracts.

### AP-PORTABILITY-001 — Portable core with explicit platform seams

Keep multi-OS contracts and state neutral; isolate system APIs, processes,
paths, packaging, and events. Evaluate targets before foundational runtime,
graphics, filesystem, or transport choices. Prove behavior on each platform,
not by compilation; name unsupported targets.

### AP-EXCEPTION-001 — Explicit local exceptions

Only an exception approved by the user or named authority may narrow, replace,
or suspend an import. Record protocol ID, exact scope, reason, approver, date,
and any expiry or review condition. Unrecorded conflicts are drift; additive
detail is an overlay.

### AP-GIT-001 — Safe repository history

Repository history is shared user state. Inspect status and local policy;
preserve unrelated or concurrent work and use the least destructive sufficient
operation.

Amend the unpublished task commit for same-task corrections; refresh downstream
commits and generated artifacts. After external reliance (push, promotion,
release, or pin), commit corrections separately. Never reset, discard,
force-push, rewrite published history, or create a branch without authority.
Verify intended diff before commit and remote state after push. Roll back with a
reviewed revert unless local policy says otherwise.

### AP-DELIVERY-001 — Installed artifact and fresh-session proof

Source checks do not prove delivery. For packaging/install/activation/
interaction changes, build exact candidate through delivery path;
record artifact/environment/commands/result; activate without checkout/stale
fallback; verify identity; dogfood from a fresh session. Narrow unexercised
claims. Compilation is not install proof; scripts are not dogfood.

Composition proof covers an unchanged embedded artifact only when recording
exact identity and exercising relied-on behavior. Repeat inner proof for
distinct install/activation or failure isolation.

### AP-PROMOTION-001 — Exact-revision channel promotion

Release channels are ordered references over one linear history; the consumer
declares order, evidence, authority, and the development channel where every
change originates. Promotion-only channels fast-forward to the exact verified
revision in their predecessor. No direct channel commits, merges, cross-channel
cherry-picks, published rebases, force-pushes, or skipped stages.

Require target-channel delivery evidence, no known critical or high-severity
regression, and explicit authority for user-facing promotion. Roll back with a
development-channel revert, then verify and promote normally. Promotion changes
exposure, not proof.

### AP-SESSION-001 — Session continuity across compaction

Treat context compaction as lossy. Write each user decision to its durable
owner when it is made, and reread the active issue after any compaction. Once
a session has compacted, start the next issue in a fresh session; within an
issue, a second compaction or any sign of lost context triggers a handoff
comment (state, next step, open decisions) and a new session. Prefer a
clean-break restart to compaction, and start a large issue fresh when most of
the context is already used.

### AP-FORK-001 — Minimal isolated fork delta

A fork owns only behavior that cannot reasonably live upstream or externally.
Start from exact upstream source and behavior; prefer configuration, extension
points, then isolated patches over rewrites. Keep fork modules, commits, and
tests separable; avoid unrelated upstream cleanup. Keep the delta cohesive and
measurable and justify growth with user-visible ownership.

### AP-FORK-002 — Upstream and removal discipline

For every fork feature, record upstream baseline and patch provenance, audit
upstream changes instead of replaying patches mechanically, contribute general
mechanisms upstream when feasible, define its removal condition, and test both
fork behavior and the upstream boundary most likely to invalidate it.

## Repository-local rules

### Yazelix Nova

Yazelix Nova is a clean architecture track for a Yazelix-like runtime with the
fewest practical lines of code and the simplest ownership model.

### Nova Boundary

Do not mechanically port Yazelix Classic. Review the relevant sources of truth
and decide explicitly what survives.

Current runtime chain:

```text
yzx -> Nova Rio -> Yazelix Zellij fork
```

The project interface is a Nix/Lix-compatible flake. `yzx` is the sole public
command name. Do not broaden Home Manager, layouts, config generation, plugins,
pane policy, or legacy compatibility unless the user chooses that scope.

### Git Channels

Keep one linear history with this ancestry invariant:

```text
stable ⊆ main ⊆ edge
```

All tracked changes originate on `edge`, including fixes, reverts,
documentation, and Beads updates. Work directly on `edge` by default.

Promote whole revisions, never selected commits. Review every accumulated
change since the target channel's current revision. No cherry-picks, merge
commits, direct channel commits, or skipped stages. If an urgent fix follows
unaccepted work, accept that work too, revert it on `edge`, or defer promotion.

`edge` is the active development and experimental dogfood channel. `main` is a
promotion-only accepted-development channel. After an `edge` revision is
accepted and verified for `main`, advance `main` to that exact revision with:

```sh
git push origin <sha>:main
```

`stable` is the promotion-only user channel. Advance it only when the user
explicitly requests promotion. A candidate must be a fast-forward from the
current `stable`, belong to `main`, pass the protected `linux`, both cache
publication checks, and `release-gate`, and have fresh-session dogfood
for user-visible runtime interaction changes. Do not promote a commit with a
known P0 or P1 regression. Promote the exact verified revision with:

```sh
git push origin <sha>:stable
```

Ordinary Stable promotions wait at least seven days after the last promotion,
including an urgent promotion. Explicitly requested urgent fixes may bypass
only this wait, never verification or whole-candidate acceptance. Routine docs
and planning wait for the next eligible promotion. Record the candidate, proof,
actual promotion time, and any urgency reason in the owning Bead.

GitHub requires linear history and status checks, including for administrators,
on `main` and `stable`. Ancestry through the immediate predecessor, cadence,
scope acceptance, and dogfood are maintainer checks before the push.
Never delete `stable`.

### Beads

Use `br` for all issue work. Serialize writes and finish with
`br sync --flush-only`.

Use `bv --robot-triage` as the graph-aware planning entry point. Use only
`bv --robot-*` commands; bare `bv` opens an interactive TUI. `bv` decides what
to work on, while `br` creates, updates, and closes issues. Before claiming a
recommendation, verify it with `br show <id> --json` or `br ready --json`.

### Hosted Check Handoffs

Do not spend an active agent run watching a long hosted check. Treat a check as
long when its workflow design or recent history indicates that it normally
takes more than about 30 minutes, including the staged `Version Gate`.

After dispatching a long check, verify that it started for the intended workflow,
ref, and exact revision. Record the run URL, revision, required conclusion,
current state, and success or failure follow-up in the current owning Bead. If
the check has no owning Bead, create one follow-up Bead for that handoff. Finish
with `br sync --flush-only`, return control to the user, and let a later run
inspect the result and continue or close the Bead. Do not perform a dependent
promotion or release before the required check succeeds.

Short and medium hosted checks expected to finish within about 30 minutes may
be watched in the active run.

### Command Formatting

Commands shown to the user must work when pasted into Nushell unless the user
explicitly requests another shell. Never use backslash line continuations. Keep
each command on one line, or show multiple complete commands on separate lines.

### LOC and Documentation

Update the LOC scorecard counts in the README and `docs/development.md`
whenever project files change. Update `CHANGELOG.md` when user-visible runtime
behavior, commands, keymaps, packaged tools, or runtime contracts change.

The website is the canonical user guide for the current Stable release.
Point the README's prominent documentation link directly to
`https://nova.yazelix.com/docs/`.

If LOC grows, justify the added behavior in the commit message and owning Bead.
The scorecards hold counts and method only, not growth narratives.
Formatting rules outrank LOC pressure; for Rust, keep `rustfmt` output rather
than compressing code manually.

### Nova Verification

For runtime flake changes, normally verify:

```sh
nix flake check
nix flake show --all-systems
nix build .#yazelix --no-link --print-build-logs
nix profile add --refresh /home/lucca/pjs/yazelix-dir/yazelix --profile <tmp>
```

### Efficient Local Verification

The user permits more than two CPU cores for future work. Choose a bounded
budget from available CPU, memory and current load, leaving room for interactive
use. Avoid overlapping heavy builds. Nix `--cores` is per build and `--max-jobs`
multiplies concurrency; client `taskset` does not constrain the Nix daemon.

Start with the smallest check that can reject the change. For plugin-only
iterations, run model tests and build the Wasm plugin against the existing native
binary before rebuilding the distribution. Batch full Nix builds after related
fixes converge, then verify the exact installed candidate with embedded plugins.
Keep candidate artifacts GC-rooted while they are still needed for verification.

A readiness timeout alone does not prove a runtime defect. Capture the last
valid snapshot, screen and elapsed time; use one bounded longer observation to
distinguish delayed convergence from persistent wrong state before changing code
or increasing deadlines. Keep quick-click input timing separate from readiness
waits. Quiet terminal capture may validly produce no output or capture file.

Measure idle CPU only after startup and interaction queues settle, across several
windows. Identify the measured process; native-server CPU does not describe Rio
or whole-session cost. Repeated full builds or inconclusive probes are a signal
to narrow the reproducer and reassess the hypothesis. Use hosted gates for costly
platform validation and follow the existing handoff rule for long checks.

### Runtime Handoff and Regression Checks

Run automated GUI checks on a private display through the pinned Kinestra
recorder. Launch the application and send input through that recorder; never
fall back to the user's desktop or host keyboard automation. A request to test
or update Nova does not authorize a visible desktop test. Use the user's display
only when they explicitly request that interaction. If isolation fails, retain
the diagnostic and stop that test.

Before handing a test command to the user, run that exact installed launcher
with its actual environment, paths, config and arguments in a fresh session.
Verify the full anima-to-workspace handoff. `--version` or a direct native-binary
probe does not exercise the launcher and cannot substitute for this check.
For Rio handoffs, exercise `launch`; an `enter` probe does not cover that path.

Isolate Zellij's session cache as well as its socket and Nova state directories.
After a disposable probe, remove only its own dead session record before handing
over the same name. A leftover resurrection record previously made the next
`launch --session bottom-bar-test` exit after anima even with a valid socket path.

Keep isolated Unix socket directories short and check the complete socket path
in bytes, including Zellij's namespace and session name, against the target
platform limit. A proof-directory socket base previously produced a 119-byte
path on Linux and caused startup to exit after anima; short test paths hid it.

For pane-order regressions, compare ordered stable pane IDs through repeated
toggles and manual rearrangements, including with a popup open. Presence sets,
geometry and focus checks previously passed while panes rotated on each restore.

When testing historical source in a shared Cargo target, use a fresh build
directory or force the relevant target to rebuild, then verify artifact identity.
Older archive timestamps previously reused an unrelated Wasm artifact.

### Linux and Darwin Gates

Classify every change before implementation as shared, explicitly
platform-specific, or platform-neutral. Treat Nix evaluation and packaging,
dependencies, Home Manager, config materialization, and runtime command, shell,
filesystem, process, or PTY behavior as shared unless the contract says
otherwise.

Shared changes must not be accepted or promoted from Linux evidence alone:

- On `edge`, require green Linux CI and `Darwin Package Smoke` on the exact
  revision. Dispatch the Darwin workflow when its scheduled run does not cover
  that revision; dispatch it after changing the workflow itself.
- On `main`, require green CI plus both `Publish Nix Cache` jobs:
  `publish_x86_64_linux` and `publish_aarch64_darwin`.
- For a release candidate, require `Version Gate`'s `release-gate`, which
  requires Linux and every Darwin job to succeed, and the protected Linux and
  Darwin cache checks before promoting `stable`. Skipped work is not proof.

`nix flake show --all-systems` and Darwin derivation evaluation on Linux prove
flake shape only; they do not replace a real `aarch64-darwin` build. Linux-only
verification is allowed only for an explicitly Linux-only surface,
platform-neutral documentation or planning, or an optional capability that is
unavailable or disabled on Darwin by contract. Record that boundary and reason,
and keep shared flake evaluation green when the shared graph changes.

### Installed Edge Updates

Home Manager owns the user's `Yazelix Nova (Edge)` desktop entry. Its
configuration in `/home/lucca/.config/home-manager` selects the `yazelix`
input's `yazelix-edge` package.

After a runtime or package change is pushed to `edge` and its required checks
pass, update only that Home Manager input to the verified candidate. Review the
lockfile and activation diff, build the activation closure and verify the selected
package before activation. Preserve unrelated Home Manager edits.

After activation, verify the locked Nova revision and the actual
`yzx-edge.desktop` Exec against that candidate's installed artifact. Follow the
fresh-session proof rules above; shell PATH or a separate Nix profile does not
prove the desktop launcher.

Documentation- and Beads-only pushes do not require a package update. Do not
add or refresh a separate `yazelix-edge` profile for this installation. Do not
update the user's Stable installation unless explicitly requested.

Do not launch GUI sessions unless the user asks or reports manual dogfooding.
