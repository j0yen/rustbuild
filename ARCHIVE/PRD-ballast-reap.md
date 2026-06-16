# PRD: ballast-reap — gated, logged reclamation of fossil build weight

**Status:** Draft v0.1
**build_target:** rust-cli
**Vision:** visions/ballast.md
**Repo:** j0yen/ballast-reap (NEVER AtScaleInc)

## TL;DR

Survey says *what* is reclaimable and *how safe* each candidate is. ballast-reap
is the keystone that actually frees the bytes — but never blindly. It consumes
`ballast-survey --json`, selects candidates by `reap_safety` and age thresholds,
shows exactly what it would delete and how much it would reclaim, and **does
nothing without `--apply`**. Every applied deletion is written to an append-only
ledger (reclaimed bytes, the classification that justified it, timestamp). It
refuses to reap a `target/` while a build holds it. ballast-reap turns 205G of
fossil into free space the way a careful operator would — auditable, reversible
in spirit, never surprising.

## Why this exists

Verified 2026-06-16: 205G of `target/` across 174 dirs on a 94%-full disk, the
biggest (`wintermute-brain/target`, 13G) provably fossil (installed `wmd` is 17
days newer — see PRD-ballast-cloudaware §Why). The reclamation is sitting right
there, but the only mechanism today is a human running `rm -rf` after a manual
`du` — exactly the kind of irreversible, unlogged operation that the box's own
boundaries say not to do casually (memory: "I do not act on irreversible
operations without explicit confirmation"). ballast-reap encodes that caution in
software: dry-run default, explicit `--apply` gate, a ledger so a reclamation is
never silent, and a hard refusal to touch in-flight builds.

This mirrors the gating discipline already proven in the fleet — mqo-goldgrow's
human-gated acceptance, mqo-drift-watch's `--accept` baseline gate — applied to
deletion instead of data growth.

## What this builds

A Rust CLI `ballast-reap` (clap).

**Pipeline**
- `intake` — read `ballast-survey --json` from stdin or `--survey <path>`;
  optionally invoke `ballast-survey` directly if `--scan` is passed.
- `select` — filter candidates by `--safety <fossil|stale-installed|...>` (a
  floor — only reap entries at or safer than the named rank; default `fossil`)
  and `--min-age <days>` and `--min-size`. Print the selection as a plan.
- `guard` — before deleting any `target/`, verify no live build holds it:
  check for a `target/.cargo-lock`/`target/debug/.cargo-lock`, and cross-check
  running processes (reuse `pevent`/`procstat` if available, else a `pgrep
  cargo` whose cwd is the crate). A held target is skipped with a logged reason,
  never force-deleted.
- `apply` — only when `--apply` is present: delete each selected path, summing
  reclaimed bytes from the survey entry. Without `--apply`, print the plan and
  the would-reclaim total, exit 0, delete nothing.
- `ledger` — append one JSONL record per applied deletion to
  `~/.local/state/ballast/reap-ledger.jsonl` (path, kind, reap_safety,
  bytes_reclaimed, crate, ts). The ledger is append-only; never rewritten.

**Safety invariants (testable)**
- Default safety floor is `fossil` — the safest class only — unless the caller
  widens it explicitly.
- `--apply` is the *only* path that deletes. Absence ⇒ zero filesystem writes
  except the (optional) ledger is also untouched on a dry run.
- A path outside the configured roots is rejected (no `--apply` can delete
  `/` or `$HOME` directly; refuse any selection whose path doesn't sit under a
  survey root).
- Never reap an entry whose `reap_safety` is `stale-uninstalled` or `recent`
  unless `--force-unsafe` is *both* passed and the entry is explicitly named.

**Deps:** `clap`, `serde`/`serde_json`, `anyhow`. Optional shell-out to
`pevent`/`procstat`/`ballast-survey` (graceful absence).

## Acceptance criteria

1. With no `--apply`, ballast-reap prints a plan + would-reclaim total and the
   target fixture tree is **unchanged** (zero deletions, no ledger write).
2. With `--apply`, selected `fossil` entries are deleted and each produces one
   append-only JSONL ledger record with `bytes_reclaimed` matching the survey
   entry.
3. Default safety floor is `fossil`; a `stale-uninstalled` entry is *not*
   selected unless `--safety stale-uninstalled` is passed.
4. A `target/` with a simulated build lock / matching live `cargo` process is
   skipped with a logged `reason: "build-in-flight"`, even under `--apply`.
5. A selection whose path resolves outside every survey root is refused with a
   non-zero exit and a clear stderr message; no deletion occurs.
6. The ledger is append-only: a second `--apply` run appends, never truncates;
   prior records survive.
7. `--force-unsafe` requires an explicitly named path and refuses to act on a
   broad glob (fixture: bare `--force-unsafe` with a directory of unsafe entries
   deletes nothing).
8. `cargo test` green; `cargo clippy` clean.
