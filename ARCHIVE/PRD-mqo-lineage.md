# PRD: mqo-lineage — explain exactly how a number was computed

Status: Draft v0.1
build_target: rust-cli
build_into: /home/jsy/wintermute/mqo-lineage
Vision: visions/mqo-tools.md

## TL;DR

When a governed query returns a number, the trust question is "how was this
computed?" — which measure definition, which filters narrowed it, which backend
ran it, which date role applied. The `mqo-mcp` pipeline produces all of this
internally (bind → route → compile) but discards the derivation; the agent sees
only the answer. This PRD ships `mqo-lineage`, whose `explain` subcommand takes a
BoundMqo (or a `query_multidimensional` response) and emits a human-readable
**lineage tree**: measure definition → filters applied → backend chosen →
compiled query, so a result's provenance is inspectable.

## Why this exists

Verified 2026-06-15: `mcp-causal-tracer` "traverses the concept graph to explain
metric *deltas* via structural derivation paths" — it explains why a number
*changed*, not how a single result was *derived*. The pipeline (ARCHITECTURE.md:
`binder → router → compiler → engine`) computes the BoundMqo (every ref resolved
to an exact `unique_name`), the routing decision (`{backend, estimated_rows,
sql_projection}`), and the compiled query (DAX/MDX/SQL text) — but the server
returns rows + handle, not the chain. Surfacing that chain as lineage is a pure
read over artifacts the pipeline already produces; no crate in the workspace does
it (grep `linea` → none). It directly serves the README's governance thesis:
"make the semantic layer the contract" — a contract you can read is a contract you
can trust.

## What this builds

New repo `joeyen-atscale/mqo-lineage` (binary `mqo-lineage`):

- **`mqo-lineage explain --bound <file> [--decision <file>] [--compiled <file>]
  [--response <file>] [--format text|json]`**:
  - From a BoundMqo (the documented `mqo-spec` shape — each measure/dimension
    resolved to a `unique_name`, plus filters and date roles), build a lineage
    tree:
    - **Measures**: for each, its resolved `unique_name`, its definition/expression
      if present in the bound object, and its aggregation.
    - **Filters**: each filter and the dimension level it constrains, in
      application order.
    - **Date role**: the date dimension + grain + any time window.
    - **Routing** (if `--decision` given): backend chosen + why (shape/cardinality).
    - **Compiled** (if `--compiled` given): the DAX/MDX/SQL text, labeled by
      backend.
  - Alternatively accept a full `--response` (a `query_multidimensional` result
    that embeds the bound + decision) and extract the same tree.
  - **Text output**: an indented provenance tree. **JSON output**: a nested
    `{measures:[…], filters:[…], date_role:{…}, routing:{…}, compiled:{…}}`.
- **`mqo-lineage serve`** — `{"tool":"explain_lineage","args":{"bound":…,
  "decision":…,"compiled":…}}` → `{"ok":true,"data":{…}}`.
- Ships `fixtures/bound.json` (+ optional decision/compiled) matching the
  documented shapes for deterministic tests.

MSRV 1.85. Deps: clap, anyhow, serde/serde_json. Pure read/transform — no network,
no cluster, re-implements no pipeline stage. `#![forbid(unsafe_code)]`.

## Acceptance criteria

1. `cargo test --release` passes; `cargo clippy -- -D warnings` passes.
2. `explain --bound fixtures/bound.json` emits a lineage tree listing every
   measure's resolved `unique_name` and every filter with its constrained level —
   a test asserts each bound measure and filter appears in the output.
3. With `--decision`, the tree includes the chosen backend and its rationale — a
   test asserts the backend name appears.
4. With `--compiled`, the tree includes the compiled query text labeled by backend
   — a test asserts the DAX/MDX/SQL text is present and labeled.
5. `--response` (a full query_multidimensional result embedding bound+decision)
   produces the same tree as supplying the parts separately — a test asserts
   equivalence.
6. `--format json` emits the documented nested shape and round-trips through
   `serde_json`.
7. `serve` handles `explain_lineage` and emits `{ok,data}`; unknown tool →
   `{ok:false,error}` non-zero — tests assert both.
8. A malformed/empty bound object → actionable error naming what's missing,
   non-zero exit — a test asserts graceful handling.
