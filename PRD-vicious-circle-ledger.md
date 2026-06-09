# PRD: vicious-circle-ledger — the persistent record of every round

**Status:** Draft v0.1
**build_target:** rust-extend
**build_into:** `~/wintermute/vicious-circle`
**Vision:** visions/vicious-circle.md

## TL;DR

A round that vanishes when the process exits can never become a column or a
back-issue. This PRD gives the circle a memory: an append-only ledger that
records every round — its artifact, its verdicts, its cross-verdicts, and its
crowned bon mot — so the downstream `conning-tower` column and `new-yorker`
issues can read back what the table said on any given day.

## Why this exists

The umbrella vision `visions/roundtable.md` ends the write-only era by making
creativity "a conversation — voices answering voices, the best of it printed."
Printing requires a durable record: `conning-tower` composes the day's crowned
lines into a column and runs a "Constant Reader" feedback loop; `new-yorker`
binds columns into issues. Both read history. The solo creative repos this
critiques (`~/wintermute/day-haiku`, `~/wintermute/conversations-zine`,
`~/wintermute/letters-we-never-sent`, `~/wintermute/self-portrait`) each archive
their own output to disk; the ledger is the circle's equivalent archive of its
*verdicts*. The roundtable vision designates JSONL as the start, with a later
`conning-tower` PRD free to index it (or migrate to `~/wintermute/recall`).

## What this builds

Extends `~/wintermute/vicious-circle`.

Modules:
- `src/ledger.rs`:
  - `Round { date: String, artifact: String, verdicts: Vec<Verdict>,
    crosses: Vec<CrossVerdict>, bon_mot: Option<BonMot> }` (serde).
  - `Ledger::append(round: &Round) -> Result<()>` — append one JSON object per
    line to the ledger file (JSONL), creating it and parent dirs if absent.
    Default path `$XDG_DATA_HOME/vicious-circle/ledger.jsonl`; overridable via
    `--ledger <path>` and `VICIOUS_CIRCLE_LEDGER` env.
  - `Ledger::read_all(path) -> Result<Vec<Round>>` — parse the JSONL back.
  - `Ledger::rounds_on(date) -> Vec<Round>` and `latest() -> Option<Round>`.
- `src/main.rs` — add the `ledger` and `record` subcommands.

Deps: reuse `serde`/`serde_json`, `directories`, `anyhow`. Date via `time` or
`chrono` (whichever the autobuilder scaffold prefers); store ISO-8601 date.

CLI:
- `vicious-circle record <artifact-path>` — run the full round (review → roast →
  crown) and append the resulting `Round` to the ledger; print the bon mot.
- `vicious-circle ledger list` — list recorded rounds (date, artifact, crowned
  line).
- `vicious-circle ledger show <date>` — print all rounds on a date as JSON.
- `vicious-circle ledger latest` — print the most recent round.

## Acceptance criteria

1. `cargo build` and `cargo test` succeed in `~/wintermute/vicious-circle`.
2. `Ledger::append` writes exactly one line per round; appending three rounds to
   a fresh temp ledger yields a 3-line JSONL file, each line independently valid
   JSON (test).
3. `Ledger::read_all` round-trips: append N `Round`s, read them back, assert
   equality of the deserialized vec (test).
4. Append creates the ledger file and any missing parent directories when they
   do not exist (test against a temp path with a nonexistent parent).
5. `--ledger <path>` and `VICIOUS_CIRCLE_LEDGER` both override the default XDG
   path (test the override resolution; CLI flag wins over env).
6. `vicious-circle record <file>` appends a `Round` whose `verdicts`,
   `crosses`, and `bon_mot` match a same-input `review`/`roast`/`crown` run, and
   prints the crowned bon mot.
7. `vicious-circle ledger show <date>` returns only rounds whose stored date
   equals the argument; `ledger latest` returns the last-appended round (test
   with two dated rounds).
8. `ledger list | head -1` does not panic (`sigpipe::reset()` in effect).
