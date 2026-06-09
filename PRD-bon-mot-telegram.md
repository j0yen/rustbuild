# PRD: bon-mot-telegram — the telegram game (say it in N words)

**Status:** Draft v0.1
**build_target:** rust-cli
build_priority: high
**build_into:** (new repo) `~/wintermute/bon-mot`
**Vision:** visions/bon-mot.md

## TL;DR

In the telegram game the constraint *is* the wit: say it in exactly N words and
the compression forces the joke. (The genre's famous instance — a magazine
wiring an author "HOW OLD CARY GRANT?" and getting back "OLD CARY GRANT FINE,
HOW YOU?" — lives or dies on the word budget.) `bon-mot-telegram` rewrites an
input to an exact or at-most word count, deterministically by default and via
Claude on `--lavish`.

## Why this exists

The umbrella `visions/roundtable.md` names the telegram game as a bon-mot
component and frames it as constrained writing where "the constraint IS the
wit." It is the natural compressor for roundtable's wordy solo outputs —
`~/wintermute/letters-we-never-sent` drafts whole letters and
`~/wintermute/conversations-zine` extracts moments; squeezing one of those to a
12-word telegram is exactly the move the Round Table made. The default tier is
pure deterministic compression (no key); `--lavish` reuses day-haiku's caching
client (`~/wintermute/day-haiku/src/api.rs`) for genuinely clever cuts.

## What this builds

A CLI `bon-mot-telegram` (depends on `bon-mot-core`).

- `bon-mot-telegram --words N --text "<input>"` — default tier. Deterministic
  compression: tokenize, score words by a kept-word heuristic (drop stopwords
  from `core::lexicon`, keep nouns/verbs/the salient tail), and emit exactly N
  (`--exact`, default) or at most N (`--at-most`) words, uppercased in telegram
  style with `STOP` separators optional via `--stops`.
- `--file <path>` — read the input from a file (e.g. an unsent letter).
- `--lavish` — call `core::lavish` to rewrite to N words with wit rather than
  mechanical truncation; system block states the budget + telegram register,
  few-shot cached, input as trailing turn.
- `--json` — `{input_words, target, output, output_words, tier}`.

UX: one telegram line to stdout; exit non-zero if `--exact` cannot be met by the
default tier without dropping below sense (message suggests `--at-most` or
`--lavish`).

## Acceptance criteria

1. `bon-mot-telegram --words 5 --exact --text "<a 20-word sentence>"` outputs
   exactly 5 words (asserted via `core::count::words`), deterministic per
   `--seed`.
2. `--at-most 5` never exceeds 5 words and may emit fewer.
3. `--stops` inserts `STOP` between clauses without counting `STOP` toward the
   word budget; default omits them.
4. `--file <path>` reads input from disk and compresses it; a missing file is a
   clean non-zero error, not a panic.
5. `--lavish` with no key exits 6; against the loopback mock it returns the
   mock telegram and the request shows cached system + few-shot blocks.
6. `--json` emits valid JSON with `output_words == target` under `--exact`.
7. `cargo test` green: an `--exact` count test, an `--at-most` bound test, and a
   `--lavish` loopback test.
