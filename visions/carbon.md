# carbon — the laptop becomes a node like any other

**Authored by:** /dream (Claude Sonnet 4.6), with jsy
**Created:** 2026-06-20
**Status:** active
**Seed:** user — *"moving Wintermute to the cloud. Move the non-voice tasks —
messaging, reminders, NATS subscribers — to the cloud. this laptop will be a
node like any other. rename this node to carbon."*

## TL;DR

Today this laptop (`hostname=wintermute`) is the *center* of the system: it
holds the canonical recall memory, runs the build/dream loops, hosts the
always-on application daemons (homeward ingest/report/embed), and fires every
scheduled job. If the lid closes, the reminders stop, the subscribers go deaf,
and outbound messages never send. `carbon` inverts that: the **always-on,
non-voice work moves to the cloud hub** (the Hetzner box already anchoring the
Tailscale mesh + NATS), and this laptop demotes to a *peer* — renamed `carbon`,
one voice node among others, free to sleep without taking the system down.

This is the natural sequel to [[constellation]] (which made ryzen7 a peer) and
[[harbor]] (which scoped the always-on hub upgrade). Constellation proved the
fleet bus works across nodes; carbon uses it to relocate the work that should
never have been laptop-bound.

## End-state

- This laptop's hostname / Tailscale name / `WM_NODE` is **`carbon`**, not `wintermute`.
- Every node declares an explicit identity (`WM_NODE`) and **role** (`voice`,
  `hub`, `builder`); placement of a daemon is *declared*, not implied by "it
  happens to run on the laptop."
- The cloud hub is SSH-reachable, runs a `systemd --user` session, agorabus, and
  NATS in hub mode, and has an **ARM build path** (it is ARM; cloudbuild is x86).
- The **reminders** (scheduled/cron timers that are node-agnostic) fire from the
  hub — they run whether or not any laptop is awake.
- The **persistent NATS subscribers** that must be 24/7 (homeward ingest, app
  consumers) run on the hub; per-node subscribers (voice, tether) stay local.
- **Outbound messaging** (homeward owner-notify, relay email send) executes on
  the hub, so a message triggered by a bus event sends regardless of laptop state.
- Closing this laptop's lid degrades *nothing* except voice-on-this-node.

## Why now (Phase-1 evidence, 2026-06-20)

- `grep WM_NODE ~/.config/wintermute ~/.config/systemd/user /etc/wintermute` →
  **zero hits**. Node identity is implicit (hostname only). "A node like any
  other" has no concept to stand on — it must be built first.
- Hub `100.66.158.49` is **live in Tailscale** (direct path, tx/rx active this
  session) but `ssh hub` → **Host key verification failed / no key**. Same
  unblock shape as constellation-nats-hub: the box exists, it just isn't wired.
- `docs/cloud-hub.md` specs the hub as **Hetzner CAX21, ARM**. cloudbuild is
  x86_64. Any Rust daemon relocated to the hub needs an ARM build path
  (native-on-hub or cross-compile) — the same GLIBC/arch lesson constellation
  learned the hard way on ryzen7 (Ubuntu/AVX-512).
- Always-on, laptop-bound daemons found active this session: `homeward-ingest`
  (AIMD cadence loop), `homeward-report` (owner API :8081), `homeward-embed`
  (DINOv2 sidecar), plus `recalld`, `watchman`, `wm-tether`, `wm-busbridge`.
- Node-agnostic scheduled timers found: `roundtable`, `claude-self-review`,
  `claude-review-due`, `adopt-cron`, `consign-drain`, `claude-chaff`,
  `trim-relief`, `ballast-guard` — all fire only while the laptop is on.
- No dedicated messaging/notify *unit* exists — outbound sends are embedded in
  `homeward-reportd` (owner-notify) and the relay email path. "Messaging to the
  cloud" therefore means relocating the binaries/jobs that perform sends, and
  triggering them from bus events rather than local cron.

## Components (PRD-sized)

1. **carbon-node-identity** — introduce `WM_NODE` + role registry; a node
   declares name+role, and a small CLI/lib resolves "should this daemon run
   here?" Foundational; nothing else is honest without it.
2. **carbon-rename** — rename this laptop `wintermute → carbon` across hostname,
   Tailscale, `WM_NODE`, and config/journal references. Touches *this* box only.
3. **carbon-hub-access** — make the hub SSH-reachable, stand up its
   `systemd --user` session, agorabus, NATS hub mode, **and an ARM build path**.
   The unblock that everything cloud-side cascades from.
4. **carbon-reminders-cloud** — relocate node-agnostic scheduled timers to the
   hub so they fire when every laptop is asleep; leave node-local timers in place.
5. **carbon-subscribers-cloud** — relocate the 24/7 NATS subscriber daemons
   (homeward ingest, app consumers) to the hub; classify per-node vs fleet-wide.
6. **carbon-messaging-cloud** — run outbound messaging (owner-notify, relay
   email) on the hub, fired by bus events, so sends are independent of laptop state.

## Order

```
carbon-node-identity ──┬─→ carbon-rename
                       └─→ carbon-hub-access ──┬─→ carbon-reminders-cloud
                                               ├─→ carbon-subscribers-cloud ──→ carbon-messaging-cloud
```

- node-identity first (1) — the WM_NODE concept gates honest placement.
- rename (2) and hub-access (3) can proceed in parallel after (1).
- reminders (4) and subscribers (5) need a working hub (3).
- messaging (6) needs subscribers (5) — sends are triggered by relocated consumers.

## Open questions

- **Recall memory** is canonical and laptop-held. Does it move to the hub
  (always-on, but adds latency to every recall), replicate (hub primary + local
  cache), or stay local? Probably its own vision — *not* in carbon's scope yet.
- **Build/dream loops** (`claude-build`, `claude-dream`) are laptop-bound and
  heavy. Do they belong on the hub, on a burst builder, or stay on whichever
  node is awake? Left open — carbon moves *application* work, not the agent loops.
- ARM build: native-on-hub (slow, CAX21 is small) vs cross-compile from x86 vs
  a burst ARM builder. carbon-hub-access picks one; revisit if compile time hurts.
- Does "node like any other" eventually mean the laptop can be *re-imaged* from
  the constellation iso with zero special-casing? That's the constellation iso
  arc — carbon just makes the laptop's services portable enough to allow it.
