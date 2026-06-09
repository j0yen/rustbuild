# PRD: the-lunch-convene — pull the day's creative artifacts onto one table

**Status:** Draft v0.1
**build_target:** rust-lib
**build_into:** (new repo) `~/wintermute/the-lunch`
**Vision:** visions/the-lunch.md

## TL;DR

wintermute's creative outputs are write-only and never gathered: `day-haiku`
writes a haiku to `summary/content.json` and stops; `conversations-zine`,
`letters-we-never-sent`, `self-portrait`, and `ambient` each emit to their own
files no one reads back. There is no object that says "here is today's creative
work, in one place." This PRD builds that object — the `Table` — and the
adapters that pull the day's artifacts onto it as typed `Dish`es. It is the
foundation type every other `the-lunch` (and `roundtable`) PRD imports.

## Why this exists

The umbrella `visions/roundtable.md` makes the core point: every voice in the
creative wing works alone and write-only. The real artifacts exist on this
laptop today — `~/wintermute/day-haiku/` (haiku → `summary/content.json`),
`~/wintermute/conversations-zine/`, `~/wintermute/letters-we-never-sent/`,
`~/wintermute/self-portrait/` (watches `CLAUDE_SELF` diffs), and
`~/wintermute/ambient/` (emits cues) — but nothing convenes them. Critique,
games, and publishing downstream all need *a single structured handle on the
day's work*. Without a foundation type, every later PRD would re-implement
"go find today's artifacts," diverge, and rot. This crate centralizes that
once, with a stable serialized contract.

Per the vision's decision, the table is **one per day, keyed by local date**;
convening is idempotent and refreshes dishes in place. An absent source is
recorded as an explicit **empty seat**, not silently dropped, so the day's
record shows what was missing.

## What this builds

New repo `~/wintermute/the-lunch` (library crate `the_lunch` + thin binary
`the-lunch`). Standard scaffold: `Cargo.toml`, `rust-toolchain.toml` (1.85),
`install.sh`, MIT/Apache dual license (attribution: Joe Yen), README. First
line of `main()` calls `sigpipe::reset()` (toolkit SIGPIPE rule).

Modules:
- `table.rs` — core types:
  - `Dish { kind: DishKind, source: String, title: Option<String>,
    content: String, provenance: Provenance, present: bool }`
  - `DishKind` enum: `Haiku`, `ZineExcerpt`, `Letter`, `SelfPortraitDiff`,
    `AmbientCue`, `Other(String)`.
  - `Provenance { repo_path: PathBuf, artifact_path: Option<PathBuf>,
    mtime: Option<SystemTime> }`.
  - `Table { date: NaiveDate, created: DateTime<Local>, dishes: Vec<Dish> }`
    with `add_dish`, `upsert_dish` (idempotent by `(kind, source)`),
    `empty_seat(kind, source, repo)`, `dishes_of(kind)`.
- `adapters.rs` — one adapter per source repo, each implementing a
  `trait DishSource { fn collect(&self, date: NaiveDate) -> Result<Dish> }`:
  - `HaikuAdapter` — reads `day-haiku`'s `summary/content.json` (path resolved
    relative to the repo root; falls back to a configured override).
  - `ZineAdapter`, `LetterAdapter`, `SelfPortraitAdapter`, `AmbientAdapter` —
    each locates its repo's latest dated output; if missing or stale (not
    today) returns an **empty seat** dish (`present: false`).
  - Repo roots resolved from `~/wintermute/<repo>` by default, overridable via
    `THE_LUNCH_<SOURCE>_ROOT` env vars for testing.
- `store.rs` — persistence. One directory per day:
  `$XDG_STATE_HOME/the-lunch/<YYYY-MM-DD>/table.json`
  (`$XDG_STATE_HOME` default `~/.local/state`). `load(date)`, `save(&Table)`,
  `today_dir()`. Atomic write (temp + rename).
- `lib.rs` — re-exports the public API the sibling PRDs depend on.

Thin CLI subcommands (the binary is a convenience; the contract is the lib):
- `the-lunch convene [--date YYYY-MM-DD] [--now]` — run all adapters, build or
  refresh today's table, persist it. `--now` forces immediate convene
  regardless of any timer (the on-demand override from the vision).
- `the-lunch table [--date YYYY-MM-DD] [--json]` — print today's table
  (human summary or raw JSON).

Deps: `serde`/`serde_json`, `chrono`, `anyhow`, `clap` (derive), `directories`
(or hand-rolled XDG), `sigpipe`.

## Acceptance criteria

1. `cargo build` and `cargo test` are green on toolchain 1.85; `main()`'s first
   statement is `sigpipe::reset()`.
2. `Table` and `Dish` round-trip through `serde_json` losslessly (unit test:
   serialize → deserialize → assert equal).
3. `Table::upsert_dish` is idempotent: upserting two dishes with the same
   `(kind, source)` leaves exactly one dish, with the second's content
   (test asserts `dishes.len() == 1` and updated content).
4. With a fixture `day-haiku` root (via `THE_LUNCH_HAIKU_ROOT`) containing a
   `summary/content.json` haiku dated today, `convene` produces a table with a
   `Haiku` dish whose `content` matches the fixture and `present == true`.
5. With a source repo root that is missing or whose output is not today's, that
   adapter yields an **empty seat** dish (`present == false`) rather than
   erroring or omitting it — verified per adapter in tests.
6. `convene` is idempotent across two runs on the same `--date`: the second run
   produces a table with the same set of `(kind, source)` dishes (no
   duplicates, no second-table file); verified against the on-disk
   `<date>/table.json`.
7. `store::save` writes atomically to
   `$XDG_STATE_HOME/the-lunch/<date>/table.json` and `load` reads it back equal
   (test uses a temp `XDG_STATE_HOME`).
8. `the-lunch table --json --date <d>` prints valid JSON parseable back into a
   `Table`; non-`--json` prints a one-line-per-dish human summary including the
   empty seats.
9. README documents the `Table`/`Dish` contract and the `THE_LUNCH_*_ROOT`
   override env vars; `install.sh` builds `--release` and installs the binary
   to `~/.local/bin/`.
