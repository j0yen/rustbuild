# PRD: lattice-ground-live — prove end-to-end grounding + context retrieval over the seeded store

Status: Draft v0.1
build_target: shell
Vision: visions/lattice.md
Depends-on: lattice-cco (full store with CCO mid-layer must exist)

## TL;DR

Once `lattice-seed` and `lattice-cco` produce the federated store, the "infinite
context" claim needs to be proved in practice: `lattice-ground resolve` and
`lattice-context get` must return useful, bridge-connected results for the entities
an AI ops agent actually works with. This PRD ships a smoke harness + integration
tests that confirm end-to-end grounding and expose where the results are thin or
missing, so the next /dream pass can target real gaps rather than assumed ones.

## Why this exists

The lattice vision's end-state says "a natural-language mention is grounded to its
best ontological class across the whole lattice, with its axiom + bridge
neighborhood." Without a live test over real ontologies this is an aspiration, not
a measurement. Live research (2026-06-16) confirmed the tools ship and the parse
bug is the one concrete blocker. Once the store exists, grounding needs verification
over the entities that actually matter: software tools, information artifacts, agents,
sessions, processes, plans, and software versions.

## What this builds

A shell harness `scripts/lattice-ground-harness.sh` (installable to
`~/.local/bin/lattice-ground-harness`) that:

**Grounding smoke suite** (each assertion is a test):
```sh
# From SWO — should resolve to software entity class
lattice-ground resolve "software tool"          # SWO_0000001 or equivalent
lattice-ground resolve "software version"       # SWO version class
lattice-ground resolve "software license"       # SWO license class
# From IAO / CCO InformationEntity — should bridge-resolve
lattice-ground resolve "information artifact"   # IAO_0000030
lattice-ground resolve "document"               # IAO or CCO term
lattice-ground resolve "plan"                   # IAO:plan / OBI:plan specification
lattice-ground resolve "specification"          # IAO term
# From CCO Agent Ontology
lattice-ground resolve "agent"                  # CCO agent class
lattice-ground resolve "role"                   # BFO_0000023 + CCO role subclasses
lattice-ground resolve "organization"           # CCO organization class
# From BFO via process / event
lattice-ground resolve "process"                # BFO_0000015
lattice-ground resolve "event"                  # CCO event or BFO process subclass
# Cross-bridge — tests that bridge axioms are actually traversed
lattice-ground link <SWO:software_tool_IRI>     # should return BFO path via bridge
```

**Subsumption walk** (tests lattice-traverse over the seeded store):
```sh
lattice-traverse subsumers <SWO:software_tool_IRI>  # walks up to BFO:continuant
lattice-traverse subsumed  <BFO:process_IRI>        # walks down through CCO events
lattice-traverse path --from <SWO:software> --to <IAO:information_artifact>
```

**Subgraph context retrieval** (tests lattice-context):
```sh
lattice-context get "software tool"          # returns structured subgraph
lattice-context get "agent"                  # agent ontology neighborhood
lattice-context get "information artifact"   # IAO/CCO neighborhood with bridge axioms
```

The harness runs all tests and emits a structured report:
```
lattice-ground-harness results:
  grounding:   14/17 passed (3 no-result — see below)
  traversal:   3/3 passed
  context:     3/3 passed
No-result terms (add to proposals):
  - "software version": SWO has SwO:0000030 but bridge to BFO not yet in store
  - ...
```
Exit 0 if ≥80% pass; exit 1 otherwise (some gaps are expected and become the input to
the next /dream pass).

## Acceptance criteria

1. `lattice-ground-harness.sh` runs to completion.
2. `lattice-ground resolve "software tool"` returns ≥1 result with an IRI traceable
   to SWO (prefix `http://www.ebi.ac.uk/swo/`).
3. `lattice-ground resolve "information artifact"` returns ≥1 result with an IRI
   traceable to IAO (`http://purl.obolibrary.org/obo/IAO_`) or CCO.
4. `lattice-ground resolve "agent"` returns ≥1 result traceable to CCO Agent Ontology
   or BFO.
5. `lattice-traverse subsumers <SWO:software_tool>` prints a chain ending at
   `BFO_0000001` (entity) — confirming the cross-bridge subsumption chain is intact.
6. `lattice-context get "agent"` returns a non-empty subgraph (JSON or Turtle).
7. The harness report names any no-result terms explicitly — no silent failures.
8. Overall pass rate ≥ 80% of the 17 grounding tests.
