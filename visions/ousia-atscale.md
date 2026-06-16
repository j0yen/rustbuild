# Vision: ousia-atscale — make the formal semantic layer executable, not just demonstrable

**Authored by:** /dream (Claude Opus 4.8), with jsy
**Created:** 2026-06-16
**Status:** active
**Fleet 1 drafted:** 5 PRDs
**Seed:** jsy (2026-06-16) — `/dream for ousia-atscale`, the day the repo went
public at `github.com/j0yen/ousia-atscale` for colleagues to try.

## TL;DR

`ousia-atscale` today is a **demonstration**: it maps AtScale model elements onto
BFO 2020 categories (`ground`), emits an annotation overlay (`annotate`), and prints
coverage (`report`). It proves the thesis from *The Ontological Semantic Layer* —
that a semantic layer can be formally grounded. But the MARKET.md pitch makes
promises the tool cannot yet keep: *"is this the same revenue as last quarter?"*
has no command; the grounding is asserted as JSON but never **reasoner-checked**;
the overlay is JSON, not RDF, so it cannot be loaded into the very OWL/SPARQL
machinery (`ousia-reason`, `ousia-sparql`) that gives it teeth; and an AI agent
cannot query the grounding live. This vision turns the demonstration into a tool
colleagues can actually use — every claim in the pitch backed by a runnable command.

## End-state

When this vision is fulfilled:

1. **The grounding is reasoner-verified.** `ousia-atscale validate` exports the
   grounded model as RDF and runs `ousia-reason check` over it — so a coverage
   report is not just "100% mapped" but "100% mapped *and OWL 2 DL consistent*".
2. **The pitch is executable.** `ousia-atscale diff modelA.json modelB.json`
   answers the headline question — "is this the same revenue?" — by comparing the
   two models' BFO groundings element-by-element and reporting semantic agreement
   vs divergence.
3. **The overlay is real RDF.** `ousia-atscale export --format turtle` emits the
   grounding as triples that load directly into `ousia-sparql` (or any triplestore),
   joining the AtScale grounding to the broader World Ontology / lattice graph.
4. **Colleagues steer the mapping.** The `bfo_hint` override documented in the
   README actually works — a colleague who disagrees with a Quality-vs-Role call
   can pin it per column.
5. **An AI agent queries grounding live.** `ousia-atscale serve` exposes
   ground/diff/report over MCP, so a Claude session analysing a model *knows* what
   each measure formally is.

## Why this is distinct from the parent ousia vision

[ousia](ousia.md) operationalises the *World Ontology* (the ethical BFO ontology).
`ousia-atscale` is its **market-facing application** — the bridge into a real,
shipping product (AtScale's semantic layer). The parent vision builds the ontology
and reasons over ethics; this vision makes the BFO grounding *usable by data teams*
and connects an AtScale model's semantics to the same OWL/SPARQL substrate. It
reuses `ousia-reason` and `ousia-sparql` wholesale.

## Components (PRD-sized)

- **ousia-atscale-bfo-hint** (rust-extend) — implement the `bfo_hint` per-column
  override the README already documents but the mapper does not honor. Small honesty fix.
- **ousia-atscale-rdf** (rust-extend) — `export` subcommand: emit the grounding as
  RDF/Turtle (and OWL/XML) so it loads into `ousia-sparql` and any triplestore. The
  foundational bridge to the rest of the ousia substrate.
- **ousia-atscale-validate** (rust-extend) — `validate` subcommand: run the RDF
  export through `ousia-reason check` to prove the grounded model is OWL 2 DL
  consistent; fold the verdict into `report`. Depends on -rdf.
- **ousia-atscale-diff** (rust-extend) — `diff` subcommand comparing two models'
  groundings element-by-element: same name + same BFO category = agreement;
  same name + different category = a flagged semantic divergence. Makes the
  headline pitch ("is this the same revenue?") executable.
- **ousia-atscale-mcp** (rust-extend) — `serve` subcommand exposing
  ground/report/diff over MCP (reuse the `ousia-mcp` pattern), so a live AI agent
  can query a model's formal grounding mid-analysis.

## Order

```
ousia-atscale-bfo-hint    (standalone — small)
ousia-atscale-rdf         (standalone — foundational)
   └── ousia-atscale-validate   (needs the RDF export)
ousia-atscale-diff        (standalone)
ousia-atscale-mcp         (standalone; best after diff so it can serve diff too)
```

## Open questions (for the next /dream pass or the user)

- **Calculated / derived measures.** `avg_order_value = revenue / order_count` is a
  GDC about a *composed* process, not a primitive measure. The mapper treats every
  measure identically. A future `ousia-atscale-calc` PRD could ground calculation
  chains structurally. Deferred — needs a real AtScale model with derived measures
  to motivate the shape.
- **Batch mode.** Colleagues will have catalogs of many models; `ousia-atscale
  report --dir models/` with aggregate coverage is an obvious follow-on. Deferred
  until the single-model commands stabilise.
- **AtScale write-back.** MARKET.md notes integrating the overlay into the model
  as a first-class property is "pending AtScale API support." Out of scope until
  that API exists; keep the overlay non-mutating.
