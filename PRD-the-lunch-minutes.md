# PRD: the-lunch-minutes — keep the durable transcript of the gathering

**Status:** Draft v0.1
**build_target:** rust-cli
**build_into:** (new repo) `~/wintermute/the-lunch`
**Vision:** visions/the-lunch.md

## TL;DR

A lunch that convenes, seats, and sets an agenda but leaves no record is just
another write-only event — the very failure mode `roundtable` exists to fix.
There is no transcript: nowhere that says *who said what, about which dish, in
what order*. This PRD builds the **minutes**: an append-only, durable record of
the gathering that `vicious-circle` writes its persona verdicts into and that
becomes the day's quotable source for `conning-tower`.

## Why this exists

`visions/roundtable.md` ends on the point that creativity here must "stop being
a pile of write-only artifacts and become a conversation … the best of it
printed." A conversation no one writes down cannot be printed. The convene,
seating, and menu PRDs produce the *plan* of a gathering (`Table`, `Seating`,
`Menu`); minutes produce the *record* of it. It is the last data layer in
`the-lunch` before the standing invitation wires the cadence: every downstream
sub-vision (`vicious-circle` verdicts, `conning-tower` crowning the bon mot,
`new-yorker` binding columns) reads or appends to the minutes. Append-only and
durable so the day's gathering is never silently overwritten — mirroring how
`daily-receipt` produces a fixed daily record.

## What this builds

Extends `~/wintermute/the-lunch`. New module `minutes.rs` in the `the_lunch`
library plus a `minutes` CLI subcommand.

Types:
- `Remark { seq: u64, at: DateTime<Local>, persona: PersonaId,
  dish_ref: Option<DishRef>, kind: RemarkKind, text: String }`.
  `RemarkKind`: `Critique`, `Roast` (a persona answering another's take),
  `Aside`, `Note` (table-keeping, e.g. an empty-seat observation).
- `Minutes { date: NaiveDate, opened: DateTime<Local>,
  host: PersonaId, remarks: Vec<Remark> }` with monotonic `seq`.

Logic (`minutes.rs`):
- `open(table, seating, menu) -> Minutes` — initializes the record for the day:
  host from the menu, opening `Note` remarks listing the menu courses and any
  empty seats (so the transcript shows what was and wasn't on the table).
- `append(&mut self, persona, dish_ref, kind, text) -> seq` — appends a remark
  with the next `seq` and current timestamp; **append-only**, never mutates or
  removes existing remarks.
- `store`: one JSONL file per day,
  `$XDG_STATE_HOME/the-lunch/<date>/minutes.jsonl`, one `Remark` per line.
  `load(date)` reconstructs `Minutes` by replaying lines in `seq` order;
  `append` writes a single line (atomic append, `O_APPEND`).
  Re-opening an existing day is idempotent: `open` is a no-op if minutes already
  exist for the date (does not wipe the transcript).
- `transcript(&self) -> String` — render the human-readable minutes:
  `[seq] HH:MM PERSONA (re DISH): text`, grouped by course in menu order.

CLI:
- `the-lunch minutes open [--date d]` — create the day's minutes from the
  persisted table/seating/menu; no-op if already open.
- `the-lunch minutes add --persona <id> [--dish <kind:source>]
  --kind <critique|roast|aside|note> --text <...>` — append a remark
  (the interface `vicious-circle` calls).
- `the-lunch minutes show [--date d] [--json]` — print the transcript or raw
  JSONL-derived `Minutes`.

Deps: reuses convene deps; JSONL append via `std::fs::OpenOptions` (`append`).

## Acceptance criteria

1. `cargo build` / `cargo test` green on toolchain 1.85.
2. `append` is strictly append-only: `seq` is monotonic and existing remarks
   are byte-identical before and after a new append (test reads the JSONL file
   before/after).
3. `minutes open` writes opening `Note` remarks that list the menu courses and
   every empty seat from the table; a second `open` on the same date is a no-op
   and leaves the existing transcript intact (test asserts unchanged file).
4. `minutes add` with a `--dish kind:source` attaches the correct `DishRef`;
   without `--dish` the remark has `dish_ref == None`.
5. The JSONL store round-trips: write N remarks, `load`, and assert the
   reconstructed `Minutes` equals the in-memory one (order + seq + content).
6. `RemarkKind::Roast` remarks are accepted and rendered distinctly from
   `Critique` in `transcript` (a persona answering another persona).
7. `transcript` groups remarks by course in **menu order** and prefixes each
   with `[seq] HH:MM PERSONA`; verified against a fixture day.
8. `minutes show --json` emits the full `Minutes` parseable back; `minutes show`
   prints the human transcript.
9. Atomic append under concurrent `add` calls does not interleave partial lines
   (test spawns two appends; both lines are well-formed JSON).
