# PRD: conning-tower-column — compose the day's crowned lines into one dated column

**Status:** Draft v0.1
**build_target:** rust-cli
**build_into:** (new repo) `~/wintermute/conning-tower`
**Vision:** visions/conning-tower.md

## TL;DR

`vicious-circle` crowns a `bon mot` each round and writes every `Verdict` to an
append-only ledger — but a crown that lands in a JSONL file is read by nothing.
There is no artifact that takes the day's best line and *prints* it. This PRD
builds the daily column: read the ledger, pull today's crowned line and the top
runner-up verdicts, and render one dated Markdown column — the artifact the rest
of `conning-tower` decorates, syndicates, and archives.

## Why this exists

`~/wintermute/conversations-zine` is the standing proof of the failure mode: its
own README states "the bottleneck is the moment-extractor step" and that
"layout/print/mail are explicitly downstream and human-driven" — so the moments
it extracts are write-only, surfaced and then unanswered. The umbrella
[roundtable.md](visions/roundtable.md) frames the column as exactly where wit
"gets published instead of composted." The upstream `PRD-vicious-circle-crown`
produces a single best line per round and `PRD-vicious-circle-ledger` persists
every verdict as JSONL; without a composer, that crown is just another row.
`~/wintermute/daily-receipt` and the `cadence` vision already establish a
daily-render ritual (a deterministic day-typed artifact emitted once per day) —
this PRD is that ritual for the Round Table's column.

## What this builds

A new repo `~/wintermute/conning-tower`: CLI `conning-tower`, lib `conning_tower`.

- **`ledger.rs`** — reader for the upstream ledger. Parse the append-only JSONL of
  `Verdict { persona, target, line, score, stance }` plus the crowned-bon-mot
  record from `PRD-vicious-circle-crown`. Ledger path is config/flag driven
  (`--ledger`, default `~/wintermute/vicious-circle/ledger.jsonl`). Filter to a
  given date.
- **`column.rs`** — the `Column` type: `{ date, headline, bon_mot: Line, runners_up:
  Vec<Line>, source_ledger }` where `Line { text, persona, target, score, stance }`.
- **`render.rs`** — render a `Column` to dated Markdown: an `# YYYY-MM-DD — The
  Conning Tower` headline, the bon mot set off as a blockquote, a "Runners-up"
  list, deterministic byte-stable output (no timestamps in body; date only).
- **CLI subcommands:**
  - `conning-tower compose [--date YYYY-MM-DD] [--ledger PATH] [--top N] [--format
    md|json] [--out PATH|-]` — read ledger for date (default today), build the
    `Column`, render. `--top N` caps runners-up (default 5). `-`/stdout default.
- Deps: `clap`, `serde`/`serde_json`, `time` or `chrono` (date only), `anyhow`.

## Acceptance criteria

1. `conning-tower compose --ledger <fixture.jsonl> --date 2026-06-08 --format json`
   on a fixture with one crowned record and ≥3 verdicts for that date emits JSON
   `{date, headline, bon_mot, runners_up, source_ledger}`; `bon_mot` is the crowned
   line; `runners_up` are the next-highest-score verdict lines for that date,
   excluding the bon mot, sorted score-descending.
2. `--format md` (default) renders the bon mot as a Markdown blockquote under an
   `# 2026-06-08 — The Conning Tower` headline and lists runners-up; rendering the
   same fixture twice is byte-identical (no wall-clock in the body).
3. `--top N` caps `runners_up` at N (default 5); a date with fewer verdicts yields
   fewer runners-up without error; a date with a crown but zero other verdicts
   yields an empty `runners_up` and still renders the headline + bon mot.
4. A `--date` with no crowned record in the ledger exits non-zero (exit 3) with a
   stderr message naming the date and the ledger path; a missing ledger file exits
   2 with stderr containing the path and `ledger not found`.
5. Only verdicts whose date matches `--date` are considered; verdicts from other
   dates in the same ledger are ignored (proven by a multi-date fixture).
6. `--out PATH` writes the rendered column to PATH and prints nothing to stdout;
   `--out -` (default) writes to stdout. Parent dir of `--out` is created if absent.
7. Each AC has a matching integration test under `tests/acceptance_ac<n>.rs` using
   committed JSONL fixtures under `tests/fixtures/`.
