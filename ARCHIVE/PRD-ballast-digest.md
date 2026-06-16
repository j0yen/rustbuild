# PRD: ballast-digest — one ranked disk block self-review pastes instead of a hunch

**Status:** Draft v0.1
**build_target:** rust-cli
**Vision:** visions/ballast.md
**Repo:** j0yen/ballast-digest (NEVER AtScaleInc)
**Depends on:** ballast-survey (shipped), PRD-ballast-trend, PRD-ballast-pilot
  (consumes the guard event log)

## TL;DR

The vision's fifth end-state wants `/self-review` to "report a number instead of
a hunch." Today it reports the hunch: the 2026-06-16 review literally printed
*"Suggest `du -sh ~/wintermute/*/target | sort -h`"* as prose for a human to run
by hand. ballast-digest is the read-only synthesizer — the disk-side counterpart
to the already-shipped drydock-digest — that fuses the latest survey
(stock + fossil classification), the trend report (flow + ETA), and the guard
event log (last SLO band, reclaimed-bytes history) into a single ranked,
copy-pasteable block self-review drops in place of the `du` suggestion.

## Why this exists

- self-review 2026-06-16 "Pending your call" section: *"⚠️ DISK 92% … Suggest
  `du -sh ~/wintermute/*/target | sort -h | tail -10` to find bloated build
  targets."* — a manual instruction, not a finding.
- The drydock vision already proved this pattern works: drydock-digest replaced
  self-review's drift "prose wall" with a single ranked lane-grouped block
  (gossip 2026-06-16). Disk has the same prose-wall problem and now (after
  ballast-trend + ballast-pilot ship) the same three structured inputs available
  to collapse it.
- The raw materials exist but are scattered across three JSON surfaces
  (`ballast-survey --json`, `ballast-trend report --json`,
  `~/.local/state/ballast/guard-events.jsonl`); no one composes them, so
  self-review falls back to a `du` suggestion every run.

## What this builds

A Rust CLI `ballast-digest` (clap), read-only, no deletion, no scheduling.

**Inputs** (each optional; degrade gracefully if a source is missing):

- latest `ballast-survey --json` (stock + fossil/reap_safety per path)
- `ballast-trend report --json` (delta + bytes/day + ETA-to-high-water)
- `~/.local/state/ballast/guard-events.jsonl` (last SLO band/exit code, recent
  reclaimed-bytes events)

**Output** — one block, human by default, `--json` available:

- a one-line headline: current usage %, free bytes, SLO band (ok/warn/breach).
- top-K reclaimable entries ranked by `size × reap-safety` (safest-biggest
  first), each annotated fossil/stale/warm and "growing N/day" if trend has it.
- a flow line: fastest-growing path + ETA-to-high-water, or "stable" if nothing
  is on a climb.
- a ledger line: bytes reclaimed in the last window (from guard events) — the
  "number instead of a hunch."
- when a source is absent, the block says so explicitly ("trend: no snapshots
  yet") rather than omitting the line silently — no false "all clear."

**Determinism:** `--now <RFC3339>` for age/ETA rendering; no wall-clock in
library code so output is testable.

## Acceptance criteria

1. `ballast-digest` with all three inputs present emits a single block
   containing: usage headline, top-K ranked reclaimable list, a flow/ETA line,
   and a reclaimed-bytes-this-window line.
2. With `ballast-trend` data absent, the block still renders and the flow line
   reads an explicit "no trend snapshots yet" — never a fabricated rate and
   never a silent omission.
3. With the guard event log absent, the reclaimed-bytes line reads "no guard
   events yet"; the digest still exits 0.
4. Reclaimable entries are ranked safest-biggest-first (`size × reap_safety`),
   and each is labeled with its fossil/stale/warm class from survey.
5. `--json` output is stable and documented for downstream embedding into
   self-review's report step.
6. The tool performs no deletion and no scheduling; it only reads the three
   sources (verified: paths it lists still exist afterward).
7. `cargo test` green with fixed `--now`; `cargo clippy` clean.

## Out of scope

- Editing the self-review skill to call this digest — that wiring is a
  follow-on the user opts into once the block proves itself (mirrors how
  drydock-digest was adopted).
- Any deletion or reaping — digest reports, it never acts.
