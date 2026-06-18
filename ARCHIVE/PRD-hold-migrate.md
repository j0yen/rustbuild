# PRD: hold-migrate — drain private target/ dirs into the shared hold, safely

**Status:** Draft v0.1
**build_target:** rust-cli
**Vision:** visions/hold.md
**Repo:** j0yen/hold-migrate (NEVER AtScaleInc)

## TL;DR

hold-anchor makes *future* builds share one hold, but the 214G of *existing*
private `target/` dirs stay on disk until something reclaims them. hold-migrate
is that reclaimer, scoped to the hold transition: for each repo, it confirms the
repo is anchored and its installed binary is current (reusing ballast's fossil
classification), then `cargo clean`s the private `target/` so the next build
repopulates the shared hold instead. It is dry-run by default, `--apply`-gated,
records every reclaimed path and byte count to an append-only ledger, and never
cleans a `target/` while a build holds it.

## Why this exists

Verified 2026-06-16: 214G across 190 private `target/` dirs. Once hold-anchor
redirects `build.target-dir`, those private dirs become pure dead weight — but
nothing removes them automatically, and a blind `rm -rf */target` is exactly the
unsafe move ballast was built to avoid (never reap a build in flight; never
delete a target whose binary isn't reproducibly installed). hold-migrate applies
ballast's safety ethic to the *transition*: clean only what the shared hold can
reproduce, only after the repo is anchored, only when no build is running.

ballast-reap reclaims *fossil* targets (dead, old). hold-migrate reclaims
*live-but-now-redundant* targets — repos still in use, whose private target is
superseded by the shared hold. Different trigger, same safety gate; hold-migrate
reuses ballast's installed-binary-current check rather than reinventing it.

## What this builds

A single Rust CLI `hold-migrate` (clap). It calls `cargo clean` (or removes the
private `target/`) per repo, gated; it writes an append-only ledger.

**Modules**
- `plan` — for each repo under the root: is the fleet anchored (a hold
  `target-dir` is in effect)? does a private `target/` still exist? is the repo's
  installed binary current vs source (call `ballast`/`binstale`/`adopt` if
  present; else a conservative mtime+`cargo build --quiet` check)? Classify each
  repo `safe-to-clean | not-anchored | binary-stale | build-in-flight | no-target`.
- `guard` — refuse to clean a `target/` if a `cargo`/`rustc` process has it open
  (scan `/proc/*/cwd` + open fds, or check for `target/.cargo-lock`); mark
  `build-in-flight` and skip.
- `reclaim` — for `safe-to-clean` repos, `cargo clean` (preferred, respects
  cargo's own layout) or remove `target/`; measure bytes freed.
- `ledger` — append one JSON line per reclaimed repo to
  `~/wintermute/.hold/migrate-ledger.jsonl` (`repo`, `bytes_freed`,
  `classification`, `ts` from env/flag); never rewrite history.
- `report` — dry-run summary: total reclaimable bytes, per-classification
  counts, the repos that would be cleaned vs skipped and why.

**Deps:** `clap`, `serde`/`serde_json`, `walkdir`, `humansize`, `anyhow`.

**UX**
- `hold-migrate plan` → dry-run JSON/table; reclaims nothing.
- `hold-migrate apply` → clean the `safe-to-clean` repos; append ledger.
- `hold-migrate apply --repo <name>` → migrate one repo.
- `hold-migrate ledger` → print the append-only ledger.

## Acceptance criteria

1. `hold-migrate --help` lists `plan`, `apply`, `ledger`; exits 0.
2. `plan` is read-only: against a fixture tree it reclaims nothing and a
   fixture checksum is unchanged after the run.
3. `plan` classifies a fixture repo with no shared anchor as `not-anchored` and
   does **not** mark it `safe-to-clean` (never clean before anchoring).
4. `plan` classifies a repo whose installed binary is older than its source as
   `binary-stale` and excludes it from `safe-to-clean`.
5. A repo with a simulated held `target/.cargo-lock` (or an open fd) is
   classified `build-in-flight` and skipped by `apply`.
6. `apply` against a `safe-to-clean` fixture removes the private `target/`,
   reports `bytes_freed` > 0, and appends exactly one ledger line for it.
7. The ledger is append-only: a second `apply` does not rewrite or remove
   existing lines; line count only grows.
8. `apply` without any `safe-to-clean` repo reclaims nothing, writes no ledger
   line, and exits 0 with a clear "nothing to migrate" report.
9. `--ts <rfc3339>` makes ledger lines deterministic for tests (no wall-clock).
