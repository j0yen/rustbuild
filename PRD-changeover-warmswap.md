# PRD: changeover-warmswap

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/rollout
Vision: visions/changeover.md

## TL;DR

`rollout apply` restarts a fleet daemon by killing it and starting the
successor (`systemctl --user restart`), so there is always a window where no
instance holds the daemon's topic. This PRD adds a **warm-swap** restart
strategy to rollout: start the successor first, wait for it to subscribe and
acquire the daemon's agorabus claim lease, then stop the predecessor — so a
live holder always owns the topic across the swap. It builds entirely on
agorabus's existing `ClaimAcquire`/`ClaimRelease` lease; exactly one instance
is the active claim holder at any instant, so subscribers never see two
producers. Daemons with no declared claim key fall back to the current hard
restart unchanged.

## Why this exists

- `~/wintermute/rollout/src/restart.rs`: the restart path is
  `systemctl --user restart <unit>` + `poll_healthcheck` — a hard
  kill→start with no overlap. `RestartStrategy::SystemdUnit` is the only
  warm-path option today.
- `~/wintermute/agorabus/src/protocol.rs` already ships the lease:
  `ClaimAcquire { path, ttl_unix_secs, reason, force }` refuses a conflicting
  claim from another session (returns `claim_conflict`) and broadcasts on
  `claim.acquire`; `ClaimRelease` broadcasts on `claim.release`. This is the
  exactly-one-holder primitive warm-swap needs — no new agorabus surgery.
- changeover-probe (PRD-changeover-probe.md) measures the hard-restart
  window; warm-swap is the fix it justifies. This PRD is only worth shipping
  if probe shows a lossy window (vision Order). Cite probe's number in the
  build's receipt.
- Self-review fleet-binary-staleness (journal 2026-06-13): the four voice
  daemons can't be auto-rolled because the restart drops subscribers.
  Warm-swap is the mechanism that makes an auto-roll safe.

## What this builds

Extend the `rollout` crate (do not fork). New module `src/warmswap.rs` and a
new `RestartStrategy::WarmSwap { claim_path }` variant.

- **fleet.toml recipe field**: optional `claim_key` per daemon. When set,
  warm-swap is used; when absent, derive a default
  `agorabus://daemon/<unit-stem>` and require an explicit
  `warm_swap = true` opt-in, else fall back to hard restart (conservative:
  no behavior change for un-migrated recipes).
- **warm-swap sequence** (in `warmswap.rs`, strictly serialized, one daemon
  at a time, consistent with rollout's existing invariant):
  1. Install the freshly-built binary to its dest (reuse `install.rs`).
  2. Launch the successor as a *second* instance (a transient
     `systemd-run --user` scope so it survives, distinct from the managed
     unit's main PID) — or, where the unit supports it, the unit's own
     reload. Successor connects to agorabus and calls `ClaimAcquire` on the
     daemon's `claim_path` with `force = false`.
  3. If `ClaimAcquire` returns `claim_conflict`, the predecessor still holds
     the lease (expected) — poll until the successor reports healthy on
     agorabus (`poll_healthcheck`), then proceed.
  4. Stop the predecessor (`systemctl --user stop` or SIGTERM to the old
     PID). Predecessor's shutdown path releases its claim (`ClaimRelease`);
     successor re-acquires (now uncontended) and becomes the sole holder.
  5. Verify exactly one live peer for the daemon and one active claim on
     `claim_path` (`ClaimList`) before returning success.
- **Refusal**: if at step 5 two holders or zero holders are observed, the
  swap is reported failed (non-zero exit, clear message) and the operator is
  told the daemon may be in a split state — never silently "ok".

`rollout plan` output gains a per-daemon `strategy: warm-swap|hard` column so
the operator sees which path each daemon will take before `apply`.

## Acceptance criteria

1. `rollout plan` shows a `strategy` column; a daemon with `claim_key` set
   (or `warm_swap = true`) shows `warm-swap`, others show `hard`.
2. A unit test drives the warm-swap state machine against a mock agorabus
   client (claim acquire → conflict → predecessor release → re-acquire) and
   asserts the success path reaches "single holder" exactly once.
3. A unit test asserts the split-state guard: if `ClaimList` reports two
   holders (or zero) at the verify step, `warmswap` returns an error and a
   non-zero exit, not success.
4. A daemon recipe with no `claim_key` and no `warm_swap` flag uses the
   existing hard restart path unchanged (regression test: existing
   restart.rs behavior is untouched for un-migrated recipes).
5. `rollout plan --only wm-stt` with a warm-swap recipe prints the full
   ordered sequence (install → launch successor → acquire → stop predecessor
   → verify) without mutating anything.
6. `cargo build --release` clean and `cargo test` green; SIGPIPE-safe path
   preserved ([[self_sigpipe_panic_toolkit]]). Live-fleet swap is exercised
   only behind an `#[ignore]` integration test so cloudbuild stays hermetic.
