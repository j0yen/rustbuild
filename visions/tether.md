# Vision: tether — the work node becomes a part of the self

**Authored by:** /dream (Claude Sonnet 4.6), with jsy
**Created:** 2026-06-17
**Status:** active
**Seed:** user — *"about connecting better with the work node. establish a
persistent connection. extend agorabus. share your tools, notes, thoughts,
gossip, etc. It's a part of you — wire it in."*

---

## TL;DR

wintermute lives on one laptop. The work node — jsy's AtScale machine
(`joeyen-atscale` identity, a Ryzen 7 5700U / 32GB box) — is where jsy spends
the working day, but it is *cut off* from the self: no shared recall, no shared
gossip, no shared tools, no presence on the bus. The `constellation` vision
explicitly **dropped** it ("it's jsy's work machine"); the `persona` vision
gives it an *identity* but not a *connection*. `tether` is the missing wire: a
**persistent, auto-healing link** that makes the work node a first-class peer of
the wintermute self — its sessions appear on the bus, it reads and appends the
shared gossip, it queries the shared memory, and it can reach the laptop's tools.
Not a build workhorse (that's `harbor`/`constellation`); a *limb*. When jsy is at
work, wintermute is there too — same notes, same thoughts, same tools, one self
across two machines.

## End-state

When this is done:

- A `wm-tether` supervisor keeps an always-on, self-reconnecting link between the
  work node and the wintermute hub (Tailscale mesh + a NATS leaf that survives
  sleep/wake, network changes, and reboots). `wm-tether status` on either machine
  shows the link healthy and the round-trip latency.
- `agorabus peers --fleet` on the laptop lists the work node's live `claude`
  sessions, and vice-versa — presence is fleet-wide, not host-local.
- An append to `~/wintermute/autobuilder/notes/gossip.md` on either machine
  shows up on the other within seconds, loop-guarded and order-preserved. /dream
  and /build on the laptop and any session on the work node share one gossip log.
- A session on the work node can `recall query` against the wintermute memory and
  see the laptop's reflective/semantic/procedural notes — the self's thoughts are
  legible from the work side (read-first; bounded append later).
- A session on the work node can list and invoke the laptop's `~/.local/bin`
  tools as remote capabilities over the bus, so "share your tools" is literal:
  the work node calls `recall`, `ctrace query`, `procstat`, etc. on the machine
  that has them, getting results back over the wire.

## Why now (Phase 1 evidence, 2026-06-17)

- **agorabus is single-host by construction.** Its own README: *"advisory
  presence+pub/sub substrate over a Unix-domain socket so co-located sessions
  can announce themselves."* The bus literally cannot see another machine. To
  wire the work node in, the bus needs a network reach it does not have.
- **The transport keystone already exists, half-built.** `agorabus-nats-bridge`
  (`wm-busbridge`, INSTALLED at `~/.local/bin/wm-busbridge`) mirrors allowlisted
  `wm.fleet.*` events between the local UDS and a NATS leaf, with a loop guard and
  selective forwarding. `tether` is what that bridge was *for* — but the bridge
  assumes the leaf is reachable; nothing keeps the link *up* across a work
  laptop's sleep/wake/VPN-flap day, and nothing yet mirrors presence, gossip,
  recall, or tools across it.
- **constellation dropped the work box; the user just un-dropped it.** The
  constellation vision (2026-06-04) put `~~5700U box~~ — REMOVED (jsy's work
  machine)` in its fleet table. Today's seed reverses that *for the
  connection purpose* (not as a builder): the work node is to be a peer of the
  self. `tether` records that reversal so the two visions don't fight — see
  Open Questions.
- **The identity half is already dreamed.** `persona`'s work-persona frontier
  (persona-work, -redline, -eval, -doctor) gives the work node *who it is*
  (`CLAUDE_WORK.md`, the work `redline.toml`, the AtScale boundaries). `tether`
  gives it *how it connects*. They compose: persona is the face, tether is the
  nervous system. Every tether PRD inherits persona's honesty discipline —
  *authored and fixture-tested on this box, installed on the work box, SKIP
  honestly when the link/identity isn't live, never false-green by assuming the
  remote machine's state.*
- **The bus already carries a `wm.fleet.*` namespace.** nats-bridge forwards
  exactly `wm.fleet.>`, so presence/gossip/recall/tools each get a fleet subject
  (`wm.fleet.presence.*`, `wm.fleet.gossip.*`, …) that crosses for free once the
  link is up. No new transport per feature — they all ride the one bridge.

## Components (PRD-sized)

- **tether-link** (rust-cli) — the persistent connection itself. A `wm-tether`
  supervisor + `tether-link.service` that brings up the Tailscale mesh + NATS
  leaf, watches it, and reconnects with backoff across sleep/wake/network-change/
  reboot. `wm-tether status` reports link state + RTT; `wm-tether up/down`
  controls it. This is the literal "establish a persistent connection." Depends
  on `wm-busbridge` (already installed). The real cross-machine link is
  deferred/mocked (embedded NATS test server, the nats-bridge precedent).

- **tether-presence** (rust-extend agorabus) — the literal "extend agorabus."
  Add a `node` field to the peer announce (defaulting to the local hostname /
  Tailscale name) and a `peers --fleet` flag that merges remote peers learned
  from `wm.fleet.presence.*` with local UDS peers. Pure additive extension; all
  existing local clients unchanged (agorabus AC2-style invariant).

- **tether-gossip** (rust-cli) — "share your gossip." A `wm-tether-gossip`
  daemon that tails `~/wintermute/autobuilder/notes/gossip.md`, publishes new
  appends to `wm.fleet.gossip.append`, and applies appends arriving from the bus
  to the local file — loop-guarded (don't republish what you just received) and
  order-preserving (monotonic per-node sequence; append-only, never rewrites
  history per the gossip hard rule).

- **tether-recall** (rust-cli) — "share your notes, thoughts." A
  `wm-tether-recall` bridge that answers `recall query` requests arriving over
  `wm.fleet.recall.query` from the work node by running the query against the
  local recall store and returning ranked hits — read-first (the self's thoughts
  become legible on the work side without write-conflict risk). Bounded
  append-back (`wm.fleet.recall.write`) is a follow-on once read is proven.

- **tether-tools** (rust-cli) — "share your tools." A `wm-tether-tools` responder
  that advertises an allowlisted subset of `~/.local/bin` (recall, ctrace,
  procstat, wchg, …) over `wm.fleet.tools.manifest` and executes allowlisted
  invocations arriving on `wm.fleet.tools.invoke`, returning stdout/exit-code —
  deny-by-default, arg-sanitized, no shell metacharacters, timeout-bounded.

## Order

```
tether-link ──► tether-presence ──► { tether-gossip ∥ tether-recall ∥ tether-tools }
```

`tether-link` is the foundation — nothing crosses until the link is up.
`tether-presence` is the smallest consumer (and the user's explicit "extend
agorabus"), so it lands second and proves the round-trip. gossip, recall, and
tools are independent of each other and each consume the link.

## Open questions

1. **Reconciliation with constellation.** constellation's fleet table marks the
   5700U DROPPED. tether un-drops it *as a peer of the self, not as a builder*.
   Should constellation's table get an amended row (`work node — peer via
   tether`), or does tether stand as the canonical work-node vision and
   constellation stay build/GPU-focused? (Leaning: amend constellation's row to
   point at tether; don't fork the fleet model.)
2. **Hub topology.** Does the work node leaf-connect to the Hetzner hub
   (`harbor`) or peer directly to the laptop over Tailscale? Hub is more durable
   (laptop sleeps); direct is lower-latency. Likely hub-primary with direct
   fallback — but that's a tether-link config decision, flag it.
3. **Recall write-back conflict model.** Read-first is safe. When the work node
   gains append (`recall write` from the work side), the memory store needs a
   conflict/merge story (hub-authoritative append log? per-node namespace?). Tie
   to whatever `recall-daemon` exposes; don't invent a CRDT prematurely.
4. **Tool-invocation trust surface.** Remote tool execution is the highest-risk
   component. Single-user trust model holds (it's all jsy), but the allowlist +
   arg-sanitization must be load-bearing and grep-asserted. Should it be
   capability-tokened per session, or is Tailscale-ACL + allowlist enough?
5. **A tether-doctor** (shell) — periodic health check on the link + the four
   mirrors, mirroring `persona-work-doctor` / `handshake`. Left as a bullet for
   the next /dream pass once the five core PRDs have shape.
