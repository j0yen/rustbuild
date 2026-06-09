# PRD: bon-mot-epigram — the epigram forge

**Status:** Draft v0.1
**build_target:** rust-cli
**build_into:** (new repo) `~/wintermute/bon-mot`
**Vision:** visions/bon-mot.md

## TL;DR

An epigram compresses an observation into one quotable, aphoristic line — the
Round Table's house form. wintermute generates plenty of *facts* (commits,
ctrace writes, journal headings) but no aphorisms about them.
`bon-mot-epigram` takes an observation — free text or a `summary.json`
telemetry fact — and forges one witty line under a length budget, deterministic
by default and Claude-powered on `--lavish`.

## Why this exists

`~/wintermute/day-haiku` already reads the daily `summary.json` and turns its
fields (`commits`, `repos`, `ctrace_top_paths`, `recall_writes`,
`journal_first_heading`) into a haiku — see its `prompt::render_ephemeral`.
The epigram forge consumes the *same* `summary.json` shape but produces a
single aphoristic line instead of three poetic ones, giving roundtable a second,
sharper register over the identical telemetry. The umbrella
`visions/roundtable.md` lists `day-haiku` among the solo outputs that "need a
shared wit layer," and names the epigram forge explicitly as a bon-mot
component. day-haiku's caching client (`src/api.rs`, `build_body`) is the
template for this PRD's `--lavish` tier.

## What this builds

A CLI `bon-mot-epigram` (depends on `bon-mot-core`).

- `bon-mot-epigram --text "<observation>"` — default tier. Selects an
  epigrammatic frame from `core::grammar` (antithesis "X, but Y"; reversal
  "the more X, the less Y"; deflation "X — which is to say, Y"), binds the
  observation's salient noun/verb (extracted by a simple heuristic) plus a
  contrasting term from `core::lexicon` synonyms/antonyms, and prints one line
  within `--max-words N` (default 14, the epigram budget).
- `bon-mot-epigram --summary summary.json` — reuse day-haiku's field set: pick
  the most striking fact (e.g. the top ctrace prefix, or commits across repos)
  and forge an epigram about it.
- `--lavish` — call `core::lavish` with a system block defining the epigram form
  + cached few-shot examples and the observation as the trailing turn.
- `--json` — `{source, epigram, frame, words, tier}`.

UX: one line to stdout; respects `--max-words`; `--seed` for reproducibility.

## Acceptance criteria

1. `bon-mot-epigram --text "I deployed at 2am again" --seed 1` prints one line
   of `--max-words`≤14, deterministic for the seed.
2. `--summary <file>` accepts a `summary.json` with day-haiku's field names
   (`commits`, `repos`, `ctrace_top_paths`, `recall_writes`,
   `journal_first_heading`) and produces an epigram referencing a real field
   value from that file.
3. The default tier never exceeds `--max-words`; a chosen `--max-words 8` is
   honored (asserted by counting `core::count::words`).
4. `--json` emits valid JSON including `frame` (the epigrammatic frame used) and
   `words` (the actual word count).
5. `--lavish` with no key exits 6; against the loopback mock it returns the mock
   epigram and the request shows cached system + few-shot blocks.
6. `cargo test` green: a `--text` case, a `--summary` case using a fixture in the
   day-haiku field shape, and a `--max-words` enforcement case.
