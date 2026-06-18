# PRD: colophon-digest — a provenance block self-review can read

Status: Draft v0.1
build_target: mixed
build_into: /home/jsy/wintermute/colophon
Vision: visions/colophon.md

## TL;DR

self-review reports cruft and stale state every day but can't say who made it.
This PRD adds `colophon digest`: run attribution over the known cruft dirs and
stale-detection over the config dirs, and emit one markdown digest block that
self-review folds into its journal — so the daily pass attributes leaks instead
of guessing.

## Why this exists

Phase-1 inspection, 2026-06-18:

- self-review already narrates exactly the questions colophon answers, without
  the answers: the 2026-06-17 journal lists `51 build-worktrees consuming 11G —
  /build creates but sometimes doesn't clean them up`, `65 dirty repos`,
  `adopt 26 stale`, `fleet-binary-staleness` — every one a who/when question
  about a file, none grounded in provenance.
- The self-review skill journals to `~/brain/journal/YYYY-MM-DD.md` and is the
  natural home for a provenance digest; it already aggregates ctrace/scribe
  rollups and a docket. A `colophon digest` block slots into that existing
  reporting surface rather than inventing a new one.
- The two analyses this digest needs — attribution and staleness — are built by
  PRD-colophon-attribute and PRD-colophon-stale; this PRD is the convergence
  that runs both over the laptop's real cruft/config dirs and formats the
  result. It must NOT reimplement either; it composes them.
- This is the integration PRD: it depends on attribute AND stale, so it ships
  last in the fleet.

## What this builds

Extends `~/wintermute/colophon/` (CLI half) plus a self-review wiring step
(config half) — hence `build_target: mixed`.

- **`colophon digest`** subcommand: runs `attribute` over a configured set of
  cruft roots (default: `~/wintermute/*/.build-worktrees`, `~/.claude/projects`)
  and `stale` over a configured set of config/state roots (default:
  `~/.claude` excluding `projects`, `~/wintermute/*/state`), and renders ONE
  markdown block: a ranked "who wrote the cruft" table + a short "stale state"
  list, each capped to `--top N`. `--format markdown|json`. Roots overridable
  via `--cruft-root`/`--config-root` (repeatable) for testing and tuning.
- **Degradation:** if provfs is absent (no `user.prov.session` anywhere the walk
  touches), the block renders a single honest line ("provenance unavailable —
  kernel not stamping") and exits 0. The digest must never break a self-review
  run.
- **self-review wiring:** add the `colophon digest` invocation as an optional
  block in the self-review skill's reporting phase (skill-doc edit; the skill is
  prompt-driven, so this is an instruction to run `colophon digest --format
  markdown` and paste the block, guarded "skip if `colophon` is not installed").
  WRAP, don't replace, any existing reporting — the digest is additive.
- Tests: over a fixture cruft tree + fixture config tree, assert the rendered
  markdown contains the expected actor table and stale entries; assert the
  no-provfs path renders the honest one-liner and exits 0.

Out of scope: acting on the digest (no deletion/quarantine — it informs the
human-run sweep); changing what ballast/careen/drydock reap; re-pointing
provenance-mcp.

## Acceptance criteria

1. `cargo build`/`cargo test` green in `~/wintermute/colophon`; `cargo clippy`
   adds no new warnings over baseline; `parse`/`attribute`/`stale` subcommands
   still work (no regression).
2. `colophon digest --cruft-root <fixtureA> --config-root <fixtureB> --format
   markdown` emits a single markdown block containing a ranked actor table (from
   attribute) and a stale-state list (from stale), each honoring `--top`.
3. The digest composes the existing layers — verified by a test asserting the
   actor totals in the digest equal what `colophon attribute` reports for the
   same root (no divergent reimplementation).
4. With cruft/config roots that contain no provfs-stamped files, `colophon
   digest` renders the honest "provenance unavailable" line and exits 0 (never
   non-zero, never a panic).
5. `colophon digest --format json` emits the same data structured, for machine
   consumers.
6. The self-review skill doc is updated to invoke `colophon digest --format
   markdown` as an additive, guarded block ("skip if colophon not installed"),
   wrapping not replacing existing reporting; the edit is idempotent and names
   no other behavior change.
7. Run live on this machine, `colophon digest --format markdown` produces a
   non-empty block attributing real `.build-worktrees` content to the skill(s)
   that wrote it (integration test, skipped with a logged note if provfs is
   absent).
