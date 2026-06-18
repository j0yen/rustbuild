# PRD: homeward-cadence-stray — poll the stray gold mine faster

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/homeward
Vision: visions/homeward.md

## TL;DR

homeward's ingest orchestrator already adapts polling cadence per source with
an AIMD loop — but it is blind to *what kind* of records a source returns. It
resets to the fast floor on any churn ≥10 and backs off on zero
(`orchestrator.rs:53-64`), treating a flood of adoptable-dog updates the same
as a flood of fresh STRAY intakes. The entire homeward thesis is that the
**stray/found population is the someone's-lost-pet population**. This PRD makes
the cadence stray-aware: a source actively producing STRAY intakes gets polled
faster; a source returning only adoptables backs off sooner.

## Why this exists

Phase-1 research (2026-06-17) read the live orchestrator:

- `ConnectorState` (orchestrator.rs:25-49) holds `interval`, `cadence_floor`,
  `max_interval`, `backoff_factor = 2.0`, `churn_threshold = 10`.
- `adapt(returned: usize)` (orchestrator.rs:53-64): `returned == 0` →
  `interval *= 2` (capped at `max_interval`); `returned >= churn_threshold` →
  reset to `cadence_floor`; otherwise unchanged. **`returned` is a flat count;
  `IntakeType` is never consulted.**
- The vision (and the original ingest PRD) explicitly motivate prioritizing
  STRAY feeds; `metro_from_name`/coverage already special-cases nothing about
  intake type in scheduling. This is the vision's own un-dreamt frontier item:
  "a scheduler that raises a source's cadence when it's actively returning
  STRAY intakes and backs off silent ones."
- `PetRecord.intake_type: IntakeType` with a `Stray` variant
  (homeward-schema intake.rs:11-26) is already populated by every connector, so
  the signal is free at ingest time — no new fetch.

## What this builds

Extends `homeward-ingest`'s orchestrator:

- Change the adaptation input from a flat `returned: usize` to a small
  `PollOutcome { total: usize, stray: usize }` (counting
  `IntakeType::Stray` + `FoundReport` as the lost-pet-relevant class),
  computed during the existing per-record loop in `tick()`
  (orchestrator.rs:122-209) at zero extra cost.
- New adaptation rule, layered on the existing AIMD so behavior degrades to
  current logic when stray==0:
  - `stray >= stray_floor_threshold` (e.g. ≥3 fresh strays) → reset to
    `cadence_floor` **and** optionally drop below floor toward a new
    `stray_floor` (a faster minimum) for the next few ticks (a short
    "hot" window with decay).
  - `stray == 0 && total > 0` (only adoptables churning) → do NOT treat as
    churn-reset; allow normal backoff to proceed (today a flood of adoptables
    pins it to the floor wastefully).
  - `total == 0` → unchanged (existing backoff).
- All thresholds (`stray_floor_threshold`, `stray_floor`, hot-window length)
  live in config with defaults; env-overridable like the existing knobs.
- Per-source counters surfaced for observability: cumulative `stray_seen`,
  current `interval`, and whether the source is in a hot window — exposed via
  the existing ingest status/metrics surface (or a new `--status` line).

No new deps. Pure logic change inside an existing crate.

## Acceptance criteria

1. The adaptation function consumes a stray-aware outcome
   (`{total, stray}`) rather than a flat count; an old-style call with
   `stray == 0` reproduces the *exact* current AIMD behavior (regression
   guard: same interval sequence as today for stray-free inputs).
2. A poll with `stray >= stray_floor_threshold` drives the interval to the
   fast minimum, proven by a unit test asserting the interval drops to
   `stray_floor`/`cadence_floor`.
3. A poll with `total > 0, stray == 0` does NOT reset to floor (adoptable
   churn no longer pins cadence), asserted against the prior behavior.
4. The hot window decays: after a stray burst, absent further strays the
   interval returns toward the normal floor/backoff curve within the
   configured window length; tested over a multi-tick sequence.
5. Stray classification counts `IntakeType::Stray` and `FoundReport` and
   excludes `Adoptable`/`OwnerSurrender`/`Transfer`; unit-tested on a mixed
   record batch.
6. Thresholds are configurable with documented defaults and env overrides;
   a test sets a non-default threshold and observes the changed behavior.
7. Per-source stray counters and hot-window state are observable through the
   ingest status surface; asserted by a test reading the emitted status.
8. `Orchestrator::tick()` computes the stray count from the records it
   already iterates — no second pass, no extra fetch (verified by code
   review note + a test that the connector is polled exactly once per tick).
9. `cargo test` green; `cargo build` green for the workspace; no new
   dependency added.

## Honesty notes

- This changes *polling priority*, never data: it does not fabricate strays,
  reclassify records, or alter what is stored.
- Default-off-equivalent: with stray==0 everywhere, the system behaves
  byte-for-byte like today, so shipping this cannot regress existing sources.
- "Below floor" hot polling must still respect upstream ToS rate limits — the
  `stray_floor` minimum must not be configurable below the slowest documented
  upstream refresh; enforce a hard lower bound and test it.
