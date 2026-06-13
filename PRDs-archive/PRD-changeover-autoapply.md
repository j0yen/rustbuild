# PRD: changeover-autoapply

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/rollout
Vision: visions/changeover.md

## TL;DR

`rollout apply` is never auto-approved: a human must run it because daemon
restarts can drop subscribers. With changeover-probe measuring the swap window
and changeover-warmswap closing it, the last piece is a gate that lets
`rollout apply --auto` proceed *only* for daemons that carry a fresh proof the
swap is loss-free. This PRD adds `rollout apply --auto`, a per-daemon proof
ledger fed by `changeover probe`, and a freshness binding to the daemon's
binary hash so a proof is invalidated the moment the exe changes. With green
proofs, the self-review fleet-staleness cron can roll the stale voice daemons
unattended; without one, auto refuses and falls back to plan-only.

## Why this exists

- Journal 2026-06-13 "Pending your call": fleet-binary-staleness lists
  `rollout plan --only wm-audio` (etc.) and notes "Never auto-applied —
  daemon restarts drop subscribers; requires explicit approval." The manual
  gate is the only thing keeping the stale fleet stale.
- changeover-probe (PRD-changeover-probe.md) produces a per-daemon
  `{deafness_ms, events_missed_window, binary_hash}` proof; this PRD is what
  consumes it. changeover-warmswap (PRD-changeover-warmswap.md) is the
  mechanism that makes a proof come back green (0 events lost). autoapply
  depends on both (vision Order: probe → warmswap → autoapply).
- [[feedback_verify_before_concluding]]: the auto-gate must trust a *measured*
  proof, not an assumption — and the proof must be invalidated when the thing
  it measured changes (the binary), so stale-green can't auto-roll a daemon
  whose swap behavior was never actually tested.
- [[self_recall_baseline_gate_red]] / hermetic builds: keep the test suite
  green without a live fleet; the auto-gate logic is pure and unit-testable.

## What this builds

Extend the `rollout` crate. New module `src/autogate.rs` and a proof ledger.

- **Proof ledger** at `~/.config/rollout/proofs.json` (XDG-respecting):
  per-daemon `{daemon, binary_hash, deafness_ms, events_missed_window,
  recorded_at, strategy}`. `changeover probe` writes entries via a small
  `rollout record-proof --from <probe-json>` subcommand (so changeover stays
  a measurement tool and rollout owns its own ledger). Append/replace by
  daemon name.
- **Gate logic** (`autogate.rs`, pure): given a daemon's current installed
  binary hash, the ledger entry, and a threshold config
  (`max_events_lost` default 0, optional `max_deafness_ms`), return
  `Allow | Refuse { reason }`. Refuse when: no proof; proof's `binary_hash`
  ≠ current installed hash (stale proof); `events_missed_window` >
  `max_events_lost`; or (if set) `deafness_ms` > `max_deafness_ms`.
- **`rollout apply --auto`**: for each stale daemon in plan order, consult
  the gate. Allowed daemons are warm-swapped (or hard-restarted if that's
  their proven strategy); refused daemons are skipped with the reason printed
  and the daemon left for manual `rollout apply`. `--auto` never restarts a
  daemon without a current Allow.
- **Self-review wiring** (doc + a one-line invocation, not a code change to
  self-review): document that the fleet-staleness finding can call
  `rollout apply --auto` once proofs are green; the existing adopt-cron /
  self-review guardrail (report-only) is the integration point. Keep the
  actual auto-roll opt-in via a config flag (`auto_enabled = false` default)
  so shipping this PRD does not silently start restarting the live fleet.

## Acceptance criteria

1. `rollout apply --help` documents `--auto`; `rollout record-proof --help`
   documents `--from`.
2. `rollout record-proof --from <changeover-probe-json>` writes/updates the
   per-daemon entry in `~/.config/rollout/proofs.json`; a second record for
   the same daemon replaces (not duplicates) the entry.
3. Unit test: gate returns `Refuse` when no proof exists for a daemon.
4. Unit test: gate returns `Refuse{reason: stale}` when the proof's
   `binary_hash` differs from the current installed hash, and `Allow` when
   it matches and `events_missed_window == 0`.
5. Unit test: gate returns `Refuse` when `events_missed_window` exceeds
   `max_events_lost`, even with a matching hash.
6. `rollout apply --auto` with `auto_enabled = false` (default) performs no
   restarts and prints that auto is disabled — proving the ship is inert
   until explicitly enabled. With it enabled and a mock ledger, the apply
   path is driven in a dry/mock mode that restarts only Allowed daemons and
   skips Refused ones with reasons.
7. `cargo build --release` clean, `cargo test` green, hermetic (no live
   fleet or bus required for the suite).
