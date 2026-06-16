# PRD: mqo-bench — NL→metric binding accuracy harness for the semantic layer

Status: Draft v0.1
build_target: rust-cli
Vision: visions/atscale-ai-strategy.md
Repo: j0yen/mqo-bench (NOT AtScaleInc; consumes public mqo-spec shapes only)

## TL;DR

The whole AtScale AI thesis — *the semantic layer makes an AI more accurate than
text-to-SQL* — is currently **asserted, never measured**. `mqo-bench` is the
keystone that turns it into a number: a golden set of `{nl_question,
expected_bound_mqo}` pairs and a runner that scores a binder's output field by
field (measure, dimensions, filters, grain) and reports accuracy. With an
optional raw-table control answer key, it publishes the "+N% over text-to-SQL"
delta the strategy needs.

## Why this exists (Phase-1 evidence)

- Probing the live AtScale MCP (`list_models`, 2026-06-16) shows three real
  models to draw golden questions from — `Tasty Bytes`, `Internet Sales`
  (BigQuery + Snowflake), `TPC-DS` — with rich measures (`ORDER_AMT`,
  `AVG_ORDER_VALUE`, `ROLLING_7D_AVG_SALES`).
- AtScale is explicitly competing in NL-analytics: `Tasty Bytes` was imported
  from a Power BI / Snowflake **Cortex Analyst** `.pbit`. The competitive
  question is binding accuracy; nobody can win it without measuring it.
- Prior `mqo-tools`/`mqo-trust` Phase-1 greps (gossip, 2026-06-15) confirmed the
  50-crate `mqo-mcp` workspace has **no** crate matching `bench|eval|accuracy|
  golden` — the evidence layer is unclaimed.
- Sibling fleets (`mqo-trust`'s binding-confidence, `ousia-mqo`'s grounded
  binding, this vision's `mqo-catalog-embed`) all *claim* to improve binding;
  none can prove it without a scorer.

## What this builds

A standalone Rust CLI `mqo-bench`:

- `mqo-bench score --golden <set.json> --predictions <preds.json>` → per-question
  field-level scoring (exact / partial / miss for measure, each dimension, each
  filter, grain) + aggregate accuracy, precision/recall on bound fields, and a
  confusion summary. Output JSON + a human table.
- `mqo-bench run --golden <set.json> --binder '<cmd>'` → invoke a binder command
  per question (subprocess speaking the documented `mqo-mcp` bind tool JSON on
  stdin/stdout), collect predictions, then score.
- `mqo-bench compare --golden <set.json> --a <predsA.json> --b <predsB.json>` →
  paired delta (e.g. semantic-layer binder vs raw-table control), with a sign
  test so "+N%" is reported with whether it's significant on the set size.
- A seed golden set under `fixtures/golden/` — ~30 hand-authored NL→BoundMqo
  pairs across the three live models, each citing its model + the MQO shape.
- `serve` subprocess mode exposing `score`/`compare` as an `mqo-mcp-server` tool.

Deps: `serde`/`serde_json`, `clap`, a small stats helper (no heavy deps). No
network, no AtScale cluster — fixture- and stdin-driven.

## Acceptance criteria

1. `mqo-bench score --golden G --predictions P` emits per-question field-level
   scores and an aggregate accuracy; a perfect P scores 100%, a P with one wrong
   measure on one question scores the documented partial.
2. Field scoring distinguishes **exact**, **partial** (right measure, missing one
   filter), and **miss**, per the documented rubric in the README.
3. `mqo-bench run --binder '<cmd>'` invokes the binder per golden question over
   the subprocess JSON protocol and feeds results into `score`; a stub binder
   fixture demonstrates the full loop with no cluster.
4. `mqo-bench compare --a --b` reports a signed accuracy delta and a significance
   verdict appropriate to the set size.
5. The shipped `fixtures/golden/` set has ≥25 pairs spanning all three live
   models, each annotated with the model and the expected BoundMqo.
6. `serve` mode answers a `score` tool call in the `mqo-mcp-server` stdin/stdout
   JSON format (golden-fixture round-trip test).
7. `cargo test` is green with no network and no AtScale instance; all scoring is
   exercised on fixtures.
8. `mqo-bench --help` documents the scoring rubric and the golden-set schema.
