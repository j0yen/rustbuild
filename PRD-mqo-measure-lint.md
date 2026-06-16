# PRD: mqo-measure-lint — lint a semantic model for AI-unfriendly naming, gaps, and redundancy

Status: Draft v0.1
build_target: rust-cli
Vision: visions/atscale-ai-strategy.md
Fleet: 6 (AUTHOR)
Repo: j0yen/mqo-measure-lint (NOT AtScaleInc; consumes public mqo-spec shapes only)

## TL;DR

The Fleet 1–5 tools make the semantic layer AI-friendly — but only if the model
was well-built. A measure named `m_rev_adj_v2_FINAL` with no description, a
`UNIT_PRICE` dimension that's actually a measure, and three measures that all
compute gross margin three different ways: all of these silently poison binding
accuracy. No existing tool tells a model *author* any of this. `mqo-measure-lint`
does: it flags AI-hostile patterns in a `describe_model` snapshot — ambiguous
names, missing descriptions, redundant definitions, type-category mismatches —
with a machine-readable severity and a human-readable fix suggestion.

## Why this exists (Phase-1 evidence)

- The live `Tasty Bytes` catalog (2026-06-16) already has `COGS_USD`, `SALE_PRICE_USD`
  listed under `MENU_ITEM` as *dimensions*, not measures — but they are numeric
  values that an agent might try to aggregate. That's a type-category mismatch
  discoverable by a linter.
- `internet_sales_no_pii` and `internet_sales` are siblings; a linter can detect
  that a field present in one is absent in the other and flag it as a potential
  governance gap (coverage inconsistency).
- The `TPC-DS` model has 30+ dimension groups — `Sold Date Dimensions`, `Ship Date
  Dimensions`, `Return Date Dimensions` — with overlapping names and no
  description. An agent trying to bind "order date" will face three candidates;
  a linter flags the ambiguity *before* an agent fails.
- `mqo-catalog-embed` (Fleet 1, shipped) improves retrieval for measures that
  *have* descriptions; measures with no description get zero benefit. The linter
  motivates the fix the author needs to make before embed can help.

## What this builds

A standalone Rust CLI `mqo-measure-lint`:

- `mqo-measure-lint lint --model <describe_model.json> [--rules <rules.toml>]
  [--format text|json|sarif]` → a list of `{rule_id, severity: error|warn|info,
  element, message, suggestion}` findings across the model.
- Built-in rule set (extendable via `--rules`):
  - `M001` (warn) — measure/column with empty or single-word description
  - `M002` (warn) — numeric column filed as a `dimension` rather than `fact`/`metric`
  - `M003` (error) — measure name contains version markers (`v2`, `FINAL`, `_adj`)
  - `M004` (warn) — two measures whose names are ≥85% string-similar (redundancy candidate)
  - `M005` (info) — measure name not in `{noun}_{qualifier}` format (binding-unfriendly)
  - `M006` (warn) — measure present in model A but absent in sibling model B when
    `--sibling <describe_model_b.json>` is supplied (coverage inconsistency)
- `--rules <rules.toml>` lets a project add/suppress rules without forking the binary.
- `--format sarif` emits SARIF 2.1 for IDE/CI integration.
- `serve` subprocess mode exposing `lint` as an `mqo-mcp-server` tool for an
  author-facing MCP session.

Deps: `serde`/`serde_json`, `toml`, `clap`. String similarity via a simple
edit-distance or token-overlap score; no ML.

## Acceptance criteria

1. `lint` applied to the live Tasty Bytes `describe_model` fixture flags `COGS_USD`
   and `SALE_PRICE_USD` under `MENU_ITEM` as `M002` (numeric dimension).
2. `M001` fires on any element with an empty or ≤5-character description.
3. `M003` fires on a measure named `revenue_adj_v2_FINAL` (or similar) in the
   fixture; does not fire on `AVG_ORDER_VALUE`.
4. `M004` fires when two measures names share ≥85% token overlap on a fixture
   with `gross_margin` and `gross_margin_pct`.
5. `--sibling B` fires `M006` for measures in model A absent in B, on a
   `internet_sales` vs `internet_sales_no_pii` fixture.
6. `--rules <rules.toml>` suppresses a rule by id and the suppressed rule does not
   appear in output.
7. `--format sarif` produces a valid SARIF 2.1 document (schema-checkable in
   tests via serde).
8. `serve` mode answers a `lint` tool call in `mqo-mcp-server` stdin/stdout JSON.
9. `cargo test` green offline; each rule has at least one positive and one negative
   fixture case.
