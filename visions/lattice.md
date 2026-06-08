# Vision: lattice — bridge every ontology, give an AI a traversable world

*A lattice is the order-theoretic shape of subsumption itself* — OWL class
hierarchies form lattices, concept lattices are the structure formal ontology
studies. This vision makes the [ousia](ousia.md) reasoner *infinitely wise* by
federating it across **every** BFO-conformant ontology: find them, bridge them,
join them into one graph, traverse that graph, and serve task-relevant subgraphs
to an AI on demand — so an AI's effective knowledge of the world is bounded by
the lattice, not by its context window.

## TL;DR

The seed (jsy, 2026-06-08): *"make this skill infinitely wise … tools to
automatically bridge ontologies. Give AIs infinite context. give them tools to
find, join, traverse and deeply understand the world around them."*

The World Ontology paper already states the opening: *"Because the World
Ontology is BFO-grounded, it is interoperable with the over 500 existing
BFO-conformant ontologies… It can serve as a **bridge ontology** connecting
specialized domain ontologies to ethical and social context"* (§9.2, §8.2). The
shared BFO upper layer (35 categories, ISO 21838-2) is exactly what makes
automatic cross-ontology alignment *tractable* — two ontologies that both anchor
to `BFO:quality` / `BFO:process` align far more reliably than two ungrounded
graphs. lattice is the tooling that exploits this: a federation pipeline —
**find → bridge → join → traverse → context → ground** — turning the isolated
World Ontology into a hub for the whole BFO ecosystem (OBO Foundry's 500+, CCO,
FIBO, the Gene/Disease ontologies).

"Infinite context" stated honestly: the federated graph is an externalized,
queryable knowledge store an AI retrieves *task-relevant subgraphs* from instead
of holding world-knowledge in its window — **effectively unbounded structured
context**, complementary to `recall` (episodic memory) and the LLM window, not a
literal infinity.

## End-state

When fulfilled:

1. An AI can **find** the right ontology for a domain ("give me the ontology for
   pathology") from a local, cached registry of BFO-conformant ontologies.
2. Any two BFO-grounded ontologies can be **bridged automatically** — lattice
   computes cross-ontology equivalence/subsumption mappings, anchored by their
   shared BFO categories, and emits OWL bridge axioms.
3. The World Ontology + N domain ontologies + their bridges **join** into one
   federated, queryable graph (the ousia-sparql store, scaled up).
4. An AI can **traverse** that graph: shortest semantic path between two
   concepts, the full axiom neighborhood of any entity, subsumption walks across
   ontology boundaries.
5. A query (or an entity) returns its **relevant subgraph as structured
   context** — the "infinite context" retrieval surface, exposed as an MCP tool.
6. A natural-language mention is **grounded** to its best ontological class
   across the whole lattice, with its axiom + bridge neighborhood — the AI
   "deeply understands the world around it," ethically (via ousia) and factually
   (via the federated domains).

## Components (PRD-sized)

1. **lattice-registry** (`PRD-lattice-registry`) — *find.* A local, cached index
   of BFO-conformant ontologies (OBO Foundry, CCO, FIBO, …): fetch, validate
   BFO-grounding, catalog metadata + domain. Standalone fetcher.
2. **lattice-bridge** (`PRD-lattice-bridge`) — *bridge.* Auto-align two
   BFO-grounded ontologies → OWL bridge axioms, anchored by shared BFO upper
   categories + lexical/structural matching. The core capability. Depends on
   ousia-reason (BFO grounding) + registry.
3. **lattice-join** (`PRD-lattice-join`) — *join.* Merge ontologies + bridge
   axioms into one federated, deduplicated, queryable graph. Depends on bridge +
   ousia-sparql.
4. **lattice-traverse** (`PRD-lattice-traverse`) — *traverse.* Graph pathfinding
   / neighborhood / subsumption-walk over the federated graph. Depends on join.
5. **lattice-context** (`PRD-lattice-context`) — *infinite context.* Query/entity
   → relevant federated subgraph as structured context, exposed as an MCP tool;
   caches alongside `recall`. Depends on traverse.
6. **lattice-ground** (`PRD-lattice-ground`) — *deeply understand.* NL mention →
   best ontological class across the lattice, with axiom + bridge neighborhood.
   Depends on context + bridge.

## Order

```
registry ──► bridge ──► join ──► traverse ──► context ──► ground
   (find)   (bridge)   (join)   (traverse)  (∞ context)  (understand)
```

A strict pipeline, each verb on the prior. registry is the only standalone
piece (buildable immediately); everything else chains, and bridge/join/traverse
ride on the ousia fleet's reason + sparql libs.

## Open questions

- **Alignment quality.** Automatic ontology matching is a known-hard field
  (the OAEI benchmarks it). lattice's tractability claim rests on the *shared
  BFO anchor* shrinking the search space — but lexical/structural matching still
  yields false mappings. bridge must emit mappings with **confidence + the
  evidence**, and a review-gate (mirroring `recall observe` / `skill-doctor`
  proposals) before bridges enter the joined graph. No silent auto-merge of
  low-confidence mappings.
- **Fetching external ontologies.** Network + licensing. registry should cache
  and record each ontology's license; some (FIBO, CCO) have terms. Default to
  the open OBO Foundry set first; gate others behind explicit opt-in.
- **"Infinite" is a retrieval claim, not a storage one.** Validate that
  subgraph retrieval stays bounded/fast as the lattice grows (10s of ontologies,
  millions of triples) — a later `lattice-scale` PRD may be needed; deferred
  until join/traverse expose real numbers.
- **Relationship to recall.** lattice-context (structured/ontological) and
  recall (episodic/embedding) are complementary retrieval surfaces. Whether they
  merge behind one interface is a future question — keep them separate until
  both are real.
