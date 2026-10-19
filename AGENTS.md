# Agent Guidelines

This file is self-contained. Canonical protocol text was rendered into it;
the source repository is needed only to update or verify the import.
Do not edit this generated file directly. Edit `.agent-protocols.local.md`
or `.agent-protocols.exceptions.json`, then render from the pinned source.

## Protocol import record

- Source: `https://github.com/Yazelix/starcompass`
- Source commit: `129746d8d56fad641a1fbc99aa3844cf51468fef`
- Profiles: `orchestrator`, `release`, `fork`
- Manifest: `.agent-protocols.json` (schema 1)

| Protocol | Version | SHA-256 |
| --- | ---: | --- |
| `AP-SCOPE-001` | 5 | `f68b0606986b0ac083a0c7949e1b2ab5d34150e0f906ddae226b9d5429ee115a` |
| `AP-CONTRACT-001` | 5 | `4e3695ec39a6d1eb01a18930d0ce3ed3241078b4dfefbee4f2e295e74fd13d1f` |
| `AP-REFERENCE-001` | 4 | `906bcda869549979647d20e957ff48909fa5f36024f584be6657bdc058d58e72` |
| `AP-MINIMAL-001` | 6 | `8dde259c77ec251719dba23eb1090b10262bbc2fb000d4899690781eb6563e11` |
| `AP-DEPENDENCY-001` | 2 | `ded49add538b82de0a9c522bc8a34720f4ebbb47f39fc2f7ddb25e7aad1700d3` |
| `AP-OWNERSHIP-001` | 2 | `d218a4e0625b659ec366284110bdfce02bd66cb229ab6ba07f2317092ea13053` |
| `AP-TEST-001` | 4 | `1c1c5eafa1f611fd7575ecf00b32a2068a477614c8be78ac4efa41630ec8b5ad` |
| `AP-PROOF-001` | 11 | `310e1f798c3334e08402f9e42f447ae9352d171fc92888afb141c6b228f84e56` |
| `AP-PLAN-001` | 12 | `5ee8deccb4b3c61ddd4a52b8ed6f69ebd30e24b65ace867871a89c75ef02a984` |
| `AP-CI-001` | 3 | `c7cb65a81ce8434d02f2d19306bf93358f3624d0b77886bc91d4de93f7a779ba` |
| `AP-ORCHESTRATOR-001` | 2 | `f78ebc0db6db525aac1d3226703af7e84cba9953c579470763831241a7d0c2ad` |
| `AP-FRONTIER-001` | 2 | `0777c74645955a26dc3829812ff26fa1a00a017c3b4cc1ac6740284f2664566c` |
| `AP-PORTABILITY-001` | 2 | `a2f5025bc7a10436b5c2f8f42c6002ccf1e361f05abbff3dea6507a3426394c0` |
| `AP-EXCEPTION-001` | 2 | `2c4f00299922edac83286821af07ad485e5a630bca6acbbce918d87614abb395` |
| `AP-GIT-001` | 5 | `f2c3254311de57a23a13aa382fe3285e65bb34e535ecd813d1be783ca79d4bf9` |
| `AP-PERF-001` | 1 | `500ddfc56c1293e74ba1e8e19dddca0129fbbec49e9a00735e975ee2e8a8790d` |
| `AP-DELIVERY-001` | 3 | `b6b0944ba57587204fd5f8f42daf69f7ff46486281056c99f2b403e8278366ee` |
| `AP-PROMOTION-001` | 2 | `07b53035fd705affe21ca1b0c07a2b766bc91fe45bc83c036c5c335577a08f93` |
| `AP-LICENSE-001` | 1 | `cc490313be0b53952c4adfd390600e243c3dfeb14713de9088fdcb8f99e874ac` |
| `AP-SESSION-001` | 4 | `51c15e878070caaf372721c2eb083354f5f63492f5c4e694cf31aaefcefa53a7` |
| `AP-FORK-001` | 2 | `11847b012a06b859b42c01c5aa3f4a2fb1d079cb25de80f27c3b129ae4f4780e` |
| `AP-FORK-002` | 2 | `4d527d23672d2541e9f8b0ba0d83613ba715bfe6e7870437eccc4bf267d0bb4d` |

### Local exceptions

No local exceptions.

## Canonical protocols

### AP-SCOPE-001 — User-owned scope

The user owns scope. Inspection, audit, diagnosis, explanation, and advice
authorize no implementation, external-state, or durable-planning write.
Implement only a chosen outcome.

Never silently add features, migrations, repositories, or planning items.
Preserve user inputs and artifacts; replace or delete an exact target only when
required, else write a distinct result. State consequential assumptions; stop
when a missing scope choice would materially change the result. Recommend and
justify an answer to every choice. “Finish” adds persistence, not authority.
Report out-of-scope findings unless the user chose a durable home.

### AP-CONTRACT-001 — Contract-driven changes

Before code shape, state the smallest observable contract: consumer, trigger,
result, and important failures. Of the contracts consistent with the request
and evidence, choose the fewest unsupported guarantees or restrictions; leave
reasonable future behavior unspecified. Give it a stable ID only when later
consumers need one. Choose the cheapest check that would falsify it.

Keep docs, help, examples, and configuration aligned with current commands,
paths, flags, defaults, and availability; label planned, partial, or gated
behavior. Update the contract first for an intentional behavior change.
Implementation details are not contracts unless a component must rely on them.

### AP-REFERENCE-001 — Evidence before code shape

Before a consequential code shape, read affected instructions, code, contracts,
tests, and required subsystem references. Record the adopted, rejected, or
unresolved mechanism; separate evidence from inference and revisit it after a
material shape change. Memory, summaries, and reputation are discovery only.

### AP-MINIMAL-001 — Minimum sufficient implementation

After identifying the contract, evidence, owner, flow, and boundaries, take the
first sufficient option: no change; existing owner, helper, or pattern;
standard library or native platform; accepted dependency that owns the
behavior; minimum correct local code.

Judge the whole lifecycle, including coordination, coupling, migration and
removal, portability, operations, proof, and agent context. Scope instructions
narrowly; retain non-obvious constraints and reusable behavior-changing
workflows, not generic or duplicated policy. Load details only when needed.

A patch is not minimal if it treats a symptom below its shared cause, bypasses
a boundary, or raises downstream cost. Prefer deletion and fewer files; use
patch size only between equally correct system shapes. Never remove required
behavior, trust-boundary validation, data-loss protection, security,
accessibility, or the cheapest runnable check for non-trivial logic.

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
representative tasks without supplying the expected conclusion when the user
authorizes a separate agent; otherwise validate in the active context and
report agent behavior as unproven. Structural checks prove structure, not
agent behavior. Test observable effects; delete duplicate proof,
implementation trivia, and scaffolding. Guard absence only when absence is a
security, licensing, size, ownership, or known-regression contract.

### AP-PROOF-001 — Explicit proof lifecycle

Proof records the revision, command or observation, environment, and result
with its surface. Never widen or reuse proof after a material change.

Use the highest boundary that exercises the changed contract. Add broader or
lower suites only for a changed owner, failure, trust boundary, distinct
delivery, or promotion, not because a suite exists, an issue lists it, or it
ran before. Exact downstream proof covers lower checks only for the identity
and behavior it exercised.

Run a complete suite once per unchanged candidate and environment; metadata,
docs, and planning follow-ups reuse it. Commands are evidence, not a checklist.
Keep logs only when reproduction is costly, disputed, or performance-sensitive,
or a summary loses detail; a temporary path is not proof. Unattended runs and
waits close unused stdin, time out, check progress early, and never match
themselves as the target; stalls fail.

Finish every change to code, tests, or scripts with review by a fresh agent
that did not write it, since authors miss their own leftovers; in Claude Code,
use the converge skill. Docs-only and planning-only work skips it. A pass that
changes anything requires another; convergence needs a full clean pass. Nits
never block it, and text-only fixes restart a review at most twice per slice.

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
the issue tool (beads_rust `br` when available), never its storage.

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

### AP-PERF-001 — Measured performance claims

Performance claims need a measured bottleneck, a fixed workload and
environment, a baseline, a correctness oracle, and a distribution or bound.
Keep negative results that constrain them.

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

### AP-LICENSE-001 — Exact license interpretation

Apply source-license terms to their exact actors, uses, conditions,
beneficiaries, and direction. A user who chooses a provider's tool does not act
for that provider. An example introduced by “including” applies only within the
condition it illustrates. Inspection is distinct from copying, adaptation,
redistribution, dependency selection, and incorporation, so a restriction on one
does not spread to another. If an interpretation would block required evidence,
identify the exact clause and roles and resolve material ambiguity with the
user. Review does not require reuse.

### AP-SESSION-001 — Session continuity across compaction

Compaction is lossy; avoid it. Write each user decision to its durable owner
when it is made. Delegate work whose raw output matters only as a conclusion,
such as broad searches, long logs and checks, or research, to a fresh
subagent; subagents are authorized unless the user or local rules say
otherwise.

Budget absolute context tokens, not window share; if the host reports none,
estimate high:

- From 400k, add no scope or new steps: close the issue if it can close by
  600k, else bring work in progress to a verified state and hand off.
- Past 600k, continue only to reach an imminent verified state or keep live
  state that cannot be written down; record why on the issue.
- Hand off before the host would compact. After any compaction, reread the
  issue and hand off.

A handoff is an issue comment: state, next step, and open decisions with
reasons. The next session primes, e.g. "prime, then continue <issue> from its
handoff".

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

Do not mechanically port Yazelix Classic; decide explicitly what survives. The
runtime chain is `yzx -> Nova Rio -> Yazelix Zellij fork`. The project
interface is a Nix/Lix-compatible flake, and `yzx` is the sole public command.
Do not broaden Home Manager, layouts, config generation, plugins, pane policy,
or legacy compatibility unless the user chooses that scope.

### Git Channels

Keep one linear history with `stable ⊆ main ⊆ edge`. Every tracked change,
including fixes, reverts, documentation, and Beads updates, originates on
`edge`; work there directly. `main` (accepted development) and `stable` (user
channel) are promotion-only: after reviewing every change since the target's
current revision, push the exact verified revision with
`git push origin <sha>:main` or `git push origin <sha>:stable`. If unaccepted
work precedes a wanted fix, accept it, revert it on `edge`, or defer.

Advance `stable` only when the user explicitly requests it. The candidate needs
green `linux`, both cache publication checks, and `release-gate`, fresh-session
dogfood for user-visible interaction changes, and no known P0 or P1 regression.
Ordinary promotions wait seven days after the last one; an explicitly requested
urgent fix may skip only that wait. Record the candidate, proof, actual
promotion time, and any urgency reason in the owning Bead. Never delete
`stable`. `docs/development.md` owns the full procedure and ancestry commands.

### Beads

Use `br` for all issue work; serialize writes and finish with
`br sync --flush-only`. `bv --robot-triage` is the planning entry point; use
only `bv --robot-*` commands, because bare `bv` opens an interactive TUI.
Verify a recommendation with `br show <id> --json` or `br ready --json` before
claiming it.

### Hosted Check Handoffs

Do not spend a run watching a hosted check that normally takes more than about
30 minutes, including the staged `Version Gate`; shorter checks may be watched.
After dispatching a long check, verify that it started for the intended
workflow, ref, and exact revision. Record its run URL, revision, required
conclusion, and follow-up in the owning Bead, creating one handoff Bead if none
owns it; sync and return control. Never perform a dependent promotion or
release before the required check succeeds.

### Command Formatting

Commands shown to the user must work when pasted into Nushell unless the user
explicitly requests another shell. Never use backslash line continuations. Keep
each command on one line, or show multiple complete commands on separate lines.

### LOC and Documentation

Update the LOC scorecard counts in the README and `docs/development.md`
whenever project files change. Justify LOC growth in the commit message and
owning Bead; the scorecards hold counts and method only. Update `CHANGELOG.md`
when user-visible runtime behavior, commands, keymaps, packaged tools, or
runtime contracts change. Formatting rules outrank LOC pressure; keep `rustfmt`
output for Rust.

The website is the canonical user guide for the current Stable release. Point
the README's prominent documentation link directly to
`https://nova.yazelix.com/docs/`.

### Local Verification

`docs/development.md` lists the local gates and focused checks. You may use more
than two CPU cores: size the budget from current CPU, memory, and load, leave
room for interactive use, and avoid overlapping heavy builds. Nix `--cores` is
per build, `--max-jobs` multiplies it, and client `taskset` does not constrain
the Nix daemon.

For plugin-only iterations, run model tests and build the Wasm plugin against
the existing native binary before rebuilding the distribution. Batch full Nix
builds after related fixes converge, and keep candidate artifacts GC-rooted
while they are still needed.

A readiness timeout alone does not prove a runtime defect. Capture the last
valid snapshot, screen, and elapsed time, then make one bounded longer
observation before changing code or deadlines. Keep quick-click input timing
separate from readiness waits; quiet terminal capture may validly produce no
output. Native-server CPU does not describe Rio or whole-session cost.

### Runtime and GUI Checks

Run automated GUI checks only on a private display through the pinned Kinestra
recorder, which launches the application and sends its input. Never fall back
to the user's desktop or host keyboard automation; if isolation fails, keep the
diagnostic and stop that test. Use the user's display or launch GUI sessions
only when the user explicitly asks for that or reports manual dogfooding; a
request to test or update Nova does not authorize a visible desktop test.

Before handing a test command to the user, run that exact installed launcher
with its actual environment, paths, config, and arguments in a fresh session
through the full anima-to-workspace handoff. `--version` and direct
native-binary probes do not exercise the launcher; only `launch` covers Rio.

Known regression traps:

- Isolate Zellij's session cache as well as its socket and Nova state
  directories. Remove only a probe's own dead session record before reusing its
  name; a leftover resurrection record made the next launch exit after anima.
- Check the complete socket path in bytes, including Zellij's namespace and
  session name, against the platform limit; a 119-byte Linux path made startup
  exit after anima while short test paths hid it.
- For pane order, compare ordered stable pane IDs through repeated toggles and
  manual rearrangements, including with a popup open; presence, geometry, and
  focus checks passed while panes rotated.
- When building historical source in a shared Cargo target, force a fresh build
  and verify artifact identity; old timestamps reused an unrelated Wasm
  artifact.

### Linux and Darwin Gates

Classify every change before implementation as shared, platform-specific, or
platform-neutral. Nix evaluation and packaging, dependencies, Home Manager,
config materialization, and runtime command, shell, filesystem, process, or PTY
behavior are shared unless a contract says otherwise. Shared changes need
Darwin evidence before acceptance or promotion:

- `edge`: green Linux CI and `Darwin Package Smoke` on the exact revision.
  Dispatch Darwin when its scheduled run misses that revision or after changing
  its workflow.
- `main`: green CI plus `Publish Nix Cache`'s `publish_x86_64_linux` and
  `publish_aarch64_darwin`.
- Release candidates: `Version Gate`'s `release-gate` plus both protected cache
  checks. Skipped work is not proof.

`nix flake show --all-systems` and Darwin evaluation on Linux prove flake shape,
not an `aarch64-darwin` build. Linux-only evidence suffices only for an
explicitly Linux-only surface, platform-neutral documentation or planning, or a
capability disabled on Darwin by contract; record that boundary and keep shared
flake evaluation green.

### Installed Edge Updates

Home Manager in `/home/lucca/.config/home-manager` owns the user's
`Yazelix Nova (Edge)` desktop entry through the `yazelix` input's
`yazelix-edge` package. As soon as a runtime or package change reaches `edge`,
update only that input to the pushed candidate. Never wait for hosted checks
first: the sooner Lucca dogfoods, the sooner problems surface. Hosted checks
still gate acceptance and promotion.
Review the lockfile and activation diff, build the activation closure, and
verify the selected package before activating; preserve unrelated Home Manager
edits. Then verify the locked Nova revision and the actual `yzx-edge.desktop`
Exec against the installed artifact from a fresh session; shell PATH or a
separate Nix profile does not prove the desktop launcher.

Documentation- and Beads-only pushes need no package update. Never add a
separate `yazelix-edge` profile or update the Stable installation unless
explicitly requested.
