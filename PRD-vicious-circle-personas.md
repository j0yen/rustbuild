# PRD: vicious-circle-personas — the critic-persona registry

**Status:** Draft v0.1
**build_target:** rust-cli
**build_into:** (new repo) `~/wintermute/vicious-circle`
**Vision:** visions/vicious-circle.md

## TL;DR

The roundtable needs a fixed cast of critic voices before anything can be
critiqued. Right now there is no registry — no definition of who Dorothy Parker
*is* as a critical stance, what tone she carries, or how her verdict should read
differently from Robert Benchley's. This PRD builds the persona registry: a
crate and CLI that load a cast of Round Table critics from a TOML file, each
with a real-era bio, a critical stance, and a signature verbal tic, so every
downstream PRD (`review`, `roast`, `crown`, `ledger`) shares one source of
truth for who is at the table.

## Why this exists

This laptop's creative wing is six write-only soloists — `~/wintermute/day-haiku`
(writes a haiku, archives it, moves on), `~/wintermute/conversations-zine`,
`~/wintermute/letters-we-never-sent`, `~/wintermute/self-portrait`,
`~/wintermute/ambient`, `~/wintermute/wintermute-music`. The umbrella vision
`visions/roundtable.md` observes these are all output with no answering voice.
Before any of them can be *critiqued*, we need the critics defined as data.

The existing `~/wintermute/concord` crate already models persona tone: its
`concord-deescalate/src/` carries `lexicon.rs`, `prompt.rs`, and `types.rs` for
voice profiles. This PRD's persona stance fields are designed to map onto a
concord tone profile later (a `concord_profile` key), so the voices are real
tone, not bare labels.

## What this builds

A new repo `~/wintermute/vicious-circle` with a library crate `vicious_circle`
and a binary `vicious-circle`.

Modules:
- `src/persona.rs` — `Persona { id, name, era_bio, stance, tic, concord_profile }`
  where `stance` is an enum (`AcidEpigram`, `GentleAbsurdist`,
  `GrandioseEnthusiast`, `StructuralEye`, `NarrativeRealist`) and `tic` is the
  signature verbal mannerism string.
- `src/registry.rs` — `Registry::load(path) -> Result<Registry>` parsing a TOML
  `personas.toml`; `get(id)`, `iter()`, `len()`. Default registry path resolves
  to `$XDG_CONFIG_HOME/vicious-circle/personas.toml`, falling back to a
  baked-in default cast embedded via `include_str!` so the tool works with zero
  config.
- `src/lib.rs` — re-exports `Persona`, `Stance`, `Registry`.
- `src/main.rs` — clap CLI.

Deps: `clap` (derive), `serde` + `toml`, `anyhow`, `directories` (XDG paths).
First line of `main()` calls `sigpipe::reset()` (per the laptop's known
println-SIGPIPE issue) — add the `sigpipe` crate.

Baked-in default `personas.toml` ships all five: Dorothy Parker / AcidEpigram /
tic "the velvet stiletto"; Robert Benchley / GentleAbsurdist / tic "the
bewildered aside"; Alexander Woollcott / GrandioseEnthusiast / tic "the
superlative cascade"; George S. Kaufman / StructuralEye / tic "the second-act
question"; Edna Ferber / NarrativeRealist / tic "the working-life detail".

CLI subcommands:
- `vicious-circle personas` — list each persona: id, name, stance, tic (table).
- `vicious-circle personas --json` — emit the registry as JSON.
- `vicious-circle personas show <id>` — print one persona's full record
  including era bio.

## Acceptance criteria

1. `cargo build` and `cargo test` succeed in `~/wintermute/vicious-circle`.
2. `vicious-circle personas` with no config file present lists exactly the five
   default personas (Parker, Benchley, Woollcott, Kaufman, Ferber) with their
   stance and tic columns.
3. `vicious-circle personas --json` emits valid JSON parsing back to a 5-element
   array; a unit test round-trips the embedded default through serde.
4. `vicious-circle personas show parker` prints Dorothy Parker's era bio,
   `AcidEpigram` stance, and the "velvet stiletto" tic.
5. `Registry::load` reads an external `personas.toml` when present and a test
   confirms a custom 1-persona file overrides the baked-in default.
6. `Persona` carries a `concord_profile` field (string, may be empty) reserved
   for mapping onto a `~/wintermute/concord` tone profile; a test asserts it is
   present and serde-(de)serializes.
7. `main()` calls `sigpipe::reset()` first; `vicious-circle personas | head -1`
   does not panic.
