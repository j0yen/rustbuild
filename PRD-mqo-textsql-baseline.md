# PRD: mqo-textsql-baseline — the honest raw-table text-to-SQL control binder

Status: Draft v0.1
build_target: rust-cli
Vision: visions/atscale-ai-strategy.md
Repo: j0yen/mqo-textsql-baseline (NOT AtScaleInc; consumes public mqo-spec shapes only)

## TL;DR

`mqo-bench` promises to publish the AtScale thesis as a number — *"the semantic
layer binds NL→metric N% more accurately than text-to-SQL"*. That number is a
lie until there is a real **text-to-SQL control** to subtract from. This builds
the control: a deliberately-naive binder that sees only physical fact/dimension
tables — no semantic measures, no grain, no grounding — and produces a best-effort
bound query per NL question. Its *mistakes* (invented joins, double counts, wrong
grain, leaked PII columns) are exactly the measurement. Feed its predictions to
`mqo-bench compare` and the "+N%" delta becomes honest.

## Why this exists (Phase-1 evidence)

- The atscale-ai-strategy vision names this as an explicit open question and a
  follow-on PRD: *"To publish the '+N% vs text-to-SQL' delta honestly, mqo-bench
  needs a text-to-SQL baseline answer key… Build a minimal raw-schema-only binder
  as the control."* (visions/atscale-ai-strategy.md, Open questions).
- Probing the live AtScale MCP (`list_models`, 2026-06-16) shows the raw surface
  the control must bind against: `internet_sales` exposes raw fact table
  `factinternetsales`; `tpcds_benchmark_model` exposes `catalog_sales`,
  `store_sales`, `web_sales` across channels; `Tasty Bytes`'s `ORDER_DETAILS`
  carries raw facts `LINE_TOTAL`, `UNIT_PRICE`, `QUANTITY` — *and* semantic
  metrics `ORDER_AMT`, `AVG_ORDER_VALUE`, `ROLLING_7D_AVG_SALES`. The control
  must be denied the metrics layer and forced onto `SUM(LINE_TOTAL)`-style raw
  guesses; that gap is the whole point.
- The competitor is concrete: `Tasty Bytes` was imported from a Power BI /
  Snowflake **Cortex Analyst** `.pbit`. A raw-table binder is a faithful proxy
  for "warehouse-locked NL→SQL", the losing move the thesis argues against.
- TPC-DS's multi-channel star (`catalog_sales`/`store_sales`/`web_sales` sharing
  a `Customer Dimension`) is a deliberate trap: a raw binder asked for "total
  sales" will double-count across channels or pick one arbitrarily — a
  measurable, reproducible failure the semantic layer avoids by definition.

## What this builds

A standalone Rust CLI `mqo-textsql-baseline`:

- `mqo-textsql-baseline bind --schema <raw_schema.json> --question "<nl>"` →
  emits a `BoundMqo`-shaped prediction (same JSON shape `mqo-bench` scores) using
  only physical tables/columns from the raw schema. No measure catalog is read
  even if present.
- The binder is rule-based and intentionally naive (this is a *control*, not a
  product): lexical column matching, greedy single-fact-table selection, FK-guess
  joins from name overlap, `SUM`/`COUNT`/`AVG` chosen by keyword. It records a
  per-bind `failure_flags[]` (e.g. `double_count_risk`, `ambiguous_fact`,
  `pii_column_selected`, `no_grain`) so failures are inspectable, not silent.
- `mqo-textsql-baseline strip --model <full_model.json>` → derive a raw-only
  schema from a full semantic model by dropping measures/grounding/grain — the
  fixture generator that turns the live models into control inputs.
- `serve` subprocess mode exposing `bind` as an `mqo-mcp-server` tool, so
  `mqo-bench run --binder 'mqo-textsql-baseline serve'` drives it directly.
- Fixtures under `fixtures/raw/` derived (via `strip`) from the three live models,
  plus a handful of golden traps (multi-channel total, PII-column leak, metric
  that only exists semantically).

## Acceptance criteria

1. `bind` on a question whose answer requires a semantic-only measure (e.g.
   `ORDER_AMT`, which has no raw column) produces a prediction that misses the
   measure and sets a `failure_flags` entry — proving the control *fails where the
   semantic layer succeeds*.
2. `bind` output validates against the same `BoundMqo` JSON shape `mqo-bench
   score` consumes (round-trips through `mqo-bench` with no schema error).
3. `bind` on the TPC-DS multi-channel "total sales" trap selects a single channel
   or flags `double_count_risk` — the failure is recorded, not hidden.
4. `strip` on a full model JSON emits a raw schema containing zero `metrics`/
   grounding fields and only physical tables/columns.
5. `serve` answers an `mqo-mcp-server` bind tool call over stdin/stdout and is
   drivable by `mqo-bench run` end-to-end against a fixture golden set.
6. Determinism: same `(schema, question)` yields byte-identical prediction across
   runs (no clock/RNG) so the published delta is reproducible.
7. `--help` documents every subcommand/flag; tests run cluster-free on fixtures.

## Non-goals

- Not a good binder. Quality is the *opposite* of the goal; it is a faithful
  proxy for raw-table NL→SQL.
- No live warehouse execution — it binds, it does not run SQL. (Execution/parity
  is `mqo-engine-parity`'s job.)
- No LLM call required; a rule-based control is more reproducible than a hosted
  model and needs no key. (An optional LLM-backed control is a follow-on.)
