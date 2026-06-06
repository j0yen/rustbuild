# PRD: constellation-voice-role — voice on boot is a per-host role, not a fleet default

Status: Draft v0.1
build_target: shell
Vision: visions/constellation.md
Depends: PRD-constellation-provision.md (the boot-to-voice wiring this PRD makes
  role-conditional — provision must exist; this refines it).

## TL;DR

provision boots **every** node straight into live voice control —
greetd→i3→`wintermute.target` with the mic/STT stack up. But the fleet isn't all
voice nodes: the cloud node has no mic, and a compute/build node shouldn't bring up
`wm-audio`/`wm-stt` (it burns cores the node is meant to spend on inference or
builds, and listens on a mic that isn't there). `constellation-voice-role` makes
voice-on-boot a **per-host role flag**: voice nodes (laptop, future companions)
boot straight to listening; compute/cloud nodes come up headless-of-voice with the
coordination stack only. One declared variable, conditional unit enablement, no
duplicated host configs.

## Why this exists

The vision's last open question, and a real per-node mismatch provision doesn't
resolve:

> **Voice on every node?** The desktop and cloud node may not want a live mic.
> Voice-on-boot should be a per-host role flag (the laptop/companion devices are
> voice nodes; the desktop is a compute node, optionally voice).
> — `visions/constellation.md`, Open questions

- **provision is unconditional today.** `PRD-constellation-provision.md` scopes
  *"greetd→i3→`wintermute.target` boot-to-voice"* with no host-role conditioning —
  grepping the shipped PRD for `role` / `voice.*boot` / `mic` returns nothing. As
  written, the cloud node would try to start the same voice stack as the laptop.
- **the stack is real and not free.** Memory `project_voice_input_null_detectors`:
  the live voice TURN runs `wm-audio` + `wm-stt` (STT on `small.en`, *~1.5× CPU vs
  distil*). On a 4-core build/inference node those are cores spent listening to a
  mic that doesn't exist. The local-LLM node decision in the vision is explicitly
  *"serve the model and do NOT run heavy builds"* — the same role-isolation logic
  says it shouldn't run the voice front-end either.
- **the per-host axis already exists everywhere else.** mesh tags nodes
  `tag:laptop`/`tag:desktop`/`tag:cloud`; chezmoi templates per-host; dispatch binds
  job classes per node capability. Voice-on-boot is the one remaining capability
  that's still hard-wired on for all. This PRD aligns it with the existing role axis.
- **`graphical-session.target` bridge is the real wiring point.** The vision already
  documents the i3→`graphical-session.target` fix (i3 issue #5186) that lets
  `wintermute.target` start the voice user units. Making that bridge — and the voice
  units it pulls — conditional on the role flag is the minimal, surgical change.

## What this builds

A `voice_node` role variable threaded through the provision/appearance Ansible flow
and a small `constellation voice` helper:

- **The flag** — a per-host `voice_node: true|false` (default by role: `laptop`/
  `companion` → true, `desktop`/`cloud` → false), set in the host's Ansible vars /
  chezmoi host data, single source of truth.
- **Conditional enablement** — when `voice_node` is false, the provision role does
  **not** enable/start the voice user units (`wm-audio`, `wm-stt`, the wake/VAD
  chain) and does not wire the i3→`graphical-session.target`→voice bridge; the
  coordination stack (`agorabus`, the bus bridge, brain ladder, dispatch worker)
  still comes up. When true, the full boot-to-voice path is unchanged from provision.
- **Brain-only voice (optional middle ground)** — a documented mode where a non-mic
  node still *serves* a brain/STT endpoint over the mesh (consumed by voice nodes per
  mesh AC4) without running its **own** mic front-end — distinguishing "has a mic and
  listens" from "answers other nodes' turns." The flag governs the mic front-end; the
  served endpoint is governed by the node's brain/dispatch role, not this flag.
- **Verification helper** — `constellation voice status` reports, for this host,
  whether the voice front-end is expected (per role) and whether it is actually
  up/down — so a misconfigured node (mic stack running on the cloud, or *not* running
  on the laptop) surfaces as a one-line mismatch instead of a silent wrong-boot.
- **Idempotent re-role** — flipping a host's `voice_node` and re-applying converges:
  turning it off stops+disables the voice units; turning it on enables+starts them;
  neither leaves orphaned units or a half-state.

Non-goals: the voice stack itself (wm-audio/wm-stt, already shipped); the brain
ladder/serving (brain-cuda/cloud); mesh reachability of a served endpoint (mesh AC4).
This PRD delivers the **per-host on/off of the boot voice front-end** only.

## Acceptance criteria

1. A per-host `voice_node` boolean exists with role-based defaults (`laptop`/
   `companion` → true, `desktop`/`cloud` → false), set in one place (Ansible host
   vars / chezmoi host data), consumed by the provision flow.
2. With `voice_node: false`, applying the provision role does **not** enable or start
   the voice user units (`wm-audio`, `wm-stt`, wake/VAD) and does not wire the
   i3→`graphical-session.target`→voice bridge (asserted: units are `disabled`/inactive
   after apply).
3. With `voice_node: false`, the coordination stack (agorabus + bus bridge + brain
   ladder/dispatch as the node's role dictates) still comes up — voice-off does not
   take the node off the bus (asserted).
4. With `voice_node: true`, the full boot-to-voice path is unchanged from provision —
   the voice units are enabled and the i3 bridge is wired (asserted; a regression test
   that provision's existing boot-to-voice behavior is preserved for voice nodes).
5. `constellation voice status` reports, for the host, the **expected** voice
   front-end state (from the role flag) and the **actual** state (units up/down) and
   exits non-zero on a mismatch (mic stack running where it shouldn't, or absent where
   it should run).
6. Flipping `voice_node` and re-applying converges idempotently: false→true
   enables+starts the voice units; true→false stops+disables them; re-applying the
   same value is a no-op (asserted, no orphaned units / half-state).
7. The "serves a brain/STT endpoint without a local mic front-end" middle-ground is
   documented and shown to be governed by the node's brain/dispatch role, **not** the
   `voice_node` flag (so a no-mic node can still answer other nodes' turns).
8. No host config is duplicated to express the role: the difference between a voice
   node and a compute node is the single flag plus conditional enablement, not two
   divergent provision paths (asserted by inspection / a documented diff).
9. `constellation voice status` is `SIGPIPE`-safe (`self_sigpipe_panic_toolkit`) and
   read-only (it reports, it does not change unit state).
10. The change is additive to provision: a node with no `voice_node` set behaves as
    its role default, so existing single-laptop provisioning is unaffected
    (backward-compatible, asserted).
