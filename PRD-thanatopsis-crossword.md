# PRD: thanatopsis-crossword — build a small cross-word from the day's vocabulary

**Status:** Draft v0.1
**build_target:** rust-extend
**build_into:** (existing repo) `~/wintermute/thanatopsis`
**Vision:** visions/thanatopsis.md

## TL;DR

The 1920s cross-word craze swept the Round Table; the Thanatopsis club solved
them between hands. The day's artifacts contain a *vocabulary* — the actual
salient words that appeared across the haiku, the zine, the letter, the
self-portrait diff — and nothing on this laptop ever reflects that vocabulary
back. This PRD harvests the day's words and lays a small interlocking cross-word
grid with clues, reusing `bon-mot`'s word primitives. The puzzle is a playful
mirror of the day's language: the words the table actually used, woven back into
a grid the table (or you) can solve.

## Why this exists

The day's table aggregates the six soloists — `~/wintermute/day-haiku`,
`~/wintermute/conversations-zine`, `~/wintermute/letters-we-never-sent`,
`~/wintermute/self-portrait`, `~/wintermute/ambient`,
`~/wintermute/wintermute-music`. The umbrella vision `visions/roundtable.md`
names the cross-word as one of the Round Table's standing games and frames the
games club as the generative counterpart to critique. A cross-word built from
the day's own vocabulary is generative in exactly that spirit — it produces a
new artifact (the puzzle) *out of* the day's artifacts.

This PRD extends the `thanatopsis` crate (scaffolded by `thanatopsis-charades`),
consuming its `Table`/`Artifact` types (from `the-lunch`, slug `the-lunch`,
drafted in parallel) and emitting the shared `GameResult`. Word extraction,
length filtering, and clue/synonym generation lean directly on `bon-mot`'s word
primitives (slug `bon-mot`, drafted in parallel) via the crate's existing
`words` façade and its built-in fallback.

## What this builds

Extends `~/wintermute/thanatopsis` with the `crossword` subcommand.

Modules:
- `src/vocab.rs` — `harvest(table, opts) -> Vocab`: collect candidate fill words
  from every artifact's text via `words` (salient-word extraction, stopword
  strip, length 3..=12, alphabetic only), dedupe, and rank by cross-artifact
  frequency so words used by *several* soloists rank highest; tag each word with
  the artifacts it came from.
- `src/grid.rs` — `Grid` (a sparse char grid) and a deterministic placement
  packer: `build(vocab, size, seed) -> Crossword` seeds the longest word
  across, then greedily places remaining words that interlock on a shared
  letter (across/down alternating), rejecting placements that collide or create
  invalid adjacencies; returns the grid plus the placed `Entry { word, row, col,
  dir, number }` list and the unplaced remainder.
- `src/clue.rs` — `clue_for(word, source_artifacts) -> String`: a clue built
  from the word's source artifact `kind` and a `bon-mot` synonym/definition hint
  (fallback: "appears in the {kind}, {len} letters"); never contains the answer
  word itself.
- `src/render.rs` — `render_grid(crossword) -> String` (monospace ASCII grid
  with numbered cells; a blank/solution variant) and a clue list split
  Across/Down.
- `src/main.rs` — add the `crossword` subcommand.

CLI subcommands:
- `thanatopsis crossword build --table <path> [--size <n>] [--seed <n>]
  [--solution] [--json]` — harvests vocabulary, builds the grid, prints the
  numbered blank grid and the Across/Down clues; `--solution` prints the filled
  grid; `--json` emits the `GameResult` (with the puzzle embedded in `detail`).
- `thanatopsis crossword vocab --table <path>` — prints the ranked harvested
  vocabulary with each word's source artifacts (the raw material).

Deps: reuse the crate's `clap`/`serde`/`serde_json`/`anyhow`/`sigpipe`; optional
`bon-mot` feature for richer clues.

A clue MUST NOT contain its own answer word, and every placed entry MUST
interlock with at least one other entry on a shared, consistent letter (no
floating words) — these are the puzzle-validity invariants.

## Acceptance criteria

1. `cargo build` and `cargo test` succeed; clippy adds no new warnings beyond
   the repo baseline.
2. `thanatopsis crossword vocab --table tests/fixtures/table.json` prints a
   non-empty ranked vocabulary; a test asserts a word appearing in two fixture
   artifacts ranks above a word appearing in one.
3. `thanatopsis crossword build --table tests/fixtures/table.json --seed 1
   --json` emits a valid `GameResult` with `game == "crossword"` and a `detail`
   carrying the grid dimensions and the placed entries.
4. Interlock invariant: a test asserts every placed `Entry` (after the seed
   word) shares at least one cell with another entry and that all shared cells
   agree on their letter (no contradictions).
5. Clue invariant: a test asserts no generated clue string contains its own
   answer word (case-insensitive).
6. Determinism: a test asserts `build` with identical `(table, size, seed)`
   produces an identical grid and clue list across two runs.
7. `--solution` renders a grid whose filled cells spell each placed entry's word
   along its row/col and direction; the crate still builds
   `--no-default-features` and `--features bon-mot`.
