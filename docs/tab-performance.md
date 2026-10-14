# Nova tab performance

Investigation: 2026-10-12. Scope: the Nova Edge desktop launcher on Linux.
Owners: investigation `yazelix-5vl-tab-performance-investigation-yia1`, mode
events `yazelix-5vl-tab-mode-events-mbhl`, and native build profile
`yazelix-5vl-native-release-profile-4bea` (all closed); hints follow-up
`yazelix-5vl-hints-tab-work-jvpj` and rendering model
`yazelix-5vl-shared-plugin-rendering-2vjp` (open). Closed sections keep results,
rejected options, and limits; open sections keep their full evidence.

The Home Manager Edge desktop entry resolved to exact revision
`47fe1ecd8dbdc6009a6d4294436b52d5e8eeafa8`, including lightweight Radar/Bar
mode events, the hints follow-up, and native release-profile settings. The
exact activated launcher passed fresh private-display anima-to-editor handoff,
creation, numbered/walking switches, and minor modes; exact Linux CI and Darwin
Package Smoke passed. The user reports it is substantially faster; this is
qualitative manual evidence, not a measured combined speedup. Raw proofs are
local workstation artifacts under `/home/lucca/.local/state/codex/proofs/`.

## Shared-plugin rendering decision

Owner: `yazelix-5vl-shared-plugin-rendering-2vjp`. The selected outcome is
**retain the current rendering model for the installed candidate**. Shared
hints demonstrate a scaling benefit, but the prototype fails immediate initial
activation and does not preserve the nested-session API. No runtime, child pin,
compatibility floor, desktop entry or user configuration changes in this pass.
This is Linux research and platform-neutral documentation, not shared-runtime
acceptance. Prerequisite Linux/Darwin delivery checks remain separate.

One fresh installed `enter` session grew from 2 to 8 to 16 tabs at 240x40 cells.
Native-server RSS grew from 157 to 226 to 362 MiB; walking-switch correct-bar
medians were 34, 41 and 93 ms. Each tab added Radar, Bar and hints panes; pane
counts do not distinguish shared slots from Wasm instances. This single curve
establishes a remaining scaling cost, not its attribution or a general forecast.

The smallest comparative slice uses the existing hints renderer. All three
file-plugin variants use native SDK source `333d7898`, identical Cargo locks,
Rust 1.96.0 and the same installed native executable. The legacy control keeps
the ordinary registration macro. A decoder-only control keeps its per-tab
lifecycle but substitutes the shared macro's byte reader and event decoder.
The shared variant keeps renderer state per client and collapse state per slot;
the [native host](https://github.com/zellij-org/zellij/pull/5662) owns visibility,
slot lifecycle and rendering targets.

Changing SDK alone fails to compile the existing nested-session handlers:
upstream uses session paths, generations and a synchronous keybinding response
instead of the fork's event/request interface. Both controls and the shared
prototype exclude those handlers identically for this top-level comparison.
That exclusion is a measurement boundary, not an accepted behavior change.

Fresh shared startup leaves hints blank even with cached permissions. Moving
permission requests to client connection or first-slot registration does not
restore initial activation within 20 seconds plus one bounded 20-second extension.
A minimal shared plugin reports permission granted with no mode; a real Pane
then Normal transition supplies the mode. The pinned host records deduplication
state before checking event permission (`plugins/shared.rs:462-495`). Together,
these observations support an initial-state replay gap; this pass does not
establish a complete native fix. Blank-frame timing values are invalid and excluded.

The comparison below explicitly performs the same Pane/Normal bootstrap in
every variant before measuring. Four runs use shared/control/control/shared;
two subsequent runs isolate decoding. No heavy builds overlap these runs.
Each checkpoint exercises eight numbered, eight walking and eight click switches.
All fixtures retain the full top-level UI, 59 explicit bindings and native defaults.
They use disposable homes and empty zoxide histories, without an agent workload.

| Endpoint at 16 tabs, two runs per variant | Legacy control | Faster decoder only | Shared hints |
| --- | ---: | ---: | ---: |
| Native-server RSS | 355.6–355.8 MiB | 325.2–325.7 MiB | 196.8–198.1 MiB |
| Observed new-tab endpoints | 553–569 ms | 460–490 ms | 327–386 ms |
| Numbered-switch correct-bar median | 95–105 ms | 85–99 ms | 64–109 ms |
| Walking-switch correct-bar median | 88–95 ms | 91–107 ms | 78–112 ms |
| Click-switch correct-bar median | 92–105 ms | 97–103 ms | 78–120 ms |

Sharing saves about 158 MiB of native-server RSS at 16 tabs relative to the
matched legacy control; decoding alone saves about 30 MiB. Creation endpoints
improve, but switch latency has no consistent large-tab benefit. At two tabs,
shared numbered/walking medians are 64–83 ms versus legacy 42–53 ms: this prototype
does not improve the small-session interaction the user already finds fast.
The 5.6-second control body-startup outlier remains in the raw records.

Creation clocks include native CLI dispatch and polling until body text, the
destination bar highlight, a Radar header and Normal hints are present. They
do not prove Radar row freshness, presentation order or Rio latency. Switch
inputs wait for readiness and are not rapid-input tests. Host load and background
I/O vary; two runs per variant do not establish a latency guarantee. RSS and
idle samples describe the native server, not Rio or all workspace processes.

Frame equality alone cannot remove the per-tab Wasm contexts measured here.
Radar already gates unchanged render inputs; a changed active-tab marker also
changes its frame. A custom visibility service would duplicate the native owner.
Retain controller/view Radar and Bar and direct per-tab hints. Reconsider one
shared migration only after permission/initial-state replay and the child's
nested-session API preserve the existing contract. A future candidate still
needs independent-client, permission-denial, resize, hide/restore, popup, nested
and lifecycle checks, exact Linux/Darwin gates and fresh installed GUI proof.
Those checks were not run for a prototype already rejected at first activation.

Commands, source, artifact identities, raw captures, failures and reproducible
analysis: `/home/lucca/.local/state/codex/proofs/nova-shared-rendering-20261013._aks6_vq`.
`analysis.json` separates recovered measurements from failed fresh startup;
`minimal-shared-1/minimal-before-input-screen.txt` and the following mode captures
preserve the smallest failure discriminator. The native executable SHA-256 is
`ec19a96362ad2dc3978fe137663d6e2a14bde135f49e30e6ad849eef54effc6e`.

## Native release-profile comparison

Owner: `yazelix-5vl-native-release-profile-4bea` (closed). The native wrapper
sets `lto = true` and `codegen-units = 1` at the application root, matching
pinned Zellij's release settings;
[Cargo's profile rules](https://doc.rust-lang.org/cargo/reference/profiles.html)
make that root the effective owner. The control already used release
optimization level 3, 16 codegen units, and thin local LTO, so this is not a
debug-to-release comparison. Control and candidate (Edge `a6623698`) differed
only in the wrapper; all 16 embedded Wasm plugins were byte-identical. Eight
fresh installed sessions in control/candidate/candidate/control order and its
reverse each created a second tab and ran eight numbered, walking, and click
switches with all 59 explicit bindings.

| Endpoint | Control | Candidate |
| --- | ---: | ---: |
| Native/package build wall time, one trial each | 6m27s | 9m50s |
| Stripped native executable | 76,362,024 bytes | 68,493,544 bytes |
| Fresh `enter` body, median (range), four sessions | 501 ms (486–512) | 430 ms (419–448) |
| New-tab body, median | 93 ms | 91 ms |
| Correct new-tab bar selection, median | 134 ms | 117 ms |
| All new-tab endpoints observed, median (range) | 275 ms (251–297) | 271 ms (237–279) |
| Numbered switch correct-bar median, 32 switches | 42 ms | 36 ms |
| Walking switch correct-bar median, 32 switches | 43 ms | 41 ms |
| Click switch correct-bar median, 32 switches | 54 ms | 51 ms |
| Observed live-thread CPU during ~3s after creation, median | 555 ms | 505 ms |
| Settled native-server CPU, median of twelve 2s windows | 0.5% of one core | 0.5% of one core |

Fresh terminal startup improved by about 71 ms at the median (14%), and the
executable is 10.3% smaller, so the settings stay. Build time rose by 3m23s
(52%) in one warm build per variant. New-tab and switch ranges overlap,
walking/click medians were slightly worse, and there is no idle gain, so this is
not an overall UI speedup. Accepted on Edge at `47fe1ecd` with
[Linux CI](https://github.com/Yazelix/nova/actions/runs/38332459290) and
[Darwin Package Smoke](https://github.com/Yazelix/nova/actions/runs/38332456177).
Proof: `nova-native-profile-20261013.0zf4p9r2`.

## Bottom-hints follow-up

Owner: `yazelix-5vl-hints-tab-work-jvpj`. The isolated
[lightweight event patch](../prototypes/zellij-distribution/zjhints-lightweight-events.patch)
uses the existing SDK's `InitialKeybinds` for the complete catalog and keeps it
across lightweight metadata events. Either startup event order, remaps and an
empty replacement catalog are covered. Native reconfiguration remains the owner
of catalog delivery; there is no new SDK, dependency or Nova config field.

Nova's direct pane sets the existing `pipe_name` to `""`. This disables unused
compatibility broadcasts and the session subscription. It settles after actual
mode metadata arrives, including empty/base Locked output. Unset and custom
names retain the child's publication and compatibility retries. Focus state is
still tracked, but it redraws hints only when the host is descended into a guest.
Width, theme, mode and nested-session updates still request the relevant redraw.
Permission startup keeps the pane selectable until the permission result and
does not render hints before permission-gated metadata arrives.

Separable catalog/direct-pane/output-cache variants were built with Rust 1.96.0
against the existing installed native binary. The output cache was rejected:
both trials made ten derivations in ten renders, giving zero cache hits. Skipping
irrelevant redraws needs no derived-output state or invalidation framework.

Two final traced pairs retain all 59 explicit bindings and native defaults.
Each creates a tab, runs 24 numbered/walking/click switches, reloads a remapped
Alt+9 binding, and checks Pane/Normal hints and Radar's session identity.

| Diagnostic endpoint | Control | Candidate |
| --- | ---: | ---: |
| Settled hints mode payload | 88,936 bytes | 830 bytes |
| Mode events requesting redraw during settled switches | 24 | 0 |
| Completed hints renders in captured run | 294 | 10 |
| Compatibility publications | 588 | 0 |
| Session events delivered to hints | 144 | 0 |
| Separate catalog deliveries, including reload | 0 | 4 |

Counts describe captured trace prefixes, not an overall UI speedup. Native
forced renders can still derive hints; no output cache is claimed. The traced
candidate had one 17.1-second new-plugin startup outlier after its body appeared
in 160 ms; it eventually converged. Another candidate selected the new tab in
867 ms. Controls selected it in 423 and 898 ms. These instrumented trials had
system load 5.13–10.02 on 16 logical CPUs, with concurrent release archive I/O.
The outlier is retained; it cannot be silently excluded or attributed to disk
activity without evidence. The bounded production follow-up below checks
recurrence without changing deadlines.

### Uninstrumented Nix-plugin comparison

After the archive work and Nix build finished, a control/candidate/candidate/control
sequence used the production Nix Wasm for all three plugins. Hints were installed
in an isolated profile; candidate source matched the applied patch exactly.
The existing native package remained
`/nix/store/8ycc2pycq5wmqj7qp2h3gv6191f2r0xf-yazelix`. Each probe used explicit
file-plugin URLs and retained all 59 bindings plus native defaults.
There were no overlapping heavy builds; one-minute load was 5.33–7.39 on 16 CPUs.
Creation values share a clock started before dispatching the new-tab action;
they are not additional delays after the body. Switch values start before input.
Endpoints use separate captures; their polling times do not prove presentation order.

| Observed endpoint | Control, two runs | Candidate, two runs |
| --- | ---: | ---: |
| New-tab body | 166–181 ms | 130–136 ms |
| New-tab bar label | 433–560 ms | 186–193 ms |
| Correct new-tab bar selection | 437–565 ms | 172–197 ms |
| New-tab hints | 236–303 ms | 186–211 ms |
| New-tab Radar | 489–539 ms | 186–212 ms |
| Numbered switch correct-bar median | 103–119 ms | 40–46 ms |
| Walking switch correct-bar median | 103–129 ms | 46–51 ms |
| Click switch correct-bar median | 116–122 ms | 39–66 ms |

The 17-second outlier did not recur in these two candidate sessions; its cause
remains unresolved. Body-switch medians did not consistently improve: control
19–26 ms, candidate 24–41 ms across methods/runs. These are small Linux terminal
observations with polling and IPC overhead, not Rio presentation timings,
macOS performance, a latency guarantee or proof of a complete UI fix.

Measured hints package:
`/nix/store/aiy8mc85bk739fyh1j80zqrnfpp7wa0f-nova-zjhints-0.5.0`.
Wasm SHA-256: `5e8509f993e981e1c4ea59becb87e6ab5bbbda9dea6e55450697b89b734904bc`.
`production-analysis.json` records independent body/plugin/selection observations.

The Nix Wasm build and all 148 native renderer/model tests pass, including catalog
ordering, reload, empty/Locked readiness, default pipe retries, resize and nested
focus. Existing `bottom_hints` and `attached_tab_focus` checks pass against the
installed file-plugin fixture: every minor mode, fitting, popup/focus, hide/restore,
pane order and multiple/mirrored clients remain covered. A fresh uncached file
plugin displays its permission prompt, accepts a keyboard grant and renders
Normal/Pane hints. Flake shape and all-system output evaluation pass.
Full rebuilt-distribution,
installed-launcher, fresh private-display and exact Linux/Darwin acceptance
proof remain separate from these existing-native/file-plugin probes.

Raw source, variant identities, logs, snapshots, elapsed times and analysis:
`/home/lucca/.local/state/codex/proofs/nova-hints-performance-20261012.nm_auw6j`.
`ablation-analysis.json` preserves exploratory variants; `final-analysis.json`
preserves the final traced pairs and the negative startup result.

## Lightweight mode-event implementation

Owner: `yazelix-5vl-tab-mode-events-mbhl` (closed). Radar
[`3ad74c18`](https://github.com/Yazelix/zj-radar/commit/3ad74c18e44cf97cdd8c6ea9121f93c5c7135dd0)
and zjstatus
[`8ccf986f`](https://github.com/Yazelix/zjstatus/commit/8ccf986f7d9b9c27f278d618db45d7b05fc95302)
subscribe to `InitialKeybinds` in their stateful roles, so the native host
delivers the keymap separately and omits it from frequent mode events; both
ignore it. Views, SDK versions, host floor, native keymaps, and hints are
unchanged.

Two instrumented control/candidate pairs (diagnostic timings, full keymap, 24
switches each):

| Endpoint | Control | Candidate |
| --- | ---: | ---: |
| Settled mode payload, both controllers | 88,936 bytes | 830 bytes |
| Radar settled decode median per run | 72.4–81.7 ms | 0.75–0.78 ms |
| Bar settled decode median per run | 40.1–44.1 ms | 0.74–1.00 ms |
| New-tab body | 158–163 ms | 154–157 ms |
| New-tab correct bar | 416–468 ms | 392–427 ms |
| New-tab Radar | 517–672 ms | 481–502 ms |
| Switch correct-bar medians across kinds/runs | 103–126 ms | 94–114 ms |
| Worst observed correct-bar switch | 212 ms | 167 ms |

Two uninstrumented installed pairs (`d372ddd0`, full configuration, 24 switches
each):

| Endpoint | Original desktop artifact | Candidate |
| --- | ---: | ---: |
| New-tab body | 122–130 ms | 122–146 ms |
| New-tab correct bar | 1,158–1,316 ms | 450–773 ms |
| New-tab Radar | 1,539–1,768 ms | 877–977 ms |
| New-tab hints | 1,319–1,519 ms | 497–685 ms |
| Switch correct-bar medians across kinds/runs | 243–285 ms | 156–195 ms |
| Worst observed correct-bar switch | 587 ms | 391 ms |

Mode events stay below 1 KB across style and minor-mode changes. Reload,
Pane/Normal hints, and Radar's session name pass assertions, and the
attached-client check covers both clients through reload, detach, and reattach.
A fresh private-display Kinestra run of the installed `launch` passed in 18.5
seconds; this proves interaction correctness, not GUI latency. Proof:
`nova-light-mode-20261012.gwbR4tSP` (matched pairs `candidate-3/4` and
`control-3/4`) and `nova-mode-review-20261012.x7x33fct`.

## Original investigation

Owner: `yazelix-5vl-tab-performance-investigation-yia1` (closed). These
observations describe the original desktop artifact (Nova 1.3.0 Edge, native
SHA-256 `509634058144474e44431d48707cdaf0920694a3f03bba9026c1cbb32f76b3ad`)
and temporary prototypes. They explain the chosen changes, not the final
candidate. The reproduction bypassed Rio, so Rio is not needed for the blank
plugin UI; its contribution remains unmeasured. A large zoxide history was not
a prerequisite. Proof: `nova-performance-20261012.mtW9Bi5s`.

The user reported slow tab creation, walking, numbered switching, and clicking
from the Edge desktop entry. An optimization must keep tab contents, focus
policy, Radar, top-bar interactions, and hints; no numerical latency target was
accepted. Pruning the user's bindings is not a fix.

| Endpoint | Observation | What it does not establish |
| --- | --- | --- |
| Native tab creation | New stable tab/pane IDs exist | Visible content or plugin readiness |
| First terminal body | Target Yazi/picker or unique terminal marker appears | Input echo, correct bar/Radar, GUI presentation |
| First plugin frame | Bar labels, Radar rows, and hints replace placeholders | All three agree with the current selection |
| Completed switch | Target body and active tab highlight agree | Rio/compositor presentation or every sidebar field |
| Input acknowledgement | A harmless input produces its expected response in the target | Not measured in the creation probes |

Each tab has three plugin panes (bar, Radar view, and hints) plus long-lived
background controllers, and new tabs start managed Yazi with
`--yzx-startup-picker`:

| Trigger | Owner before the shared native switch/create path |
| --- | --- |
| `Alt Shift T`, tab-mode `n` | Native `NewTab`, installed layout, home cwd |
| `Alt 1`…`Alt 9` | Native `GoToTab` |
| `Alt h/l`, left/right | Pane orchestrator: focus a pane or cross a tab boundary |
| Top-bar tab click | Bar view → bar controller → native action |
| Radar row click | Radar view → Radar controller → native action |
| `Ctrl Tab`, `Ctrl Shift Tab` | Radar cross-session cycling; a separate operation |

Findings:

- **Body versus plugins.** The installed launcher showed a new tab's body in
  about 120 ms, while the bar took 1.16–1.25 s and Radar 1.62–1.72 s. Replacing
  Radar with a terminal still left a 0.51–0.85 s bar delay.
- **Interpreted plugin work.** Two plugin executor threads accumulated about
  1.3–1.9 CPU seconds in the first three seconds. A three-second `perf` profile
  put 91% of native user CPU in `wasmi` instruction execution under
  `apply_event_to_plugin`.
- **Settled switching.** Body medians were 19–28 ms; correct-bar medians were
  297–338 ms, with maxima of 793–834 ms.
- **Mode payloads.** Every full mode event carried an 89,496-byte protobuf
  keymap. Decoding took 88–120 ms per call in Radar's controller, 39–44 ms in
  Bar's, and 47–73 ms in hints, although Radar used only the session name and
  Bar discarded the bindings. A three-rule keymap configured before startup
  cut correct-bar switch medians from 111–122 ms to 40–41 ms. A Radar-only
  `InitialKeybinds` prototype removed its decode cost but not the switch delay
  while Bar and hints kept theirs.
- **Hints amplification.** Hints readiness stayed false in Normal mode, so
  ignored `SessionUpdate` events requested renders (33 renders, 620 ms in
  aggregate, in three seconds). Each render broadcast unused `zjhints` and
  `zjstatus_hints` compatibility pipes, which Bar's controller stored and
  republished to every view. Single-change prototypes improved some runs but
  not consistently.
- **Frame fan-out.** Radar and Bar controllers render once per size but send
  frames to every registered view without equality checks; Bar also refreshes
  each second. Native pane/tab/mode delivery filters inactive tab plugins, but
  session updates reach every subscriber.
- **Not causes.** The orchestrator's 500 ms timer is a recovery timer, not a
  per-keystroke delay; Yazi's two-second size loop is a deadline, not a sleep.
  Synthetic 3- and 12-tab body timings were similar across native, Radar-only,
  and bar-only graphs and do not clear the combined graph.
- **Build profile.** The native wrapper lacked upstream's `lto = true` and
  `codegen-units = 1`; Wasmi recommends both for its embedding application.

Pinned source at investigation: Radar `9beefdf9`, pane orchestrator
`f47af174`, Nova Bar `7295e997`, zjstatus `0338e5ea`, native Zellij `333d7898`
plus local patches, and hints `709d5629` plus the grouping patch.

## Further findings and improvement options

Upstream's [Wasmtime-to-Wasmi migration](https://github.com/zellij-org/zellij/pull/4449)
traded compilation and cache machinery for interpretation and documented a cost
for compute-heavy plugins.

### Upstream already owns shared plugin instances

Zellij merged [resource usage optimizations, PR #5662](https://github.com/zellij-org/zellij/pull/5662)
on 2026-09-28, before Nova's 2026-10-09 native pin. That pin contains
`ZellijSharedPlugin`, `register_shared_plugin!`, slots and client context.
The native host deduplicates equal mode/tab/pane events before calling a shared
instance, merges render requests, and renders visible slot/client targets.
The implementation is in pinned `zellij-tile/src/lib.rs` and
`zellij-server/src/plugins/shared.rs`.

Radar and Bar currently implement their own controller/view fan-out; hints is
instantiated per tab. They do not opt into this API. Updating the native pin
alone therefore cannot remove their instances or message protocol. Migrating
a plugin could replace that protocol with native slot ownership. This requires
SDK and child compatibility decisions, not just a layout flag. Radar's declared
0.44.3 host floor cannot silently be dropped.

There is also a decoder difference: the legacy registration macro parses a
JSON array of protobuf bytes, then decodes and converts the event. The current
shared macro uses `protobuf_bytes_from_stdin` and `event_from_protobuf_bytes`
instead. A newer SDK alone leaves the legacy macro path in place. Measure any
decoder benefit separately from instance deduplication and render batching.

### Options, in the order I would evaluate them

| Option | Owner and concrete change | Expected value | Cost and proof boundary |
| --- | --- | --- | --- |
| Remove avoidable plugin work | Radar/Bar `InitialKeybinds` is installed; hints catalog, unused broadcasts and readiness/subscriptions are implemented above | Reduces decoding and render amplification while retaining the UI | Hints acceptance is pending. Output caching was rejected. Preserve reload, nested sessions and compatibility-pipe consumers. |
| Restore native release optimizations | Native root fat LTO and one codegen unit are measured above; no hints profile change | Consistent fresh terminal-startup gain and smaller executable; tab-creation/walking benefit remains uncertain | Accepted on Edge with exact Linux/Darwin checks and installed desktop proof. Two Cargo settings, 3m23s longer measured build. |
| Adopt native shared instances | Migrate one expensive plugin to `ZellijSharedPlugin`; delete its controller/view protocol when equivalent | Removes per-tab Wasm initialization, frame messages and duplicate state delivery through an existing host owner | Medium to large child change. Prove client-local selection, slot lifecycle, resize, click targets and compatibility policy. |
| Reduce legacy frame fan-out | Radar/Bar: skip unchanged frames, use visibility, refresh immediately on activation | Less pipe decoding and rendering, especially with many tabs | Medium. Equality must include interaction targets. Prove first frame, activation and multiple clients. Avoid building a second visibility owner if migrating to shared instances. |
| Upgrade Wasmi | Native Zellij: evaluate Wasmi/WASI 2.x with correct features | Faster interpreter without replacing the plugin model | Medium upstream/fork work. Nova pins 1.1.0; migration is not a version-only edit. Upstream benchmark gains do not predict Nova latency. |
| Use a smaller UI graph | Existing configuration can replace Radar; optionally evaluate omitting hints | Immediate reduction in plugin work | Loses visible functionality. Radar-off still left 0.51–0.85 s bar delay. This is a user-selected tradeoff. |
| Compiled Wasm engine | Native host: evaluate Wasmtime with build-time compilation or caching | Potentially reduces sustained callback execution cost | Large engine, packaging and maintenance change. Keep behind evidence from the cheaper options. |

The [Wasmi 2.0 release](https://github.com/wasmi-labs/wasmi/releases/tag/v2.0.0)
reports roughly 2.2× geometric-mean execution improvement on its benchmark suite.
Its [migration guide](https://github.com/wasmi-labs/wasmi/blob/v2.0.0/docs/migration-v1-to-v2.md)
covers changed resource limits, WASI and feature selection. Nova's native
dependency disables default features; a migration must deliberately retain
validation and select the intended dispatch path. No Wasmi 2.x Nova candidate
has been built or measured.

Wasmtime documents [precompilation](https://docs.wasmtime.dev/examples-pre-compiling-wasm.html)
and [caching/Winch](https://docs.wasmtime.dev/examples-fast-compilation.html)
as ways to control compilation latency. Precompiled artifacts require matching
engine configuration and trusted provenance; external plugins need an explicit
loading policy. This would enlarge the native fork and require Linux/macOS
delivery proof. A general engine benchmark is not proof that this change beats
fixing redundant callbacks in Nova.

## Next experiment and decision boundary

Finish exact-revision acceptance of the hints candidate
before choosing one plugin's shared-instance prototype. Hints output caching
has been rejected; the native profile comparison remains separate from plugin
changes. Retain the full keymap/UI
and test reload, minor modes, nested sessions, permission startup, tab closure and
multiple clients alongside creation and all three switch methods.

The remaining trace gap is native queue waiting and presentation. Correlate
action receipt, plugin enqueue/start/end, worker/lock waiting, view registration,
frame send/receipt and client output using stable IDs and monotonic timestamps.
Use private Kinestra for Rio presentation. Do not interpret elapsed callback
sums as critical-path wall time or add threads before measuring contention.

Repeat paired comparisons at 1, 3, 12 and a representative larger tab count,
including rapid inputs during convergence. Check warm switches separately from
new tabs. Then extend to long-lived and busy-agent sessions. If frame fan-out
remains expensive, compare a one-plugin shared-instance prototype with legacy
frame equality/visibility gating; avoid two owners for visibility and client state.

Future candidates need exact installed-launcher proof with body, correct plugin
frames and input acknowledgement; GUI latency needs timed private Kinestra
presentation observations. Shared changes require Linux and Darwin gates.
Defer extra executor threads and engine replacement until measurements justify them.

## Limits and excluded evidence

- Host: 16 logical CPUs, approximately 31 GiB RAM, tmux 3.4, Rust 1.96.0.
  Background load varied during the original investigation; matched candidate
  comparisons excluded heavy builds. No p95 or p99 claim follows from these samples.
- Tests used disposable state and small histories. The user’s long-lived session,
  many projects, agent activity, and actual tab count remain unmeasured.
- Terminal captures prove native output, not the instant Rio presents pixels.
  Radar/Bar has rebuilt-distribution, private GUI and user desktop delivery proof.
  The current Edge desktop includes hints and native-profile changes, with
  delivery evidence recorded above. Hints acceptance remains with its owning bead.
  There is no timed GUI comparison or Darwin performance measurement. Temporary
  instrumented builds support experiments only and are absent from production artifacts.
- Shared `/tmp` logs contained top-bar panics, but the isolated reproductions
  have no corresponding crashes. They do not establish the cause of this report.
- The first synthetic harness counted an intentional one-second attachment
  settling delay in `STARTUP`. Those values are excluded from startup claims.
  An early “native-only” attempt retained the inline bar controller and is
  labeled invalid in the proof directory.

The native release-profile candidate has exact Linux/Darwin package checks,
installed Edge desktop proof and qualitative manual dogfood. Darwin interaction
and performance remain unverified.
