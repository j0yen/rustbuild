# PRD: rosetta-serve — dereferenceable IRIs + a SPARQL endpoint over the lattice

Status: Draft v0.1
build_target: rust-cli
Vision: visions/rosetta.md

## TL;DR

The World Ontology lives in an `oxigraph` store (`lattice-join`), but its IRIs —
`wo:SentientBeing`, `wo:Dignity`, every emitted decision graph — are **not
dereferenceable**, and there is no SPARQL *endpoint*, only a CLI. A semantic-web
client that holds the IRI `http://wintermute.local/wo/Dignity` cannot GET a
description of it, and an agent cannot point a standard SPARQL client at the
graph. `rosetta-serve` is a small local HTTP server that makes the joined
lattice + the PROV-O decision graphs (`rosetta-prov`) **addressable linked
data**: content-negotiated IRI dereferencing (Turtle / JSON-LD / HTML) and a
read-only SPARQL 1.1 query endpoint.

## Why this exists

Phase-1 live inspection (2026-06-15):

- `lattice-join`'s README confirms it loads everything into a single persistent
  `oxigraph 0.4` store with live bridge materialization — the data is there and
  query-ready, but reachable only via a CLI in-process.
- `lattice-context`'s README states the "give AIs infinite context" mechanism is
  *retrieval of task-relevant subgraphs* — which presupposes the subgraph is
  *addressable*. Today it is computed and printed, not served at a stable IRI.
- `ousia-mcp` exposes SPARQL over MCP (agent-facing), but there is **no W3C
  SPARQL Protocol HTTP endpoint** any standard semantic-web tool (a browser, a
  `curl`, a Python `SPARQLWrapper`) could hit, and **no dereferenceable IRIs**.
- The ecosystem already uses `axum`/`tokio` (see `wintermute-bootstrap`,
  `wintermute-platform`); `oxigraph 0.4` exposes a query API directly.
- `grep` over all PRDs + visions: zero dereferenceable-IRI / SPARQL-endpoint
  coverage. Unclaimed.

## What this builds

New repo `~/wintermute/rosetta-serve/` (rust-cli, edition 2021, MSRV 1.85).

**Deps:** `oxigraph 0.4` (the store + SPARQL query engine — same as
lattice-join), `axum` + `tokio` (HTTP), `clap` v4. **Read-only by construction:
SPARQL UPDATE is rejected, mirroring `ousia-mcp`'s read-only stance.**

**Surfaces:**

1. **SPARQL Protocol endpoint** — `GET|POST /sparql?query=...` per the W3C SPARQL
   1.1 Protocol. SELECT → `application/sparql-results+json` (and CSV via
   `Accept`); CONSTRUCT/DESCRIBE → Turtle/JSON-LD via content negotiation; ASK →
   results-json. UPDATE verbs → 405. A query timeout guards runaway queries.

2. **Dereferenceable IRIs** — `GET /{prefix}/{local}` (e.g. `/wo/Dignity`)
   returns a `DESCRIBE`-style bounded description of that resource, content-
   negotiated by `Accept`: `text/turtle`, `application/ld+json`, or a minimal
   `text/html` human view. A 404 for unknown IRIs.

3. **Store binding** — `--store <path>` points at the lattice-join oxigraph
   store; `--load <ttl>...` additionally loads emitted decision graphs
   (rosetta-prov output) at startup so verdicts are queryable alongside the
   ontology.

**CLI:**

```
rosetta-serve up --store ~/.local/share/lattice/store [--load decisions/*.ttl]
                 [--bind 127.0.0.1:7180] [--timeout 30s]
rosetta-serve check --store <path>     # validate store loads + print triple count, exit
```

Binds `127.0.0.1` by default (local-only; public hosting is a deferred
constellation/herald concern per the vision's open questions).

## Acceptance criteria

1. `rosetta-serve check --store <fixture>` loads the store, prints a triple
   count > 0, and exits 0; a missing/corrupt store exits non-zero with a
   diagnostic.
2. With the server up, `GET /sparql?query=ASK%20{%20?s%20?p%20?o%20}` returns
   `application/sparql-results+json` with `"boolean": true`.
3. A SELECT query over `Accept: application/sparql-results+json` returns
   well-formed SPARQL JSON results with the requested bindings.
4. A CONSTRUCT query with `Accept: text/turtle` returns Turtle that round-trips
   through `oxrdfio`; the same query with `Accept: application/ld+json` returns
   valid JSON-LD.
5. `GET /wo/Dignity` with `Accept: text/turtle` returns a non-empty bounded
   description of that IRI; with `Accept: application/ld+json` returns valid
   JSON-LD; an unknown IRI returns 404.
6. A SPARQL UPDATE request (e.g. `INSERT DATA`) is rejected with HTTP 405 and
   the store is unchanged (verified by triple count before/after).
7. `--load` of a rosetta-prov decision graph makes its `prov:Activity` / verdict
   triples queryable via `/sparql` (a SELECT returns the loaded verdict).
8. A query exceeding `--timeout` is aborted with a 5xx/diagnostic rather than
   hanging the server; subsequent requests still succeed.
