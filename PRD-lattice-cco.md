# PRD: lattice-cco — ingest the Common Core Ontologies mid-layer

Status: Draft v0.1
build_target: shell
Vision: visions/lattice.md
Depends-on: lattice-seed (base store must exist)

## TL;DR

The Common Core Ontologies (CCO) v2.0 is a BFO-grounded, DoD/IEEE-evaluated
mid-layer of 11 ontologies that sits between BFO's ~35 upper classes and fully
domain-specific ontologies. For an AI ops agent, CCO's **Agent Ontology**
(agents, roles, organizations), **Information Entity Ontology** (information
artifacts, records, plans), **Event Ontology** (processes, tasks, actions),
**Time Ontology** (temporal regions), and **Extended Relation Ontology** (richer
cross-domain relations) are directly relevant — providing formal class hierarchies
for exactly the entities this box's agents manipulate. This PRD drives the relevant
CCO subset into the lattice federated store.

## Why this exists

Research (2026-06-16) on the BFO ecosystem shows CCO as the strongest candidate
mid-layer for a general AI agent (not biomedical):
- 11 Turtle modules at `github.com/CommonCoreOntology/CommonCoreOntologies/src/cco-modules/`
- Confirmed BFO 2020-grounded (README: "extends from Basic Formal Ontology (BFO), ISO-standard")
- DoD/Intelligence Community directive as baseline standard; IEEE PAR3195.1 under review
- Key modules for AI agent use: AgentOntology (107KB), InformationEntityOntology (164KB),
  EventOntology (209KB), TimeOntology (43KB), ExtendedRelationOntology (44KB) — total ~567KB
- License: **BSD 3.1** — permissive, requires attribution, no copyleft restrictions
  on use or derivative works
- v2.0 breaking change: opaque IRI namespace
  (`https://www.commoncoreontologies.org/ont00001234`); mapping file available at
  `src/cco-modules/documentation/mapping-new-iris/`

## What this builds

A shell script `scripts/lattice-cco-ingest.sh` (installable to
`~/.local/bin/lattice-cco-ingest`) that:

**Phase 1 — fetch CCO modules**:
```sh
BASE="https://raw.githubusercontent.com/CommonCoreOntology/CommonCoreOntologies/master/src/cco-modules"
for module in AgentOntology InformationEntityOntology EventOntology TimeOntology ExtendedRelationOntology; do
  curl -sL "$BASE/${module}.ttl" -o ~/.cache/lattice/cco/${module}.ttl
done
```
Skip: ArtifactOntology (heavy, overlaps IAO/SWO), QualityOntology, GeospatialOntology,
FacilityOntology, UnitsOfMeasureOntology, CurrencyUnitOntology (domain-specific).

**Phase 2 — add to registry**:
```sh
for f in ~/.cache/lattice/cco/*.ttl; do
  lattice-registry add "file://$f"   # or the raw GitHub URL if add supports URLs
done
```
Log each module's class_count and bfo_grounded status. If bfo_grounded=false for any
module, log a WARNING but do not abort — CCO may not use `owl:imports bfo.owl` explicitly
and the detection heuristic may misfire; the BFO-grounding is confirmed by README and
the BFO structure of its class hierarchy.

**Phase 3 — bridge CCO modules into the existing store**:
```sh
# CCO Agent Ontology bridges to IAO (agents produce information artifacts)
lattice-bridge align --a ~/.cache/lattice/cco/AgentOntology.ttl \
                      --b $(lattice-registry path iao) \
                      --out ~/.cache/lattice/bridges/cco-agent-iao.owl \
                      --proposals ~/.cache/lattice/bridges/cco-agent-iao.proposals.jsonl
# CCO Information Entity Ontology vs IAO — expected high overlap / confirmed equivalences
lattice-bridge align --a ~/.cache/lattice/cco/InformationEntityOntology.ttl \
                      --b $(lattice-registry path iao) \
                      --out ~/.cache/lattice/bridges/cco-info-iao.owl \
                      --proposals ~/.cache/lattice/bridges/cco-info-iao.proposals.jsonl
# CCO Event Ontology bridges to BFO Process
lattice-bridge align --a ~/.cache/lattice/cco/EventOntology.ttl \
                      --b $(lattice-registry path bfo) \
                      --out ~/.cache/lattice/bridges/cco-event-bfo.owl \
                      --proposals ~/.cache/lattice/bridges/cco-event-bfo.proposals.jsonl
```

**Phase 4 — incremental join**:
```sh
for module in AgentOntology InformationEntityOntology EventOntology TimeOntology ExtendedRelationOntology; do
  lattice-join add --store ~/.cache/lattice/store/ \
    ~/.cache/lattice/cco/${module}.ttl \
    --allow-unreviewed
done
# add the CCO bridges
for bridge in ~/.cache/lattice/bridges/cco-*.owl; do
  lattice-join add --store ~/.cache/lattice/store/ "$bridge" --allow-unreviewed
done
```

**Phase 5 — validation queries**:
```sh
lattice-join stats ~/.cache/lattice/store/   # triple count should jump significantly
lattice-ground resolve "agent"               # should hit CCO Agent Ontology class
lattice-ground resolve "plan"                # should hit CCO Event Ontology / IAO
lattice-ground resolve "role"                # should hit CCO Agent + BFO Role
```

## Acceptance criteria

1. All 5 CCO modules are cached at `~/.cache/lattice/cco/*.ttl`.
2. `lattice-join stats` shows ≥ 5000 triples after CCO ingest (combining the seed store).
3. `lattice-ground resolve "agent"` returns ≥1 result with an IRI in the CCO namespace
   (`https://www.commoncoreontologies.org/`).
4. `lattice-ground resolve "information artifact"` returns ≥1 result bridged across
   IAO and CCO InformationEntityOntology (bridge axioms visible in the results).
5. `lattice-ground resolve "process"` returns ≥1 result with BFO_0000015 (process)
   in the subsumption chain.
6. A `~/.cache/lattice/cco/ATTRIBUTION.md` file is written noting: CCO v2.0,
   `github.com/CommonCoreOntology/CommonCoreOntologies`, BSD 3.1 license — satisfying
   the license attribution requirement.
7. Script is idempotent.
