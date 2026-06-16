# PRD: mqo-report-pack — bundle metric answers, charts, and narrative into a shareable report

Status: Draft v0.1
build_target: rust-cli
Vision: visions/atscale-ai-strategy.md
Fleet: 5 (NARRATE)
Repo: j0yen/mqo-report-pack (NOT AtScaleInc; consumes public mqo-spec shapes only)

## TL;DR

`mqo-insight-extract` extracts findings, `mqo-narrative-compose` writes prose,
`mqo-chart-caption` labels the chart. Nothing packages them into a single,
shareable artifact. `mqo-report-pack` is the assembler: given a report manifest
(a list of `{question, model, audience}` entries), it runs each through the
`mqo-agent` pipeline, composes the narrative, captions the chart, and emits a
self-contained Markdown (or HTML) report a colleague can open or paste into a
deck. This is the NARRATE fleet's end-state: from NL question to shared insight
document, with no human in the loop between "run the query" and "here is the
report."

## Why this exists (Phase-1 evidence)

- Without a packager, the narrate fleet produces three separate outputs (findings
  JSON, prose paragraph, captioned chart spec) that a human must manually
  assemble. The last-mile gap is assembly, not content.
- The Tasty Bytes "Summit 2026" scenario in the live catalog (2026-06-16) is
  explicitly a slide-deck / executive-briefing use case. A running `mqo-report-pack`
  turns that into: "give me the Summit report" → Markdown report, no manual work.
- AtScale's competitor Cortex Analyst (evidenced by the `.pbit` import) targets
  NL-to-report workflows. A packager makes the same claim credible with a
  runnable artifact.
- The Fleet 3 `mqo-decision-log` (shipped) logs every agent decision; a report
  pack that sources its content from the decision log's replay means the report is
  *provenance-stamped* — every number cites the session that produced it.

## What this builds

A standalone Rust CLI `mqo-report-pack`:

- `mqo-report-pack run --manifest <report.toml> [--out report.md] [--format
  markdown|html] [--agent '<mqo-agent-cmd>']` → a self-contained report with:
  - A title and date header (sourced from the manifest)
  - Per-question sections: prose narrative (`mqo-narrative-compose` output),
    chart (Vega-Lite JSON-embedded or rendered as ASCII table in Markdown),
    and a provenance footnote (session id from `mqo-decision-log`)
- `report.toml` is a minimal config:
  ```toml
  title = "Tasty Bytes Q3 Summit Report"
  audience = "executive"
  [[sections]]
  question = "Show me revenue by region year over year"
  model = "Tasty Bytes"
  [[sections]]
  question = "Which truck brands are trending?"
  model = "Tasty Bytes"
  ```
- `--agent` is a subprocess command so `report-pack` is independent of any
  particular agent implementation; a stub agent fixture makes tests cluster-free.
- `serve` subprocess mode exposing `run` as an `mqo-mcp-server` tool.

Deps: `serde`/`serde_json`, `toml`, `clap`. Vega-Lite JSON embedded verbatim in
HTML; ASCII table rendering for Markdown (no external renderer).

## Acceptance criteria

1. `run --manifest M` produces a Markdown report with one section per manifest
   entry; each section contains a prose paragraph and a chart representation.
2. The prose paragraph comes from `mqo-narrative-compose` output routed through the
   insight extraction → compose pipeline; stubbed by default in tests.
3. `--format html` emits valid HTML with the Vega-Lite spec embedded as a
   `<script type="application/json">` block renderable by a browser with the
   Vega-Embed CDN script.
4. Each section's provenance footnote names the `mqo-decision-log` session id that
   produced the answer, or "fixture" in stub mode.
5. A missing section (agent returns no result) produces a "no data" placeholder,
   not a crash; the rest of the report continues.
6. `--agent '<cmd>'` correctly routes each section's question through the
   subprocess and feeds its output to the insight/narrative pipeline.
7. `serve` mode answers a `run` tool call in `mqo-mcp-server` stdin/stdout JSON.
8. `cargo test` green offline; a two-section fixture with stub agent exercises
   both Markdown and HTML output paths and the missing-section fallback.
