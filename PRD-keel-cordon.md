# PRD: keel-cordon — stop re-discovering a dead tier every turn

**Status:** Draft v0.1
**build_target:** rust-extend
**build_into:** `~/wintermute/keel`
**Vision:** visions/keel.md

## TL;DR

When the cloud is keyless/exhausted — which on this box is most days — the brain
attempts the cloud rung, eats the failure, and degrades **on every single
turn**. The deadness is known after the first failure but never remembered.
**keel-cordon** adds the memory: a `should_attempt(tier) -> Decision` function
that consults pulse + ledger health and an exponential cooldown, so a known-dead
rung is *skipped* instead of re-attempted. It ships the decision library + a
`keel cordon` CLI; wiring it into the brain's per-turn dispatch is a separate,
user-gated PRD (it touches the live local-model path, not cloud-safe).

## Why this exists

- `wintermute-brain/src/ladder.rs` dispatches each rung and maps a transport
  error / no-key / 402 to `LadderOutcome::Degraded` (lines ~352–519). This is
  correct but **stateless**: the next turn repeats the same doomed attempt. With
  cloud down for 6+ self-review runs (docket `wm-anthropic-key-empty`), that is a
  wasted cloud round-trip *per conversational turn*, every turn, for days —
  latency the user pays on a voice path that is supposed to feel immediate
  (`project_voice_input_null_detectors`: the turn must feel live).
- The brain already supports skipping rungs statically
  (`WM_BRAIN_SKIP_TIERS`, `project_brain_local_first_ladder`). keel-cordon is the
  *dynamic* analog: skip a rung because it's *currently* dead, with automatic
  re-probe after a cooldown, rather than skipping it forever by hand-editing
  config.
- Builds directly on keel-pulse (`TierHealth`/`TierStatus`) and keel-ledger
  (durable `Exhausted`/`Keyless` stamps) — consumes both; do not start until
  keel-pulse has SHIPPED.

## What this builds

rust-extend of `~/wintermute/keel`.

- **`Cordon`** — holds per-tier `{status, consecutive_failures, cooldown_until}`,
  loaded from keel-pulse health + keel-ledger status stamps.
- **`should_attempt(tier, now) -> Decision`** where `Decision` is
  `Attempt` | `Skip { reason, retry_after }`. A tier marked
  `Keyless`/`Exhausted`/`Unreachable` returns `Skip` until `cooldown_until`;
  after that, one `Attempt` is allowed (half-open probe). A `Reachable` tier
  always `Attempt`s and resets its failure count.
- **Exponential cooldown** — `record_failure(tier, now)` bumps
  `consecutive_failures` and sets `cooldown_until = now + base * 2^(n-1)`, capped
  at a documented ceiling (e.g. 1h). `record_success` clears it. `base`/cap are
  config, injected in tests. (Keyless is a special case: cooldown keyed to "key
  present?" not time — re-probe cheaply each call since checking an env var is
  free, but still skip the socket.)
- **`keel cordon`** CLI — prints each tier's current `Decision` and
  `retry_after`, so `keel cordon` answers "which rungs would the brain even try
  right now?" `--format json` for the eventual brain consumer.
- Pure/deterministic: `now` injected, no `Date::now`; all state read through the
  pulse/ledger traits with fixtures. Fully cloud-build-safe. (The *consumer* —
  `LadderClient` calling `should_attempt` before dispatch — is the held-out
  `brain-keel-wire` PRD in the vision, because it links the non-cloud-safe local
  backend.)

## Acceptance criteria

1. `cargo build` + `cargo test` succeed offline; all health/ledger inputs are
   fixtures (no network, no real clock).
2. A tier stamped `Exhausted` at `t0` returns `Skip` for every `now < t0 +
   cooldown`, then exactly one `Attempt` (half-open) at `now >= cooldown`.
3. `record_failure` grows `cooldown_until` exponentially across successive calls
   and is capped at the documented ceiling; a table test covers n=1..6.
4. `record_success` resets `consecutive_failures` to 0 and clears the cooldown so
   the next `should_attempt` returns `Attempt`.
5. A `Keyless` tier returns `Skip { reason: "keyless" }` without consulting the
   network; if the key later appears (fixture env flips), the next call returns
   `Attempt`.
6. `keel cordon --format json` lists every configured tier with its `Decision`
   and `retry_after`, consistent with the injected health state.
7. `Cordon` consumes keel-pulse `TierHealth` and keel-ledger status stamps
   directly (no duplicated status type) — a test constructs a `Cordon` from both
   and asserts the merged view.
8. Integration test entry file (e.g. `tests/cordon.rs`) appears in `cargo test`
   output (`self_orphaned_mock_tests` guard).
