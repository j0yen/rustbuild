# PRD: hold-survey — measure the dependency duplication across the fleet

**Status:** Draft v0.1
**build_target:** rust-cli
**Vision:** visions/hold.md
**Repo:** j0yen/hold-survey (NEVER AtScaleInc)

## TL;DR

The laptop is 97% full and 214G of that is Rust `target/` dirs, but nobody can
say how much of it is *redundant* — the same dependency compiled into 190
private build directories. hold-survey is the read-only measurement layer the
rest of the hold fleet trusts: it walks every repo under `~/wintermute`, parses
each `Cargo.lock`, sizes each repo's dependency-build output, and computes the
**sum-of-per-repo-dep-builds vs the deduplicated union** (by package + version +
features). It deletes nothing and changes no config. It turns "a shared target
would save a lot" into a measured byte figure.

## Why this exists

Verified 2026-06-16:

- `df -h /` → **97% used, 18G free of 468G**.
- `du -sch ~/wintermute/*/target` → **214G across 190 `target/` dirs**.
- `echo $CARGO_TARGET_DIR` → empty; `which sccache` → **not found**. There is no
  sharing and no local compilation cache.
- Per-repo dep builds dominate and overlap: `recall/target/debug/deps` = 9.2G,
  `wintermute-brain` 5.7G deps + 3.3G incremental, `agorabus` 3.1G deps.
  `Cargo.lock` package counts (recall 406, agorabus 92, mqo-bench 65) share the
  common Rust crates (`serde`, `tokio`, `clap`, `anyhow`) recompiled fleet-wide.

ballast-survey (shipped) inventories reclaimable *fossil* weight — dirs safe to
delete because they're dead. It does **not** answer "how much is reclaimable by
*sharing* live builds." hold-survey adds that orthogonal measurement so
hold-anchor flips the shared-target switch against a number, not a hope.

## What this builds

A single Rust CLI `hold-survey` (clap), no network, no deletion, no config writes.

**Modules**
- `discover` — find every repo under configured roots (default `~/wintermute`):
  a dir containing `Cargo.toml`. Record path, crate name, and pinned toolchain
  (read `rust-toolchain.toml` / `rust-toolchain` if present, else `default`).
- `locks` — parse each repo's `Cargo.lock`; collect the set of
  `(name, version, source)` packages. Compute, per repo, the dependency package
  set; across the fleet, the union and the per-package repo-multiplicity.
- `size` — for each repo, size `target/` and specifically `target/*/deps` and
  `target/*/incremental`; record bytes + newest-file mtime + age_days.
- `dedup` — estimate redundant bytes: sum of per-repo dep-build bytes minus an
  estimate of the union (apportion a repo's dep-build bytes across its locked
  deps; a package locked in N repos counts once toward the union, N times toward
  the sum). This is an estimate — label it as such; exact union needs a real
  shared build. Bucket the estimate by pinned toolchain (incompatible artifacts
  don't dedup across toolchains).
- `concurrency` (optional, best-effort) — if `ctrace` is available, query recent
  history for overlapping local `cargo` invocations to inform the shared-target
  lock-contention open question; degrade silently if absent.
- `emit` — JSON: a `summary` header (`total_target_bytes`,
  `sum_dep_build_bytes`, `estimated_union_bytes`, `estimated_reclaimable_bytes`,
  `repo_count`, `toolchain_buckets`, `disk_used_pct`, `disk_free_bytes`,
  `scanned_at` from env/flag — no wall-clock in tests) plus a `repos` array and a
  `hot_packages` array (top packages by repo-multiplicity × estimated bytes).

**Deps:** `clap`, `serde`/`serde_json`, `walkdir`, `toml`, `humansize`, `anyhow`.

**UX**
- `hold-survey` → JSON to stdout.
- `hold-survey --root <dir>` (repeatable) → override scan roots.
- `hold-survey --format table` → human summary (top reclaimable-by-sharing
  packages + the headline union-vs-sum number).
- `--scanned-at <rfc3339>` / `HOLD_SCANNED_AT` → deterministic timestamp for tests.

## Acceptance criteria

1. `hold-survey --help` lists `--root`, `--format`, `--scanned-at`; exits 0.
2. Against a fixture tree of ≥3 fake repos (each a `Cargo.toml` + `Cargo.lock` +
   a `target/debug/deps` dir of known size) with overlapping locked packages,
   the JSON `summary.estimated_reclaimable_bytes` > 0 and equals
   `sum_dep_build_bytes - estimated_union_bytes`.
3. A package locked in all fixture repos appears in `hot_packages` with
   `repo_multiplicity` equal to the repo count.
4. `repos[].toolchain` reflects a fixture `rust-toolchain.toml`; repos with
   differing toolchains land in distinct `summary.toolchain_buckets` and their
   dep bytes are not deduplicated against each other.
5. The tool writes no files and modifies no config (verify a fixture-tree
   checksum is unchanged after a run).
6. With `--scanned-at` fixed, output is byte-identical across two runs
   (deterministic; no wall-clock).
7. `--format table` prints the headline `sum → union (reclaimable)` line and the
   top-10 `hot_packages`; exits 0.
8. Missing/`unparseable` `Cargo.lock` in one repo is reported in a
   `warnings` array and does not abort the scan of the others.
