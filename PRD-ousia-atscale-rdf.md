# PRD: ousia-atscale-rdf — export the grounding as real RDF, not just JSON

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/ousia-atscale
Vision: visions/ousia-atscale.md

## TL;DR

`ousia-atscale annotate` emits the grounding as a JSON overlay. That overlay is
human-readable but it is **not RDF** — it cannot be loaded into `ousia-sparql`,
queried with SPARQL, or checked by `ousia-reason`. The grounding is therefore
isolated from the very OWL/SPARQL substrate that gives BFO its power. This PRD adds
an `export` subcommand that emits the grounded model as RDF (Turtle and OWL/XML),
with each model element as an individual typed to its BFO class and annotated with
the `philosophicalGrounding` / `domainModule` / `aristotelianDefinition` vocabulary
as RDF properties.

## Why this exists

Verified 2026-06-16: `src/annotate.rs` emits `GroundedOverlay` as JSON; the sibling
tools `ousia-sparql` (SPARQL 1.1 over oxigraph) and `ousia-reason` (OWL 2 DL) are
installed at `~/.local/bin/` and consume RDF/OWL, not JSON. The MARKET.md pitch says
"the overlay [is] ready for OWL import" — but there is no command that produces OWL.
This is the missing bridge between the AtScale grounding and the rest of ousia.

## What this builds

Extend `~/wintermute/ousia-atscale`:

- **New subcommand `export`** in `src/main.rs`:
  ```
  ousia-atscale export --model <m.json> --format turtle|owl [--out grounded.ttl]
  ```
  with the same `--from-mcp` path as the other subcommands.
- **New module `src/rdf.rs`**: render the grounded model as RDF triples:
  - Mint an IRI per model element under a model-scoped namespace
    (e.g. `https://atscale.example/<catalog>/<schema>/<table>#<element>`).
  - Each element individual: `rdf:type` its BFO class IRI
    (e.g. `obo:BFO_0000031` for a measure GDC).
  - Attach annotation properties using the paper vocabulary IRIs:
    `philosophicalGrounding`, `domainModule`, `aristotelianDefinition`
    (reuse the IRIs already defined in `src/annotate.rs` so JSON and RDF agree).
  - Declare the model itself as an individual (catalog/schema/table) and relate
    each element to it (e.g. `obo:BFO_0000050` part-of, or a model-membership property).
  - Emit `owl:imports` of BFO (`http://purl.obolibrary.org/obo/bfo.owl`) in the OWL
    output so a reasoner can resolve the BFO classes.
- Use a Turtle/RDF serialization crate already in the ousia workspace if one is
  shared (check `ousia-sparql`/`ousia-reason` Cargo.toml for `oxrdf`/`oxttl`/
  `oxigraph`); otherwise hand-emit Turtle (it is simple, line-oriented text) and
  gate OWL/XML behind the same emitter. Prefer reusing the ousia crate stack for
  byte-compatibility with `ousia-sparql load`.

No change to existing `ground`/`annotate`/`report` behavior. MSRV: match whatever
the RDF crate requires (ousia-reason's MSRV if reused).

## Acceptance criteria

1. `cargo test --release` passes; `cargo clippy -- -D warnings` passes.
2. `ousia-atscale export --model fixtures/sales_model.json --format turtle --out /tmp/s.ttl`
   writes a non-empty Turtle file.
3. The Turtle output contains one typed individual per model element (13 for the
   sales fixture), each with an `rdf:type` of a `BFO_` class IRI.
4. The Turtle parses cleanly — verified by loading it with `ousia-sparql load`
   (skip with a logged note if `ousia-sparql` is not on `$PATH`).
5. `ousia-atscale export --format owl` emits valid OWL/XML that includes an
   `owl:imports` of `http://purl.obolibrary.org/obo/bfo.owl`.
6. A SPARQL query over the loaded Turtle returns the measures:
   `SELECT ?m WHERE { ?m a obo:BFO_0000031 }` returns ≥3 results for the sales fixture
   (skip if `ousia-sparql` absent).
7. The RDF annotation property IRIs match the JSON overlay's IRIs exactly (a test
   asserts the two emitters use the same constant).
