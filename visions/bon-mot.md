# Vision: bon-mot — the wit engine

> Sub-vision of [roundtable](roundtable.md) — the Algonquin Round Table brought
> to wintermute's creative tooling. Where `roundtable` is the lunch table,
> `bon-mot` is what the table *did*: play language games and forge epigrams
> under constraint.

## TL;DR

The Round Table was famous less for what its members wrote alone than for what
they did *at the table*: word games. Dorothy Parker's "You can lead a
horticulture but you can't make her think" was not an essay — it was a move in
the parlor game "I can give you a sentence." The wit lived in the **constraint**:
a given word, a homophone, a fixed length, an anagram. `bon-mot` builds those
constraints as composable Rust primitives — a sentence-game engine, an epigram
forge, an anagram engine, a telegram (say-it-in-N-words) game, a wit-scorer,
and cross-form transforms — so the rest of `roundtable` (the personas of
`vicious-circle`, the column of `conning-tower`, the games of `thanatopsis`) has
a shared, testable wit layer to compose instead of each re-inventing prompts.

## The tiering decision (answers roundtable's open question)

roundtable asks: should the wit be Claude-API-generated (real wit, cost per
lunch) or template/grammar-driven (deterministic, free)? **bon-mot decides:
every engine is tiered, cheap-deterministic-first.**

- **Default tier — grammar/lexicon, deterministic, free, offline.** Every
  subcommand produces a real answer with zero network and a seedable RNG, so it
  is fast, reproducible, CI-testable, and never blocks on a missing key. The
  constraint engines (anagram, telegram length, syllable counts) are exact
  arithmetic; the generative ones (sentence, epigram) use hand-authored grammar
  templates + a homophone/pun lexicon shipped in-repo.
- **`--lavish` tier — Claude API.** Only when the user opts in does an engine
  call the Anthropic Messages API for genuinely novel wit. This reuses the
  **proven day-haiku pattern**: a sync `ureq` client, `$ANTHROPIC_API_KEY`,
  `anthropic-version: 2023-06-01`, and `cache_control: {"type":"ephemeral"}` on
  the stable system + few-shot blocks so the per-call cost collapses to the
  novel turn after the first call (see `~/wintermute/day-haiku/src/api.rs`,
  `build_body`). Missing key is exit code 6, not a crash — the default tier is
  always reachable.

A shared `bon-mot-core` lib crate holds the lexicon loader, the grammar/template
runtime, the syllable/grapheme counters, and the (optional) Anthropic client, so
the six CLIs and all downstream roundtable crates depend on one wit core.

## End-state

A `bon-mot` repo at `~/wintermute/bon-mot` exposing a `bon-mot-core` lib and a
family of small CLIs. Given the day's telemetry (the same `summary.json`
day-haiku already consumes) or any input word/line, the engines can: turn a word
into a Parker-style pun sentence, compress a fact into an aphorism, anagram a
vocabulary, squeeze prose into an N-word telegram, score any candidate line for
wit, and transform a fact between forms (fact→couplet, haiku→epigram,
prose→telegram). Each runs free and deterministic by default and brilliant on
`--lavish`. `vicious-circle` personas call the scorer to rank verdicts;
`conning-tower` calls the transforms to set a crowned line in column form.

## Components (one bullet per PRD)

- **bon-mot-core** — the shared lib: lexicon loader (homophones/puns/synonyms,
  TOML), grammar-template runtime, syllable + grapheme counters, seedable RNG,
  and the optional day-haiku-pattern Anthropic client behind a `lavish` feature.
- **bon-mot-sentence** — the "I can give you a sentence" game: a word in, a
  witty sentence out that turns on a homophone/pun (the horticulture gag).
  Grammar+lexicon by default, Claude on `--lavish`.
- **bon-mot-epigram** — the epigram forge: compress an observation or a
  `summary.json` telemetry fact into one aphoristic line under a length budget.
- **bon-mot-anagram** — anagram + wordplay engine over a supplied vocabulary:
  exact anagrams, subanagrams, and "near-grams" the Round Table's games used.
- **bon-mot-telegram** — the telegram game: rewrite an input to exactly/at-most
  N words, where the constraint itself is the wit; deterministic compression by
  default, Claude on `--lavish`.
- **bon-mot-score** — the wit-scorer: rate a candidate line on surprise,
  compression, and aptness into a 0–1 composite so downstream crates can rank.
- **bon-mot-transform** — cross-form transforms (fact→couplet, haiku→epigram,
  prose→telegram), composing the other engines + scorer to pick the best output.

## Order

```
bon-mot-core ──┬──> bon-mot-sentence
               ├──> bon-mot-epigram
               ├──> bon-mot-anagram
               ├──> bon-mot-telegram
               ├──> bon-mot-score
               └──> bon-mot-transform  (also depends on score + the engines)
```

- `bon-mot-core` ships first; everything depends on it.
- `sentence`, `epigram`, `anagram`, `telegram`, `score` are independent siblings
  and can build in any order / in parallel once core lands.
- `bon-mot-transform` ships last: it orchestrates the engines and uses `score`
  to choose the best transform.

## Open questions

- **Lexicon provenance.** The homophone/pun/synonym lexicon ships in-repo as
  curated TOML (a few hundred entries, enough for the Round Table's registers).
  Should it later draw from a system wordlist (`/usr/share/dict`) or WordNet?
  Default: ship a curated TOML; treat external wordlists as a later enrichment.
- **Scorer calibration.** `bon-mot-score`'s surprise/compression/aptness weights
  are hand-set v0.1. Calibrating them against `vicious-circle`'s crowned vs.
  composted lines is deferred to that sub-vision (the held-out set must be
  independent of bon-mot's own fixtures — see the tautology note in memory).
- **Shared lavish client home.** The Anthropic client is duplicated from
  day-haiku into `bon-mot-core` for now. Whether roundtable later extracts one
  `wintermute-anthropic` crate is deferred to the umbrella; bon-mot stays
  self-contained.
