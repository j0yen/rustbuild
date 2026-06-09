# PRD: newyorker-issue — the issue-assembler: bind N daily columns into one periodical issue

**Status:** Draft v0.1
**build_target:** rust-cli
**build_into:** (new repo) `~/wintermute/new-yorker`
**Vision:** visions/new-yorker.md

## TL;DR

`roundtable` accumulates a daily `conning-tower` column (PRD
`conning-tower-column`) — over a period that is a pile of dated Markdown files
that no tool binds into anything. `visions/new-yorker.md` names this exact
failure: "a pile of 30 dated Markdown columns is not an issue of anything." This
PRD is the keystone — the binder that reads N columns from the column archive,
flattens their five persona voices through `house-style`, stamps the
`masthead`'s identity and folio, lays out the crowned bon mots and the period's
best games, and writes one numbered, dated periodical issue (Markdown, plus an
optional rendered form on the `daily-receipt` deterministic path).

## Why this exists

`~/wintermute/conversations-zine/` is wintermute's existing periodical, but its
`zine` binary stops at moment-extraction — its README states layout/print are
"explicitly downstream and human-driven." `newyorker-issue` is precisely that
downstream binding step the zine left undone, now automated.
`~/wintermute/daily-receipt/` proves an issue artifact can be rendered
**deterministically** (`render --summary --content --out`, AC3 byte-stability,
AC7 verbatim ISO date) — the optional rendered form here follows that contract.
This PRD consumes upstream: it reads columns produced by `conning-tower-column`
and archived per `visions/conning-tower.md`, stamps `newyorker-masthead`'s
folio, and runs prose through `newyorker-house-style`. The umbrella
`visions/roundtable.md` frames this as the durable artifact that outlives any
single lunch.

## What this builds

A binary crate `newyorker-issue` (binary `issue`) in the `~/wintermute/new-yorker`
workspace. Depends path-locally on the `newyorker-masthead` lib and shells out
to / links the `newyorker-house-style` `clean` pass.

Modules:
- `intake` — read the column archive. A `Column { date, contributor_handle,
  bon_mot, body, games: Vec<GameResult> }` deserialized from the
  `conning-tower` archive layout (JSON or front-mattered Markdown — match
  whatever `conning-tower-column` emits; documented as a small adapter trait
  `ColumnSource` so the exact upstream shape is swappable). `--since <date>` /
  `--until <date>` window selection over the archive.
- `assemble` — bind the windowed columns into an `Issue`: pick the folio
  (`Folio::next` from masthead, given the previous issue number found in the
  `issues/` tree), run each column body through house-style `clean`, collect the
  window's crowned bon mots into a "Talk of the Town" front section, collect the
  best games into a back section, order sections deterministically (by date then
  contributor handle — no map-iteration nondeterminism).
- `layout` — render the `Issue` to Markdown: masthead block (from
  `newyorker-masthead::render`), folio, front bon-mot section, the columns,
  the games section, a colophon. Pure function `issue_markdown(&Issue) -> String`.
- `render` — optional `--render` flag: emit a deterministic rendered form on the
  `daily-receipt` precedent (a byte-stable text/typeset artifact; the actual
  thermal/PDF wrapper stays downstream, out of scope — this PRD's `--render`
  produces a deterministic intermediate, not a printed page).
- `write` — land the issue under `~/wintermute/new-yorker/issues/NNNN/` (zero-
  padded issue number): `issue.md`, `issue.json` (the structured `Issue`), and
  a `folio.txt`. Path root overridable via `--issues-dir`.

Deps: `newyorker-masthead` (path), `serde`/`serde_derive`, `serde_json`,
`chrono` (NaiveDate), `clap`, `thiserror`, `walkdir`. No network, no clock for
the folio (date passed via `--date`, defaulting to the newest column's date, not
`now()`, to preserve determinism in tests).

CLI subcommands:
- `issue assemble --since <YYYY-MM-DD> [--until <YYYY-MM-DD>] [--columns-dir <d>]
  [--issues-dir <d>] [--date <YYYY-MM-DD>] [--render]` — bind and write the issue;
  prints the folio and the written path.
- `issue show <issue-number> [--issues-dir <d>]` — print a previously assembled
  issue's Markdown.
- `issue list [--issues-dir <d>]` — list assembled issues as `NNNN — folio —
  date — N columns`.

## Acceptance criteria

1. Given a fixture `--columns-dir` holding 3 columns dated 2026-06-06..08,
   `issue assemble --since 2026-06-06 --until 2026-06-08 --date 2026-06-08`
   writes `issues/0001/issue.md` containing all three columns' bodies and exits 0.
2. The written `issue.md` begins with the masthead block (contains the title and
   the folio string) and the folio's ISO date appears verbatim as `2026-06-08`
   (daily-receipt AC7 contract).
3. Every column body in the output has been run through house-style `clean`: a
   column whose source body contains `I think` and a double-space appears in the
   issue with the hedge removed and the spacing collapsed (proves the
   house-style pass is actually applied, not bypassed).
4. **Determinism**: two `issue assemble` runs over byte-identical inputs (same
   columns, same `--date`, same prior `issues/` state) produce byte-identical
   `issue.md` and `issue.json` (daily-receipt AC3 contract; section ordering is
   stable by date-then-handle, no map iteration leak).
5. Issue numbering is monotonic via masthead's `Folio::next`: assembling a
   second issue when `issues/0001/` already exists writes `issues/0002/` with
   `issue_number == 2`; it does **not** overwrite 0001.
6. The window selector is honored: `--since`/`--until` excludes out-of-window
   columns — a column dated outside the range does not appear in the issue.
7. `--render` writes a deterministic additional artifact whose bytes are stable
   across two identical runs; absence of `--render` writes only `issue.md` +
   `issue.json` + `folio.txt`.
8. A missing/empty `--columns-dir`, or a window matching zero columns, exits
   non-zero with a `thiserror` message (no empty issue silently written, no
   panic).
9. `cargo test` passes and `cargo build --release` produces the `issue` binary.
