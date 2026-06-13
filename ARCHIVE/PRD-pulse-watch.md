# PRD: pulse-watch — a liveness state machine that knows deaf from quiet

**Status:** Draft v0.1
**build_target:** rust-extend
**build_into:** /home/jsy/wintermute/wintermute-presence
**Vision:** visions/pulse.md

## TL;DR

pulse-hearing-probe mints a point-in-time `wm.health.hearing` envelope. One
envelope can't distinguish a momentary blip from a genuinely deaf device, and
nothing runs the check on a cadence. This PRD adds a cadenced hearing-liveness
watcher to `wintermute-presence` with a HEARING / DEGRADED / DEAF state
machine, persists the result per waking-hours window alongside the presence
state that already lives there, and emits `wm.health.hearing.fail` on the DEAF
edge — the signal both reach PRDs key on.

## Why this exists

Phase-1 evidence (2026-06-13):

- `wintermute-presence` already subscribes `wm.audio.wake` + `wm.stt.final`,
  tracks an interaction count and a waking-hours window, and persists
  per-window state (`state.rs` `silence_emitted_for_window`, daemon.rs:29
  `run`, silence.rs). It knows "did anything happen today" but **cannot tell
  deaf from quiet** — both produce zero interactions.
- The active hearing check that *can* tell them apart now exists as
  pulse-hearing-probe's `run_hearing_probe` / `wm.health.hearing` envelope.
- Nothing currently runs that check on a cadence or debounces it into a
  liveness verdict. A single failed probe is not a deaf device; K consecutive
  failures is.
- Presence is the natural home: it already owns the waking-hours window that
  decides *when* hearing matters (a box "deaf" at 3am because Mom is asleep is
  not an emergency).

## What this builds

A hearing-liveness extension to the presence daemon.

- **Cadenced probe.** On a configurable interval (default conservative — see
  vision OQ#1), within the waking-hours window, trigger a hearing check:
  either call `run_hearing_probe` in-process or subscribe to `wm.health.hearing`
  emitted by an external `wm-audio selftest --emit` timer. Drafted: subscribe
  to the envelope (keeps inference in the audio crate, presence stays a
  consumer) with the cadence owned by a `wm-audio selftest --emit` systemd
  timer this PRD ships.
- **Liveness state machine.** `Hearing` / `Degraded` / `Deaf`:
  - `ok` envelope → `Hearing` (reset failure counter).
  - `degraded` envelope → `Degraded` (no edge event).
  - `deaf` envelope, or K consecutive non-`ok` (default K configurable) →
    `Deaf`.
  - Immediate `Deaf` when an envelope reports `model_present:false` or
    `detector_loaded:false` (a structural fault, not a transient).
- **Per-window persistence.** Record, alongside existing presence state, a
  `hearing_confirmed_in_window: bool` (true once an `ok` envelope landed inside
  the window) and the current liveness state, so pulse-silence-gate can read
  whether the device was provably hearing this window.
- **Edge events.** Emit `wm.health.hearing.fail` once on the `→ Deaf`
  transition and `wm.health.hearing.ok` once on `Deaf → Hearing` recovery.
  Apply the self-emitted-topic filter (presence already does for its
  `wm.presence.*` publishes).
- **Config.** Extend `PresenceConfig`: `hearing_probe_interval_s`,
  `hearing_deaf_threshold_k`, `hearing_watch_enabled` (default ON — it is a
  safety signal, but only acts within the existing waking-hours window).
- **Status.** Extend `wm-presence status` to print current liveness state +
  last `ok` timestamp.

Ship a `wm-audio selftest --emit` companion systemd timer + the presence
subscribe wiring. MSRV 1.85, no let-chains, `sigpipe::reset()` already in main.

## Acceptance criteria

1. Feeding a sequence of `wm.health.hearing` envelopes (`ok`, then K
   consecutive non-`ok`) drives the state machine `Hearing → … → Deaf` exactly
   at the Kth failure, and a single `ok` returns it to `Hearing`; unit-tested
   for K = the configured default and K-1 (no false DEAF).
2. An envelope with `model_present:false` (or `detector_loaded:false`) forces
   `Deaf` immediately regardless of the failure counter; unit-tested.
3. `wm.health.hearing.fail` is emitted exactly once on the `→ Deaf` edge (not
   per failing probe), and `wm.health.hearing.ok` exactly once on recovery;
   re-entering the same state emits nothing (edge-triggered, tested).
4. Per-window state records `hearing_confirmed_in_window` (true iff an `ok`
   envelope arrived inside the waking-hours window) and persists across a
   daemon restart (write-then-reload test, mirroring the existing presence
   state persistence test).
5. With `hearing_watch_enabled:false`, no probing, no state transitions, no
   edge events — the daemon's pre-existing `wm.presence.*` behavior is
   unchanged (regression test).
6. Hearing checks outside the waking-hours window do not produce alerts (a
   sleeping-hours deaf reading does not emit `.fail`); tested with a window
   boundary.
7. `cargo test` green; `cargo build --release` clean.
