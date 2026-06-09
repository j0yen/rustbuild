# PRD: thanatopsis-charades — describe an artifact without naming it; the table guesses

**Status:** Draft v0.1
**build_target:** rust-cli
**build_into:** (new repo) `~/wintermute/thanatopsis`
**Vision:** visions/thanatopsis.md

## TL;DR

The roundtable can critique an artifact but it never tests whether the day's
outputs are *legible and distinct from one another*. Two haiku that read the
same, a zine excerpt indistinguishable from a letter — critique scores each in
isolation and never notices. This PRD builds charades: a persona is dealt one
artifact off the day's table and must describe it *without naming or quoting
it*; the other personas guess which artifact on the table was being described.
A round that everyone guesses instantly means the work is vivid (or trite); a
round no one can guess means the work is muddy. The game is the legibility test
critique can't perform.

## Why this exists

This laptop's creative wing is six write-only soloists whose outputs land on the
day's table: `~/wintermute/day-haiku` (a daily haiku), `~/wintermute/conversations-zine`
(extracted moments), `~/wintermute/letters-we-never-sent` (unsent letters),
`~/wintermute/self-portrait` (CLAUDE_SELF diffs), `~/wintermute/ambient` and
`~/wintermute/wintermute-music` (sound cues). The umbrella vision
`visions/roundtable.md` frames games as the *generative, playful counterpart* to
critique — the Round Table played charades nightly. Charades is exactly the game
that surfaces whether these six outputs are distinguishable: if a persona can
describe one and the others can pick it out of the pile, the day's work is
legible.

This PRD is the first to land in the `thanatopsis` repo, so it scaffolds the
shared input types it consumes from upstream `the-lunch` (drafted in parallel,
slug `the-lunch`): a `Table` JSON object of the day's `Artifact`s. The clue a
persona builds reuses word primitives from `bon-mot` (drafted in parallel, slug
`bon-mot`) — its salient-word extraction and synonym/grammar helpers — so the
clue is "the velvet-stiletto persona's paraphrase," not a literal quote.

## What this builds

A new repo `~/wintermute/thanatopsis` with a library crate `thanatopsis` and a
binary `thanatopsis`. This PRD scaffolds the crate and adds the `charades`
subcommand.

Modules:
- `src/table.rs` — the shared upstream input types: `Artifact { id, source,
  kind, text, path }` and `Table { date, artifacts: Vec<Artifact> }`, with
  `Table::load(path) -> Result<Table>` parsing `the-lunch`'s table JSON, plus a
  glob fallback `Table::from_globs(roots) -> Result<Table>` that reads the six
  creative repos' output dirs when no table file is given.
- `src/game.rs` — the shared output type `GameResult { game, date, players,
  outcome, detail }` (serde) that every thanatopsis game emits and the
  parlor-ledger persists.
- `src/words.rs` — a thin façade over `bon-mot` word primitives behind a
  `bon-mot` cargo feature (optional path dep on `~/wintermute/bon-mot`); a small
  built-in fallback (stopword strip + lowercase tokenize + frequency rank) so
  this PRD builds before `bon-mot` lands.
- `src/charades.rs` — the game. `describe(artifact, persona_seed) -> Clue`
  builds a non-naming, non-quoting description from the artifact's salient words
  (via `words`), the artifact `kind`, and the persona's tic; `guess(clue, table)
  -> Guess { artifact_id, confidence }` scores each artifact on the table by
  overlap between the clue's salient words and the artifact's, returning the
  best match; `play(table, deal) -> CharadesRound` deals one artifact, builds a
  clue, has each *other* persona guess, and records hits/misses.
- `src/lib.rs` — re-exports `Artifact`, `Table`, `GameResult`, `Clue`,
  `CharadesRound`.
- `src/main.rs` — clap CLI; first line of `main()` calls `sigpipe::reset()`
  (per the laptop's known println-SIGPIPE issue).

CLI subcommands:
- `thanatopsis charades play --table <path> [--deal <artifact_id>] [--seed <n>]`
  — runs a round, prints the clue, each persona's guess, and whether the table
  guessed right; `--json` emits the `GameResult`.
- `thanatopsis charades describe --table <path> --artifact <id>` — prints just
  the non-naming clue for one artifact (the "act it out" step).

Deps: `clap` (derive), `serde` + `serde_json`, `anyhow`, `glob`, `sigpipe`.
Optional path dep `bon-mot` behind feature `bon-mot`.

The clue MUST NOT contain the artifact's title, id, file path, or any verbatim
trigram from its text — this is the core invariant (charades is *not* quoting).

## Acceptance criteria

1. `cargo build` and `cargo test` succeed; `cargo clippy` adds no new warnings
   beyond the repo baseline.
2. `thanatopsis charades describe --table tests/fixtures/table.json --artifact
   haiku-1` prints a clue that contains **no** verbatim trigram from the
   artifact's text and **not** its id or title (asserted by a test that scans
   the clue against the artifact).
3. `thanatopsis charades play --table tests/fixtures/table.json --seed 1
   --json` emits a valid `GameResult` with `game == "charades"`, a non-empty
   `players` list, and an `outcome` recording the dealt artifact and each
   guesser's chosen artifact id.
4. Given a fixture table of ≥3 clearly distinct artifacts, a deterministic test
   shows the guessers identify the dealt artifact (the highest-confidence guess
   equals the dealt id) for at least one seed — proving legible artifacts are
   guessable.
5. Given a fixture table containing two near-duplicate artifacts, a test shows
   the guess confidence margin between them is below a threshold — proving the
   game detects muddiness rather than always guessing right.
6. With `--table` omitted, `Table::from_globs` reads a fixture directory tree
   and produces a non-empty table; with a malformed table JSON the CLI exits
   non-zero with a clear error.
7. The crate builds with `--no-default-features` (built-in word fallback) and
   with `--features bon-mot` resolving the path dep; `--json` output is valid
   parseable JSON in both.
