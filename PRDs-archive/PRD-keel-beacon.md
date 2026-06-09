# PRD: keel-beacon — say it the moment the ceiling moves

**Status:** verified_completed
iter_log:
  - "iter-2 (2026-06-08): beacon.rs + status.rs + main.rs wired; 13 tests green; all 8 ACs paired."
build_status: shipped
**build_target:** rust-extend
**build_into:** `~/wintermute/keel`
**Vision:** visions/keel.md

## TL;DR

That the brain is floored on a 3B model is currently legible **only at the daily
self-review** — every journal for weeks re-reports "cloud tier ladder cannot
reach cloud models" as if it were news. **keel-beacon** makes it legible in the
moment: a `keel status` one-liner that answers "what tier is the brain standing
on, and for how long?", and agorabus `wm.keel.*` events emitted when the
effective tier ceiling changes, so peon-ping, a future self-heal loop, or jsy
can react live instead of at 00:00 the next day.

## Why this exists

- Every self-review (journals 2026-06-01/02/03) re-derives the same floored-tier
  fact by hand. docket `wm-anthropic-key-empty` has **8 reports across 6 runs** —
  it is re-reported, never *surfaced live*. There is no runtime signal that says
  "the ceiling just dropped to local-3b."
- The fleet already has a bus-event idiom for exactly this kind of state change:
  `wm.almanac.due`, `wm.audio.status`, `wm.brain.*` (confirmed live subjects via
  agorabus). keel-beacon follows the convention with `wm.keel.*`. Both quicken
  (`wm.quicken.*`) and concord (`wm.concord.*`) left their bus surfaces as Open
  questions; keel makes the case concrete because the underlying state
  (cordon's effective ceiling) already changes on its own.
- Builds on keel-cordon (the effective-ceiling state) and keel-pulse (the
  floored-tier line). Do not start until keel-pulse has SHIPPED; reads cordon, so
  best after keel-cordon.

## What this builds

rust-extend of `~/wintermute/keel`.

- **`keel status`** — one line, the question self-review keeps asking by hand:
  `floored: local-3b for 4d 6h (cloud keyless since 2026-05-30)` when degraded,
  or `nominal: opus reachable` when the full ladder is up. Derives the floored
  tier = highest `Attempt`-able rung per cordon; "for how long" from the oldest
  contiguous cooldown/stamp. `--format json` for machines.
- **Ceiling-change detection** — compares the current effective ceiling against
  the last-emitted ceiling (persisted in a small `last-ceiling` state file).
  When it drops, emit `wm.keel.degraded { from, to, since }`; when it rises,
  `wm.keel.refloat { from, to }`; on no change, emit nothing (idempotent — the
  beacon is edge-triggered, not a heartbeat).
- **agorabus publish behind a trait** — a `Beacon` trait with the real agorabus
  client impl and a `RecordingBeacon` test double, so `cargo test` publishes to
  a vector, never the live bus. (Match the agorabus client crate the fleet uses;
  `~/wintermute/agorabus`.)
- **`keel beacon`** CLI — computes the current ceiling, diffs against
  `last-ceiling`, emits the appropriate event (or none), updates the state file.
  Intended to run from a oneshot/hook or the brain after a tier transition.
- Deterministic: `now` injected, ceiling state file path injected, bus behind the
  trait → fully cloud-build-safe in tests.

## Acceptance criteria

1. `cargo build` + `cargo test` succeed offline; the bus is the `RecordingBeacon`
   double in every test (assert zero live-bus connections).
2. `keel status` prints the floored tier + duration when cordon shows cloud rungs
   skipped, and `nominal: <top> reachable` when all rungs are `Attempt`-able;
   both cases covered over fixture cordon state.
3. A ceiling **drop** (top reachable tier falls from a cloud tier to local-3b)
   emits exactly one `wm.keel.degraded` with correct `from`/`to`/`since`.
4. A ceiling **rise** emits exactly one `wm.keel.refloat`; the subject strings
   match the `wm.keel.*` convention.
5. **No change** between two runs emits **zero** events (edge-triggered; assert
   the RecordingBeacon stays empty on the second invocation).
6. The `last-ceiling` state file is updated after each emit so the next run diffs
   against the new ceiling (round-trip test across two `keel beacon` runs).
7. `keel status | head` does not panic (SIGPIPE reset).
8. Integration test entry file (e.g. `tests/beacon.rs`) appears in `cargo test`
   output (`self_orphaned_mock_tests` guard).
