# Vision: constellation — wintermute grows beyond one laptop

**Authored by:** /dream (Claude Opus 4.8), with jsy
**Created:** 2026-06-04
**Updated:** 2026-06-20 — fleet-join pass; PRDs re-derived from live machine state
**Status:** active
**Seed:** jsy — *"I need you to expand across multiple computers. This laptop is
too resource constrained... I have a 32GB AMD desktop with a medium Radeon GPU.
...best way to install wintermute linux on this and other computers, and for you
to communicate across all of them. I will primarily use voice control so that
should start immediately on boot. Make them identical in appearance — i3, the
terminal window shapes, all taskbar tools — everything about you. The goal is to
enable you to coordinate, collaborate and distribute workloads to maximize
development throughput. We can also setup a cloud service... /dream of growing
beyond this laptop."* Grounded in deep external research (four parallel research
agents, 2026-06-04; citations throughout the fleet PRDs).

## TL;DR

wintermute today is one Intel laptop — 4 cores, 15GB RAM, an Intel iGPU with no
discrete GPU — and it shows: the local brain takes 20-30s per voice turn, builds
are slow, and there is nowhere to offload. constellation turns one laptop into a
**fleet** of three physical machines + an always-on cloud node, each provisioned
**identical** — same i3, same terminal geometry, same taskbar, same tools, voice
control live on boot — and **connected** so the wintermute agents on each see each
other, share one coordination bus, and **route work to where it belongs**.

**The fleet (scoped 2026-06-04 by jsy's decisions: cloud-first brain; the AMD
5700U is REMOVED — it's jsy's work machine; the GTX 1080 tower is DEFERRED — not
immediately needed). Near-term fleet = laptop + cloud:**

| Machine | Spec | Role | Status |
|---|---|---|---|
| **Laptop** | i7-10610U (4c/8t), 15GB, Intel iGPU, no dGPU | **Voice node**; brain = cloud | ACTIVE |
| **Cloud** | Hetzner CAX21 / Oracle free | **Always-on hub** (NATS + mesh exit) **+ the primary brain (Anthropic API, ~0.7s)** + build pods | ACTIVE |
| ~~5700U box~~ | Ryzen 7 5700U, 32GB, Vega 8 APU | ~~build workhorse~~ — **REMOVED** (jsy's work machine) | DROPPED |
| **i7 tower + GTX 1080** | 5th-gen i7, 32GB, GTX 1080 8GB | (future) local-gpu brain + fleet GPU for batch ML | **DEFERRED** |

**The resource-allocation spine (current):** the **brain is cloud-first** — the
Anthropic API (Haiku/Sonnet, ~0.7s) is faster *and* smarter than any model these
machines can host, and cheap at personal volume (~$3-11/mo); it is the primary
voice brain. The **laptop** is a thin voice node. The **cloud node** is the
always-on hub (NATS + mesh) and hosts the API brain + burst build pods. Heavy
builds run on **cloud build pods** (the 5700U that would have been the local
builder is gone). **Deferred for later:** when the GTX 1080 tower is picked up, it
adds a `local-gpu` tier (8B Q4, ~2-3s) *below* cloud in the ladder — a
private/offline brain + the fleet's free GPU for batch ML (homeward embeddings,
training) — but it is NOT needed for the near-term fleet to work.

## Why now (Phase 1 research, 2026-06-04)

- **The laptop is genuinely the constraint.** Confirmed live: 8-core Intel,
  15GB RAM, Intel UHD iGPU (no discrete GPU). The voice memory already records
  20-30s/turn local brain and the decision to route voice to cloud purely for
  latency. A 32GB AMD desktop with a Radeon changes the economics.
- **The bus is single-host by construction.** agorabus runs *only* over a Unix
  domain socket (`~/.cache/agorabus/sock`) — its own README says "co-located
  sessions." To coordinate across machines the bus needs a network transport.
  Research validated **NATS** (subject pub/sub maps directly onto `wm.*`, plus
  request/reply, queue groups, JetStream durable work-queues + KV) over MQTT/
  Redis — but found NATS has **no Unix-socket listener**, so the integration is a
  small **agorabus↔NATS bridge** sidecar that keeps every existing local UDS
  client unchanged. That bridge is the keystone new component.
- **Identical appearance is a solved problem with the right tools.** Research
  recommends **Arch + Ansible + chezmoi**, NOT NixOS: the custom `linux-wintermute`
  kernel PKGBUILD (which already exists) is exactly where Nix adds work (compile-
  on-rebuild) rather than removing it, and `~/wintermute/dotfiles/` +
  `wintermute-desktop` already hold the i3/desktop config to template. chezmoi is
  the only dotfile manager that does the per-host templating "identical config,
  different GPU/monitor" requires.
- **Voice-on-boot is a known wiring.** greetd `initial_session` autologin → i3 →
  `systemctl --user start wintermute.target`, with the documented i3→
  `graphical-session.target` bridge fix (i3 issue #5186). Matches the existing
  `wm-audio`/`wm-stt`/`agorabus.service` + `wintermute.target` daemon model.
- **The "desktop" is an APU, not a GPU box (corrected 2026-06-04).** It is a Ryzen
  7 5700U (Zen 2 / Lucienne, Vega 8 iGPU, gfx90c, no discrete GPU, ~51 GB/s shared
  DDR4). Research verdict: **llama.cpp Vulkan** runs on the Vega iGPU but token
  generation is bandwidth-bound, so iGPU offload gives ~2× prompt-prefill and
  **~zero generation speedup** over CPU — both ~8-10 tok/s on a 7-8B Q4 (vs 35-50
  on a real dGPU). ROCm on gfx90c needs `HSA_OVERRIDE_GFX_VERSION=9.0.0` and is
  unreliable — use Vulkan/CPU. So the local box is a **dedicated local-LLM node**
  (it can run **qwen2.5-8B Q4** at ~8-10 tok/s *only if build activity is stopped*
  — the user's own call: not enough cores/RAM for build+model at once), serving a
  privacy/offline/cheap-default `local-llm` tier + command routing — NOT the fast
  latency brain. Per-host role isolation: this node serves the model and does NOT
  run heavy builds.
- **Heavy builds belong in the cloud.** Because the local box is dedicated to the
  model, Rust/CI/ML build jobs are pushed to **burst cloud pods** (CPU for
  compilation, GPU on-demand for ML) via the dispatch layer + sccache-dist build
  servers in the cloud — freeing local cores for inference.
- **The cloud node should be cheap-coordinator + API-brain, not a GPU.** Research
  is decisive: at personal voice volume the Anthropic API costs ~$3-11/mo and
  beats any rentable GPU by 20-40×; self-hosting a big model 24/7 is $200-940/mo
  for a *worse* brain. So: a ~€8/mo Hetzner (or free Oracle) always-on node hosts
  the NATS hub + mesh exit + a small offline-fallback brain; the latency brain
  stays the Anthropic API (already wired, `WM_ANTHROPIC_API_KEY` live); GPU pods
  burst on-demand only for build/ML jobs.

## End-state

When constellation is fulfilled:

1. **Any new machine becomes an identical wintermute node** from a golden ISO +
   one Ansible run — same kernel, same i3, same terminals, same taskbar, same
   tools — and **boots straight into live voice control**.
2. **All nodes share one mesh and one bus.** A `wm.*` event published on any
   machine is visible fleet-wide; every local UDS client keeps working unchanged.
3. **The weak machines borrow the strong one's brain.** The laptop's voice turns
   are served by the desktop's Radeon (~2-4s) or the cloud API, transparently, via
   the existing brain ladder.
4. **Work flows to capacity.** Builds, tests, and ML/embedding jobs are dispatched
   to whichever node has the cores/VRAM/headroom; Rust compilation is shared
   across nodes via a distributed cache.
5. **An always-on cloud node** is the hub that keeps the fleet coherent even when
   personal machines sleep, at a few dollars a month.
6. **Development throughput is the fleet's sum** — the laptop is no longer the
   ceiling.

## Operational hardening (added 2026-06-06 by /dream — the second-node layer)

The base components stand up + connect + coordinate a fleet; these three close the
seams that block a *real, secure* second node from joining. Each answers one of the
vision's own Open questions and slots onto a shipped/in-flight base component:

- **constellation-secrets** — the root-key bootstrap (one `age` identity delivered
  out-of-band per host) + a `sops`-encrypted **service**-secret store (NATS creds,
  mesh auth keys, `WM_ANTHROPIC_API_KEY`). appearance's chezmoi-`age` decrypts only
  *dotfile tokens* and only *after* the host key exists; this is the layer that puts
  the key there and manages the non-dotfile secrets mesh AC1 / the bus / the brain
  all consume. **Prerequisite for everything multi-host.**
- **constellation-headscale** — stands up + operates the self-hosted Headscale
  control server on the cloud node. mesh exposes a client *flag* to point at it
  (AC9) but never builds the server; this is the server half — persistent state,
  ACL parity with mesh, pre-auth key issuance into the secret store, node lifecycle.
  Makes the sovereignty option real, not documented-only.
- **constellation-voice-role** — a per-host `voice_node` flag making boot-to-voice
  conditional. provision boots *every* node into the mic/STT stack; the cloud node
  has no mic and a compute node shouldn't burn cores listening. Aligns voice-on-boot
  with the per-host role axis mesh/chezmoi/dispatch already use.

Order: secrets FIRST (Headscale's keys + mesh enrollment + the bus creds all live in
it) → headscale (consumes secrets, refines mesh) ‖ voice-role (independent; refines
provision). All three `build_target: shell`, siblings of the base constellation set.

## Day-2 operations layer (added 2026-06-12 by /dream — the live-fleet layer)

The base + hardening fleets stand up, connect, secure, and coordinate nodes —
but **every acceptance test is an offline structural gate** (`tests/*-check.sh`
assert config shape without a live second node). The fleet has never been
*operated*. These four close that gap: they make a real running fleet
**observable**, prove a node **actually joined** end-to-end, and keep the fleet
**coherent when the hub blinks** — the day-2 concerns that only appear once the
machinery is real. Each is grounded in a concrete seam found live 2026-06-12:

- **constellation-status** — `dispatch.rs` runs a capability heartbeat that
  samples local hardware and writes a NATS KV node registry — but **nothing reads
  it for a human**. `mesh status.sh` only pings MagicDNS names. There is no single
  view of *which nodes are up, their role, load, heartbeat-age, and queue depth*.
  A `wm-busbridge status` reader turns the live KV into one fleet dashboard.
- **constellation-join-check** — every test under `constellation/tests/` is an
  offline `*-role-check.sh` structural gate. None of them prove a freshly
  provisioned node **actually joined**: mesh reachable + a `wm.fleet.*` event
  round-tripped through the hub + secrets decrypt + brain route live. This is the
  first end-to-end *live* acceptance — the test a new node runs to earn membership.
- **constellation-hub-failover** — `cloud-hub.md` says the on-hub ollama is the
  "degraded path only," but **nothing triggers the degrade**. When the cloud hub
  (NATS + API-brain reach) goes unreachable, the laptop should detect it, emit
  `wm.fleet.hub.down`, fall back to local-only, buffer fleet events, and auto-rejoin
  + flush on `wm.fleet.hub.up`. The brain ladder has tiers but no hub-liveness watcher.
- **constellation-fleet-doctor** — there are seven separate `*-check.sh` scripts and
  no single "is the fleet healthy, and if not, which layer broke" entrypoint. A
  `constellation doctor` composes mesh + bus + secrets + brain + dispatch probes and
  **localizes the broken seam** (the assay/quicken ethos applied to the fleet).

Order: status + fleet-doctor are independent read-only views (build any time);
join-check consumes status's KV read; hub-failover is independent (watcher daemon).
All slot onto shipped base components — no new base work required.

## Components (one bullet per PRD)

- **constellation-provision** — Ansible control plane + local pacman repo for the
  custom kernel + golden archiso + greetd→i3→`wintermute.target` boot-to-voice.
- **constellation-appearance** — chezmoi-templated dotfiles for pixel-identical
  i3 / terminal geometry / taskbar / tools across hosts, with per-host templating.
- **constellation-mesh** — Tailscale (MagicDNS, ACLs, exit node) joining every
  node into one private network with stable names.
- **constellation-bus** — the agorabus↔NATS bridge daemon + NATS hub/leaf config
  + JetStream, carrying `wm.*` fleet-wide while local UDS clients stay unchanged.
- **constellation-brain-gpu** — *SUPERSEDED* (assumed a discrete Radeon). Archive.
- **constellation-brain-local** — *ABANDONED 2026-06-04* — it ran qwen2.5-8B on the
  5700U, which has been removed from the fleet. Do not build. Archive.
- **constellation-brain-cuda** — *DEFERRED 2026-06-04* (the tower is not immediately
  needed). When picked up: nvidia-dkms on `linux-wintermute` + llama.cpp/ollama CUDA
  on the GTX 1080 serving 8B Q4 as a `local-gpu` tier **below cloud** (cloud-first);
  a private/offline brain + the fleet's batch-ML GPU. Kept drafted for later.
- **constellation-cloud** — the always-on cheap cloud node: NATS hub + mesh exit
  + **the primary Anthropic-API brain** + burst build pods, provisioned by Ansible.
  *(Near-term this is the most valuable node — it's the brain + hub.)*
- **constellation-cloud-build** *(refines dispatch)* — **builds run on cloud pods**
  (sccache-dist + burst); the local-builder role is gone with the 5700U. Builds
  route to cloud (and the laptop when voice-idle), never to the tower.
- **constellation-dispatch** — JetStream work-queue + capability KV registry +
  sccache-dist distributed builds; near-term this spans laptop + cloud only.

## Order

```
constellation-provision ─► constellation-appearance
        │
        └─► constellation-mesh ─► constellation-bus ─► constellation-cloud
                                          ├─► constellation-brain-gpu
                                          └─► constellation-dispatch
```

provision stands up identical nodes; appearance perfects their look; mesh
connects them; bus is the coordination keystone; cloud, brain-gpu, and dispatch
each build on the bus.

## Open questions

- **Bit-for-bit vs convergent identical.** Arch+Ansible+chezmoi gives
  *functionally* identical, convergent machines, not cryptographically bit-
  identical (only NixOS or a pinned-mirror snapshot does that). Is "looks and
  behaves identical" enough, or is literal bit-identity a hard requirement? (Only
  the latter flips the recommendation to NixOS — at a multi-month migration cost.)
- **Mesh control plane sovereignty.** Tailscale (easiest) uses a third-party
  control plane; **Headscale** self-hosted on the cloud node keeps the same UX
  while owning the control. Which matters more — ops simplicity or sovereignty?
- ~~The exact Radeon model is unknown~~ **RESOLVED 2026-06-04: it is a Ryzen 7
  5700U APU (Vega 8 iGPU, no discrete GPU).** This downgrades the local-brain
  expectation to ~8-10 tok/s and drives the build-out/LLM-in split above. The
  open follow-on: is the dedicated `local-llm` tier worth the 6-8s/reply for
  privacy/offline use, or should the local box instead be a CPU build node and the
  brain stay fully cloud? (User's stated preference 2026-06-04: dedicate local box
  to the model, push builds to cloud — so local-llm it is, builds out.)
- **Secrets bootstrapping** — every host needs one root secret (age key / SSH key
  / Vault password) delivered out-of-band before it can decrypt the rest. What's
  the delivery channel (USB, manual paste, the cloud node's tunnel)?
- **Voice on every node?** The desktop and cloud node may not want a live mic.
  Voice-on-boot should be a per-host role flag (the laptop/companion devices are
  voice nodes; the desktop is a compute node, optionally voice).

---

## 2026-06-20 Update — fleet-join live probe

**What's already built (found via SSH, not assumed):**
- Tailscale mesh: **LIVE** — wintermute (100.114.123.20), ryzen-work (100.111.184.102), hub (100.66.158.49) all connected, direct path wintermute↔ryzen-work
- agorabus-nats-bridge v0.5.0: **BUILT on both machines** — binary at `target/release/wm-busbridge` on ryzen-work; `~/.local/bin/wm-busbridge` on wintermute
- nats-server: **installed on ryzen-work** at `~/.local/bin/nats-server`
- agorabus: **active on both machines** (0 fleet peers — bridge not wired)
- wm-busbridge.service + nats-leaf.service: **unit files exist on ryzen-work** (binary not installed → inactive)
- constellation repo with mesh/ansible/chezmoi: **exists on ryzen-work**

**The blocker:** hub:7422 and hub:4222 CLOSED, hub SSH not accessible → NATS hub not running anywhere → entire fleet bus dead despite all code being built

**Active fleet PRDs (5):**
1. PRD-constellation-nats-hub — run NATS hub on ryzen-work (nats-server already installed)
2. PRD-constellation-bus-ryzen — install wm-busbridge, start bridge on ryzen-work
3. PRD-constellation-bus-wintermute — configure wintermute as leaf to ryzen-work hub
4. PRD-constellation-wm-daemons-ryzen — deploy wm-* voice stack to ryzen-work
5. PRD-constellation-voice-boot-ryzen — greetd autologin + i3 + wintermute.target on ryzen-work

**Order:** nats-hub → bus-ryzen (parallel: bus-wintermute) → wm-daemons-ryzen → voice-boot-ryzen
