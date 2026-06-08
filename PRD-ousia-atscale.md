# PRD: ousia-atscale — BFO grounding as a semantic-layer market bridge

Status: Draft v0.1
build_target: rust-cli
Vision: visions/ousia.md

## TL;DR

This is the "market success" PRD. Joe's day-job thesis (the book outline *The
Ontological Semantic Layer*, `~/Notes/AtScale/book-outline-ontological-semantic-layer.md`)
is that the semantic-layer category lacks formal semantics — "YAML files and
property graphs aren't enough." `ousia-atscale` bridges the World Ontology into
AtScale's semantic layer: it maps BFO categories onto AtScale model concepts and
annotates/emits AtScale model definitions with BFO grounding + provenance,
driven against the **live AtScale MCP** connected in interactive sessions. It
makes "BFO-grounded semantics" a demonstrable, sellable property of a real
semantic layer.

## Why this exists

- **The market thesis already exists in Joe's writing.** The book outline pitches
  formal ontology (OWL 2 / BFO) to "senior data engineers, analytics leaders,
  CTOs" as the fix for metric inconsistency ("ten tools, ten definitions of
  revenue"). ousia-atscale is that thesis made executable against the actual
  product Joe works on.
- **The AtScale MCP is live and callable right now.** This session has
  `claude.ai Non-prod MCP` (AtScale semantic layer) attached with
  `list_models / describe_model / search_columns / run_query`. The connector the
  gossip log flagged as the un-covered outward seam is present — this PRD uses it
  for real, not hypothetically.
- **BFO grounding answers the "AI-readiness" question the outline raises.** A
  semantic layer whose measures/dimensions trace to BFO categories (quality,
  role, GDC, process) is one an AI can reason over deductively — exactly the
  ousia-guard / ousia-reason value, applied to enterprise metrics.

## What this builds

A `cargo` Rust CLI `ousia-atscale` at `~/wintermute/ousia-atscale/`.

**Deps (pinned at build time):** `ousia-sparql` (lib), `ousia-forge` (spec
types), `serde`, `serde_json`, `clap`, an HTTP client for the AtScale MCP/REST
surface (exact transport resolved at build time).

**Two directions.**
1. **Ground AtScale → BFO.** Read an AtScale model (via MCP `list_models` /
   `describe_model` output, supplied as JSON so the tool is testable offline),
   and propose a BFO category mapping for each measure/dimension/column-group:
   a dimension member is typically a BFO `role` or `quality`; a measure is an
   information GDC about a process; a fact table participates in a `process`.
   Emit the mapping as annotations (`philosophicalGrounding` / `domainModule`
   in the paper's annotation vocabulary, §4.4).
2. **Emit BFO-grounded model scaffolding.** From a forged ontology subset, emit
   AtScale-shaped model definitions (or annotation overlays) that carry BFO
   provenance, so a customer's semantic layer inherits formal grounding.

**UX.**
```
ousia-atscale ground  --model atscale-model.json --owl world-ontology.owl   # propose BFO mapping
ousia-atscale annotate --model atscale-model.json --out grounded.json       # emit grounded overlay
ousia-atscale report  --model atscale-model.json                            # coverage: % grounded
```

The MCP-driven live path (`--from-mcp <catalog.schema.table>` pulling
`describe_model` directly) is **interactive-only** — it depends on the AtScale
connector being attached, which per the memory note "may be absent in
headless/cron runs." The offline JSON-input path is the build-auto-testable
core; the live path is an interactive convenience layered on top.

## Acceptance criteria

1. `ground --model <json>` reads an AtScale model description (the JSON shape
   returned by `describe_model`) and emits, per column/measure/dimension, a
   proposed BFO category (quality | role | GDC | process | disposition) with a
   one-line rationale.
2. The mapping rules are explicit and testable: dimension members → role/quality,
   measures → information GDC about a process, fact participation → process —
   verifiable on a fixture model.
3. `annotate` emits a grounded overlay using the paper's annotation vocabulary
   (`philosophicalGrounding`, `domainModule`, `aristotelianDefinition`, §4.4)
   without mutating the source model file.
4. `report` prints grounding coverage (% of model elements with a proposed BFO
   mapping) for a model.
5. The offline path (JSON input) works with **no** network/MCP dependency — all
   ACs above are testable from fixture JSON, satisfying build-auto.
6. `--from-mcp` (live path) is clearly gated: it errors with an actionable
   message ("AtScale MCP connector not attached; use --model <json> for offline
   grounding") when the connector is absent, rather than hanging or panicking.
7. Ships a short `MARKET.md` tying the tool to the book outline's thesis (the
   "formal semantics for the semantic layer" pitch) so the market framing is
   captured with the code.
8. `cargo test` covers ground/annotate/report on a fixture AtScale model and the
   `--from-mcp`-absent error path. No test requires a live connector.
