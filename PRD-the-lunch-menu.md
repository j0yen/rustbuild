# PRD: the-lunch-menu — set the agenda: order, discussion, time-box

**Status:** Draft v0.1
**build_target:** rust-cli
**build_into:** (new repo) `~/wintermute/the-lunch`
**Vision:** visions/the-lunch.md

## TL;DR

A convened, seated table still has no order of business — nothing says which
dish gets discussed first, which gets skipped, or how long the table spends on
each before the critique round runs away with the day (and, downstream, the
`--lavish` API budget). This PRD builds the **menu**: the agenda that orders the
dishes, marks what is discussed, and time-boxes the gathering, so
`vicious-circle`'s critique round runs against a bounded, deterministic plan.

## Why this exists

`visions/roundtable.md` and `visions/the-lunch.md` describe a lunch that
*convenes, seats, then discusses* — but a discussion with no agenda is either
unbounded (every dish gets the full ensemble, cost and minutes balloon) or
arbitrary (whatever order the adapters happened to run). The Round Table had a
host and an implicit order of business; this crate makes that explicit and
testable. It sits third in the vision's order: it reads the `Table` (convene)
and the `Seating` (seating) and emits a `Menu` that `the-lunch-minutes` records
against and `vicious-circle` consumes as its run plan. Time-boxing is the
mechanism that keeps the once-daily ritual — and any per-lunch API spend
decided in `bon-mot` — legible and bounded.

## What this builds

Extends `~/wintermute/the-lunch`. New module `menu.rs` in the `the_lunch`
library plus a `menu` CLI subcommand.

Types:
- `Course { dish_ref: DishRef, discussed: bool, time_box: Duration,
  lead: PersonaId, note: Option<String> }` — one agenda item.
  `DishRef` identifies a dish by `(kind, source)` (stable across reloads).
- `Menu { date: NaiveDate, courses: Vec<Course>, total_box: Duration,
  host: PersonaId }`.

Logic (`menu.rs`):
- `compose(table: &Table, seating: &Seating, cfg: &MenuConfig) -> Menu`:
  - Orders present dishes by a configurable priority over `DishKind`
    (default: `Letter` > `SelfPortraitDiff` > `Haiku` > `ZineExcerpt` >
    `AmbientCue` — narrative weight first, ambient last).
  - `discussed = true` only when at least one seated persona `draws` that dish
    kind; otherwise the course is plated but `discussed = false` (it's on the
    table, no one bites).
  - `lead` = a seated persona drawn by that dish (the host if the host qualifies,
    else the first drawing persona in fixed order).
  - `time_box` per course from `MenuConfig` (default 90s), scaled down so the
    sum of *discussed* courses never exceeds `total_box`
    (default 600s) — proportional shrink, floor 30s.
  - `MenuConfig` loadable from `$XDG_CONFIG_HOME/the-lunch/menu.toml`
    (priority order, default/total boxes) with a built-in default.
- Empty table (no present dishes / empty seating from the seating PRD): a
  `Menu` with zero courses and `total_box = 0`, host carried from seating.

CLI:
- `the-lunch menu [--date YYYY-MM-DD] [--json]` — load today's table + seating
  from the store, compose, persist to
  `$XDG_STATE_HOME/the-lunch/<date>/menu.json`, print the agenda (or `--json`).
  Errors clearly if seating hasn't been produced yet for the date.

Deps: reuses convene/seating deps; `chrono::Duration` for time-boxes.

## Acceptance criteria

1. `cargo build` / `cargo test` green on toolchain 1.85.
2. `compose` orders courses by the default `DishKind` priority (`Letter` first,
   `AmbientCue` last); test asserts the emitted `courses` order.
3. A dish whose kind no seated persona `draws` is plated with
   `discussed == false`; a dish a seated persona draws is `discussed == true`.
4. The sum of `time_box` over **discussed** courses is `<= total_box`; when raw
   defaults would exceed it, boxes shrink proportionally with a 30s floor
   (test with enough courses to force the shrink).
5. Each discussed course's `lead` is a persona that is both seated and draws
   that dish kind; the host is preferred when eligible (test).
6. A `menu.toml` override changing the priority order changes the course
   ordering accordingly (fixture `$XDG_CONFIG_HOME`).
7. Empty table/seating → `Menu` with no courses and `total_box == 0`, host
   preserved from the seating; no panic.
8. `the-lunch menu` errors with a clear message when `<date>/seating.json` is
   absent (run before `seat`).
9. `the-lunch menu --json` emits a `Menu` that round-trips through serde and is
   persisted to `<date>/menu.json`; human form prints courses in order with
   lead, time-box, and discussed/skipped marker.
