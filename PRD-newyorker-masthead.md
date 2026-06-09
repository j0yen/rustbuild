# PRD: newyorker-masthead — the publication identity: title, persona masthead, issue folio

**Status:** Draft v0.1
**build_target:** rust-lib
**build_into:** (new repo) `~/wintermute/new-yorker`
**Vision:** visions/new-yorker.md

## TL;DR

`roundtable` produces a *daily* column of crowned bon mots attributed to five
persona contributors (PRD `conning-tower-column`). But there is no stable
*publication identity* binding those columns: no title, no masthead listing who
contributed, no issue number, no date, no volume. Without a canonical identity
layer, every downstream binder (`newyorker-issue`, `newyorker-cover`) would
re-invent its own ad-hoc header and the issues would not even agree on what
number they are. This PRD ships the one stable thing everything else stamps: the
masthead and the folio.

## Why this exists

`~/wintermute/conversations-zine/` is the closest prior art — a periodical
(the quarterly zine) — yet its `zine` binary stops at moment-extraction and
leaves layout/identity human-driven; there is no machine-readable masthead. The
umbrella `visions/roundtable.md` explicitly calls for `new-yorker` to have "a
masthead" and `visions/new-yorker.md` makes masthead the first component
everyone depends on. `~/wintermute/daily-receipt/` establishes the precedent
that an issue must embed its ISO date verbatim and deterministically (its AC7):
the folio here follows that contract. The persona roster (Parker, Benchley,
Woollcott, Kaufman, Ferber) is defined upstream in `visions/vicious-circle.md`;
this lib is the canonical place their *bylines* live for print.

## What this builds

A library crate `newyorker-masthead` (+ a thin `masthead` CLI for inspection),
in the new repo `~/wintermute/new-yorker` (workspace member, so `newyorker-issue`
and `newyorker-cover` depend on it path-locally).

Modules:
- `identity` — the publication constants: title (`"The Round Table"` by
  default, overridable), tagline ("not for the old lady in Dubuque"), founding
  date. Loaded from an optional `masthead.toml`, else built-in defaults.
- `roster` — the contributor roster: `Contributor { handle, display_name,
  byline, persona_id }`. `persona_id` keys to the upstream vicious-circle
  persona registry. Serde-(de)serializable.
- `folio` — issue numbering/dating: `Folio { issue_number: u32, volume: u32,
  date: NaiveDate, season: Option<String> }`. `Folio::next(prev, date)` derives
  a monotonic successor deterministically (no clock read inside the type — date
  is always passed in, mirroring daily-receipt's no-RNG/no-clock rule). Renders
  to a canonical folio string: `"Vol. 1, No. 7 — 2026-06-08"`.
- `render` — `masthead_block(&Identity, &Roster, &Folio) -> String`, the
  Markdown masthead header every component prepends. Pure, deterministic.

Deps: `serde` + `serde_derive`, `toml`, `chrono` (NaiveDate only), `thiserror`.
No clock, no RNG, no network.

CLI subcommands (thin wrapper over the lib, for humans/tests):
- `masthead show [--config masthead.toml]` — prints the rendered masthead block.
- `masthead folio --prev <N> --date <YYYY-MM-DD>` — prints the next folio string.
- `masthead roster [--config masthead.toml]` — lists contributors as `handle —
  display_name — byline`.

## Acceptance criteria

1. `masthead show` with no config prints a masthead block containing the default
   title `The Round Table` and the tagline `not for the old lady in Dubuque`,
   and exits 0.
2. `masthead show --config <f>` where `<f>` overrides the title prints the
   overridden title (built-in defaults are overridable, not hard-coded).
3. `Folio::next` is deterministic and monotonic: `next(prev=6, date=2026-06-08)`
   yields `issue_number == 7`; calling it twice with identical inputs yields
   byte-identical folio strings (no clock, no RNG inside the type).
4. The rendered folio string for `{issue_number:7, volume:1, date:2026-06-08}`
   contains the literal ASCII substring `2026-06-08` verbatim (daily-receipt
   AC7 contract).
5. `masthead roster` with the default roster lists exactly the five
   vicious-circle personas (Parker, Benchley, Woollcott, Kaufman, Ferber), each
   with a non-empty byline; a malformed `masthead.toml` (e.g. missing `handle`)
   exits non-zero with a `thiserror` message naming the bad field, not a panic.
6. `masthead_block` is a pure function: same `(Identity, Roster, Folio)` →
   byte-identical String across two calls (covered by a unit test).
7. `cargo test` passes and `cargo build --release` produces both the
   `newyorker-masthead` lib and the `masthead` binary.
