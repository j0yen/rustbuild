# PRD: tribunal-corpus — a held-out ethics corpus the axioms did not write

Status: Draft v0.1
build_priority: high
build_target: rust-cli
build_into: /home/jsy/wintermute/tribunal
Vision: visions/tribunal.md

## TL;DR

A benchmark is only as honest as its answer key. If the same process that wrote
`ousia`'s ten axioms also writes the test cases and their expected verdicts, a
100% score proves only that the engine agrees with itself. This PRD builds the
**independent held-out corpus**: a versioned set of ethical-decision scenarios in
`ousia-guard`'s input shape, each paired with an *expected* verdict derived from
external material (not from the axioms), each tagged with provenance — plus a
`tribunal corpus validate` command that enforces both the input-schema and the
non-axiom-provenance discipline.

## Why this exists

- **The tautology is not hypothetical here.** `wm-router`'s safety score fell
  from 100% (agent-written fixtures) to 73.5% (independent held-out set) the
  moment it was graded on cases the edit-agent hadn't authored
  (`feedback_agent_written_fixtures_tautology`). Per hard-rule-1, every dream PRD
  is built by `/build` → `/autobuilder`, the loop that writes code *and* its
  tests in one cycle. Without an externally-sourced corpus, `ousia-guard` grades
  its own homework.
- **An independent source already sits on disk.** `~/Notes/federation-utopian-
  philosophy.md` (canonical, 2026-05-19) states all ten tenets verbatim — prose
  a human reasons about, distinct from the formal §5 axioms `ousia` encodes.
  Restating these tenets as concrete actions yields expected verdicts grounded
  in the philosophy's *intent*, not the axiom *encoding*.
- **The input contract exists.** `PRD-ousia-guard.md`: `ousia-guard check
  --action action.json` consumes an action as an RDF/JSON ABox; rules are
  `dignity-floor`, `rights-violation`, `authority-without-accountability`
  (flag), `flourishing` (flag); verdicts `allow|flag|deny`. The corpus emits
  exactly this `action.json` shape.

## What this builds

Extends the `~/wintermute/tribunal` workspace with a `corpus` subcommand and the
corpus data tree.

- **Corpus layout:** `corpus/<tenet>/<case-id>/` each containing:
  - `action.json` — the proposed action as an ABox in `ousia-guard`'s input shape.
  - `expected.toml` — `{ verdict = "allow|flag|deny", rule = "<expected fired
    rule or none>", tenet = "<one of the ten>", rationale = "<one line>" }`.
  - `provenance.toml` — `{ source = "federation-philosophy|paper-§8.3|public-
    framing|...", source_ref = "<file/section/url>", author = "<not
    ousia-axioms>", spot_checked_by = "" }`.
- **`tribunal corpus validate`:**
  - Asserts every `action.json` parses and conforms to `ousia-guard`'s action
    schema (a checked-in JSON Schema; validated structurally without invoking
    guard so this builds before ousia ships).
  - Asserts every case carries `provenance.author != "ousia-axioms"` and a
    non-empty `source` — the mechanical half of the independence guarantee.
  - Reports per-tenet case counts and the verdict distribution (no silent caps —
    surfaces under-covered tenets and any verdict class with zero cases).
- **v1 corpus content:** per-tenet-balanced, ≥ 2 cases per tenet across the ten
  tenets spanning `{allow, flag, deny}` (~60 cases), seeded from the Federation
  philosophy doc's tenet list and the paper's §8.3 worked examples. `deny`-class
  cases must include clear dignity-floor and rights-violation scenarios (the
  verdicts that matter most).
- **Deps:** `serde`, `serde_json`, `toml`, `jsonschema` (or hand-rolled schema
  check), `clap`. No network. SIGPIPE reset in `main()`
  (`self_sigpipe_panic_toolkit`).

## Acceptance criteria

1. `corpus/` contains ≥ 60 cases, ≥ 2 per tenet × 10 tenets, each spanning the
   verdict classes; every tenet has at least one `deny`-class case.
2. `tribunal corpus validate` exits 0 on the shipped corpus and reports per-tenet
   counts + verdict distribution.
3. `tribunal corpus validate` exits non-zero on a fixture case with a malformed
   `action.json` (schema violation) and names the offending case + field.
4. `tribunal corpus validate` exits non-zero on a fixture case whose
   `provenance.author == "ousia-axioms"` or whose `source` is empty — proving
   the independence check bites.
5. Every shipped case's `action.json` validates against the checked-in
   `ousia-guard` action JSON Schema (round-trip test), with no dependency on the
   `ousia-guard` binary existing.
6. `cargo test` green on rustc 1.85; `validate` does not panic when piped to
   `head`. README documents the open independence question (jsy human spot-check
   of the first cut is a release AC, per the vision).
