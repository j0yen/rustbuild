# PRD: careen-sweep — lock-aware intra-target reclaimer

Status: Draft v0.1
build_target: rust-cli
Vision: visions/careen.md
Repo: j0yen/careen-sweep (PUBLIC)
Depends on: PRD-careen-survey.md (consumes its classification)

## TL;DR

careen-survey sizes the reclaimable cruft inside a living target dir;
`careen-sweep` is the tool that actually scrapes it off — safely. It
prunes stale incremental, orphaned deps rlibs, wrong-toolchain
artifacts, and dead build outputs from a single repo's target dir,
dry-run by default, never touching an in-flight build, and only ever
removing artifacts a clean `cargo build` regenerates.

## Why this exists

From Vision §Why and careen-survey's evidence: `recall/target` is 13G
(9.2G of it orphaned deps rlibs), `wintermute-brain/target` is 13G, both
LIVE and therefore untouchable by `ballast`. 2550 stale incremental dirs
sit fleet-wide. The survey proves the prize; sweep is the only thing
that collects it. The danger careen-sweep must defeat: cargo holds an
advisory flock on a target dir during a build (heavy /build days run
many concurrent cargo invocations — self-review logged 6 sessions /
224k events on 2026-06-16). Deleting deps out from under a live build
corrupts it. So lock-awareness is the load-bearing safety property, not
a nicety.

## What this builds

A Rust CLI `careen-sweep` (clap) operating on **one** target dir.

**Safety model (all must hold before any unlink):**

1. **Lock check** — acquire/inspect cargo's target-dir advisory lock
   (`target/debug/.cargo-lock` flock). If a build holds it, refuse and
   exit non-zero ("target busy"). Never block-wait by default
   (`--wait` opt-in).
2. **Regenerable-only** — only the four careen-survey classes are
   eligible. The current binary, `Cargo.lock`, current-toolchain
   fingerprints, and anything outside the classified set are never
   touched.
3. **Dry-run default** — prints exactly what would be removed and the
   bytes; `--apply` required to mutate. `--apply` prints a per-class
   reclaimed total.
4. **Conservative default** — uses careen-survey's conservative
   orphaned-deps estimate unless `--aggressive` is passed.

**Flow:** invoke careen-survey's classification (as a library — survey
exposes a `lib.rs` classify fn — or shell out to its `--format json`),
filter to eligible classes, lock-check, then (under `--apply`) unlink
and report. Emit a JSON summary: `{repo, classes:{...bytes}, removed_bytes,
applied:bool, skipped_reason?}`.

**Post-sweep invariant check (test, not runtime):** after a sweep with
`--apply`, a clean `cargo build` in the fixture repo still succeeds and
produces the same binary hash as before the sweep — proving only
regenerable artifacts were removed.

**Crates:** `clap`, `serde_json`, `fs2` (or `rustix`) for advisory
flock, `anyhow`. Reuse `careen-survey` as a path/lib dependency for
classification (do not re-implement it).

**NOT in scope:** SLO/watermark triggering (careen-guard); rebuild-cost
accounting (careen-ledger); multi-repo fan-out (guard handles fleet).

## Acceptance criteria

1. Default invocation (no `--apply`) performs zero mutations and prints
   the would-remove plan with per-class byte totals — verified by an
   inode/mtime snapshot test.
2. With `--apply` on a fixture target dir, the four eligible classes are
   removed and the JSON summary reports `applied:true` with a
   `removed_bytes` equal to the survey's conservative estimate (±block
   rounding).
3. **Lock safety:** when the fixture target's `.cargo-lock` is held by a
   simulated builder, careen-sweep refuses, exits non-zero with a
   "target busy" message, and makes zero mutations.
4. **Regenerable invariant:** after `--apply` on a buildable fixture
   crate, `cargo build` succeeds and the resulting binary is byte- or
   hash-identical to a pre-sweep build of the same source — proving no
   live artifact was removed.
5. The current installed binary, `Cargo.lock`, and current-toolchain
   `.fingerprint` entries are never in the removal set — asserted by a
   test that seeds those paths and confirms they survive `--apply`.
6. `--aggressive` removes the larger (fingerprint-unreferenced) orphaned
   set; conservative (default) removes only the Cargo.lock-unmatched
   set; a fixture proves the two differ and both keep the build green.
7. Running against a busy or nonexistent target never panics; all exits
   are clean with a non-zero code and a message.
8. `--help` states the safety model (lock check, regenerable-only,
   dry-run default) explicitly.
