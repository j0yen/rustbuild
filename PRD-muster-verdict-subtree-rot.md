# PRD: muster-verdict-subtree-rot

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/muster
Vision: visions/muster.md

## TL;DR

A session can be perfectly `live` and still be dragging a rotten bus subtree —
deleted-exe subscribers and a pile of leaked workers. The current verdict
taxonomy (live | duplicate | orphan | stale) has no bucket for "healthy session,
sick subtree." This PRD adds a `subtree-rot` annotation orthogonal to the main
verdict, so muster surfaces the exact condition self-review keeps re-discovering
by hand.

## Why this exists

Verified live (2026-06-16): pid 1054 is classed `live`/`duplicate` by
`muster verdict`, yet it owns 21 leaked workers and 3 deleted-exe subscribers
(pids 1906/2201/579203). The deleted-exe subscribers are exactly the
"fleet-binary-staleness" finding self-review has carried **open for 3+
consecutive runs** (journals 2026-06-14/15/16). muster owns the
session→subtree map (once `PRD-muster-subtree-census` populates it) and is the
natural adjudicator of subtree rot — but its verdict is silent on it because rot
is not a session-liveness state, it's a property of the subtree.

## What this builds

Extends `~/wintermute/muster/src/verdict.rs`:

1. **`subtree-rot` annotation**, orthogonal to `VerdictKind`. A roster entry
   keeps its primary verdict (`live`/`duplicate`/`orphan`/`stale`) AND gains an
   optional `subtree_rot` field with: `deleted_exe` count, `worker_generations`
   count, and a reason string naming the offending counts.
2. **Rot predicate** (consumes the `subtree` block from subtree-census):
   - any `deleted_exe >= 1` → rot (stale binaries are always rot, regardless of
     count);
   - `worker_generations > threshold` → rot (default threshold 3, override via
     `--max-worker-generations <N>`; see vision open question on tuning).
3. **Output:** text gains a `ROT` marker + reason on flagged rows; JSON gains
   the `subtree_rot` object (absent/null when clean).
4. Rot is reported for `live` sessions too — the whole point is "this live
   session is healthy but its children are not."

Deps: none new; consumes the census `subtree` block.

## Acceptance criteria

1. A session whose `subtree.deleted_exe >= 1` is annotated `subtree_rot` with a
   reason naming the deleted-exe count, regardless of its primary verdict
   (unit test over a `live` session with 3 deleted-exe children → rot present,
   primary verdict unchanged).
2. A session with `worker_generations` above the threshold is annotated
   `subtree_rot`; one at or below is not (boundary test at the default
   threshold of 3 and via `--max-worker-generations`).
3. A clean session (`deleted_exe == 0`, generations ≤ threshold) has
   `subtree_rot` absent/null in JSON and no `ROT` marker in text.
4. The primary `VerdictKind` for every entry is unchanged from pre-PRD behavior
   on the same input (additive annotation, proven by the existing verdict tests
   still passing).
5. `muster verdict --format json` round-trips the `subtree_rot` object;
   `--format selfreview` (if present) includes a one-line rot count.
6. `cargo test` green; `cargo build` clean.
