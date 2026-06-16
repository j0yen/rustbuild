# PRD: doxa-reason — evaluate a scenario within a single chosen framework

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/doxa
Vision: visions/doxa.md
Depends-on: doxa-frameworks (needs built per-framework ontologies)

## TL;DR

With framework ontologies built (doxa-frameworks), this PRD makes them *do* something:
`doxa reason <framework> --scenario <abox>` loads a scenario (an Action with its
intention, consequences, agents, and the maxim it instantiates) as an ABox, materializes
the chosen framework's axioms over it via `ousia-reason`, and reports whether the action
is a `RightAction` / `WrongAction` / `PermissibleAction` *under that framework*, with the
justifying axiom chain.

## Why this exists

Verified 2026-06-16: `ousia-reason` at `~/.local/bin/ousia-reason` does OWL 2 DL
entailment — `classify` ("materialize inferences over an ABox and emit them as Turtle"),
`check` (consistency), `explain` ("ordered justification chain for inferences about an
entity"). doxa-frameworks produces the TBoxes; ousia-reason is exactly the engine to
run an ABox against one. Without this PRD, the frameworks are inert ontologies; with it,
they answer the question a philosophy is *for* — "is this act right?".

## What this builds

Extend `~/wintermute/doxa`:

- **`doxa reason <framework> --scenario <abox.ttl> [--explain]`**:
  1. Resolve the framework's built OWL (build on demand via the doxa-frameworks path if
     not cached).
  2. Combine the framework TBox with the scenario ABox.
  3. Run `ousia-reason classify` to materialize entailments; check whether the
     scenario's action individual is entailed to be a `RightAction` / `WrongAction` /
     `PermissibleAction`.
  4. With `--explain`, call `ousia-reason explain <action-iri>` to print the ordered
     axiom chain that produced the verdict.
- **A worked example scenario** shipped under `scenarios/trolley.ttl` (or similar): an
  action with consequences (lives saved/lost), an intention, and the maxim it
  instantiates — chosen so the three frameworks plausibly *differ* on it (the classic
  case where consequentialism and deontology diverge). This doubles as a test fixture.
- **Output**: `<framework>: <action> is <RightAction|WrongAction|PermissibleAction|undetermined>`
  plus, with `--explain`, the chain. `undetermined` is a first-class result (the
  framework's axioms don't decide this scenario) — never fabricate a verdict.
- Reuse `ousia-reason`; do not re-implement reasoning. Resolve it from `$PATH` or
  `--reasoner`; actionable error if absent.

MSRV: match ousia-reason if a shared lib is used; else 1.85 with a subprocess call.

## Acceptance criteria

1. `cargo test --release` passes; `cargo clippy -- -D warnings` passes.
2. `doxa reason consequentialism --scenario scenarios/trolley.ttl` exits 0 and prints a
   verdict of `RightAction`, `WrongAction`, `PermissibleAction`, or `undetermined` for
   the scenario's action (skip live reasoning if `ousia-reason` absent, but unit-test
   the verdict-classification logic on a recorded `ousia-reason` output).
3. `doxa reason deontology --scenario scenarios/trolley.ttl` runs and produces a
   verdict for the same action.
4. The shipped trolley scenario yields **different** verdicts under at least two
   frameworks (a test asserts consequentialism ≠ deontology on it) — proving the
   modules genuinely diverge.
5. `--explain` prints a non-empty justification chain for a decided verdict.
6. An `undetermined` verdict is returned (not an error, not a fabricated verdict) when
   the framework's axioms don't entail any of the three evaluative classes for the action.
7. Malformed scenario ABox → actionable error, non-zero exit.
