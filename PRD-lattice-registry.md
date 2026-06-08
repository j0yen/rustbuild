# PRD: lattice-registry — find and catalog the world's BFO ontologies

Status: Draft v0.1
build_target: rust-cli
Vision: visions/lattice.md

## TL;DR

Before you can bridge ontologies you have to *find* them. `lattice-registry` is
a Rust CLI that builds a local, cached index of BFO-conformant ontologies —
fetch them from their registries (OBO Foundry, Common Core, FIBO, …), verify
each actually grounds in BFO, and catalog its domain, class count, and license —
so the rest of the lattice fleet has a known, offline-available corpus to work
over.

## Why this exists

- **The corpus is named but not assembled.** The World Ontology paper (§1, §8.2,
  §9.2) repeatedly cites "the over 500 existing BFO-conformant ontologies… the
  OBO Foundry… Common Core Ontologies… FIBO… the Gene Ontology… the Disease
  Ontology" as the interoperability target — but none of them is on this laptop,
  and there is no index of which exist, where, or under what license.
- **Bridging needs candidates.** `lattice-bridge` aligns *two* ontologies; it
  needs a way to discover and load the second one. registry is that discovery
  layer — without it, bridging is a manual file-hunt.
- **Provenance/licensing matters for an honest tool.** Some BFO ontologies
  (FIBO, CCO) carry terms; the registry records each one's license so downstream
  joins don't silently mix incompatible sources (vision open-question).

## What this builds

A `cargo` Rust CLI `lattice-registry` at `~/wintermute/lattice-registry/`.

**Deps (pinned at build time):** `reqwest`/`ureq` (fetch), `horned-owl` (parse +
BFO-grounding check), `serde`/`serde_json` (catalog), `clap`. A bundled seed
list of OBO Foundry ontology PURLs (the Foundry publishes a machine-readable
registry — `ontologies.yml` — which registry can ingest).

**UX.**
```
lattice-registry sync                       # fetch/refresh the OBO Foundry registry + seed set
lattice-registry add <iri-or-url>           # fetch+catalog one ontology
lattice-registry list [--domain pathology]  # browse the local catalog
lattice-registry show <id>                  # metadata: domain, classes, BFO-grounded?, license
lattice-registry path <id>                  # local cached .owl path (for bridge/join to consume)
```

Cache + catalog live under `~/.cache/lattice/registry/`.

## Acceptance criteria

1. `lattice-registry sync` ingests the OBO Foundry machine-readable registry and
   populates a local catalog with ≥1 real ontology entry (id, title, domain,
   source URL).
2. `lattice-registry add <url>` fetches an OWL/OBO file, parses it with
   horned-owl, and records: class count, whether it imports/grounds in BFO
   (detected via BFO PURL references), and any declared license.
3. An ontology that does **not** reference BFO is cataloged but flagged
   `bfo_grounded: false` — registry reports it without crashing (the lattice
   targets BFO-grounded ontologies but must not assume every fetched file is).
4. `lattice-registry list --domain <d>` filters the catalog by domain; `show
   <id>` prints full metadata including license.
5. `lattice-registry path <id>` returns the local cached `.owl` path, the
   contract `lattice-bridge`/`lattice-join` consume; missing id → non-zero exit.
6. Fetches are cached: a second `add`/`sync` of an unchanged source does not
   re-download (etag/modtime check) and is offline-safe against the cache.
7. Network failure on `sync`/`add` degrades gracefully — reports which sources
   failed, keeps the existing catalog intact, non-zero exit only if nothing
   could be fetched and no cache exists.
8. `cargo test` covers: cataloging a fixture OWL file (BFO-grounded and
   not-grounded), license extraction, `path` resolution, and the cache-hit
   no-redownload path (using a local fixture server or file:// URLs — no live
   network required in CI).
