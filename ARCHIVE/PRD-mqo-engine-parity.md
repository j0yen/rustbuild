# PRD: mqo-engine-parity — prove the same metric yields the same number on every engine

Status: Draft v0.1
build_target: rust-cli
Vision: visions/atscale-ai-strategy.md
Repo: j0yen/mqo-engine-parity (NOT AtScaleInc; consumes public mqo-spec shapes only)

## TL;DR

AtScale's headline promise is "consistent metrics everywhere" — define revenue
once, get the same revenue on Snowflake, BigQuery, anything. The live MCP proves
the setup is real (the *same* `internet_sales` model is registered against both a
BigQuery and a Snowflake catalog) but **nothing on this box verifies the
numbers actually match**. `mqo-engine-parity` executes one MQO against two engine
bindings and proves (or refutes) numeric parity within a tolerance.

## Why this exists (Phase-1 evidence)

- `list_models` (2026-06-16) returns `internet_sales` under BOTH
  `internet_sales_catalog_BigQuery` and `internet_sales_catalog_Snowflake` — the
  same logical model, two engines. The consistency claim is now *testable*.
- The existing federation tooling (`mcp-cross-cluster-diff`, `ousia-mqo-diff`)
  compares model *structure/semantics*, not *executed values*. A measure can be
  structurally identical and still drift numerically (timezone, null handling,
  rounding, late-arriving rows). That gap is unverified today.
- "The same number everywhere" is end-state #3 of `visions/atscale-ai-strategy.md`
  and has no tool behind it.

## What this builds

A standalone Rust CLI `mqo-engine-parity`:

- `mqo-engine-parity check --mqo <mqo.json> --a <resultA.json> --b <resultB.json>
  [--tolerance 1e-6] [--relative]` → align the two result rowsets on their
  dimension keys, compare each measure cell, and emit
  `{parity: ok|drift, max_abs_delta, max_rel_delta, offending_cells: [...],
  rows_only_in_a, rows_only_in_b}`.
- `mqo-engine-parity run --mqo <mqo.json> --engine-a '<cmd>' --engine-b '<cmd>'`
  → execute the MQO against two binder/runner subprocesses (documented `mqo-mcp`
  execute protocol) and pipe both results into `check`.
- Tolerance is per-measure-aware: integer counts compared exactly, floats within
  abs/rel tolerance, with a `--unit-aware` flag that refuses to compare measures
  whose declared units differ (composes with `mqo-unit-guard`).
- `serve` subprocess mode exposing `check` as an `mqo-mcp-server` tool so an agent
  can assert parity inline before trusting a federated answer.

Deps: `serde`/`serde_json`, `clap`. Pure comparison logic; fixture-driven.

## Acceptance criteria

1. `check` aligns two rowsets by dimension key and reports `parity: ok` when all
   measure cells match within tolerance; `parity: drift` otherwise.
2. `offending_cells` names the dimension key, measure, and both values for every
   cell exceeding tolerance.
3. Rows present in only one result are reported in `rows_only_in_a` /
   `rows_only_in_b`, not silently dropped (a missing row is a parity failure, not
   a match).
4. Tolerance handles both absolute and relative modes; integer-typed measures are
   compared exactly regardless of `--tolerance`.
5. `--unit-aware` refuses (non-zero exit + clear error) to compare two measures
   with mismatched declared units.
6. `run` executes the MQO against two subprocess engines and feeds both into
   `check`; a two-stub fixture demonstrates an `ok` and a `drift` case with no
   real warehouse.
7. `serve` mode answers a `check` tool call in `mqo-mcp-server` stdin/stdout JSON.
8. `cargo test` is green offline; both `ok` and `drift` paths covered on fixtures.
