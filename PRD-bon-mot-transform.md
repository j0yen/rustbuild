# PRD: bon-mot-transform — cross-form transforms

**Status:** Draft v0.1
**build_target:** rust-cli
**build_into:** (new repo) `~/wintermute/bon-mot`
**Vision:** visions/bon-mot.md

## TL;DR

The Round Table crossed forms fluidly — theater critics wrote verse, journalists
wrote epigrams, a poet wrote a play by Friday. `bon-mot-transform` turns one
form into another: a telemetry fact into a couplet, a haiku into an epigram,
prose into a telegram. It is the composer that orchestrates the other bon-mot
engines and uses `bon-mot-score` to pick the best output, deterministic by
default and Claude-powered on `--lavish`.

## Why this exists

The umbrella `visions/roundtable.md` names cross-form transforms as a bon-mot
component ("turn a journal fact into a couplet, a haiku into an epigram, prose
into a telegram") and grounds it in the Round Table crossing
theater/journalism/verse. The source forms already exist on this laptop:
`~/wintermute/day-haiku` emits haiku (3 lines, the input to haiku→epigram) over
a `summary.json` whose fields are the input to fact→couplet (see day-haiku's
`prompt::render_ephemeral`); `~/wintermute/letters-we-never-sent` and
`conversations-zine` emit prose (the input to prose→telegram). This PRD ships
last because it composes `bon-mot-epigram`, `bon-mot-telegram`, and
`bon-mot-score`; its `--lavish` tier reuses day-haiku's caching client pattern
(`src/api.rs`).

## What this builds

A CLI `bon-mot-transform` (depends on `bon-mot-core`, `bon-mot-score`, and the
relevant engine crates / their library entry points).

- `bon-mot-transform fact-to-couplet --summary summary.json` — pick a salient
  fact, render a rhymed couplet (two lines, near-equal syllable counts via
  `core::count::syllables`, end-rhyme from `core::lexicon`).
- `bon-mot-transform haiku-to-epigram --file haiku.txt` — read a 3-line haiku,
  compress its image into one epigram (delegates to `bon-mot-epigram`'s lib).
- `bon-mot-transform prose-to-telegram --words N --file letter.txt` — delegate
  to `bon-mot-telegram` to compress prose to N words.
- Each transform generates `--candidates K` (default 3) and uses
  `bon-mot-score` to print the highest-scoring output; `--all` prints all
  scored.
- `--lavish` — route the transform through `core::lavish` (Claude) instead of
  the deterministic engines; scoring/selection still applies.
- `--json` — `{transform, source, output, score, tier, candidates?}`.

## Acceptance criteria

1. `fact-to-couplet --summary <fixture>` prints exactly two lines whose
   syllable counts (via `core::count`) differ by ≤2, referencing a real field
   from the fixture (day-haiku field shape).
2. `haiku-to-epigram --file <3-line fixture>` prints one epigram line within the
   epigram word budget and is deterministic per `--seed`.
3. `prose-to-telegram --words 6 --file <fixture>` prints exactly 6 words (via
   `core::count::words`).
4. With `--candidates 3`, the printed output is the one `bon-mot-score` ranks
   highest among the three (verified by re-scoring in the test); `--all` prints
   all three with scores.
5. `--lavish` with no key exits 6; against the loopback mock each transform
   returns the mock output and the request shows cached system + few-shot
   blocks.
6. `--json` emits valid JSON with `transform`, `output`, and `score` for every
   subcommand.
7. `cargo test` green covering one case per transform plus the candidate-ranking
   selection.
