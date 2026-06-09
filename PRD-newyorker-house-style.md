# PRD: newyorker-house-style — the house-style enforcer: lint & clean column prose to one urbane voice

**Status:** Draft v0.1
**build_target:** rust-cli
**build_into:** (new repo) `~/wintermute/new-yorker`
**Vision:** visions/new-yorker.md

## TL;DR

The `conning-tower` column (PRD `conning-tower-column`) attributes lines to five
distinct persona contributors — by design they sound different: Parker is acid,
Benchley meanders, Woollcott gushes. That polyphony is the point at the *lunch*.
But a *publication* has one editorial voice. Harold Ross's *New Yorker* was
famous for a precise, urbane, ruthlessly de-hedged house style ("not edited for
the old lady in Dubuque"). Binding five voices into an issue without flattening
them yields an incoherent magazine. This PRD ships the enforcer: a deterministic
lint/transform pass that flags and optionally cleans the clichés, hedges, and
sloppiness that violate the house standard.

## Why this exists

`~/wintermute/self-portrait/` and the `cadence` vision already establish that
wintermute does **house-voice / diff-narration** work — there is precedent on
this laptop for a crate that owns a writing voice. `~/wintermute/conversations-
zine/` curates moments for print but does no copy-editing; the editor (the
author) does that by hand, which `visions/new-yorker.md` names as the binding
bottleneck. The umbrella `visions/roundtable.md` cites Ross's standard verbatim
("not for the old lady in Dubuque") as the house-style requirement, and the
sub-vision's open questions decide this enforcer is **deterministic** (free,
reproducible, testable) with an `--lavish` API rewrite deferred. This PRD is
that deterministic pass.

## What this builds

A binary crate `newyorker-house-style` (binary `house-style`) in the
`~/wintermute/new-yorker` workspace.

Modules:
- `rules` — the rule set, data-driven from a built-in `rules.toml` (overridable
  via `--rules <f>`). Three rule kinds:
  - `Hedge` — flag hedges/qualifiers: `I think`, `sort of`, `kind of`,
    `arguably`, `it could be argued`, `perhaps`, `somewhat`, `a bit`, `just`
    (as filler), `very`/`really` (intensifier inflation).
  - `Cliche` — flag worn phrases: `at the end of the day`, `needless to say`,
    `last but not least`, `in this day and age`, `low-hanging fruit`.
  - `Sloppy` — flag mechanical sloppiness: double spaces, trailing whitespace,
    `  ` runs, repeated words (`the the`), and Oxford-comma absence in a 3+
    list (configurable; Ross was a comma zealot).
  Each rule carries `{ id, kind, pattern (literal or regex), replacement:
  Option<String>, severity }`.
- `lint` — `lint(text, &Rules) -> Vec<Finding>` where `Finding { rule_id, span,
  excerpt, severity, suggestion }`. Reports only; never mutates.
- `clean` — `clean(text, &Rules) -> String` applies every rule that has a
  `replacement` deterministically, left-to-right, non-overlapping, idempotent
  (running `clean` on already-clean text is a fixpoint). Rules without a
  replacement (judgment calls) are flagged by `lint` but never auto-applied.
- `report` — render findings as text (default) or `--format json`.

Deps: `regex`, `serde`/`serde_derive`, `toml`, `thiserror`, `clap`. No clock,
no RNG, no network.

CLI subcommands:
- `house-style lint <file|-> [--rules <f>] [--format text|json]` — print
  findings; exit code: 0 if clean, 1 if any `error`-severity finding (so it can
  gate `newyorker-issue`).
- `house-style clean <file|-> [--rules <f>] [--out <path>]` — emit the
  de-hedged, de-cliché'd, mechanically-tidied text; deterministic.
- `house-style rules [--rules <f>]` — list active rules (`id — kind —
  severity — pattern`).

## Acceptance criteria

1. `house-style lint` on text containing `I think it's, at the end of the day,
   sort of good` reports at least three findings (two hedges + one cliché) with
   correct `rule_id`s, and exits 1; on already-clean text it reports zero
   findings and exits 0.
2. `house-style clean` rewrites `I think it's good` → `it's good` and `The the
   table` → `The table` and collapses `a  b` (double space) → `a b`, writing the
   result to `--out` or stdout.
3. `clean` is **deterministic**: two runs on byte-identical input + rules
   produce byte-identical output (daily-receipt AC3 contract; no RNG, no map
   iteration order leaking).
4. `clean` is **idempotent**: `clean(clean(x)) == clean(x)` for every fixture in
   the test corpus (a fixpoint test over ≥10 inputs).
5. `--format json` emits valid JSON: an array of objects each with `rule_id`,
   `span` (`[start,end]` byte offsets), `severity`, and `suggestion`; spans are
   correct byte offsets into the input (verified by slicing the input at the
   span and matching the reported excerpt).
6. Rules are data-driven: a custom `--rules <f>` adding one new `Cliche` pattern
   causes `lint` to flag that pattern, with no recompile; a malformed rules file
   exits non-zero with a `thiserror` message, not a panic.
7. A `lint` finding without a `replacement` (a flagged judgment call) is
   reported by `lint` but left untouched by `clean` — verified by a fixture
   whose only issue is replacement-less, where `clean` is a no-op but `lint`
   still reports it.
8. `cargo test` passes and `cargo build --release` produces the `house-style`
   binary.
