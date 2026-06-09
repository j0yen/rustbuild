# PRD: conning-tower-contributors — attribute each printed line and render a masthead

**Status:** Draft v0.1
**build_target:** rust-extend
**build_into:** (existing repo) `~/wintermute/conning-tower`
**Vision:** visions/conning-tower.md

## TL;DR

A column that prints a brilliant line over no name is half a column — FPA's "The
Conning Tower" made Round Table members famous precisely *because* it credited
them by initial. `PRD-conning-tower-column` renders the day's lines but stops at
the bon mot's persona field. This PRD turns each printed line into a fully
attributed credit — persona *and* the source artifact it critiqued — and renders
a contributor masthead so the column names its table.

## Why this exists

The umbrella [roundtable.md](visions/roundtable.md) describes the crowned line as
"attributed, and published into a running column" — attribution is load-bearing,
not decorative; the whole point of the Circle was *whose* line it was. The
upstream ledger record from `PRD-vicious-circle-ledger` already carries `persona`
(who said it) and `target` (which artifact it critiqued — a `day-haiku` triple, a
`conversations-zine` excerpt, a `letters-we-never-sent` draft), so the data to
credit fully is present and currently dropped on the floor. `~/wintermute/recall`
and the wider fleet identify artifacts by source path; this PRD resolves a raw
`target` into a human credit line. Without it, the column says "— Parker" but
never "on the haiku of 2026-06-08," and the masthead — the Round Table's own
visible membership — never appears.

## What this builds

Extends `~/wintermute/conning-tower` (lib `conning_tower`, CLI `conning-tower`).

- **`attribution.rs`** — resolve a ledger `Verdict` into an `Attribution { persona,
  persona_display, source_kind, source_label }`. `source_kind` is derived from the
  `target` (e.g. a path or id under `day-haiku/`, `conversations-zine/`,
  `letters-we-never-sent/`, `self-portrait/`) via a small ordered rule table;
  unknown targets resolve to `source_kind: "artifact"` with the raw target as label.
- **`masthead.rs`** — collect the distinct contributing personas in a column and
  render a masthead block: each persona's display name + the count of lines they
  placed in the column, sorted by lines-placed then name.
- Extend the `Line` type from `PRD-conning-tower-column` with the resolved
  `Attribution`, and extend `render.rs` so each printed line carries `— <persona>,
  on <source_label>` and the column footer carries the masthead.
- **CLI:**
  - `conning-tower compose` gains `--masthead/--no-masthead` (default on) and
    attribution is always applied.
  - `conning-tower credits [--date] [--ledger] [--format md|json]` — print just the
    masthead/credits for a date without the full column.

## Acceptance criteria

1. Composing a column from a fixture where the bon mot's `target` is a path under
   `day-haiku/` attributes the line `source_kind: "day-haiku"` with a label derived
   from the target; a `target` under `conversations-zine/` resolves to
   `source_kind: "conversations-zine"`; an unrecognized target resolves to
   `source_kind: "artifact"` with the raw target as `source_label` (no error).
2. The rendered Markdown credits each printed line `— <persona_display>, on
   <source_label>`; the JSON form carries an `attribution` object per line with
   `persona, persona_display, source_kind, source_label`.
3. `--masthead` (default) appends a masthead block listing each distinct
   contributing persona once with their line count; `--no-masthead` omits it. A
   persona placing two lines appears once with count 2.
4. Masthead ordering is deterministic: lines-placed descending, then persona name
   ascending; rendering the same fixture twice is byte-identical.
5. `conning-tower credits --format json` emits `{date, contributors: [{persona,
   persona_display, lines_placed, source_kinds: [...]}]}` and exits 0; on a date
   with no crowned record it exits 3 (same contract as `compose`).
6. Each AC has a matching integration test under `tests/acceptance_ac<n>.rs` using
   committed fixtures; the AC1 source-kind rule table is covered by a unit test
   enumerating each known prefix plus the unknown fallback.
