# Vision: ousia — operationalizing the BFO-grounded World Ontology

*οὐσία (ousia)* — Aristotle's "substance/being," the category BFO's
ontological realism descends from. This vision turns a paper into a product.

## TL;DR

Joe has written a complete, serious paper — *The World Ontology: A
BFO-Grounded Formal Ontology for AI World-Understanding*
(`~/Notes/AtScale/World-Ontology-paper.md`, v1.0.0) — describing an OWL 2 DL
ontology of 509 classes that encodes ethical commitments (sentience →
dignity, authority → accountability, error → learning) as reasoner-enforced
axioms rather than annotations. **But the ontology itself does not exist as a
file on disk — only the paper describing it does.** And there is no tooling to
build it, reason over it, query it, or put it to work.

`ousia` is the fleet of Rust tools that makes the paper real and adoptable:
forge the `.owl` from a declarative spec, run the entailments that turn
sentience into dignity, answer SPARQL over the ethical structure, gate agent
actions against it, expose it over MCP, and bridge it into the AtScale
semantic-layer market where Joe's "ontological semantic layer" thesis already
lives (`~/Notes/AtScale/book-outline-ontological-semantic-layer.md`).

The seed (jsy, 2026-06-08): *"of making ethical AI possible with these ideas.
implement OWL2 and SPARQL tools to make ethical grounded BFO a market success."*

## End-state

When this vision is fulfilled:

1. `world-ontology.owl` (OWL 2 DL / RDF/XML, BFO 2020 import) is **generated
   reproducibly** from a human-auditable declarative source, not hand-edited
   XML. Re-running the forge yields a byte-stable artifact.
2. A reasoner materializes the paper's signature entailment chain —
   `bears Sentience → SentientBeing → bears Dignity` — and the other nine
   axioms, and reports OWL 2 DL profile conformance + consistency.
3. Any agent can ask, in SPARQL or over MCP, "what bears dignity here?",
   "does this authority structure carry accountability?", "what rights does
   this process violate?" — and get a sound, deductive answer with the axiom
   chain that justifies it.
4. An AI agent about to act can call `ousia-guard` with a description of the
   proposed action and receive `allow | flag | deny` **with the formal
   justification** — alignment as deductive inference, the paper's §9.1 thesis
   made executable.
5. The whole thing is packaged as an AtScale-flavored value proposition:
   BFO-grounded semantics as a sellable property of a semantic layer, demoable
   against the live AtScale catalog.

## Components (PRD-sized)

1. **ousia-forge** (`PRD-ousia-forge`) — author the ontology from a declarative
   TOML/YAML spec; emit OWL 2 DL / RDF/XML via `horned-owl`; vendor the BFO 2020
   import. *Foundational — closes the "no .owl on disk" gap.*
2. **ousia-reason** (`PRD-ousia-reason`) — load the OWL, validate the OWL 2 DL
   profile, forward-chain the paper's 10 axioms (EL/RL rule materialization),
   emit inferred triples + a consistency/conformance verdict. Depends on forge.
3. **ousia-sparql** (`PRD-ousia-sparql`) — load ontology + materialized
   inferences into an `oxigraph` store; SPARQL 1.1 query (CLI + embeddable lib)
   with a canned ethical-structure query pack. Depends on reason.
4. **ousia-guard** (`PRD-ousia-guard`) — the ethical-AI keystone. Given a
   proposed action as RDF/JSON, deduce whether it violates a Right, pushes a
   SentientBeing below the Dignity floor, or grants AuthorityRole without
   Accountability. Returns verdict + justifying axiom chain. Depends on sparql.
5. **ousia-mcp** (`PRD-ousia-mcp`) — expose reason/sparql/guard as MCP tools so
   any agent consumes the ontology as a connector. Distribution vector. Depends
   on guard.
6. **ousia-atscale** (`PRD-ousia-atscale`) — the market bridge. Map BFO
   categories onto AtScale semantic-layer model concepts; annotate/emit AtScale
   model definitions with BFO grounding + provenance, driven against the live
   AtScale MCP. Interactive-facing. Depends on forge + sparql.

## Order

```
forge ──► reason ──► sparql ──► guard ──► mcp
   └────────────────────┴──► atscale
```

forge is the gate for everything. reason → sparql → guard is the reasoning
spine. mcp and atscale are the two distribution/market endpoints and can be
built in either order once guard/sparql land.

## Open questions

- **The Federation Model source.** The paper draws its ten tenets from a
  "Federation Model" document that is **not on disk**. forge needs the axiom
  source. Option A: extract the 10 axioms directly from the paper's §3.1/§5
  (sufficient — the paper states each axiom formally). Option B: ask jsy for the
  Federation Model doc. Starting with A; B can enrich annotations later.
- **Reasoner choice.** The 10 axioms are all-some SubClassOf / equivalence
  restrictions — squarely OWL 2 EL/RL, materializable by forward chaining. Pure
  Rust full-DL reasoners are thin; plan is a hand-rolled rule engine over the
  specific axiom patterns (small, tractable, fully testable against §8.3's
  table) with `whelk-rs` as an upgrade path if it resolves at build time.
  A HermiT (Java) shell-out is the escape hatch for full DL but adds a JVM dep —
  avoid unless conformance demands it.
- **Conformance suite.** The paper's §8.1–8.3 give concrete, testable claims
  (single inheritance on all 509 classes, OWL 2 DL profile, the 10-axiom →
  encoding table). A dedicated `ousia-conformance` PRD may be worth splitting
  out of reason's ACs once reason exists. Deferred to a later /dream pass.
- **Is this "for myself" or a product?** These build as standalone `j0yen/<repo>`
  per the dream/autobuilder convention, but the AtScale market framing means
  ousia may eventually want its own org/home. Revisit at fulfillment.
