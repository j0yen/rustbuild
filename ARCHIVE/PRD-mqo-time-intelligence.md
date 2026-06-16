# PRD: mqo-time-intelligence — derive period-over-period MQOs from a base query

Status: Draft v0.1
build_target: rust-cli
build_into: /home/jsy/wintermute/mqo-time-intelligence
Vision: visions/mqo-tools.md

## TL;DR

Period-over-period analysis — year-over-year, quarter-over-quarter, rolling
N-period averages, MTD/QTD/YTD — is the single most common ask against a BI
semantic layer, and the `mqo-mcp` workspace has no tool for it: an agent must
hand-build a second MQO with shifted date filters and hope it composed the
comparison correctly. This PRD ships `mqo-time-intelligence`, a standalone tool
whose `derive` subcommand takes a base MQO plus a time-intelligence spec and
emits a *derived MQO* (or a pair of MQOs + a comparison directive) that the
existing `bind → route → compile → execute` pipeline runs unchanged.

## Why this exists

Verified 2026-06-15 by listing all 50 crates in `joeyen-atscale/mqo-mcp` via
`gh api` and grepping for `time|period`: there is no time-intelligence crate. The
server's tool registry (`mqo-mcp-server/src/mcp.rs`) exposes
`query_multidimensional` over a typed MQO but nothing that derives a
time-comparison MQO from a base one. The MQO is "a selection-only typed object"
(README) — so a YoY comparison is expressible as a pair of selections with
shifted date roles, which is exactly what this tool generates deterministically
rather than leaving the model to assemble (and silently get wrong — the README's
own thesis is that *coherent-but-wrong* date-role queries are the dangerous
failure).

## What this builds

New repo `joeyen-atscale/mqo-time-intelligence` (binary `mqo-time-intelligence`):

- **`mqo-time-intelligence derive --mqo <file> --spec <spec> [--date-field <name>]
  [--format json]`**:
  - `<spec>` ∈ `yoy | qoq | mom | wow | rolling:N | mtd | qtd | ytd | pop:<grain>`.
  - Parse the base MQO (the documented `mqo-spec` shape — a measure selection + a
    date dimension at some grain). Identify the date field (explicit `--date-field`
    or inferred from the MQO's date-role member).
  - Emit a **derived MQO bundle**: for comparison specs (yoy/qoq/...), a `{base,
    comparison, comparison_kind}` object where `comparison` is the base MQO with
    the date filter shifted by the appropriate offset (YoY = −1 year, QoQ = −1
    quarter, etc.); for window specs (rolling:N, *td), a single MQO with the date
    filter widened to the window and a `window: {kind, periods}` annotation the
    pipeline/profiler can honor.
  - Never invent measures or dimensions not in the base MQO — only shift/widen the
    existing date selection. A spec that can't apply to the base MQO (no date
    role) → actionable error, non-zero exit.
- **`mqo-time-intelligence serve`** — stdin/stdout subprocess-tool mode for
  mqo-mcp-server: `{"tool":"derive_time_intelligence","args":{"mqo":…,"spec":…}}`
  → `{"ok":true,"data":{base,comparison,comparison_kind}}`.
- Ships `fixtures/base_mqo.json` matching the documented `mqo-spec` shape; a
  `README` note states the real schema must be confirmed against `mqo-spec` at
  build time (the crate is not a path dep — it serializes/deserializes the JSON
  shape).

MSRV 1.85. Deps: clap, anyhow, serde/serde_json, time or chrono (date offset math).
`#![forbid(unsafe_code)]`.

## Acceptance criteria

1. `cargo test --release` passes; `cargo clippy -- -D warnings` passes.
2. `derive --mqo fixtures/base_mqo.json --spec yoy` emits a `{base, comparison,
   comparison_kind:"yoy"}` bundle where `comparison`'s date filter is exactly the
   base's shifted back one year and all measures/dimensions are otherwise
   identical — a test asserts the date shift and measure-set equality.
3. `--spec qoq` shifts by one quarter; `--spec mom` by one month — tests assert
   each offset.
4. `--spec rolling:3` emits a single MQO widening the date filter to the trailing
   3 periods with a `window:{kind:"rolling",periods:3}` annotation — a test asserts
   the window shape.
5. `--spec ytd` widens the filter from the start of the year to the base period —
   a test asserts the lower bound.
6. A base MQO with no date role + `--spec yoy` exits non-zero with a message
   naming the missing date field — a test asserts this.
7. `serve` handles `{"tool":"derive_time_intelligence","args":{…}}` and emits
   `{"ok":true,"data":{…}}`; an unknown tool emits `{"ok":false,"error":…}` and
   exits non-zero — tests assert both.
8. `--format json` round-trips through `serde_json`; never mutates the base MQO's
   measure/dimension selection (only the date filter) — a test asserts the
   measure set is byte-identical between base and comparison.
