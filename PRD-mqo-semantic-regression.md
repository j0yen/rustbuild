# PRD: mqo-semantic-regression — CI gate for the agent-facing metric contract

Status: Draft v0.1
build_target: rust-cli
Vision: visions/atscale-ai-strategy.md
Repo: j0yen/mqo-semantic-regression (NOT AtScaleInc; consumes public mqo-spec shapes only)

## TL;DR

When a model changes — a measure's grain shifts, a definition is rewritten, a
once-redacted column loses its PII tag — every AI agent that bound against the
old contract silently breaks. `ousia-atscale-diff` can compare two model files by
hand, but there is no **gate** that fails a build when a change breaks the
agent-facing metric contract. `mqo-semantic-regression` snapshots a model's
contract and fails CI when a subsequent snapshot regresses it.

## Why this exists (Phase-1 evidence)

- The live MCP shows model-variant proliferation: `internet_sales` ships beside
  `internet_sales_no_pii`, and the *same* model exists on BigQuery and Snowflake
  catalogs (`list_models`, 2026-06-16). Variants and re-platforming are exactly
  where a metric contract silently drifts.
- `ousia-atscale-diff` (queued in `visions/ousia-atscale.md`) answers "are these
  two models semantically the same?" as a one-shot manual compare — it is not a
  pass/fail gate and has no notion of *which* changes are breaking.
- A PII tag regressing (a column that was sensitive becoming un-flagged) is a
  governance incident; nothing watches for it. `mqo-sensitivity-scan` (Trust)
  detects PII in one snapshot but does not diff sensitivity across versions.
- The strategy's Trust + Ground pillars are only as good as the contract's
  stability over time; this is the missing temporal guard.

## What this builds

A standalone Rust CLI `mqo-semantic-regression`:

- `mqo-semantic-regression snapshot --model <describe_model.json> [--grounding
  <ground.json>] --out <contract.json>` → a normalized contract: per-measure
  `{name, definition_hash, grain, declared_unit, bfo_grounding?, sensitivity?}`.
- `mqo-semantic-regression check --base <contractA.json> --head <contractB.json>
  [--policy <policy.toml>]` → classify each change as `safe` (added measure,
  doc-only), `warn` (definition reworded, new dimension), or `breaking` (grain
  change, unit change, BFO regrounding, measure removed, PII tag dropped); exit
  non-zero if any `breaking` (or `warn` under a strict policy).
- A default policy with sensible severities, overridable per-project via TOML.
- Emits a human report + machine JSON, suitable for a CI step or a scheduled
  watch over the live MCP's `describe_model` output.
- `serve` subprocess mode exposing `check` as an `mqo-mcp-server` tool.

Deps: `serde`/`serde_json`, `clap`, `toml`. Pure diff logic over documented model
+ grounding shapes; fixture-driven, no cluster.

## Acceptance criteria

1. `snapshot` produces a normalized, stable contract file from a `describe_model`
   fixture (re-running on unchanged input yields a byte-identical contract).
2. `check` classifies a grain change, a unit change, a measure removal, a BFO
   regrounding, and a dropped PII/sensitivity tag each as `breaking`.
3. `check` classifies an added measure and a documentation-only change as `safe`;
   a reworded definition and an added dimension as `warn`.
4. `check` exits non-zero when any `breaking` change is present; `--policy strict`
   also fails on `warn`.
5. A `--policy <toml>` override changes a specific change-type's severity and is
   reflected in the verdict and exit code.
6. The report names each change with its from→to values and assigned severity.
7. `serve` mode answers a `check` tool call in `mqo-mcp-server` stdin/stdout JSON.
8. `cargo test` is green offline; safe/warn/breaking paths and exit codes each
   covered on fixtures, including the dropped-PII-tag case motivated by the live
   `_no_pii` variant.
