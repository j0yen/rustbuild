# PRD: ballast-trend — measure the *flow* of disk weight, not just the stock

**Status:** Draft v0.1
**build_target:** rust-cli
**Vision:** visions/ballast.md
**Repo:** j0yen/ballast-trend (NEVER AtScaleInc)
**Depends on:** ballast-survey (consumes its `--json` snapshots) — already shipped

## TL;DR

ballast-survey answers "what is big *right now*." It cannot answer "what is
*growing*, how fast, and when do we hit the wall" — and that flow question is
exactly what self-review keeps failing to answer. On 2026-06-16 the journal
could only say disk "jumped 92% in one day — major growth event… build targets
are the likely culprit" — a *guess*, because nothing on the box records the
derivative. ballast-trend snapshots successive survey runs into a small ring,
diffs them, and reports per-directory growth rate (bytes/day), the
fastest-growing entries, and a projected ETA to the high-water mark. It turns
"the disk surprised us again" into "recall/target is adding 1.4G/day; you have
~6 days."

## Why this exists

The disk moved **86% → 92% → 96%** across three days (self-review journals
2026-06-14/15/16). Each review measured the *level* and guessed at the *cause*:

- 2026-06-16 journal: *"Disk jumped 92% in one day — major growth event. Build
  targets are the likely culprit … Suggest `du -sh ~/wintermute/*/target`."*

That is the signature of a missing derivative. The vision's fifth end-state
explicitly asks the loop to answer *"what keeps re-growing"* — but every shipped
ballast PRD measures a single point in time. `du` at draft time shows
`recall/target` 13G and `wintermute-brain/target` 13G; survey's cloudaware layer
can tell us brain's target is fossil (its binary is newer), but **only a
time-series can tell us which target is the one actively re-inflating** after
each reap — the difference between a one-time win and a recurring leak.

## What this builds

A Rust CLI `ballast-trend` (clap), reading the same `ballast-survey --json`
schema the rest of the fleet trusts. No deletion, no mutation of survey state —
pure measurement.

**Subcommands:**

- `ballast-trend snapshot` — run (or read piped) `ballast-survey --json`, stamp
  it with a timestamp, and append to a bounded ring at
  `~/.local/state/ballast/trend/` (default keep last N=30 snapshots; oldest
  pruned). Deterministic time via `--now <RFC3339>` for tests.
- `ballast-trend report` — diff the two most recent snapshots (or `--since
  <RFC3339>` / `--last <N>` to span a window), compute per-path delta bytes and
  bytes/day, rank the fastest-growing paths, and project ETA to a configurable
  high-water mark (`--high-water-pct`, default reads from the trend's own flag,
  not coupled to guard). Human table by default; `--json` for machines.
- `ballast-trend report --json` feeds PRD-ballast-digest.

**Honesty constraints:**

- A path present in the new snapshot but absent in the old is reported as "new,
  no rate yet" — never extrapolated from a single point.
- A path that shrank (e.g. after a reap) shows a negative rate and is excluded
  from ETA-to-full, with a note, so a reclamation doesn't read as a leak.
- All sizes are taken from survey's measured bytes; ballast-trend never re-walks
  the filesystem itself (single source of truth = survey).

## Acceptance criteria

1. `ballast-trend snapshot --now <T1>` then `--now <T2>` writes two timestamped
   snapshots to the ring; the ring never exceeds the keep-N bound.
2. `ballast-trend report` over two snapshots emits, for each changed path, the
   delta bytes and a bytes/day rate consistent with the `(T2 - T1)` interval.
3. A path appearing only in the newer snapshot is labeled "new — no rate" and is
   never assigned a fabricated rate.
4. A path that shrank between snapshots shows a negative delta and is excluded
   from the ETA-to-high-water projection (with an explanatory note).
5. `--json` output is stable, documented, and round-trips through `jq`;
   PRD-ballast-digest consumes it.
6. `ballast-trend` performs no deletion and no filesystem walk of its own — it
   reads only `ballast-survey --json` and its own snapshot ring (verified: an
   `strace`/dry inspection shows no `unlink`; functionally, paths it reports as
   growing still exist and are untouched).
7. `cargo test` green, `cargo clippy` clean; ETA math is unit-tested with fixed
   `--now` values (no wall-clock in library code).

## Out of scope

- Acting on the trend (reaping the leak) — that remains guard+reap's job.
- Cross-mount or non-`~/wintermute` roots beyond whatever survey was pointed at.
