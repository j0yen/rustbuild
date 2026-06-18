# PRD: careen-survey — intra-target reclaimable inventory

Status: Draft v0.1
build_target: rust-cli
Vision: visions/careen.md
Repo: j0yen/careen-survey (PUBLIC)

## TL;DR

The heaviest disk weight on this box is *inside* the target dirs of
living repos, where `ballast` (whole-dir reaping) and `hold` (target
sharing) structurally can't reach. `careen-survey` is a read-only tool
that classifies, per target dir, how many bytes are reclaimable by
*scraping the dir clean* without scrapping it — distinct from ballast's
reclaimable-by-deletion. It sizes the prize before any sweep touches a
byte.

## Why this exists

Evidence gathered 2026-06-16:

- `df /` = 97%, **17G free**; self-review flagged 86%→92%→97% over three
  runs.
- `du -sch ~/wintermute/*/target` = **215G**. The two largest,
  `recall/target` and `wintermute-brain/target`, are **13G each** and
  both LIVE — ballast can't reap them (installed binary current), hold
  can't drain them mid-flight.
- `recall/target/debug/deps` = **9.2G**, 295 `.rlib` files — most are
  superseded dep versions cargo never garbage-collects.
- `find ~/wintermute/*/target/debug/incremental -maxdepth 1 -type d
  -mtime +7` = **2550 stale incremental dirs** fleet-wide.
- `cargo-sweep` is not installed; nothing sizes intra-target reclaim
  today.

Without a survey, a sweep would be flying blind — careen-survey is the
keystone that makes the rest of the fleet honest (per Vision §Order, it
ships first and standalone).

## What this builds

A Rust CLI `careen-survey` (clap), read-only, no deletion path at all.

**Inputs:** one or more target-dir roots (default: glob
`~/wintermute/*/target`), `--toolchain-current <ver>` (default: detect
via `rustc --version`), `--stale-days <N>` (default 7).

**Classification (four reclaimable classes), per target dir:**

1. **stale-incremental** — `debug/incremental/*` (and `release/`) dirs
   with mtime older than `--stale-days`. Cargo always regenerates these.
2. **orphaned-deps** — `.rlib`/`.rmeta`/`.d` in `debug/deps/` whose
   crate+hash is not referenced by the repo's current `Cargo.lock`
   resolved set nor by any live `.fingerprint`. (Cheap-path heuristic;
   Vision open question #2 — survey reports both a conservative and an
   aggressive estimate so sweep can pick.)
3. **wrong-toolchain** — fingerprint/dep artifacts bucketed to a
   toolchain other than `--toolchain-current` (1.85 vs 1.88 split, read
   from `.fingerprint` rustc hash where derivable).
4. **dead-build** — `debug/build/<pkg>-<hash>/` dirs for pkg+hash combos
   no longer in the resolved dependency set.

**Output:** a JSON document — per target dir, bytes per class, a
`reclaimable_total`, and a fleet rollup with a `reclaimable_by_careening`
grand total. Human table on a TTY; `--format json` for machines.

**Crates:** `clap`, `serde`/`serde_json`, `walkdir`, `cargo_metadata`
(or `cargo-lock` crate to parse Cargo.lock without invoking cargo),
`anyhow`.

**Explicitly NOT in scope:** no deletion (that's careen-sweep); no
whole-dir reaping (that's ballast-survey — careen-survey must never
double-count a dir ballast would reap; it operates only on dirs whose
binary is current/in-use, and documents that boundary in `--help`).

## Acceptance criteria

1. `careen-survey --format json` over a fixture tree emits valid JSON
   with per-dir entries, each carrying byte counts for all four classes
   plus a `reclaimable_total`, and a fleet `reclaimable_by_careening`
   rollup.
2. The tool performs **zero** filesystem mutations — verified by a test
   that snapshots the fixture tree's inode/mtime set before and after a
   run and asserts no change.
3. stale-incremental classification respects `--stale-days`: a fixture
   incremental dir with mtime N days old is included iff N ≥ threshold.
4. orphaned-deps reports BOTH a conservative estimate (only rlibs with
   no Cargo.lock match) and an aggressive estimate (also unreferenced by
   live `.fingerprint`); a fixture with a known-orphan rlib and a
   known-live rlib classifies each correctly.
5. wrong-toolchain bucketing: a fixture with artifacts tagged to two
   rustc hashes reports the non-current bucket's bytes separately.
6. Running against a path that is NOT a cargo target dir exits non-zero
   with a clear message, not a panic.
7. `--help` documents the four classes and states the boundary vs
   ballast (whole-dir) explicitly.
8. The reported `reclaimable_total` per dir equals the sum of `du` over
   exactly the classified paths (±filesystem block rounding), proven by
   a test that cross-checks against a real `du` on the fixture.
