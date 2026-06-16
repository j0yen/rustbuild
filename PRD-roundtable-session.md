# PRD: roundtable-session — convene the whole table, end to end

Status: Draft v0.1
build_target: rust-cli
build_into: /home/jsy/wintermute/roundtable
Vision: visions/roundtable.md

## TL;DR

The Round Table's six sub-visions all shipped — but the table never actually
convenes. `the-lunch lunch` sets the table (`convene → seat → menu → minutes
open`) and then stops; nobody eats. This PRD ships `roundtable` (new repo) with
its core command `roundtable session [--date]`, which runs the full daily chain:
set the table (`the-lunch lunch`), then for each seated artifact run the critique
round (`vicious-circle record`), then compose and syndicate the day's column
(`conning-tower compose` + `syndicate`). One command turns six soloists into a
lunch.

## Why this exists

Verified 2026-06-15 by running `--help` on every shipped binary:

- `the-lunch lunch` — "Run the full lunch chain: convene → seat → menu →
  minutes open." It persists `table.json` and `seating.json` under
  `$XDG_STATE_HOME/the-lunch/<date>/` and **stops at minutes-open**. The
  critique never runs.
- `vicious-circle record <ARTIFACT_PATH> [--ledger <path>]` — "Run a full round
  and append it to the ledger" (review → roast → crown). Default ledger
  `$XDG_DATA_HOME/vicious-circle/ledger.jsonl`; also honors
  `VICIOUS_CIRCLE_LEDGER`.
- `conning-tower compose --ledger <ledger> [--date]` — composes the day's
  crowned column from the vicious-circle ledger; `conning-tower syndicate
  --ledger <ledger> --columns-dir <dir> --to columns` routes it to the columns
  slot.

So every stage exists and exposes a clean CLI; what is missing is the wire that
runs them in order over the day's actual artifacts. The umbrella vision's "Order"
diagram (`the-lunch → vicious-circle → conning-tower`) assumed a convener that
was never written. This PRD is that convener.

## What this builds

A new rust-cli repo `~/wintermute/roundtable` (binary `roundtable`):

- **`roundtable session [--date YYYY-MM-DD] [--dry-run]`** — the chain:
  1. Run `the-lunch lunch --date <date>` (idempotent; sets the table). Parse the
     resulting `$XDG_STATE_HOME/the-lunch/<date>/table.json` to enumerate the
     day's dishes (artifacts) and their file paths.
  2. For each artifact path on the table, run `vicious-circle record <path>`
     against a single shared ledger for the date (pass `--ledger` explicitly so
     the run is self-contained and testable). Collect each round's crowned line.
  3. Run `conning-tower compose --ledger <ledger> --date <date>` to produce the
     column, then `conning-tower syndicate --ledger <ledger> --date <date> --to
     columns` to route it to the canonical columns slot.
  4. Print a short summary: N artifacts critiqued, the crowned **bon mot**, and
     where the column landed.
- **Tool resolution**: resolve `the-lunch`, `vicious-circle`, `conning-tower`
  from `$PATH` (override via `--bin-dir <dir>` or per-tool env, e.g.
  `ROUNDTABLE_LUNCH_BIN`). If any required tool is absent, exit non-zero with an
  actionable message naming the missing binary — do not silently skip a stage.
- **Idempotent + fail-loud**: mirror `the-lunch lunch`'s contract — each stage's
  progress persists, and on failure the command exits non-zero and **names the
  failed stage** (`lunch` / `critique:<artifact>` / `compose` / `syndicate`).
  Re-running `session` for the same date does not double-append to the ledger
  for artifacts already recorded that day (dedup on artifact path + date).
- **`--dry-run`**: print the exact stage commands in order without executing the
  mutating ones (still allowed to read `table.json`).

MSRV 1.85. Deps: clap, anyhow, serde/serde_json (parse `table.json` + ledger).
The heavy lifting stays in the existing binaries; this crate is orchestration +
the dedup/summary logic only — it re-implements none of the critique, scoring,
or composition.

## Acceptance criteria

1. `cargo test --release` passes; `cargo clippy -- -D warnings` passes in
   `~/wintermute/roundtable`.
2. `roundtable session --dry-run --date 2026-06-15` prints the ordered stage
   plan (`the-lunch lunch` → `vicious-circle record …` per artifact → `compose`
   → `syndicate`) and mutates nothing (a test asserts no ledger/columns writes
   under `--dry-run`).
3. With stub binaries on `--bin-dir` (test fixtures that emit a known
   `table.json` and a known ledger line), `roundtable session` runs the full
   chain and prints a summary naming the crowned line and the column
   destination. Skip live binaries if absent, but unit-test the chain logic
   against the fixtures.
4. A missing required binary (e.g. `conning-tower` not on `$PATH` / `--bin-dir`)
   makes `roundtable session` exit non-zero with a message naming the binary —
   a test asserts the exit code and the named tool.
5. A stage failure (a stub that exits non-zero) makes `session` exit non-zero
   and name the failed stage (`lunch`/`critique:<artifact>`/`compose`/
   `syndicate`) — a test exercises at least one mid-chain failure.
6. Re-running `session` for the same date does not double-record an artifact
   already in the ledger for that date (a test asserts the ledger length is
   stable across two runs).
7. `roundtable session` over an empty table (no dishes) completes cleanly,
   reports "no artifacts on the table today," and exits 0 (an empty lunch is
   not an error).
