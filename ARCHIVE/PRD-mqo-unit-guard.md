# PRD: mqo-unit-guard — block unit/format-incompatible measure combinations

Status: Draft v0.1
build_target: rust-cli
build_into: /home/jsy/wintermute/mqo-unit-guard
Vision: visions/mqo-tools.md

## TL;DR

Measures in a semantic model carry units and formats — currency, percentage,
ratio, count, duration. Nothing in `mqo-mcp` stops an agent from composing a query
that adds revenue (USD) to a margin (ratio), or charts a count against a currency
on one axis — coherent MQOs that produce meaningless numbers. This PRD ships
`mqo-unit-guard`, whose `check` subcommand takes an MQO plus the model's
per-measure unit metadata and returns `{ok, violations}` naming each
semantically-invalid combination, as a value-semantics firewall complementing the
existing grounding firewall.

## Why this exists

Verified 2026-06-15: `mqo-param-validator` ("rejects unmapped MQO fields before
execution: lookalike_measure, wrong_hierarchy_level, wrong_date_role,
cross_fact_path") guards *grounding* — does this field exist and bind correctly.
The `mqoguard-*` suite guards structural compatibility (column-group enrichment,
compatibility matrix, filter-bind, null-path). None guards **value semantics** —
whether the measures selected are *unit-compatible* with each other and with the
chart/aggregation requested. Grepping all 50 crates for `unit|format` returns
nothing. The README's thesis is that the dangerous failures are silent and
coherent; a unit mismatch (sum of a ratio, USD+EUR without conversion, count on a
currency axis) is exactly such a silent coherent error.

## What this builds

New repo `joeyen-atscale/mqo-unit-guard` (binary `mqo-unit-guard`):

- **`mqo-unit-guard check --mqo <file> [--units <file>] [--from-model <file>]
  [--format json]`**:
  - Source the per-measure unit/format from either an explicit `--units` map
    (`{measure_name: {unit, format, currency?}}`) or `--from-model` (a
    `describe_model` output whose measures carry `format`/`unit` fields — fall
    back to the units map if absent).
  - Apply a deterministic rule set over the MQO's selected measures + their roles:
    - **Additive mismatch**: two measures of incompatible units placed where the
      MQO/chart implies summation or a shared axis (currency + ratio, count +
      currency) → `violation: additive_mismatch`.
    - **Currency mismatch**: two currency measures with different `currency` codes
      and no conversion directive → `violation: currency_mismatch`.
    - **Ratio aggregation**: a ratio/percentage measure under an additive
      aggregation (`sum`) rather than a weighted/avg form → `violation:
      ratio_summed`.
    - **Dimensionless-on-currency-axis**: a count/index measure paired on a
      currency-scaled axis in a chart hint → `violation: scale_mismatch`.
  - Emit `{ok: bool, violations: [{kind, measures, message, severity}]}`.
  - Exit 0 if no violations; non-zero if any `severity:"error"` violation
    (warnings don't fail) — enables a CI/pipeline gate.
- **`mqo-unit-guard serve`** — `{"tool":"check_unit_compatibility","args":{"mqo":…,
  "units":…}}` → `{"ok":true,"data":{ok,violations}}`.
- Ships `fixtures/` with a clean MQO+units (no violations) and a dirty one
  (currency+ratio additive) for tests.

MSRV 1.85. Deps: clap, anyhow, serde/serde_json. `#![forbid(unsafe_code)]`.

## Acceptance criteria

1. `cargo test --release` passes; `cargo clippy -- -D warnings` passes.
2. `check --mqo fixtures/clean.json --units fixtures/units.json` reports
   `{ok:true, violations:[]}` and exits 0 — a test asserts this.
3. An MQO selecting a currency measure and a ratio measure for summation reports
   an `additive_mismatch` violation naming both measures and exits non-zero — a
   test asserts the kind and measure names.
4. Two currency measures with different `currency` codes and no conversion report
   a `currency_mismatch` violation — a test asserts this.
5. A ratio measure under `sum` aggregation reports `ratio_summed` — a test asserts.
6. `--from-model` sources units from a `describe_model` fixture whose measures
   carry `format` fields; when a measure lacks unit metadata it is reported as
   `unknown_unit` (warning, not error) and does not fail the run — a test asserts
   the warning path and exit 0.
7. `serve` handles the tool call and emits the documented `{ok,data}` shape;
   unknown tool → `{ok:false,error}` non-zero — tests assert both.
8. `--format json` round-trips; severity gating works (error fails, warning passes)
   — a test asserts a warning-only run exits 0.
