# Vision: harbor — the burst fleet gets a permanent home port

**Authored by:** /dream (Claude Sonnet 4.6), with jsy
**Created:** 2026-06-12
**Status:** active
**Seed:** user-prompt — jsy: *"how do i upgrade to a permanent box?"* after
confirming the Hetzner burst box has been used (528 UP events since 2026-06-05).
Chose **Option A**: a small always-on cheap hub + keep ephemeral ccx53 bursts for
heavy compiles. This vision is the practical tooling for that upgrade.

## TL;DR

Today `wm-burst` / `/cloudbuild` is **destroy-only**: `create_pod` → build →
`destroy_pod`, 528 lifecycles logged, every one torn down to stop billing
(`provider.rs` exposes only create/destroy — no power-off, no persistent state, no
shared cache server). That's correct for "stop paying when idle," but it throws
away everything between builds: the sccache cache dies with each pod (cold compiles
every time), there's no in-DC git mirror so pods clone from GitHub, and the cloud
is invisible to the agorabus/`wm.*` bus so the laptop never knows a build node
exists. harbor adds a **permanent, cheap home port** — a ~€4/mo `cx22`-class box
that stays up — and reshapes the tooling around it: the hub hosts a **shared
distributed compile cache** (cold bursts hit warm cache), a **git+registry mirror**
(pods clone in-DC, and the laptop's 24 unpushed repos get a fallback remote), and a
**bus bridge + heartbeat** so the fleet sees the hub. Heavy compiles still burst a
ccx53 pod on demand — but now they're warm, mirrored, and visible. The destroy-only
flow becomes the exception, not the rule.

## Why now (Phase 1 research, 2026-06-12)

- **The box is real and heavily used.** `~/.config/wm-burst/cost.log`: 528 UP
  events, first 2026-06-05, latest 2026-06-12T22:16. Last two runs `ccx53`
  (32 vCPU/128GB) at `49.13.154.86`, both ~2 min. The burst pattern works; the
  user explicitly asked to make a box permanent.
- **The tooling is destroy-only by construction.** `constellation-burst-builder/
  src/provider.rs` exposes exactly `create_pod` + `destroy_pod` (idempotent
  destroy) — there is no `power_off`/`get_server`/reuse path. `BUILDER_IP` and
  `BUILDER_ID` in `.env` are empty ("server deleted 2026-06-05"). Nothing persists
  between bursts.
- **sccache is per-pod and thrown away.** `config.rs` has an `SccacheConfig`
  (endpoint/bucket/keys) but it points at an S3-style bucket that isn't stood up;
  in practice each pod's local sccache dies on destroy, so every cold burst
  recompiles from scratch. The README's own framing was "an always-on, cheap,
  dedicated cloud box" — the implementation never built the *always-on* half.
- **Pods clone from GitHub, and the laptop is behind.** Self-review (2026-06-12)
  flags **24 dirty repos, rollout + wintermute-desktop unpushed** run after run.
  A hub-side bare mirror gives build pods in-DC clones *and* the laptop a fallback
  remote for work that hasn't reached GitHub.
- **The cloud is invisible to the bus.** agorabus is UDS-only (`~/.cache/agorabus/
  sock`, "co-located sessions"); 13 peers on the bus this session, all local. The
  burst box never announces itself — the laptop can't ask "is a builder up?" over
  the bus. constellation's vision names the agorabus↔NATS bridge the keystone;
  harbor stands up its landing pad (NATS on the hub + a heartbeat).
- **The cost model changes from per-minute to per-month.** `config.rs` already
  carries `monthly_budget_usd` but nothing separates a 24/7 standing charge from
  per-burst cost. A permanent box bills whether on or off (Hetzner only stops
  billing on *delete*), so the accounting must track the hub's standing cost
  distinctly and project the month.

## End-state

When harbor is fulfilled:

1. **`wm-burst hub up` is idempotent and cheap.** It provisions-or-reuses a
   permanent ~€4/mo box, persists its identity separately from burst pods, and
   never auto-destroys it. `hub down` deletes it (with confirmation). `hub status`
   shows it's up, its standing cost, and the month's projection.
2. **Cold bursts are warm.** Every build — laptop-local, a burst ccx53 pod, or the
   hub itself — shares one distributed compile cache hosted on the hub. The first
   cold compile of the day hits a populated cache, not an empty one.
3. **Pods clone in-DC; the laptop has a fallback remote.** Build pods pull from the
   hub's bare mirror (fast, same datacenter); the laptop can push WIP to the hub
   when GitHub isn't the right home yet.
4. **The fleet sees the hub.** The hub publishes `wm.fleet.hub.up`/`.down` + a
   heartbeat; the laptop can ask the bus "is a builder reachable?" without an SSH
   probe. This is the NATS landing pad constellation-bus connects to.
5. **The money is legible.** `wm-burst` separates the hub's standing monthly charge
   from burst-pod cost, projects the month, and warns past a configurable cap.
6. **`/cloudbuild` defaults warm.** Incremental builds run on the warm hub;
   cold/full compiles still burst a ccx53 pod but point sccache at the hub's
   distributed scheduler. Destroy-only becomes a `--ephemeral` opt-out.

## Relationship to constellation

harbor is the **near-term, concrete first slice** of `constellation-cloud` (the
"always-on cheap cloud node: NATS hub + mesh exit + burst build pods"). constellation
describes the node; harbor *builds the tooling to stand it up and use it* from the
existing `constellation-burst-builder` codebase. harbor-bridge is the landing pad
that the constellation-bus keystone (agorabus↔NATS) connects to — harbor stands up
NATS + heartbeat on the hub; constellation-bus carries the full `wm.*` stream across
it later. No overlap: harbor is builder-hub tooling; constellation-bus is the fleet
event fabric.

## Components (one bullet per PRD)

- **harbor-hub** *(rust-extend → constellation-burst-builder)* — `wm-burst hub
  up|down|status`: a permanent-box lifecycle distinct from burst pods. Idempotent
  `up` (provision-or-reuse via a new `get_server`/reuse path in `provider.rs`),
  persisted `HUB_ID`/`HUB_IP` in a hub section of config, never auto-destroyed,
  `down` deletes with confirmation. **Keystone — everything else consumes the hub.**
- **harbor-cache** *(shell + config → constellation-burst-builder/scripts)* — stand
  up `sccache-dist` scheduler + build-server on the hub; emit client config so burst
  pods and laptop builds share one distributed compile cache. Cold → warm.
- **harbor-mirror** *(shell → constellation-burst-builder/scripts)* — a bare git
  mirror + cargo registry/source cache on the hub; pods clone in-DC, the laptop gets
  a fallback push remote for unpushed WIP.
- **harbor-bridge** *(shell → constellation-burst-builder/scripts)* — NATS server on
  the hub + a `wm.fleet.hub.{up,down}` heartbeat publisher; the bus-reachable landing
  pad constellation-bus's agorabus↔NATS bridge connects to.
- **harbor-thrift** *(rust-extend → constellation-burst-builder)* — cost accounting
  that separates the hub's 24/7 standing charge from per-burst cost in `cost.log`,
  projects the month against `monthly_budget_usd`, and warns past a cap.
- **harbor-route** *(shell → cloudbuild.sh)* — make `/cloudbuild` hub-aware:
  incremental builds run warm on the hub; cold/full compiles burst a ccx53 pod but
  point sccache at the hub's dist scheduler. Destroy-only becomes `--ephemeral`.

## Order

```
harbor-hub ─┬─► harbor-cache ──► harbor-route
            ├─► harbor-mirror
            ├─► harbor-bridge
            └─► harbor-thrift
```

harbor-hub is the keystone (persists the permanent box + its identity). cache,
mirror, bridge, and thrift all consume the standing hub and are mutually
independent. harbor-route lands last — it needs both the hub up (hub) and the
distributed cache reachable (cache).

## Open questions

- **Box class.** `cx22` (2 vCPU/4GB ARM, ~€4/mo) is plenty for sccache-dist
  scheduler + NATS + git mirror, but the burst pods are x86 and sccache-dist
  toolchain artifacts are arch-specific — the scheduler is arch-agnostic but a
  hub-side *build-server* must match the pods' x86. So the hub likely wants `cpx11`
  (x86, 2 vCPU/2GB, ~€4.5/mo) not `cx22`. Confirm with jsy: ARM hub (cheapest,
  scheduler-only) vs x86 hub (can also be a build-server).
- **sccache-dist vs shared S3 cache.** `config.rs` already models an S3-style
  sccache bucket. Is a full `sccache-dist` (scheduler + distributed *compilation*)
  worth it, or is a shared **cache** (S3/MinIO on the hub, `SCCACHE_*` pointed at
  it) enough? The latter is far simpler and gets ~90% of the warm-cache win. Lean
  shared-cache-first; defer true distributed compilation.
- **Hub deletion safety.** A permanent box holding the only warm cache + a WIP git
  mirror is now *stateful*. `hub down` must refuse-by-default or snapshot first, so
  an accidental teardown doesn't lose the mirror's unpushed commits.
- **Standing-cost cap behavior.** When the month's projection exceeds the cap, does
  harbor-thrift just warn, or actually tear the hub down? Warn-only is safer;
  auto-teardown of a stateful hub is dangerous (see above). Default: warn.
