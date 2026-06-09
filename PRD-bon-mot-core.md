# PRD: bon-mot-core — the shared wit core (lexicon, grammar runtime, counters, optional lavish client)

**Status:** Draft v0.1
**build_target:** rust-lib
**build_into:** (new repo) `~/wintermute/bon-mot`
**Vision:** visions/bon-mot.md

## TL;DR

The six bon-mot CLIs all need the same primitives — a homophone/pun/synonym
lexicon, a grammar-template runtime, syllable + grapheme counters, a seedable
RNG, and (only on `--lavish`) an Anthropic client. Without a shared core each
CLI would re-author prompts and re-implement counting, and the
`--lavish`/default tiering would drift between tools. This crate is the one wit
core every other bon-mot PRD and every downstream roundtable crate depends on.

## Why this exists

wintermute's creative wing is a set of soloists that each hand-roll their own
plumbing. `~/wintermute/day-haiku/src/api.rs` already proves the
Anthropic-from-Rust pattern works on this laptop: a sync `ureq` client reading
`$ANTHROPIC_API_KEY`, `anthropic-version: 2023-06-01`, and—critically—
`cache_control: {"type":"ephemeral"}` on the stable system + final few-shot
blocks (`build_body`) so the cache hits every call after the first and only the
novel turn is billed. The umbrella `visions/roundtable.md` calls out that its
solo outputs (`day-haiku`, `conversations-zine`, `letters-we-never-sent`,
`ambient`, `wintermute-music`) "need a shared wit layer." bon-mot-core *is* that
layer; it lifts day-haiku's caching client pattern into a reusable home behind a
`lavish` feature so the default tier never needs a network or a key.

## What this builds

A library crate `bon_mot_core` (no binary). Modules:

- `lexicon` — loads a TOML lexicon (`homophones`, `puns`, `synonyms` tables) from
  an embedded default (`include_str!`) overridable by `$BON_MOT_LEXICON` path.
  Exposes lookups: `homophones_of(word)`, `puns_for(word)`, `synonyms_of(word)`.
- `grammar` — a tiny template runtime: a template is a string with `{slot}`
  placeholders filled from a `Bindings` map; supports an alternation list
  `{a|b|c}` chosen via the seedable RNG. Pure, no I/O.
- `count` — `syllables(word)` (heuristic vowel-group counter), `graphemes(s)`
  and `words(s)` using `unicode-segmentation` (same dep day-haiku uses).
- `rng` — `seeded(u64)` wrapper so every engine is reproducible under a fixed
  `--seed`; default seed derived from the input hash (deterministic per input).
- `lavish` (feature-gated) — the Anthropic client ported from day-haiku's
  `api.rs`: `LavishError` with the same exit-code mapping (missing key=6,
  4xx=4, 5xx/network=5, bad response=3), `build_body` applying `cache_control`
  ephemeral on system + final few-shot, `$ANTHROPIC_BASE_URL` override for the
  test loopback, `call(req)`.

Deps: `serde`, `serde_json`, `toml`, `unicode-segmentation`, `rand` (or a small
seeded PRNG), `anyhow`; `ureq` only under the `lavish` feature.

## Acceptance criteria

1. `cargo build` (default features) succeeds with **no** `ureq`/network dep
   compiled in; `cargo build --features lavish` adds the client.
2. `lexicon::homophones_of("horticulture")` returns a non-empty set including
   the seed entry that makes the Parker gag possible (e.g. relates to "whore");
   the embedded default lexicon parses and has ≥100 total entries.
3. `$BON_MOT_LEXICON=/path/to/file.toml` overrides the embedded lexicon; a
   malformed file returns an `Err` (not a panic).
4. `grammar::render("{a|b}", seed)` is deterministic for a fixed seed and selects
   from the alternation; `{slot}` is filled from bindings; unknown slot is an
   `Err`.
5. `count::syllables`, `graphemes`, `words` return correct values on a fixtured
   table (≥8 cases incl. multibyte/emoji for graphemes).
6. With the `lavish` feature, a loopback mock at `$ANTHROPIC_BASE_URL` shows the
   request body carries `cache_control:{"type":"ephemeral"}` on the system block
   and the final few-shot block but **not** the trailing turn (mirrors
   day-haiku's `last_fewshot_cached_today_uncached` test); missing key yields
   `exit_code()==6` with no network call.
7. `cargo test` is green (`--features lavish` included in the test matrix).
