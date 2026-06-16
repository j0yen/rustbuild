# PRD: mqo-aggregate-advisor — make AI agents aggregate-cost-aware

Status: Draft v0.1
build_target: rust-cli
Vision: visions/atscale-ai-strategy.md
Repo: j0yen/mqo-aggregate-advisor (NOT AtScaleInc; consumes public mqo-spec shapes only)

## TL;DR

AtScale's signature differentiator is **autonomous aggregates** — it accelerates
queries by routing them to pre-computed summary tables. But an AI agent fires
queries blind: it doesn't know whether an MQO will hit a fast aggregate or trigger
an expensive base-table scan, and it can't tell a user "this will be slow/costly."
`mqo-aggregate-advisor` gives the agent that awareness: estimate an MQO's cost
tier and advise whether a covering aggregate exists or should be recommended.

## Why this exists (Phase-1 evidence)

- The live models are real star schemas with grain that matters: `TPC-DS` has
  multiple fact tables (`store_sales`, `web_sales`, `catalog_sales`, `inventory`)
  and dozens of shared dimensions (`column_groups` in `list_models`, 2026-06-16) —
  exactly the surface where aggregate routing makes or breaks query cost.
- Pre-built metric tiers in the live models (`Net Profit Tier`, `Sales Price
  Tier`, `ROLLING_7D_AVG_SALES`) show aggregation/derivation is first-class in the
  product — but nothing exposes *cost awareness* to an agent.
- The `mqo-trust` fleet covers correctness/safety and `mqo-result-cache` covers
  re-execution, but **none** of the queued fleets makes an agent reason about
  *acceleration* — AtScale's core performance story is invisible to the AI.
- An agent that fires a base-scan when a covering aggregate exists wastes the
  exact advantage AtScale sells.

## What this builds

A standalone Rust CLI `mqo-aggregate-advisor`:

- `mqo-aggregate-advisor estimate --mqo <mqo.json> --model <describe_model.json>
  [--aggregates <aggs.json>]` → `{cost_tier: aggregate_hit|partial|base_scan,
  estimated_grain, scanned_facts, reasons[]}` derived from the MQO's measures +
  required dimensions vs the model's declared grain and any aggregate definitions.
- `mqo-aggregate-advisor recommend --workload <mqos.json> --model <m.json>` →
  given a set of MQOs (e.g. an agent's recent queries), suggest the covering
  aggregate(s) that would convert the most `base_scan`s to `aggregate_hit`s,
  ranked by benefit.
- Cost is a *relative tier* (not a wall-clock promise): it reasons over grain
  coverage and fact-table fan-out, honestly labeled as an estimate.
- `serve` subprocess mode exposing `estimate` as an `mqo-mcp-server` tool so an
  agent can warn before firing ("this query has no covering aggregate and will
  scan `store_sales` at day grain").

Deps: `serde`/`serde_json`, `clap`. Pure analysis over documented model + MQO
shapes; fixture-driven, no cluster.

## Acceptance criteria

1. `estimate` returns `aggregate_hit` when the MQO's grain is covered by a
   supplied aggregate definition, `base_scan` when no aggregate covers it, and
   `partial` when only some required dimensions are covered.
2. `reasons[]` cites the specific grain/dimension mismatch driving the tier (e.g.
   "requires `MENU_ITEM` grain; nearest aggregate is `DATE`-only").
3. `scanned_facts` lists which fact table(s) the MQO touches, derived from the
   model's `column_groups`.
4. `recommend` over a workload proposes aggregate definition(s) and reports how
   many workload MQOs each would accelerate, ranked by count.
5. The estimate is explicitly labeled a relative cost tier, not a time/byte
   promise, in both output and `--help`.
6. `serve` mode answers an `estimate` tool call in `mqo-mcp-server` stdin/stdout
   JSON.
7. `cargo test` is green offline; `aggregate_hit`, `partial`, and `base_scan`
   paths each covered on fixtures built from the live TPC-DS column_groups shape.
8. Works with `--aggregates` omitted (treats all queries as `base_scan` and still
   reports grain/scanned_facts) — degrades gracefully when aggregate metadata is
   unavailable.
