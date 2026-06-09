# PRD: bon-mot-score — the wit-scorer

**Status:** Draft v0.1
**build_target:** rust-cli
build_priority: high
**build_into:** (new repo) `~/wintermute/bon-mot`
**Vision:** visions/bon-mot.md

## TL;DR

When several candidate lines exist, something has to crown one. The Round
Table's whole social mechanic was ranking each other's wit. `bon-mot-score`
rates a candidate line on three measurable axes — surprise, compression,
aptness — into a 0–1 composite, so downstream roundtable crates can rank
candidates deterministically. It is a scorer, not a generator: no `--lavish`
generation, optionally `--lavish` as an LLM-judge second opinion.

## Why this exists

The umbrella `visions/roundtable.md` says the crowning step (`vicious-circle`
selecting the day's `bon mot`, `conning-tower` ranking column lines) needs a
wit-scorer "so vicious-circle-crown and conning-tower can rank." Those crates
are downstream sub-visions; this PRD gives them the shared, testable ranking
function instead of each inventing one. The axes map to real signals already on
this laptop: compression is `core::count::words`/`graphemes` (the same counting
day-haiku does on its 3-line/40-grapheme haiku shape, see its `validate`
module), surprise leans on lexicon rarity, aptness on overlap with the source
observation. The default scorer is pure and deterministic; `--lavish` is an
optional Claude *judge*, not a generator, reusing day-haiku's client pattern.

## What this builds

A CLI `bon-mot-score` (depends on `bon-mot-core`).

- `bon-mot-score --line "<candidate>" [--source "<observation>"]` — default
  tier. Computes:
  - `surprise` — mean lexicon-rarity of content words + a bonus for an
    unexpected pivot (homophone/antithesis detected via `core::lexicon`);
  - `compression` — wit-per-word: brevity vs. information, from
    `core::count::words`/`graphemes` (shorter, denser scores higher, normalized);
  - `aptness` — content-word overlap / semantic relation between the line and
    `--source` (if given; defaults to neutral 0.5 when absent).
  Emits a composite `score = w_s*surprise + w_c*compression + w_a*aptness`
  (weights from `core::` defaults, overridable via `--weights s,c,a`).
- `--rank` — read newline-delimited candidates from stdin, print them sorted by
  score descending with scores.
- `--lavish` — additionally obtain a Claude judge score (0–1) for the line and
  report it alongside; the composite stays deterministic unless `--judge-weight`
  is set.
- `--json` — `{line, surprise, compression, aptness, score, judge?}`.

## Acceptance criteria

1. `bon-mot-score --line "<witty line>" --json` emits all of `surprise,
   compression, aptness, score` each in `[0,1]`, deterministic across runs.
2. A terser, denser line scores strictly higher on `compression` than a verbose
   paraphrase of the same content (fixtured pair).
3. `aptness` is higher when `--source` shares content words with the line than
   when it does not (fixtured pair); absent `--source` it is the neutral
   default.
4. `--weights 1,0,0` makes the composite equal `surprise` (weight plumbing
   works); weights are normalized so the composite stays in `[0,1]`.
5. `--rank` over stdin returns candidates sorted by descending score, stable for
   ties.
6. `--lavish` with no key exits 6; against the loopback mock it returns a judge
   score and the request shows cached system + few-shot blocks.
7. `cargo test` green covering the compression-ordering and aptness-ordering
   fixtures and the weights plumbing.
