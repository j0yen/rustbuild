# PRD: ousia-atscale-validate — prove the grounded model is OWL 2 DL consistent

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/ousia-atscale
Vision: visions/ousia-atscale.md
Depends-on: ousia-atscale-rdf (needs the RDF/OWL export)

## TL;DR

`ousia-atscale report` says "100% of elements mapped" — but "mapped" only means a
BFO category was *assigned*, not that the resulting grounding is *logically
consistent*. A grounding that, say, types the same individual as both a Quality and
a Process would be 100% "covered" and yet incoherent. This PRD adds a `validate`
subcommand that exports the grounded model to OWL (via `ousia-atscale-rdf`) and runs
`ousia-reason check` over it, reporting OWL 2 DL profile conformance + consistency —
upgrading the coverage claim from "mapped" to "mapped *and sound*".

## Why this exists

Verified 2026-06-16: `ousia-reason check` exists at `~/.local/bin/ousia-reason` and
"Validate[s] the OWL 2 DL profile and (with an ABox) report[s] consistency". The
ousia vision's whole premise is reasoner-enforced grounding; ousia-atscale currently
asserts grounding without ever invoking the reasoner. The MARKET.md pitch claims the
grounding is "machine-checkable … validated with OWL reasoners" — but no command does
the check. This closes that gap.

## What this builds

Extend `~/wintermute/ousia-atscale`:

- **New subcommand `validate`** in `src/main.rs`:
  ```
  ousia-atscale validate --model <m.json> [--reasoner <path-to-ousia-reason>]
  ```
  with the same `--from-mcp` path.
- Implementation:
  1. Ground the model and export it to OWL into a temp file (reuse the
     `ousia-atscale-rdf` export path — call the in-process function, do not shell to self).
  2. Locate `ousia-reason` (the `--reasoner` flag, else `$PATH`). If absent, exit
     with an actionable error ("ousia-reason not found; install the ousia fleet or
     pass --reasoner").
  3. Run `ousia-reason check <owl-file>`, capture stdout/stderr + exit code.
  4. Report a verdict: `consistent` (reasoner exit 0), `inconsistent` (reasoner
     reports inconsistency — surface the offending axioms), or `not-dl` (profile
     violation — surface which construct). Exit non-zero on inconsistent/not-dl.
- **Fold into `report`**: add a `--validate` flag to the existing `report` subcommand
  that appends a "Consistency: <verdict>" line to the coverage report, so a colleague
  gets coverage + soundness in one command.

No change to `ground`/`annotate`/`export` behavior. Reuses the RDF export from the
-rdf PRD; this PRD must not re-implement RDF emission.

## Acceptance criteria

1. `cargo test --release` passes; `cargo clippy -- -D warnings` passes.
2. `ousia-atscale validate --model fixtures/sales_model.json` exits 0 and prints
   `Consistency: consistent` when `ousia-reason` is on `$PATH`.
3. When `ousia-reason` is absent, `validate` exits non-zero with an actionable
   error message naming the `--reasoner` flag (test via a PATH-stripped invocation
   or a stubbed reasoner-locator).
4. `ousia-atscale report --model fixtures/sales_model.json --validate` prints both
   the coverage table and a `Consistency:` line.
5. A deliberately-inconsistent grounding fixture (an individual typed to two
   disjoint BFO classes, hand-authored as a test OWL file or via a `bfo_hint` that
   creates a contradiction) yields `inconsistent` and a non-zero exit (skip the
   live-reasoner portion if `ousia-reason` is absent, but still unit-test the
   verdict-parsing logic).
6. The validate path writes its temp OWL to a transient location and cleans it up
   (no litter in the cwd).
