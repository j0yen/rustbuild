# PRD: recourse-amend — an upheld field contest becomes a held-out corpus case

Status: Draft v0.1
build_priority: high
build_target: mixed
build_into: /home/jsy/wintermute/recourse
Vision: visions/recourse.md

## TL;DR

A contest that a human upholds is a verified statement that the engine got a
real-world case wrong. That is gold — it is exactly the held-out test the axioms'
authors could not have written. This PRD turns an upheld contest into a
**tribunal-corpus case** in the precise shape `tribunal` already consumes, then
**re-runs `tribunal corpus validate` + `tribunal gate`** so the new world-case is
proven against the engine before anything ships. It is the edge that closes the
field's disagreement back onto the answer key.

## Why this exists

- The whole `recourse` cycle exists to let the world write held-out cases (vision
  recourse §"Why"); `amend` is where a field contest actually *enters* the proof
  layer.
- **The independence guarantee falls out for free.** PRD-tribunal-corpus requires
  every case carry `provenance.author != "ousia-axioms"`. A field contest's author
  is a downstream human — so an amended case satisfies the mechanical half of
  tribunal's independence check **by construction**. This is the vision's central
  hinge; it must be enforced in an AC, not assumed.
- The corpus wire contract is fully specified (PRD-tribunal-corpus): `action.json`
  (ousia-guard ABox) + `expected.toml` (`{verdict, rule, tenet, rationale}`) +
  `provenance.toml` (`{source, source_ref, author, spot_checked_by}`). `amend`
  emits exactly this — no schema negotiation.

## What this builds

Adds the `amend` subcommand to the `recourse` crate (mixed: Rust CLI + a
documented shell-out to the `tribunal` binary, stubbed for tests).

- **`recourse amend <contest-id>`** — requires the contest to be in `upheld.ndjson`
  (refuses `pending`/`rejected`); requires `--store-raw` to have captured the
  action (or a `--action <file>` override), since the corpus needs the real ABox,
  not just its digest. Emits a corpus case directory
  `<tribunal-corpus>/cases/field-<contest-id>/`:
  - `action.json` — the canonical action (from the local raw store / `--action`).
  - `expected.toml` — `verdict` = the contest's upheld `claimed_verdict`; `rule`/
    `tenet` carried from the disputed receipt or set by the reviewer; `rationale` =
    the contest reason (one line).
  - `provenance.toml` — `source = "field-contest"`, `source_ref =
    "receipt:<receipt_id> contest:<contest_id>"`, `author = "downstream:<contestant>"`
    (guaranteed `!= "ousia-axioms"`), `spot_checked_by = "<reviewer>"`.
- After emitting, runs `tribunal corpus validate` then `tribunal gate` against the
  amended corpus and reports pass/fail. In tests these are a **recorded stub** with
  the documented CLI contract asserted (cross-vision: real `tribunal` may be
  unbuilt; `amend` must build and test without it).
- `--dry-run` writes the case to a temp dir and prints it without touching the real
  corpus or invoking tribunal.

**Deps:** shares the `recourse` lib; `serde`, `toml`, `serde_json`, `clap`. SIGPIPE
reset. rustc 1.85, no let-chains.

## Acceptance criteria

1. `recourse amend <upheld-contest-id>` produces `cases/field-<id>/` with all three
   files; each parses under tribunal-corpus's checked-in schemas (action JSON
   schema + the two TOML shapes).
2. **Independence AC (the hinge):** the emitted `provenance.toml` has
   `author` starting `downstream:` and **never** equal to `"ousia-axioms"`, and a
   non-empty `source = "field-contest"`. A test asserts both — this is what makes
   the case a valid held-out entry.
3. `expected.toml.verdict` equals the contest's upheld `claimed_verdict` (the
   field's corrected answer), **not** the receipt's observed verdict.
4. `amend` refuses a contest that is not in `upheld.ndjson` (pending or rejected →
   non-zero, no files written).
5. `amend` refuses when no raw action is available and no `--action` is given (the
   corpus needs the ABox) — clear error, non-zero, no partial case dir.
6. After emitting, `amend` invokes `tribunal corpus validate` then `tribunal gate`
   in that order against the amended corpus; a test with the recorded tribunal-stub
   asserts both are called with the amended corpus path and that a stub `gate`
   failure makes `amend` exit non-zero.
7. `--dry-run` touches neither the real corpus dir nor the tribunal binary; output
   shows the three rendered files.
8. SIGPIPE-safe; `cargo test` green (against the tribunal stub); `amend --help`
   documents `--dry-run`, `--action`.
