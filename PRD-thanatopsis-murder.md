# PRD: thanatopsis-murder — plant a flaw in an artifact; the table hunts the murderer

**Status:** Draft v0.1
**build_target:** rust-extend
**build_into:** (existing repo) `~/wintermute/thanatopsis`
**Vision:** visions/thanatopsis.md

## TL;DR

"Murder" was the Round Table's favorite parlor game: one player is secretly the
murderer, and the rest deduce who. This PRD turns that into creative QA. It
takes one artifact off the day's table, *plants a deliberate defect* in it (a
broken rhyme, a swapped word, an off-by-one in a count, a snapped line break),
shuffles the tampered artifact back onto the table, and the personas hunt the
"murderer" — the artifact that was tampered with, and where. A round the table
solves means the day's work is robust enough that a planted flaw stands out; a
round it can't solve means the work is loose enough to hide a defect. The game
is a find-the-planted-defect test no straight critique runs.

## Why this exists

The six soloists' outputs land on the day's table — `~/wintermute/day-haiku`
(rhyme/meter), `~/wintermute/conversations-zine` and
`~/wintermute/letters-we-never-sent` (prose), `~/wintermute/self-portrait` (diff
text), `~/wintermute/ambient` / `~/wintermute/wintermute-music` (cue lists). The
umbrella vision `visions/roundtable.md` lists "the game of Murder" among the
table's games and frames the games club as the playful counterpart to critique.
Murder gives the table a QA dimension critique lacks: critique judges what *is*
there; Murder asks whether the table would *notice* if something were wrong —
mutation-testing the day's creative output for free.

This PRD extends the `thanatopsis` crate (scaffolded by `thanatopsis-charades`),
reusing its `Table`/`Artifact` types (from `the-lunch`, slug `the-lunch`,
drafted in parallel) and emitting the shared `GameResult` for
`thanatopsis-parlor-ledger`. The plausibility of a planted swap (choosing a
swap-in word that *looks* like it could belong) reuses `bon-mot` word primitives
(slug `bon-mot`, drafted in parallel) so the defect is a convincing forgery, not
obvious gibberish.

## What this builds

Extends `~/wintermute/thanatopsis` with the `murder` subcommand.

Modules:
- `src/defect.rs` — `Defect` enum (`WordSwap`, `RhymeBreak`, `LineSnap`,
  `CountOffByOne`, `Transpose`) and `plant(artifact, kind, seed) ->
  (Artifact, Wound)` returning the tampered artifact plus a `Wound { defect,
  location, original, planted }` describing exactly what was changed and where.
  Swap-in words come from `words`/`bon-mot` so the forgery is plausible.
- `src/detective.rs` — `investigate(table, persona_seed) -> Accusation
  { artifact_id, location, confidence, reason }`: each persona scans the table
  for anomalies (a non-rhyming line in a form that rhymes, a word that breaks
  local collocation, a count that doesn't match, a transposition) and accuses
  the most anomalous artifact + location. Detection uses the *clean* table as
  the alibi when available, or intra-artifact consistency heuristics otherwise.
- `src/murder.rs` — `play(table, seed) -> GameResult`: plants one defect in one
  chosen artifact, shuffles it back, runs every other persona's investigation,
  and scores the round — `solved` iff a majority correctly accuse the tampered
  artifact, `pinpointed` iff at least one also names the right location; wraps
  into `GameResult` with `game == "murder"` and the `Wound` revealed in
  `detail`.
- `src/main.rs` — add the `murder` subcommand.

CLI subcommands:
- `thanatopsis murder play --table <path> [--target <artifact_id>]
  [--defect <kind>] [--seed <n>] [--json]` — plants a defect, runs the hunt,
  prints each persona's accusation, then the reveal (who the murderer was and
  the wound); `--json` emits the `GameResult`.
- `thanatopsis murder plant --table <path> --target <id> [--defect <kind>]
  [--seed <n>]` — prints just the tampered artifact and the hidden `Wound` (for
  building fixtures / inspecting the forgery).

Deps: reuse the crate's `clap`/`serde`/`serde_json`/`anyhow`/`sigpipe`; optional
`bon-mot` feature for plausible swap-in words.

`plant` MUST change exactly one thing and the returned `Wound` MUST faithfully
locate it, so the game is scorable — the reveal is ground truth.

## Acceptance criteria

1. `cargo build` and `cargo test` succeed; clippy adds no new warnings beyond
   the repo baseline.
2. `thanatopsis murder plant --table tests/fixtures/table.json --target haiku-1
   --defect word-swap --seed 1` returns a tampered artifact differing from the
   original by exactly one change, and a `Wound` whose `location`/`original`/
   `planted` fields exactly describe that change (asserted by a test diffing
   original vs tampered).
3. `thanatopsis murder play --table tests/fixtures/table.json --seed 1 --json`
   emits a valid `GameResult` with `game == "murder"`, a revealed `Wound`, and
   one accusation per investigating persona.
4. On a fixture table of well-formed artifacts with one planted `RhymeBreak`,
   a test asserts the round is `solved` (majority correctly accuse the tampered
   artifact) — proving a planted flaw in robust work is found.
5. On a fixture table of already-loose artifacts, a test shows the round is
   *not* reliably solved (the accusation spreads across artifacts) — proving the
   game distinguishes robust from loose material rather than always solving.
6. Determinism: identical `(table, target, defect, seed)` yields an identical
   `Wound` and identical accusations across two runs.
7. Every `Defect` variant has a `plant` test asserting it produces a locatable
   single-change `Wound`; the crate still builds `--no-default-features` and
   `--features bon-mot`.
