# PRD: pulse-deaf-escalation — a deaf box reaches Joe, durably

**Status:** Draft v0.1
**build_target:** rust-extend
**build_into:** /home/jsy/wintermute/wintermute-reach
**Vision:** visions/pulse.md

## TL;DR

When the companion goes deaf, jsy needs to know — fast and reliably, because a
deaf box at his mother's home means she can call for help into a void. This PRD
adds a deaf-device escalation path to `wintermute-reach`: subscribe
`wm.health.hearing.fail`, deliver a deaf-device alert to jsy over the existing
durable distress ladder (retry + fallback transport), debounced so a flapping
mic sends one alert per outage, not one per probe. It reuses the safety
delivery path kin already shipped; it does not invent a new transport.

## Why this exists

Phase-1 evidence (2026-06-13):

- `wintermute-reach/src/distress_delivery.rs:41 run_distress_ladder()` already
  implements bounded retry (`policy.backoff_ms`) + multi-transport fallback for
  kin's Mom-initiated distress. A deaf device is a safety event of the same
  class — the delivery requirements are identical (must not end at a single
  `delivered:false` nack).
- pulse-watch emits `wm.health.hearing.fail` on the `→ Deaf` edge (and `.ok` on
  recovery), but nothing delivers it off-device. Today a deaf companion is
  invisible to jsy.
- kin distress is *Mom-initiated* ("I've fallen"); a deaf-device alert is
  *device-initiated* and must work precisely when other paths may be down — so
  it must not depend on the Claude API (a down brain is itself a reason the box
  might look deaf).

## What this builds

A new escalation class in the reach daemon, reusing the distress ladder.

- **Subscribe `wm.health.hearing.fail`** in the reach daemon's subscribe loop
  (daemon.rs); on receipt, compose a deaf-device alert: "Wintermute at Mom's
  hasn't been able to hear for ~N minutes — it may need attention," carrying
  the fail timestamp and last-`ok` age from the envelope.
- **Deliver over `run_distress_ladder()`** — the same retry + fallback-transport
  escalation distress uses. A deaf-device alert that nacks the primary transport
  retries and escalates exactly like distress; it is not allowed to die as one
  `delivered:false`.
- **Debounce / rate-limit.** Persist a small escalation state: once a deaf alert
  is sent, suppress further deaf alerts until either a `wm.health.hearing.ok`
  arrives or a configurable cool-down elapses (default generous). A detector
  flapping deaf↔hearing must not spam jsy — one alert per sustained outage.
- **Recovery note.** On `wm.health.hearing.ok` following a sent alert, deliver a
  single "hearing restored" note (best-effort, not over the full ladder) and
  clear the suppression state.
- **Config.** Extend reach config: `deaf_escalation_enabled` (default ON — a
  safety signal), `deaf_escalation_cooldown_s`. Reuse the enrolled jsy
  transport + the distress policy/fallbacks already configured.
- **Self-emitted-topic filter** so the subscriber never loops on its own
  publications; deaf-escalation publishes no `wm.health.*` itself.

No API-path dependency (deterministic bus-event → transport). No new transport
crate. MSRV 1.85, no let-chains, `sigpipe::reset()` already in main.

## Acceptance criteria

1. A `wm.health.hearing.fail` event triggers exactly one deaf-device alert
   delivered via `run_distress_ladder()`; tested with a stub transport asserting
   the ladder was invoked (not a bare one-shot publish).
2. When the primary stub transport nacks, the alert retries per
   `policy.backoff_ms` and escalates to a fallback transport, mirroring the
   distress-ladder behavior; the alert acks `delivered:true` via the fallback
   (tested with primary-fail + fallback-ok stubs).
3. Debounce: a second `wm.health.hearing.fail` arriving within the cool-down (no
   intervening `.ok`) sends **no** additional alert; after a `wm.health.hearing.ok`
   a subsequent `.fail` does send again. Tested with a fake clock / injected
   timestamps (no `Date::now()` in tests).
4. A `wm.health.hearing.ok` following a sent alert delivers exactly one
   "hearing restored" note and clears suppression; an `.ok` with no prior alert
   delivers nothing (tested).
5. With `deaf_escalation_enabled:false`, no subscription action and no delivery
   on `.fail` (regression: existing distress/message/reply behavior unchanged).
6. The escalation path performs no Claude-API / brain call — verified by
   inspection + a test that delivers a deaf alert with no brain dependency wired.
7. `cargo test` green; `cargo build --release` clean; reach `version` bumped.
