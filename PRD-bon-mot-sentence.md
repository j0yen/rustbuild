# PRD: bon-mot-sentence — the "I can give you a sentence" game

**Status:** Draft v0.1
**build_target:** rust-cli
build_priority: high
**build_into:** (new repo) `~/wintermute/bon-mot`
**Vision:** visions/bon-mot.md

## TL;DR

Dorothy Parker's "You can lead a horticulture but you can't make her think" was
a move in the Round Table game "I can give you a sentence": someone names a
word, you produce a witty sentence that turns on it — usually via a homophone or
pun. There is no tool on this laptop that plays this game. `bon-mot-sentence`
takes a word and emits such a sentence, deterministically from grammar+lexicon
by default and from Claude on `--lavish`.

## Why this exists

The umbrella `visions/roundtable.md` names this exact game and this exact Parker
line as the seed of the whole `bon-mot` sub-vision ("Dorothy Parker's '...' came
from the game 'I can give you a sentence'"). roundtable's solo creative repos —
`~/wintermute/day-haiku`, `conversations-zine`, `letters-we-never-sent`,
`ambient` — each generate text alone and "need a shared wit layer"; the sentence
game is the most literal instance of that layer. `day-haiku/src/api.rs` already
proves the Anthropic-from-Rust + prompt-caching pattern works here, so the
`--lavish` tier is a known-good path rather than new risk; the default tier owes
nothing to the network.

## What this builds

A CLI `bon-mot-sentence` (depends on `bon-mot-core`).

- `bon-mot-sentence <word>` — default tier. Looks up `homophones_of`/`puns_for`
  the word in the core lexicon, picks a sentence template from `core::grammar`
  whose structure sets up the pun (the "lead a ___ / make her ___" frame is one
  template), fills slots, and prints one sentence. Seedable via `--seed` for
  reproducibility; `-n K` prints K distinct candidates.
- `--lavish` — instead calls `core::lavish` (Claude) with a system block
  describing the game + a few cached few-shot examples (the horticulture gag as
  the canonical shot) and the target word as the trailing uncached turn.
- `--json` — emit `{word, sentence, pivot, tier, seed}` where `pivot` is the
  homophone/pun the line turned on.
- If a word has no lexicon pivot in the default tier: exit non-zero with a clear
  message suggesting `--lavish` or a different word (no fabricated pun).

UX: prints exactly one sentence per candidate to stdout; diagnostics to stderr.

## Acceptance criteria

1. `bon-mot-sentence horticulture --seed 1` prints a single sentence that
   contains the pivot word and is deterministic across runs for that seed.
2. `bon-mot-sentence horticulture -n 3 --seed 1` prints 3 distinct sentences.
3. `--json` emits valid JSON with all of `word, sentence, pivot, tier:"default",
   seed`; `pivot` is a real lexicon homophone/pun of the input.
4. A word with no lexicon pivot exits non-zero with a message naming `--lavish`
   as the alternative; it does **not** invent a homophone.
5. `--lavish` with no `$ANTHROPIC_API_KEY` exits code 6 (per core) and the
   message points back to the free default tier; `--lavish` against a loopback
   mock (`$ANTHROPIC_BASE_URL`) returns the mock sentence and shows the request
   used cached system + few-shot blocks.
6. `cargo test` green: at least one end-to-end test of the default tier and one
   of the `--lavish` path against the loopback mock.
