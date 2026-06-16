# PRD: drydock-ledger — prove the fleet is converging, not just churning

**Status:** Draft v0.1
**build_target:** rust-extend
**build_into:** /home/jsy/wintermute/drydock-survey
**Vision:** visions/drydock.md
**Repo:** j0yen/drydock-survey (NEVER AtScaleInc)

## TL;DR

A digest shows today's drift, but it can't tell you whether yesterday's drift
got fixed or just rolled forward unchanged. drydock-ledger is the convergence
instrument: an append-only JSONL record of each run's per-lane counts and item
fingerprints. It asserts the `auto` lane is non-increasing over time (if the
safe lane grows, either a regression shipped or the auto-drain isn't running),
and it ages the long-tail `window`/`reboot`/`approval` items so an item parked
too long (wm-stt 9d) gets escalated instead of silently re-listed.

## Why this exists

Verified 2026-06-16:

- The fixpoint vision already named this exact failure mode for the *adopt*
  loop: *"Run after run, self-review still reports a flat `N/N not-current`.
  Nobody has wired the cure to run autonomously"* — and fixpoint's answer was a
  convergence ledger that "asserts the count is non-increasing." drydock needs
  the same instrument but across **all** lanes, not just adopt markers.
- The recurring evidence is precisely a non-converging count: self-reviews
  2026-06-12 → 2026-06-16 each re-report "adopt … installed-stale (many)" and
  "fleet-binary-staleness" with no run-over-run trend — the human cannot tell if
  it's getting better or worse because nothing records the series.
- Long-tail items rot silently: wm-stt went from listed to "9d stale" with no
  point at which the system said "this has been parked too long, escalate."

## What this builds

Extends the `drydock-survey` crate with a `ledger` module and a `drydock-ledger`
subcommand/binary.

**Ledger format** — append-only JSONL at `~/.config/drydock/ledger.jsonl`
(override via `--ledger`). One record per run:
```
{ "run_ts": "<rfc3339, caller-supplied>", "counts": {"auto":3,"window":1,
  "reboot":1,"approval":2}, "items": [{"item":"wm-stt","lane":"window",
  "first_seen":"<ts>","age_days":9}, ...] }
```
`first_seen` is carried forward from the earliest prior record that names the
item (so age reflects how long it has been *parked*, distinct from the binary's
own build age).

**Operations**
- `drydock-ledger record` — consume classify JSON, append a run record,
  carrying `first_seen` forward per item.
- `drydock-ledger check` — read the last two records; **exit non-zero if the
  `auto` lane count increased** (regression or drain-not-running), exit 0
  otherwise. Print the per-lane delta.
- `drydock-ledger escalate --days N` — list items whose parked age
  (`run_ts - first_seen`) exceeds N, for self-review to surface as urgent.
- `drydock-ledger delta` — emit the per-lane Δ vs previous run as JSON, for
  drydock-digest's header.

**Determinism:** all timestamps are caller-supplied (`--now`/`DRYDOCK_NOW`); no
wall-clock in library code, so the carry-forward and escalation logic are
unit-testable against fixed series.

**Deps:** inherits survey's; `serde_json` lines. No network.

**UX**
```
drydock-classify --json | drydock-ledger record --now <ts>
drydock-ledger check          # CI/self-review gate: auto lane must not grow
drydock-ledger escalate --days 7
```

## Acceptance criteria

1. `drydock-ledger record` appends one JSONL run record with per-lane `counts`
   and an `items` array; the file is append-only (prior lines untouched).
2. `first_seen` is carried forward: an item present in run N-1 keeps its earlier
   `first_seen` in run N; a new item gets `run_ts` as its `first_seen`.
3. `drydock-ledger check` exits non-zero when the `auto` lane count is higher
   than the previous record's, exit 0 when equal or lower; prints per-lane Δ.
4. `drydock-ledger escalate --days N` lists exactly the items whose
   `run_ts - first_seen` exceeds N days; verified against a fixed series fixture.
5. `drydock-ledger delta` emits per-lane Δ vs the previous run as JSON consumable
   by drydock-digest.
6. All time math is deterministic under `--now`/`DRYDOCK_NOW`; a multi-run
   fixture yields known `first_seen`, ages, and deltas with no wall-clock.
7. A first-ever run (empty ledger) records cleanly and `check` exits 0.
8. `--help` documents every flag; `cargo test` green; `cargo clippy` clean on the
   crate's own code.
