# PRD: bon-mot-anagram — anagram & wordplay engine over a vocabulary

**Status:** Draft v0.1
**build_target:** rust-cli
build_priority: high
**build_into:** (new repo) `~/wintermute/bon-mot`
**Vision:** visions/bon-mot.md

## TL;DR

The Round Table loved word games, and the anagram is the purest: rearrange the
letters of a word or phrase into another that comments on it. This is exact
combinatorial work — no LLM needed — so it is the most deterministic engine in
bon-mot. `bon-mot-anagram` finds anagrams, subanagrams, and "near-grams" of an
input over a supplied vocabulary.

## Why this exists

The umbrella `visions/roundtable.md` lists anagrams among the Round Table games
bon-mot must supply ("the epigram forge, anagrams, the telegram game") and notes
`thanatopsis` (the games club, with a collaborative cross-word "from the day's
vocabulary") will consume them. The day's vocabulary is real and already on this
laptop: the words in `~/wintermute/day-haiku`'s `summary.json` (repo names,
journal headings) and the lexicon shipped in `bon-mot-core`. This engine is
deterministic and free by design — it has **no** `--lavish` tier, demonstrating
that bon-mot's default tier is a first-class citizen, not a fallback.

## What this builds

A CLI `bon-mot-anagram` (depends on `bon-mot-core` for the lexicon + counters).

- `bon-mot-anagram <input>` — print exact anagrams of `<input>` found in the
  active vocabulary (multiset-of-letters equality, case/space/diacritic
  normalized via `core::count`/unicode handling).
- `--sub` — also print subanagrams (vocabulary words whose letter-multiset is a
  subset of the input's), longest first.
- `--near K` — print "near-grams": vocabulary words reachable within K letter
  substitutions/additions/removals of an anagram (edit distance on sorted
  letter-multisets), for the looser parlor variant.
- `--vocab <file>` — newline word list to anagram against; default is the
  `bon-mot-core` lexicon keys plus `$BON_MOT_VOCAB` if set.
- `--json` — `{input, anagrams:[...], subanagrams:[...], nears:[...]}`.

Pure CPU, deterministic, offline. No network, no key, no `--lavish`.

## Acceptance criteria

1. `bon-mot-anagram listen --vocab <file-with-"silent">` returns `silent` as an
   exact anagram; a non-anagram in the vocab is **not** returned.
2. Normalization: `"Listen!"` and `"listen"` anagram identically; trailing
   punctuation/case/whitespace do not break the match.
3. `--sub` returns vocabulary words that are strict subanagrams, ordered longest
   first, and excludes the input itself unless it is in the vocab.
4. `--near 1` over a fixtured vocab returns words within one letter edit of an
   anagram and excludes words at distance >1.
5. `--json` emits valid JSON with all four arrays present (possibly empty).
6. Runs with no `$ANTHROPIC_API_KEY` and no network access (assert no `ureq`
   feature is required to build this binary); `cargo test` green over a fixtured
   vocabulary covering exact / sub / near cases.
