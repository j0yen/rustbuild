# PRD: ousia-sparql — query the ethical structure

Status: Draft v0.1
build_priority: high
build_target: rust-cli
Vision: visions/ousia.md

## TL;DR

Once `ousia-reason` has materialized the entailments, the ontology becomes a
queryable knowledge graph. `ousia-sparql` loads the ontology + inferred triples
into an `oxigraph` store and exposes SPARQL 1.1 query — as a CLI, an embeddable
library, and (optionally) a read-only HTTP endpoint — with a curated pack of
canned queries over the ethical structure ("what bears dignity?", "which
processes violate rights?", "which authority roles lack accountability?").

## Why this exists

- **The paper names querying as a core capability.** §1: ontologies "support…
  query answering that is sound and complete." SPARQL is the standard surface;
  none of the fleet exposes it yet.
- **guard and atscale both need a query layer.** Rather than each re-implement
  graph traversal, they consume one embeddable SPARQL lib. This PRD is the
  shared substrate (the dependency the vision's order diagram funnels through).
- **The demo lives here.** §9 (Applications) lists policy/governance and
  education uses that are fundamentally queries. A canned query pack turns the
  abstract paper into a clickable "ask the ontology a moral question" demo —
  the seed's "market success" needs a demo.

## What this builds

A `cargo` Rust workspace `ousia-sparql` at `~/wintermute/ousia-sparql/`: a lib
crate (`ousia_sparql`) + a thin CLI binary.

**Deps (pinned at build time):**
- `oxigraph` — in-memory + on-disk RDF store with SPARQL 1.1 query.
- `ousia-reason` (lib) — to materialize inferences before loading.
- `clap`, `serde_json` (JSON result serialization).

**UX.**
```
ousia-sparql load  --owl world-ontology.owl --abox facts.ttl   # build store (materialized)
ousia-sparql query --store store/ -q 'SELECT ?b WHERE { ?b a :SentientBeing }'
ousia-sparql ask   --store store/ --canned dignity-bearers     # named query from the pack
ousia-sparql serve --store store/ --read-only --port 7070      # optional HTTP SPARQL endpoint
```

**Canned query pack** (`queries/*.rq`, each documented): `dignity-bearers`,
`rights-violations`, `unaccountable-authority`, `just-societies`,
`unjust-systems`, `flourishing-prerequisites`, `agents-not-sentient`. Each maps
to a paper section so the pack doubles as executable documentation of §5–§6.

## Acceptance criteria

1. `load` materializes inferences via ousia-reason, then ingests ontology +
   inferred triples into an oxigraph store; `query` returns correct results
   against it.
2. The `dignity-bearers` canned query returns every individual the reasoner
   classified as bearing Dignity — i.e. it reflects the materialized
   sentience→dignity chain, not just asserted triples.
3. The `unaccountable-authority` canned query returns empty on a consistent
   ABox (because reason materializes Accountability) and is the basis of a
   governance check — paper §9.3.
4. `query` accepts arbitrary SPARQL 1.1 SELECT/ASK/CONSTRUCT and emits results
   as JSON (SPARQL results JSON) and as a table for TTY.
5. `ask --canned <name>` runs a named query from the pack and lists available
   names with `ask --list`.
6. The embeddable lib exposes a stable `Store::load(owl, abox) -> Store` and
   `Store::query(&str) -> Results` API documented for guard/atscale to consume.
7. `serve --read-only` rejects SPARQL UPDATE and only answers query; refuses to
   start without `--read-only` in this version (no write surface yet).
8. `cargo test` covers: load+query round-trip on a fixture ABox; the
   `dignity-bearers` and `rights-violations` canned queries return expected rows;
   a malformed SPARQL string returns a clean error, not a panic.
