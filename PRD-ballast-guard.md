# PRD: ballast-guard — defend a disk high/low-water SLO autonomously

**Status:** Draft v0.1
**build_target:** rust-cli
**Vision:** visions/ballast.md
**Repo:** j0yen/ballast-guard (NEVER AtScaleInc)

## TL;DR

survey measures, reap frees — but both are pull operations a human still has to
trigger. ballast-guard closes the loop: it watches the disk against a
high/low-water SLO and acts before the laptop hits the wall. Under the
high-water mark it reaps the *safest* candidates (fossil first) until usage
drops below the low-water mark, emitting a structured alert event at each
transition (no transport hard-coded — a notifier composes it). Below high-water
it just reports. The 94%-full surprise that prompted this whole vision becomes a
self-correcting dip the operator reads about after the fact.

## Why this exists

Verified 2026-06-16: the disk went from ~88% to **94% in a single day** during a
heavy `/build` run — the journal calls it a "major growth event." Nothing
watched it; it was caught only by the next morning's `/self-review`. By then
there was 28G left of 468G. A box that builds Rust continuously and routes the
heavy compiles to the cloud (leaving fossil targets behind — see
PRD-ballast-cloudaware) needs a *standing* defense, not a daily manual sweep.
This is the same SLO-watcher shape the fleet already uses for accuracy/parity
(mqo-drift-watch: exit-code SLO contract, `--accept` baseline gate), pointed at
free bytes instead.

## What this builds

A Rust CLI `ballast-guard` (clap) that orchestrates survey + reap behind an SLO.

**Config** (`~/.config/ballast/guard.toml`):
- `high_water_pct` (default 90) — at/above this, reap.
- `low_water_pct` (default 80) — reap until below this, then stop.
- `advisory_pct` (default 85) — at/above this but below high-water, alert-only.
- `max_safety` — the *least* safe class guard is allowed to reap autonomously
  (default `fossil`; never widens itself past this without a human edit).
- `roots` — scan roots passed through to survey.

**Loop (single pass — `ballast-guard run`)**
1. Read `df` for the target filesystem → `used_pct`, `free_bytes`.
2. If `used_pct < advisory_pct` → emit `ok` event, exit 0.
3. If `advisory_pct ≤ used_pct < high_water_pct` → emit `warn` event with the
   reclaimable total ballast-survey would find, exit 2 (warn). Reap nothing.
4. If `used_pct ≥ high_water_pct` → run survey, select candidates at/safer than
   `max_safety`, and reap (via ballast-reap `--apply`) in `reap_safety` order,
   largest-first within a class, **stopping as soon as projected free crosses
   `low_water_pct`**. Emit a `breach` event recording bytes reclaimed and the
   resulting `used_pct`. Exit 3 (breach-acted).
5. If high-water is still breached after exhausting `max_safety` candidates →
   emit a `breach-unresolved` event (human must widen safety or move data),
   exit 4.

**Events** — structured JSON to stdout and/or an event file
(`--event-sink <path>`); fields: `level` (ok|warn|breach|breach-unresolved),
`used_pct_before`, `used_pct_after`, `bytes_reclaimed`, `candidates`, `ts`. No
mail/slack/bus transport in this crate — it emits, a notifier composes.

**Composition** — guard shells out to the installed `ballast-survey` and
`ballast-reap`; it does not re-implement them. If either is absent it exits
non-zero with a clear "ballast-reap not on PATH" message rather than guessing.

**Deps:** `clap`, `serde`/`serde_json`, `toml`, `nix`/`libc` (or `statvfs`
crate) for `df`, `anyhow`.

## Acceptance criteria

1. With simulated `used_pct` below `advisory_pct`, guard emits an `ok` event,
   reaps nothing, exits 0.
2. With `used_pct` in the advisory band, guard emits a `warn` event including a
   reclaimable-bytes estimate, reaps nothing, exits 2.
3. With `used_pct ≥ high_water_pct`, guard invokes reap on `fossil` candidates
   largest-first and **stops** once projected free crosses `low_water_pct`
   (verified: it does not over-reap a class it didn't need).
4. guard never reaps a class less safe than `max_safety`; with only
   `stale-uninstalled` candidates left it emits `breach-unresolved` and exits 4
   rather than deleting unsafe state.
5. Every transition emits a structured event with `level`, `used_pct_before`,
   `used_pct_after`, `bytes_reclaimed`, `ts`; `--event-sink` appends to a file.
6. Config thresholds are read from `guard.toml`; invalid ordering
   (`low_water ≥ high_water`) is rejected at startup with a clear error.
7. With `ballast-reap` or `ballast-survey` absent from PATH, guard exits
   non-zero with a precise message and reaps nothing.
8. Exit codes follow the SLO contract (0 ok / 2 warn / 3 breach-acted /
   4 breach-unresolved); `cargo test` + `cargo clippy` green.
