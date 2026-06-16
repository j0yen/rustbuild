# PRD: careen-guard — SLO-triggered careen of live target giants

Status: Draft v0.1
build_target: rust-cli
Vision: visions/careen.md
Repo: j0yen/careen-guard (PUBLIC)
Depends on: PRD-careen-sweep.md (reclaim engine), PRD-careen-survey.md

## TL;DR

`ballast-guard` watches the disk SLO and reaps whole dead target dirs
when usage breaches a watermark — but it structurally *cannot* touch the
two largest dirs on the box (`recall/target`, `wintermute-brain/target`,
13G each) because their binaries are current and in use. `careen-guard`
is the arm that catches exactly those: when disk breaches, it careens
the largest LIVE target dirs using careen-sweep, emitting the *same*
SLO event schema ballast-guard already defines so the two compose into
one disk-defense story rather than two forks.

## Why this exists

- `df /` = 97%, 17G free; growth accelerating (self-review: 86→92→97%
  over three runs). ballast-guard shipped 2026-06-16 but cannot resolve
  this alone — its fossil-first reaper skips binary-current/in-use dirs,
  which is precisely where 26G+ (recall 13G + brain 13G) sits.
- ballast-guard already defines a clean `Event` schema
  (`~/wintermute/ballast-guard/src/event.rs`): `Level`
  (Ok/Warn/Breach/BreachUnresolved), `used_pct_before`, `used_pct_after`,
  `bytes_reclaimed`, `reclaimable_bytes`, `candidates`, `ts`. Forking it
  would split the disk-event stream; careen-guard emits the same shape.
- ballast-pilot already wires a systemd timer for ballast-guard — the
  pattern to follow for careen-guard's own cadence (or, per Vision open
  question #1, a shared evaluator).

## What this builds

A Rust CLI `careen-guard run` mirroring ballast-guard's surface:

- `--mount <path>` (default `/`), `--config <guard.toml>`,
  `--event-sink <file>`.
- Reads high/low-water thresholds from config (same TOML shape family as
  ballast's `guard.toml`).
- On a pass: read disk usage. If below advisory → emit `Ok`. In advisory
  band → emit `Warn` with a `reclaimable_bytes` estimate (from
  careen-survey over the live giants). On breach → select the largest
  LIVE target dirs (binary-current, not in-flight), run careen-sweep
  (conservative by default) against them in descending-size order until
  usage drops below high-water or safe candidates exhaust, then emit
  `Breach` (resolved) or `BreachUnresolved`.
- **Coordination with ballast-guard:** careen-guard only considers dirs
  ballast would NOT reap (current binary, in-use). Document the
  partition so the two guards never double-act on the same dir. Vision
  open question #1 (shared watermark evaluator) is noted as the
  follow-on; this PRD ships the careen arm standalone first.

**Event compatibility (hard requirement):** careen-guard's emitted JSON
must deserialize against ballast-guard's `Event`/`Level` types
unchanged. Achieve this by depending on ballast-guard's event module as
a library (preferred) or by reproducing the exact serde contract and
proving compatibility in a test.

**Crates:** `clap`, `serde`/`serde_json`, `toml`, `anyhow`; reuse
`careen-sweep` (reclaim) and `careen-survey` (estimate) as deps; depend
on `ballast-guard`'s event types if exposed, else replicate + test.

**NOT in scope:** the systemd timer unit (a `careen-pilot` follow-on,
or fold into ballast-pilot per open question #1); rebuild-cost
accounting (careen-ledger).

## Acceptance criteria

1. `careen-guard run --mount <fixture>` below the advisory threshold
   emits a single `Ok` JSON event and runs no sweep.
2. In the advisory band, it emits a `Warn` event carrying a non-zero
   `reclaimable_bytes` estimate and performs zero deletions.
3. On a simulated breach, it selects live target dirs in
   descending-size order, invokes careen-sweep against them, and emits
   `Breach` with `bytes_reclaimed > 0` and the swept paths in
   `candidates`.
4. When safe candidates are exhausted but usage is still above
   high-water, it emits `BreachUnresolved` (not a panic, not a silent
   exit).
5. **Schema compatibility:** a careen-guard event JSON is successfully
   deserialized by ballast-guard's `Event` type (or an equivalent test
   proving field-for-field serde parity).
6. careen-guard never selects a dir whose binary is stale/uninstalled
   (ballast's territory) — asserted by a fixture mixing a
   ballast-eligible dir and a careen-eligible dir; only the latter is
   touched.
7. `--event-sink` appends one JSON line per pass to the named file in
   addition to stdout, matching ballast-guard's sink behavior.
8. A breach pass that calls careen-sweep respects sweep's lock-safety:
   if a selected target is build-locked, it is skipped (not corrupted)
   and the pass continues to the next candidate.
