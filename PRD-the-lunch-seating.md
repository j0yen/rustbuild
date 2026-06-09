# PRD: the-lunch-seating — seat the right personas for what's on the table

**Status:** Draft v0.1
**build_target:** rust-cli
**build_into:** (new repo) `~/wintermute/the-lunch`
**Vision:** visions/the-lunch.md

## TL;DR

A convened table (from `the-lunch-convene`) is just a pile of dishes; the Round
Table's voices were not interchangeable — Ferber answered narrative, Parker
answered the epigram. There is no logic that decides *which personas attend a
given lunch* based on *what is actually on the table*. This PRD builds that:
declarative seating rules keyed on dish kinds that produce a `Seating` the
menu and the critique ensemble downstream consume.

## Why this exists

`visions/roundtable.md` describes the end-state where "a circle of personas,
each with a distinct critical voice modeled on a real member, reviews them" —
but a circle that always seats everyone for everything is noise. A self-portrait
day and a letter day should not draw the same voices. The convene PRD produces
a typed `Table` of `Dish`es (`Haiku`, `ZineExcerpt`, `Letter`,
`SelfPortraitDiff`, `AmbientCue`); seating turns "what's on the table" into
"who's at the table" so the downstream critique round (`vicious-circle`) and
the agenda (`the-lunch-menu`) operate on a focused, justified roster rather
than a fixed list. This is the second link in the vision's order: it consumes
the convene crate and is consumed by menu and minutes.

## What this builds

Extends the `~/wintermute/the-lunch` repo. New module `seating.rs` in the
`the_lunch` library plus a `seat` CLI subcommand on the existing binary.

Types:
- `Persona { id: PersonaId, name: String, after: String,
  draws: Vec<DishKind>, voice: &'static str }` — the real-member basis:
  `Parker` (acid epigram; draws `Haiku`, `ZineExcerpt`),
  `Ferber` (narrative realism; draws `Letter`, `ZineExcerpt`),
  `Woollcott` (grandiose enthusiasm; draws `SelfPortraitDiff`, `AmbientCue`),
  `Benchley` (gentle absurdism; draws `AmbientCue`, `Haiku`),
  `Kaufman` (structural eye; draws any dish, capped — see rule 4).
- `Seat { persona: PersonaId, drawn_by: Vec<DishKind>, reason: String }`.
- `Seating { date: NaiveDate, seats: Vec<Seat>, host: PersonaId }`.

Logic (`seating.rs`):
- A declarative `SeatingRules` table mapping `DishKind -> [PersonaId]`
  (the `draws` relation above), loadable from a default plus an optional
  `$XDG_CONFIG_HOME/the-lunch/seating.toml` override.
- `decide(table: &Table, rules: &SeatingRules) -> Seating`:
  - A persona is seated iff at least one **present** dish (not an empty seat)
    matches its `draws`. Empty-seat dishes do not draw anyone.
  - `drawn_by` records which dish kinds pulled each persona; `reason` is a
    human string ("Parker drawn by today's haiku").
  - `Kaufman` (the structural eye) is seated only when ≥2 distinct present dish
    kinds are on the table (he convenes when there's structure to critique).
  - `host` is the persona drawn by the most dishes (ties broken by a fixed
    persona order); the host owns agenda framing downstream.
- `seating::EMPTY` rule: if the table has zero present dishes, return a
  `Seating` with no seats and `host = Kaufman` (a quiet table still has a chair
  to note the absence) — exercised by minutes downstream.

CLI:
- `the-lunch seat [--date YYYY-MM-DD] [--json]` — load today's table (from the
  convene store), decide seating, persist to
  `$XDG_STATE_HOME/the-lunch/<date>/seating.json`, print roster (or `--json`).

Deps: reuses convene crate's deps (`serde`, `chrono`, `anyhow`, `clap`); reads
the table via `store::load`.

## Acceptance criteria

1. `cargo build` / `cargo test` green on toolchain 1.85.
2. A table with only a `Haiku` dish seats `Parker` (and `Benchley`, who draws
   `Haiku`) but **not** `Ferber`; asserted by `decide`.
3. A table with a `Letter` dish seats `Ferber`; a table with a
   `SelfPortraitDiff` dish seats `Woollcott`.
4. `Kaufman` is seated only when ≥2 distinct present dish kinds are present:
   one-kind table → no Kaufman seat; two-kind table → Kaufman seated.
5. Empty-seat dishes (`present == false`) draw no personas: a table whose only
   dish is an empty seat yields an empty roster with `host == Kaufman`.
6. `drawn_by` and `reason` are populated for every seat and name the actual
   triggering dish kind(s); verified in tests.
7. `host` is the persona drawn by the most present dishes, deterministic under
   ties (fixed order test).
8. A `seating.toml` override that adds a `DishKind -> Persona` mapping changes
   the roster accordingly (test points `$XDG_CONFIG_HOME` at a fixture).
9. `the-lunch seat --json` emits a `Seating` that round-trips through serde and
   is persisted to `<date>/seating.json`; the human form lists each seat with
   its reason.
