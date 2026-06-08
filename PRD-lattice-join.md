# PRD: lattice-join — merge ontologies + bridges into one federated graph

Status: Draft v0.1
build_priority: high
build_target: rust-cli
Vision: visions/lattice.md

## TL;DR

`lattice-join` takes the World Ontology, N domain ontologies, and the bridge
axioms that connect them (from `lattice-bridge`) and merges them into a single
federated, deduplicated, queryable graph — the scaled-up ousia-sparql store that
`lattice-traverse` and `lattice-context` query. It is the "join" verb of the
seed: many isolated ontologies become one navigable world.

## Why this exists

- **Bridges are inert until the ontologies share a store.** `lattice-bridge`
  emits `EquivalentClasses`/`SubClassOf` axioms across ontology boundaries, but
  those axioms only *do* anything when both source ontologies and the bridge are
  loaded together and reasoned over. join is what makes a bridge live.
- **ousia-sparql already loads one ontology; this generalizes it.** PRD-ousia-
  sparql (present on disk, 2026-06-08) loads ontology + inferences into an
  `oxigraph` store. join extends that to *many* ontologies + bridge axioms,
  resolving `owl:imports`, deduplicating shared BFO classes (every BFO-grounded
  ontology re-imports the same 35 BFO categories — naive concatenation would
  duplicate them), and re-running ousia-reason so cross-ontology entailments
  materialize.
- **The federated graph is the substrate for "infinite context."** Everything
  downstream (traverse, context, ground) queries this one store; without a clean
  join there is no world to traverse.

## What this builds

A `cargo` Rust CLI `lattice-join` at `~/wintermute/lattice-join/`, building a
persistent federated store.

**Deps (pinned at build time):** `oxigraph` (the federated store),
`ousia-sparql` (lib — load/query), `ousia-reason` (lib — materialize cross-
ontology entailments), `lattice-bridge` (lib — bridge axiom types), `clap`.

**UX.**
```
lattice-join build  --base world-ontology.owl \
                    --add disease.owl --add fibo.owl \
                    --bridge disease.bridge.owl --bridge fibo.bridge.owl \
                    --store lattice-store/
lattice-join add    --store lattice-store/ --ontology env.owl --bridge env.bridge.owl
lattice-join stats  --store lattice-store/      # ontologies, classes, bridges, dedup count
```

## Acceptance criteria

1. `join build` loads a base + ≥1 domain ontology + their bridge axioms into one
   oxigraph store and reports success with counts.
2. Shared BFO classes are **deduplicated**: the 35 BFO upper categories appear
   once in the federated store regardless of how many member ontologies import
   them (verified by a count assertion).
3. After join, a cross-ontology entailment materializes: given a bridge
   `domain:Foo ≡ wo:Bar` and a fact `:x a domain:Foo`, the store answers `:x a
   wo:Bar` — proving bridges are live, not inert text.
4. `join add` incrementally adds an ontology + bridge to an existing store
   without rebuilding from scratch, and is idempotent (re-adding the same
   ontology does not duplicate triples).
5. The federated store is queryable by `ousia-sparql query --store
   lattice-store/` — i.e. join produces a store the existing sparql tooling
   reads unchanged.
6. `join stats` reports: number of member ontologies, total named classes,
   bridge-axiom count, and dedup count (BFO + other shared classes collapsed).
7. A join that includes a low-confidence/unreviewed bridge is **refused** (or
   gated behind an explicit `--allow-unreviewed`): only review-accepted
   `bridge.owl` axioms enter by default — enforcing the vision's no-silent-merge
   rule.
8. `cargo test` covers: a two-ontology + bridge build; BFO dedup; the cross-
   ontology entailment; idempotent `add`; and the unreviewed-bridge refusal.
