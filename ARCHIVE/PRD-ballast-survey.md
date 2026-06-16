# PRD: ballast-survey — honest inventory of reclaimable disk weight

**Status:** Draft v0.1
**build_target:** rust-cli
**Vision:** visions/ballast.md
**Repo:** j0yen/ballast-survey (NEVER AtScaleInc)

## TL;DR

The laptop is 94% full and nobody can say *which* directories are safe to
delete, because there is no inventory. ballast-survey is the read-only
measurement layer the rest of the ballast fleet trusts: it walks a set of roots
(default `~/wintermute`), finds every reclaimable subtree — `target/` dirs,
cargo registry/git caches, stray `node_modules`/`.venv`, big `~/.cache`
children — sizes each, records its last-modified mtime and age, and emits a
structured JSON inventory sorted by reclaimable bytes. It deletes nothing. It is
the ground truth `ballast-reap` and `ballast-guard` act on.

## Why this exists

Verified on 2026-06-16:

- `df -h /` → **94% used, 28G free of 468G**. The self-review journal for
  2026-06-16 names this the day's headline: *"Disk jumped 92% in one day …
  Build targets are the likely culprit … Priority: `du -sh ~/wintermute/*/target`
  sweep."* That `du` suggestion is the entire current remedy — manual, ad hoc,
  human-run.
- `du -sch ~/wintermute/*/target` → **205G total across 174 `target/` dirs**.
  Top offenders: `wintermute-brain/target` 13G, `recall/target` 13G,
  `wintermute-audio/target` 11G, `agorabus/target` 6.7G.
- `~/.cache` is **46G**; `~/.cargo/registry` is 1.6G. These are reclaimable but
  uninventoried.

There is no tool that turns "the disk is full" into "here are the N directories,
their sizes, their ages, ranked." Every reclamation decision today is a guess.
ballast-survey makes the measurement first-class so deletion is never blind.

## What this builds

A single Rust CLI `ballast-survey` (clap), no network, no deletion.

**Modules**
- `roots` — resolve scan roots from config + `--root` flags (default
  `~/wintermute`); expand `~`.
- `walk` — for each root, find reclaimable subtrees by rule:
  - any dir literally named `target/` that contains a `CARGO_OK`/`.rustc_info.json`
    or sits beside a `Cargo.toml` (Rust build dir),
  - `node_modules/`, `.venv/`, `__pycache__/`,
  - top-level children of `~/.cache` over a size floor,
  - `~/.cargo/registry/cache` and `~/.cargo/git/checkouts`.
  Do not descend into a reclaimable dir once matched (don't double-count nested).
- `size` — sum apparent size via `walkdir`; record entry count.
- `classify` — per entry: `kind` (rust-target | node-modules | venv | pycache |
  cargo-cache | cache-child), `bytes`, `entries`, `mtime` (newest file within),
  `age_days`, owning crate/dir name.
- `emit` — JSON array sorted by `bytes` desc, plus a summary header
  (`total_bytes`, `reclaimable_bytes`, `disk_used_pct`, `disk_free_bytes`,
  `scanned_at` from a caller-supplied or env timestamp — no wall-clock in tests).

**Deps:** `clap`, `serde`/`serde_json`, `walkdir`, `humansize`, `anyhow`.
No `Date::now()` in library code — accept `--now <rfc3339>` or `BALLAST_NOW`
for deterministic age math; CLI default reads the clock at the boundary only.

**UX**
```
ballast-survey                      # human table, biggest first
ballast-survey --json               # machine inventory (reap/guard consume this)
ballast-survey --root ~/wintermute --root ~/.cache
ballast-survey --min-size 100M      # floor
```

## Acceptance criteria

1. `ballast-survey --json` against a fixture tree emits a JSON array of entries,
   each with `path`, `kind`, `bytes`, `entries`, `mtime`, `age_days`, `crate`.
2. Entries are sorted by `bytes` descending; the summary header reports
   `reclaimable_bytes` equal to the sum of entry `bytes`.
3. A nested reclaimable dir (e.g. a `node_modules` inside a `target`) is counted
   exactly once — the outer match wins, no double-counting.
4. `--min-size 100M` excludes every entry below the floor.
5. Age math is deterministic under `--now <rfc3339>` (no dependence on wall
   clock); a fixture with a known mtime yields a known `age_days`.
6. The tool never writes to or deletes any scanned path (verified: a read-only
   fixture dir survives a full run unchanged).
7. `--help` documents every flag; exit 0 on success, non-zero on an unreadable
   root with a clear stderr message.
8. `cargo test` green; `cargo clippy` clean on the crate's own code.
