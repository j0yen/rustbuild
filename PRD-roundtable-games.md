# PRD: roundtable-games — the parlor games after the critique

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/roundtable
Vision: visions/roundtable.md
Depends-on: roundtable-session (the chain + table parsing it extends)

## TL;DR

The real Round Table was also the Thanatopsis Literary & Inside Straight Club —
the same people who critiqued each other's work at lunch also played poker,
charades, and word games after. `thanatopsis` shipped all four games as a
standalone binary, but nothing deals the day's actual table into them. This PRD
extends `roundtable` with `roundtable games [--date]` (and `session
--with-games`), which after the critique round deals the day's seated artifacts
into `thanatopsis poker` and `charades` and records the outcomes to the games
ledger.

## Why this exists

Verified 2026-06-15 via `thanatopsis --help`: the binary exposes
`poker` (personas bet chips on the best artifact), `charades` (describe an
artifact without naming it), `crossword`, `murder`, and `ledger` (persistent
game outcomes). All five subcommands are implemented (repo at v0.5.0). But like
the critique stage before roundtable-session, the games are never fed the day's
real table — they are demoware until something deals them the artifacts that
`the-lunch convene` already gathered into `table.json`. roundtable-session
established both the table-parsing and the tool-resolution patterns; this PRD
reuses them to add the after-lunch games as one more orchestrated stage.

## What this builds

Extend `~/wintermute/roundtable`:

- **`roundtable games [--date YYYY-MM-DD] [--dry-run]`** — read the day's
  `table.json` (same parse path as roundtable-session), then:
  - `thanatopsis poker` over the day's artifacts (personas bet on the best),
  - `thanatopsis charades` on a dealt artifact,
  - append both outcomes to the thanatopsis ledger (pass the ledger path
    explicitly so the run is self-contained).
- **`roundtable session --with-games`** — run the full critique chain
  (roundtable-session) and then the games stage in the same invocation. Without
  the flag, `session` behaves exactly as roundtable-session shipped it (the
  flag is additive; a test asserts the default is unchanged).
- Reuse roundtable-session's tool resolution (`$PATH` / `--bin-dir` /
  per-tool env, here `ROUNDTABLE_THANATOPSIS_BIN`); a missing `thanatopsis`
  binary is an actionable non-zero error, not a silent skip.
- Same fail-loud + idempotent contract: name the failed game on error
  (`games:poker` / `games:charades`); re-running for the same date does not
  double-append a game already recorded that day.
- `--dry-run` prints the game commands without executing the mutating ones.

MSRV 1.85. No new heavy deps; reuses the orchestration + table-parse code from
roundtable-session. The games themselves stay in `thanatopsis`.

## Acceptance criteria

1. `cargo test --release` passes; `cargo clippy -- -D warnings` passes.
2. `roundtable games --dry-run --date 2026-06-15` prints the ordered game plan
   (`thanatopsis poker …`, `thanatopsis charades …`) and mutates nothing.
3. With a stub `thanatopsis` on `--bin-dir` and a fixture `table.json`,
   `roundtable games` deals the day's artifacts into both games and reports the
   outcomes — unit-tested against the fixture.
4. `roundtable session --with-games` runs the critique chain **and** the games
   stage; `roundtable session` (no flag) runs the critique chain only — a test
   asserts the games stage is skipped without the flag.
5. A missing `thanatopsis` binary makes the games stage exit non-zero naming the
   binary — a test asserts exit code and named tool.
6. A failing game (stub exits non-zero) makes the stage exit non-zero and name
   the failed game (`games:poker`/`games:charades`).
7. Re-running `games` for the same date does not double-append a game already
   recorded that day (a test asserts ledger stability across two runs).
8. `roundtable games` over an empty table completes cleanly, reports no game
   played, and exits 0.
