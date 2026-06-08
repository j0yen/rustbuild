# PRD: lattice-context — task-relevant subgraphs as on-demand context

Status: Draft v0.1
build_target: rust-cli
Vision: visions/lattice.md

## TL;DR

This is the "give AIs infinite context" verb — stated honestly. `lattice-context`
turns a query or an entity into the *relevant subgraph* of the federated lattice,
serialized as structured context an AI can consume, and exposes it as an MCP tool
so any agent retrieves world-knowledge on demand instead of holding it in its
window. Effectively unbounded structured context — bounded by the lattice, not
the context window — complementary to `recall` (episodic memory).

## Why this exists

- **The seed's headline:** "Give AIs infinite context." The honest mechanism is
  retrieval: a 500-ontology federated graph is far larger than any context
  window, but an AI only ever needs the *region relevant to the current task*.
  context computes and serves that region. "Infinite" = the store is effectively
  unbounded and externalized; the window holds only the retrieved slice.
- **The primitives already exist after traverse.** `lattice-traverse` computes
  neighborhoods and paths; `ousia-sparql` answers structured queries. context is
  the retrieval policy + packaging + MCP surface over them — thin, not new
  reasoning.
- **It complements, not duplicates, recall.** Phase-1 research (2026-06-08):
  `recall` is episodic semantic memory (BGE embeddings + FTS5 over markdown
  notes). lattice-context is *structured ontological* retrieval (axioms, classes,
  bridges). Different substrate, different query; the vision keeps them separate
  surfaces. Mirrors `ousia-mcp`'s pattern of exposing a capability as an MCP tool.

## What this builds

A `cargo` Rust CLI + MCP server `lattice-context` at
`~/wintermute/lattice-context/`.

**Deps (pinned at build time):** `lattice-traverse` (lib), `ousia-sparql` (lib),
an MCP server crate (or hand-rolled stdio JSON-RPC, per ousia-mcp's PRD),
`serde_json`, `clap`. Optional embedding hook to `recall`'s embedder for
NL-query → seed-entity selection.

**Retrieval flow.**
1. Resolve the query to seed entities (exact IRI, label match, or — optional —
   embedding similarity via recall's BGE model).
2. Expand to the relevant subgraph via `lattice-traverse neighborhood` (k-hop,
   budget-bounded).
3. Rank + truncate to a token/triple budget (the "fit it in the window" step).
4. Serialize as structured context: classes, their axioms, bridge links, and
   short NL glosses (the World Ontology's `aiGuidance` annotations where
   present).

**UX.**
```
lattice-context get   --store lattice-store/ --query "tuberculosis welfare" --budget 4000
lattice-context get   --store ... --entity wo:Dignity --hops 2 --format json
lattice-context serve --store ... --owl world-ontology.owl     # MCP server
```

## Acceptance criteria

1. `get --query <text>` resolves the query to seed entities and returns a
   connected, relevant subgraph (classes + axioms + bridge links) as structured
   JSON/markdown.
2. Retrieval is budget-bounded: `--budget <N>` caps the output size; a query
   whose full neighborhood exceeds the budget returns a ranked, truncated subgraph
   and **logs that it truncated** (no silent drop — the result states how much was
   omitted).
3. `get --entity <IRI>` returns that entity's k-hop neighborhood including
   cross-ontology bridge links, suitable for direct injection into an AI prompt.
4. `serve` runs an MCP server exposing at least `context_get(query|entity,
   budget)` returning the same payload as the CLI (CLI/MCP parity test, mirroring
   ousia-mcp AC-2/3).
5. Output includes NL glosses where the source ontology provides them (e.g. the
   World Ontology `aiGuidance`/`philosophicalGrounding` annotations) so the
   context is human- and AI-readable, not bare IRIs.
6. The retrieval is read-only and bounded in latency: a query against a
   multi-ontology fixture store returns within a stated budget without scanning
   the whole graph (seed-then-expand, not full-store scan).
7. Clear separation from recall: context queries the ontological store only; a
   README section documents when to use lattice-context (structured world-
   knowledge) vs recall (episodic memory).
8. `cargo test` covers: query→seed→subgraph on a fixture; budget truncation with
   the omission report; entity neighborhood retrieval; CLI/MCP parity; and gloss
   inclusion.
