# PRD: mqo-anomaly-scan — surface outliers in a result without paging the rows

Status: Draft v0.1
build_target: rust-cli
build_into: /home/jsy/wintermute/mqo-anomaly-scan
Vision: visions/mqo-tools.md

## TL;DR

When `query_multidimensional` returns a 50,000-row result, an agent wanting "which
rows are anomalous?" must page through the handle with `next_page` and eyeball
them — defeating the whole point of keeping rows server-side behind handles. This
PRD ships `mqo-anomaly-scan`, whose `scan` subcommand runs a statistical outlier
detector over a result rowset for a chosen measure and returns only the ranked
outlier rows with their scores, so the model gets "row 47 is 3.2σ above the mean"
without ever seeing the full result.

## Why this exists

Verified 2026-06-15: the handle protocol (ARCHITECTURE.md) keeps result rows
server-side and exposes handle ops; the only statistical op visible in
`handle_ops.rs` is `topk`. `mqo-result-profiler` profiles a result (shape,
cardinality) to recommend a chart — it does not detect outliers. No crate in the
50-member workspace matches `anom|outlier`. The handle store
(`mqo-duckdb-handle-store`) already persists rows in DuckDB, so a statistical scan
is computable server-side and returns a *small* result (the outliers only) — a
perfect fit for the "model orchestrates, never calculates" thesis.

## What this builds

New repo `joeyen-atscale/mqo-anomaly-scan` (binary `mqo-anomaly-scan`):

- **`mqo-anomaly-scan scan --rows <file> --measure <name> [--method zscore|iqr|mad]
  [--threshold <f>] [--top <N>] [--by <dim>] [--format json]`**:
  - `--rows` is a JSON rowset (the shape `query_multidimensional` returns inline,
    or a handle export) — `[{col: value, ...}, ...]`. (Server integration can pipe
    a handle export; the tool itself is pure over a rowset, no DB dependency.)
  - Compute outliers on `--measure`:
    - `zscore` (default): rows with `|z| ≥ threshold` (default 3.0).
    - `iqr`: rows outside `[Q1 − k·IQR, Q3 + k·IQR]` (k = threshold, default 1.5).
    - `mad`: median-absolute-deviation robust score ≥ threshold.
  - `--by <dim>` computes outliers *within each group* of that dimension (e.g.
    outlier months per region), not globally.
  - Return `{method, threshold, total_rows, outliers: [{row, score, direction}]}`
    sorted by `|score|` descending, capped at `--top` (default 20).
- **`mqo-anomaly-scan serve`** — `{"tool":"scan_anomalies","args":{"rows":…,
  "measure":…,"method":…}}` → `{"ok":true,"data":{…}}`.
- Ships `fixtures/rows.json` with a known planted outlier for deterministic tests.

MSRV 1.85. Deps: clap, anyhow, serde/serde_json. No DuckDB dependency — pure stats
over a rowset (keeps the tool independently testable; the server feeds it a handle
export). `#![forbid(unsafe_code)]`.

## Acceptance criteria

1. `cargo test --release` passes; `cargo clippy -- -D warnings` passes.
2. `scan --rows fixtures/rows.json --measure revenue --method zscore` returns the
   planted outlier row with `|score| ≥ 3.0` ranked first — a test asserts the
   outlier's row identity and that non-outlier rows are excluded.
3. `--method iqr` flags rows outside the IQR fence on the same fixture — a test
   asserts the IQR path detects the planted outlier.
4. `--method mad` returns a robust-score ranking — a test asserts it also catches
   the planted outlier (robustness sanity).
5. `--by region` computes per-group outliers; a row that is normal globally but a
   group-local outlier is flagged — a test asserts the grouped path differs from
   the global path on a crafted fixture.
6. `--top 5` caps output to 5 outliers ranked by `|score|`; a result with no
   outliers returns `{outliers:[]}` and exits 0 — tests assert both.
7. `serve` handles `scan_anomalies` and emits `{ok,data}`; unknown tool →
   `{ok:false,error}` non-zero — tests assert both.
8. A `--measure` not present in the rows → actionable error, non-zero exit — a
   test asserts the message names the missing column.
