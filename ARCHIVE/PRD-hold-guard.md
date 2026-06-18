# PRD: hold-guard — bound the shared hold so it can't regrow to 214G

**Status:** Draft v0.1
**build_target:** rust-cli
**Vision:** visions/hold.md
**Repo:** j0yen/hold-guard (NEVER AtScaleInc)

## TL;DR

A shared hold dedups dependency builds, but a single growing directory is still
unbounded — without a cap, the union of every crate the fleet ever builds
accretes until `/` fills again. hold-guard caps the shared hold
(`~/wintermute/.hold/target`) at a configurable size and LRU-evicts the
least-recently-used crate fingerprint subdirectories when it crosses the cap,
emitting the **same structured SLO event schema `ballast-guard` already uses** so
a notifier can surface it. It never evicts artifacts a build is actively using,
and dry-run is the default.

## Why this exists

Verified 2026-06-16: the disk hit 97% precisely because 214G of build artifacts
accreted with nothing bounding them. hold-anchor + hold-migrate collapse the
*sum* to the *union*, but the union itself grows every time the fleet locks a new
dependency version. ballast-guard defends a whole-disk SLO by reaping fossils;
hold-guard defends the *hold's own* budget so the shared commons can't become the
next 214G. They compose: hold-guard emits ballast-guard's event schema rather
than inventing a parallel one (vision open question #2), so one notifier handles
both.

cargo's target layout stores per-fingerprint artifacts under `target/<profile>/`
with stable subdir naming, and access/mtime reflects last use — making LRU
eviction by fingerprint dir both possible and safe (cargo simply rebuilds an
evicted artifact on next use).

## What this builds

A single Rust CLI `hold-guard` (clap). It measures the hold, evicts LRU
fingerprint dirs over a cap, emits events. No network.

**Modules**
- `measure` — size the hold; enumerate evictable units (per-fingerprint subdirs
  under `deps/`, `.fingerprint/`, `incremental/`) with bytes + last-access/mtime.
- `policy` — given a `--max-size` cap and a `--low-water` target, select the
  LRU units to evict until the hold drops below low-water; never select a unit
  whose artifact a running build holds (reuse hold-migrate's in-flight guard
  approach: open-fd / `.cargo-lock` check).
- `evict` — dry-run by default; `--apply` removes the selected units; measure
  bytes reclaimed.
- `event` — emit a structured SLO event matching ballast-guard's schema
  (`kind`, `severity`, `hold_bytes`, `cap_bytes`, `reclaimed_bytes`,
  `ts` from env/flag); no transport hardcoded — print JSON to stdout / a
  configured event sink so a notifier composes it.
- `ledger` — append evictions to `~/wintermute/.hold/guard-ledger.jsonl`
  (append-only).

**Deps:** `clap`, `serde`/`serde_json`, `walkdir`, `humansize`, `anyhow`.

**UX**
- `hold-guard check --max-size 60G` → dry-run: over/under cap? what would evict?
- `hold-guard enforce --max-size 60G --low-water 45G --apply` → evict LRU to
  low-water; emit event; append ledger.
- `hold-guard status` → hold size vs cap, last enforcement, ledger tail.

## Acceptance criteria

1. `hold-guard --help` lists `check`, `enforce`, `status`; exits 0.
2. `check --max-size <N>` against a fixture hold under the cap reports
   `over_cap: false`, selects nothing, reclaims nothing (read-only).
3. `check --max-size <N>` against a fixture hold over the cap reports
   `over_cap: true` and lists the LRU units it *would* evict, oldest-access
   first, until the projected size is below `--low-water`.
4. `enforce --apply` removes exactly the selected LRU units, reports
   `reclaimed_bytes` > 0, brings the hold below `--low-water`, and appends
   ledger lines.
5. The emitted event validates against ballast-guard's event schema (same field
   names/types); a fixture-based schema check passes.
6. A fingerprint unit with a simulated held lock / open fd is never selected for
   eviction even when it is the LRU candidate.
7. `enforce` is dry-run unless `--apply` is given: without `--apply` it reclaims
   nothing and writes no ledger line.
8. The ledger is append-only across runs; line count only grows.
9. `--ts <rfc3339>` makes event + ledger timestamps deterministic (no wall-clock
   in tests).
