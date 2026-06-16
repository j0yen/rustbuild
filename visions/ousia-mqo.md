# Vision: ousia-mqo — formal semantic grounding in the AtScale MQO pipeline

> Drafted 2026-06-15. Seed: user prompt — "organize ousia and related work
> especially the federation layer; wire it into the mqo-mcp server."

## TL;DR

`mqo-mcp` federates AtScale clusters structurally today — it compares column
names and types across `describe_model` snapshots. `ousia-atscale` already
grounds AtScale model elements to **BFO 2020 upper-level categories** (measures
→ GDC, dimensions → Quality/Role, dates → TemporalRegion). The `lattice-*`
stack can align those BFO-grounded classes across models using real ontology
federation.

`ousia-mqo` bridges the two: a standalone repo (`joeyen-atscale/ousia-mqo`)
that takes an AtScale `describe_model` response, grounds it through
`ousia-atscale`, and exposes two new semantic capabilities to `mqo-mcp-server`
as subprocess tools:

1. **Semantic cross-cluster diff**: not "column A.revenue ≠ column B.rev" but
   "both are `BFO:GDC → FinancialProcess` — semantically equivalent even if
   named differently; they diverge at the `domainModule` level."

2. **Grounded NL binding**: "gross margin" → `lattice-ground` resolves to
   `BFO:GDC` → find all measures in the grounded model with that class → bind
   to the right measure even when it's named `net_margin` or `margin_pct` in a
   different cluster.

## End-state

When `mqo-mcp-server` is running with `ousia-mqo` on `$PATH`, an agent asking
"is revenue in the EMEA cluster the same metric as revenue in AMER?" gets a
formal answer: both ground to `BFO_0000033 (GDC) / FinancialProcess` → same
genus; their `aristotelianDefinition.differentia` paths match → semantically
equivalent. And "find me the gross margin measure" resolves through BFO even
if the cluster calls it `gross_profit_rate`.

## Components

- **ousia-mqo-ground** (`rust-cli` + lib, new repo `joeyen-atscale/ousia-mqo`) —
  shell `ousia-atscale annotate` over a model JSON, cache the grounded overlay,
  expose as a standalone bin + Rust lib for the extend PRDs.
- **ousia-mqo-diff** (`rust-extend ousia-mqo`) — semantic cross-cluster diff:
  ground two models → compare by BFO IRI (same genus = equivalent) → classify
  each element as `agree|diverge|only_a|only_b`; richer than `mcp-cross-cluster-
  diff`'s string matching.
- **ousia-mqo-bind** (`rust-extend ousia-mqo`) — grounded NL binding: NL phrase
  → `lattice-ground resolve` → BFO class → search grounded model annotations for
  matching IRI; emits ranked candidates for `mqo-catalog-binder` to prefer.
- **ousia-mqo-mcp** (`rust-extend ousia-mqo`) — expose ground/diff/bind as MCP
  subprocess tools in the exact stdin/stdout JSON format `mqo-mcp-server`'s
  `ToolPaths` protocol expects; register them in `ARCHITECTURE.md` + the server's
  `ToolPaths` config.

## Order

```
ousia-mqo-ground (foundation + new repo gate)
  → ousia-mqo-diff (extend; depends on ground lib)
  → ousia-mqo-bind (extend; depends on ground lib + lattice-ground CLI)
  → ousia-mqo-mcp  (extend; wraps all three in subprocess-tool protocol)
```

## Open questions

- Should the grounded overlay be cached per model snapshot hash, or recomputed
  per request? (Annotation is deterministic and cheap; a hash-keyed cache under
  `~/.cache/ousia-mqo/` is probably right.)
- Should `ousia-mqo-bind` call `lattice-ground` as a subprocess or link it? The
  `lattice-ground` crate is local Rust — linking is cleaner if the license is
  compatible (it is: MIT/Apache-2.0). Prefer linking.
- Where does `ousia-mqo-mcp`'s `ToolPaths` config live in `mqo-mcp-server`? A
  `[ousia_mqo]` section in the server's existing `config.toml` or a new env-var
  block. Investigate in the extend PRD.
