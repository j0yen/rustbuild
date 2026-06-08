# PRD: lattice-traverse — walk the federated world graph

Status: Draft v0.1
build_target: rust-cli
Vision: visions/lattice.md

## TL;DR

A federated graph is only useful if you can move through it. `lattice-traverse`
provides the "traverse" verb: shortest semantic path between two concepts, the
full axiom neighborhood of any entity, and subsumption walks — all crossing
ontology boundaries via the bridge axioms `lattice-join` materialized. It is the
navigation primitive `lattice-context` and `lattice-ground` build on.

## Why this exists

- **The seed names it directly:** "tools to find, join, **traverse** and deeply
  understand." join produces the world; traverse is how an AI moves through it.
- **Bridges make cross-domain paths possible — but only if something walks
  them.** After join, a path can run `disease:Tuberculosis → (bridge) →
  wo:DiseaseProcess → wo:undermines → wo:WelfareCondition → wo:Flourishing`,
  connecting a biomedical fact to the World Ontology's ethical structure (the
  §9.2 healthcare example, made walkable). No tool computes such a path today.
- **Neighborhood retrieval is the unit of "context."** `lattice-context` needs
  "the relevant region around entity X"; traverse defines and computes that
  region (k-hop axiom neighborhood). Building it as a clean primitive keeps
  context thin.

## What this builds

A `cargo` Rust CLI `lattice-traverse` at `~/wintermute/lattice-traverse/`, with
a reusable lib (consumed by lattice-context + lattice-ground).

**Deps (pinned at build time):** `oxigraph` (query the federated store via
`ousia-sparql` lib), a graph algorithms crate (e.g. `petgraph`) for
pathfinding, `clap`, `serde_json`.

**Operations.**
- **path** — shortest/ranked semantic path between two entities across the
  federated graph, edges = object properties + bridge axioms, with each hop
  labeled by the property/axiom traversed.
- **neighborhood** — the k-hop axiom neighborhood of an entity (its classes,
  properties, the axioms it participates in, bridge-linked equivalents).
- **subsumers / subsumed** — walk up/down the is_a + bridged subsumption
  lattice, crossing ontology boundaries.

**UX.**
```
lattice-traverse path --store lattice-store/ --from disease:Tuberculosis --to wo:Flourishing
lattice-traverse neighborhood --store ... --entity wo:Dignity --hops 2 --format json
lattice-traverse subsumers --store ... --entity disease:InfectiousDisease
```

## Acceptance criteria

1. `path --from A --to B` returns a connected path across the federated graph
   with each hop labeled by the object property or bridge axiom traversed, or a
   clear "no path" result — non-zero exit only on error, not on legitimate
   no-path.
2. A path that must cross a bridge does so: a fixture with `domain:Foo ≡ wo:Bar`
   yields a path from a `domain:Foo`-region entity to a `wo:Bar`-region entity
   that includes the bridge hop, labeled as a bridge.
3. `neighborhood --entity X --hops k` returns exactly the entities/axioms within
   k hops of X (verified against a fixture with known structure), as JSON
   suitable for `lattice-context` to package.
4. `subsumers`/`subsumed` walk the combined asserted+bridged subsumption lattice
   and include cross-ontology superclasses introduced by bridge axioms.
5. Traversal is bounded and predictable: `--hops`/path-length limits are honored
   and a large/cyclic graph does not hang (cycle detection; a default max).
6. All operations read the `lattice-join` store through the `ousia-sparql` lib
   (no separate graph copy) and emit machine-readable JSON.
7. `cargo test` covers: a known shortest path; a bridge-crossing path; k-hop
   neighborhood exactness; subsumer walk across a bridge; and the cycle/limit
   guard (no hang on a cyclic fixture).
