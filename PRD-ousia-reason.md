# PRD: ousia-reason — materialize the ontology's ethical entailments

Status: Draft v0.1
build_target: rust-cli
Vision: visions/ousia.md

## TL;DR

The World Ontology's whole point is that ethical facts are **logical
entailments**, not annotations: assert that something bears Sentience and the
reasoner concludes it is a SentientBeing that bears Dignity (paper §5.1–5.2,
§9.1). `ousia-reason` loads the forged ontology, validates that it sits in the
OWL 2 DL profile, forward-chains the paper's ten foundational axioms over an
ABox, and emits the inferred triples plus a consistency/conformance verdict.

## Why this exists

- **The paper's thesis is executable inference.** §9.1: *"If the system
  determines [an entity] bears sentience, the reasoner automatically classifies
  it as a SentientBeing with inherent dignity… The moral inference is a logical
  entailment."* Without a reasoner, the ontology is an inert taxonomy; the
  entailment is the product.
- **forge produces only the TBox.** Per the paper §8.4, the ontology is TBox-only
  (no individuals). reason is what closes the loop: feed it an ABox (an
  individual asserted to bear Sentience) and it must derive SentientBeing +
  Dignity. Nothing else in the fleet does this.
- **Conformance needs a profile checker.** §8.1 claims OWL 2 DL compliance
  (no nominals in class expressions, no cardinality on transitive properties).
  reason should verify that mechanically so regressions in forge's spec are
  caught.

## What this builds

A `cargo` Rust CLI crate `ousia-reason` at `~/wintermute/ousia-reason/`, with a
reusable `ousia-reason` lib crate (consumed by ousia-sparql and ousia-guard).

**Deps (pinned at build time):**
- `horned-owl` — parse the OWL, walk axioms.
- `sophia` or `oxigraph`'s model — RDF triple representation for
  materialized output.
- `clap`, `serde`.

**Reasoner strategy.** The ten axioms (§8.3 table) are all-some SubClassOf and
equivalence restrictions — OWL 2 EL/RL territory, materializable by forward
chaining. Plan: a hand-rolled rule engine implementing exactly the entailment
patterns the paper uses:
- equivalence-class membership (`bearer_of some Sentience` ⇒ SentientBeing),
- SubClassOf propagation (SentientBeing ⊑ `bearer_of some Dignity`),
- the AuthorityRole ⇒ Accountability constraint (infer, or flag inconsistency
  if Accountability explicitly denied — §5.3, open-world).
`whelk-rs` (OWL EL reasoner) is an upgrade path if it resolves at build time;
a HermiT shell-out is the documented escape hatch for full DL but is **not** in
scope here (avoid the JVM dep).

**UX.**
```
ousia-reason classify --owl world-ontology.owl --abox facts.ttl   # materialize
ousia-reason check    --owl world-ontology.owl                    # DL profile + consistency
ousia-reason explain  --owl ... --abox ... --entity :Robot7       # axiom chain for an inference
```

## Acceptance criteria

1. Given the forged ontology and an ABox asserting an individual `:X bearer_of
   some Sentience` (or `:X` of a class entailing it), `classify` materializes
   `:X rdf:type :SentientBeing` **and** `:X bearer_of some :Dignity`.
2. `classify` materializes the AuthorityRole → Accountability entailment: an
   individual bearing an AuthorityRole gains the Accountability disposition (or,
   if Accountability is explicitly negated, `check` reports inconsistency) —
   paper §5.3.
3. `classify` materializes the remaining signature entailments testably: Person
   (SentientBeing ∧ PersonRole), Agent (CognitiveCapacity), Error ⊑ enables
   Learning, Flourishing's welfare requirement, InjusticeProcess ⊑ violates
   Right (§5.4–5.7, §6).
4. `check` validates the OWL 2 DL profile and reports any violation (nominal in
   a class expression, cardinality on a transitive property) with the offending
   axiom — paper §8.1.
5. `check` reports overall consistency: an ABox that forces a contradiction
   (e.g. AuthorityRole-bearer asserted `not bearer_of Accountability`) returns a
   non-zero exit and names the conflicting axioms.
6. `explain --entity :X` prints the ordered axiom chain that produced an
   inference (e.g. `:X bears Sentience → SentientBeing(:X) [eqv §6.1] → :X bears
   Dignity [SubClassOf §5.2]`). This justification text is what guard and the
   MCP layer surface to callers.
7. `classify` output is valid Turtle/RDF re-loadable by ousia-sparql.
8. `cargo test` covers the four signature chains (sentience→dignity,
   authority→accountability, error→learning, injustice→violates-right) on
   fixture ABoxes, plus one DL-profile-violation fixture and one
   inconsistency fixture.
