# PRD: thanatopsis-parlor-ledger — a persistent record of games played and their outcomes

**Status:** Draft v0.1
**build_target:** rust-extend
**build_into:** (existing repo) `~/wintermute/thanatopsis`
**Vision:** visions/thanatopsis.md

## TL;DR

The four games — charades, poker, crossword, murder — each emit a `GameResult`
and then it scrolls off the terminal. Nothing remembers that yesterday's poker
crowned the haiku, that Tuesday's Murder went unsolved, that the crossword keeps
harvesting the same five words. Without a record, the games can't compose with
the rest of the roundtable: `conning-tower` can't publish "the table's favorite,
by a three-chip margin," and `back-issues` can't show the games' history. This
PRD builds the parlor ledger: an append-only, queryable record of every game
played and its outcome, so the playful side of the table becomes durable signal
the publication can read.

## Why this exists

The umbrella vision `visions/roundtable.md` is explicit that the best of the
table's output gets *published* (`conning-tower`, after FPA's column) and bound
into back-issues. The games produce crownings, solved/unsolved verdicts, and
puzzles — all worth publishing — but only if they persist. The vicious-circle
fleet already established this pattern with its `vicious-circle-ledger` (an
append-only JSONL the downstream column reads); `thanatopsis` needs the parallel
record for the games so `conning-tower` (slug `conning-tower`, drafted in the
roundtable fleet) and `back-issues` can compose with it on equal footing with
the critique ledger.

This PRD extends the `thanatopsis` crate and is the natural last piece: it
consumes the shared `GameResult` type that `charades`, `poker`, `crossword`, and
`murder` all emit, and persists it. It builds against fixture `GameResult`s, so
it does not block on the games — it persists whatever results it is handed.

## What this builds

Extends `~/wintermute/thanatopsis` with the `ledger` subcommand and wires the
other games to record through it.

Modules:
- `src/ledger.rs` —
  - `Ledger::open(path) -> Result<Ledger>`; default path resolves to
    `$XDG_DATA_HOME/thanatopsis/parlor.jsonl` (via `directories`), creating
    parent dirs.
  - `append(&self, result: &GameResult) -> Result<()>`: append one JSON line;
    each line stamped with an ISO-8601 `recorded_at` and a stable `game_id`
    (hash of game + date + seed) so re-runs are idempotent-detectable.
  - `read_all(&self) -> Result<Vec<LedgerEntry>>` and
    `query(filter) -> Vec<LedgerEntry>` with filters by `game`, by `date`/date
    range, and `latest(n)`.
  - `summary(entries) -> Standings`: aggregate stats — per-game counts, poker's
    most-crowned artifact, murder's solved-rate, charades' guess-accuracy,
    crossword's most-harvested word — the "house standings."
- `src/main.rs` — add the `ledger` subcommand; add a shared `--record` flag to
  the four game `play` subcommands that appends the emitted `GameResult` to the
  ledger after printing.

CLI subcommands:
- `thanatopsis ledger record --file <gameresult.json>` — append a `GameResult`
  read from a file or stdin (the integration seam the games use).
- `thanatopsis ledger show [--game <name>] [--date <d>] [--since <d>]
  [--latest <n>] [--json]` — print matching entries.
- `thanatopsis ledger standings [--since <d>] [--json]` — print the aggregated
  house standings.

Deps: reuse the crate's `clap`/`serde`/`serde_json`/`anyhow`/`sigpipe`; add
`directories` (XDG paths) and `chrono` (timestamps).

The ledger is append-only — `record` MUST NOT rewrite or reorder existing lines,
and a corrupt/partial trailing line MUST be skipped on read with a warning, not
abort the whole query (so a crash mid-append never bricks the ledger).

## Acceptance criteria

1. `cargo build` and `cargo test` succeed; clippy adds no new warnings beyond
   the repo baseline.
2. `thanatopsis ledger record` reading a fixture `GameResult` JSON appends
   exactly one line to a temp ledger; a test asserts the file gains one parseable
   `LedgerEntry` with a populated `recorded_at` and `game_id`.
3. Append-only: a test appends three results, then a fourth, and asserts the
   first three lines are byte-identical before and after the fourth append.
4. Corruption tolerance: a test writes a ledger with a valid line, a truncated
   trailing line, then calls `read_all` and asserts it returns the valid entry
   and skips the bad line without erroring.
5. `thanatopsis ledger show --game poker --json` returns only poker entries from
   a mixed fixture ledger; `--latest 2` returns the two most recent across all
   games in recency order.
6. `thanatopsis ledger standings --json` over a fixture ledger reports correct
   per-game counts, the most-crowned poker artifact, and the murder solved-rate
   (asserted against hand-computed fixture expectations).
7. A game `play` subcommand invoked with `--record --ledger <tmp>` both prints
   its result and appends it to that ledger (asserted end-to-end for at least
   one game); the crate still builds `--no-default-features` and
   `--features bon-mot`.
