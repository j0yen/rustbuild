# PRD: colophon-stale — state a dead session wrote and nobody refreshed

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/colophon
Vision: visions/colophon.md

## TL;DR

Config and state files accumulate that were written by sessions long gone,
sometimes before the binary that reads them was last rebuilt. This PRD adds
`colophon stale <dir>`: flag files whose provenance shows the writing session
is dead or whose `prov.ts` predates the consuming binary — a provenance-grounded
orphaned-state detector.

## Why this exists

Phase-1 inspection, 2026-06-18:

- Stale state is a standing self-review theme that today is found by ad-hoc
  heuristics, not provenance. Recent journals carry `adopt 26 stale (25
  installed-stale)`, `fleet-binary-staleness` (daemons whose running exe is
  behind HEAD), and `plumb probe memlog-active uncalibrated` — all variants of
  "an artifact outlived the context that produced it." provfs Phase 1 README
  names this exact use case: *"stale-config detection."*
- provfs now records the two facts staleness needs, on every file:
  `user.prov.session` (the writing pid, recoverable from the live comm-chain —
  e.g. `…;pid:2662703;uid:1000`) and `user.prov.ts` (`1781768028` on this pass's
  gossip write). With pid + timestamp stamped, "was this written by a process
  that's still alive?" and "was this written before its consumer was rebuilt?"
  become decidable.
- No tool answers either question from provenance today: the only consumer
  (`provenance-mcp`) returns the raw blob and never extracts pid or ts for a
  liveness/freshness check.
- colophon-parse already yields `pid`, `ts`, and `Provenance`; this PRD is the
  freshness/liveness judgment on top of it.

## What this builds

Extends `~/wintermute/colophon/` (the crate from PRD-colophon-parse). No new
repo. Independent of colophon-attribute (both extend parse; either order).

- **`Staleness` verdict model:** per-file — `path`, `reason` enum
  (`WriterDead` — stamped pid no longer alive; `OlderThanConsumer` — `prov.ts`
  < a reference binary's mtime; `Both`), the writing `Provenance`, and the
  age in days.
- **`fn stale(root, opts) -> Vec<Staleness>`** — walk `root`, parse each file's
  provenance, and judge:
  - `WriterDead`: the stamped `pid` is absent from `/proc` (and the file's
    `prov.ts` is old enough to exclude a freshly-recycled pid — a configurable
    `--min-age` floor, default 1h, guards against pid reuse).
  - `OlderThanConsumer`: `prov.ts` < `mtime` of a `--consumer <binary>` path.
  - Skip-prefixed paths are excluded (same skip set as attribute).
- **`colophon stale <dir>`** subcommand: `--format text|json`,
  `--consumer <path>` (optional; enables the OlderThanConsumer check),
  `--min-age <dur>` (WriterDead floor), `--reason writer-dead|older|any`.
  Strictly read-only — emits a list the user/sweeper acts on; deletes nothing.
- Tests over a tempdir fixture: a file stamped with a definitely-dead pid
  (e.g. an absurd pid + old ts) → `WriterDead`; a file with `prov.ts` before a
  fixture "binary" mtime → `OlderThanConsumer`; a file stamped with the test's
  own live pid and recent ts → not flagged.

Out of scope: deleting/quarantining flagged files (report only); the
self-review digest that consumes this (colophon-digest); attribution
(colophon-attribute).

## Acceptance criteria

1. `cargo build`/`cargo test` green in `~/wintermute/colophon`; `cargo clippy`
   adds no new warnings over the autobuilder baseline; `colophon parse` and
   `colophon attribute` still work (no regression).
2. A fixture file stamped with a non-existent pid and a `prov.ts` older than
   `--min-age` is flagged `WriterDead`; a fixture file stamped with the test
   process's own live pid is NOT flagged.
3. The pid-reuse guard holds: a file stamped with a dead pid but a `prov.ts`
   *newer* than `--min-age` is NOT flagged `WriterDead` (too recent to trust
   the pid is truly gone).
4. With `--consumer <path>`, a file whose `prov.ts` < the consumer's mtime is
   flagged `OlderThanConsumer`; a file newer than the consumer is not.
5. A file matching both conditions is reported once with reason `Both`.
6. `--reason writer-dead` filters output to only WriterDead verdicts;
   `--format json` emits the structured verdict list.
7. `colophon stale` is read-only (fixture tree unchanged after run) and
   SIGPIPE-safe when piped to `head`.
