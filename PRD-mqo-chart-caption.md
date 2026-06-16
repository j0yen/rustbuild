# PRD: mqo-chart-caption — generate natural-language captions for semantic-layer visualizations

Status: Draft v0.1
build_target: rust-cli
Vision: visions/atscale-ai-strategy.md
Fleet: 5 (NARRATE)
Repo: j0yen/mqo-chart-caption (NOT AtScaleInc; consumes public mqo-spec shapes only)

## TL;DR

`mqo-mcp`'s `build_vega_spec` (shipped, in the existing server) produces a chart
spec from a result handle. The chart has no title and no caption; the user sees
bars and a legend, not a statement. `mqo-chart-caption` closes this: given a
Vega-Lite spec + the underlying insight extract, it generates a title, subtitle,
and one-sentence caption that describes *what the chart says*, not what the chart
*is* ("Revenue by Region, Q3 2025" vs "EMEA leads in Q3 with revenue 28% above
the AMER average").

## Why this exists (Phase-1 evidence)

- `build_vega_spec` and `recommend_chart` are existing tools in the live
  `mqo-mcp-server` (confirmed 2026-06-16 from the MCP tool list in prior
  sessions). They produce a `VegaSpec` JSON but carry no title/caption field;
  the chart is unlabeled.
- The Tasty Bytes Summit model (live catalog, 2026-06-16) is explicitly a
  demo/summit scenario where chart-ready output matters — unlabeled bars undercut
  the demo.
- `mqo-insight-extract` (Fleet 5, this pass) already identifies the headline
  finding. The chart caption is that finding reworded as a *statement about the
  visualization* — a small transform, not a new analysis. Natural place to wire
  the two tools together.
- Charts without captions require a human to interpret and title them. That defeats
  "AI-generated analysis" at the moment it's shown to someone.

## What this builds

A standalone Rust CLI `mqo-chart-caption`:

- `mqo-chart-caption caption --spec <vega.json> [--insights <insights.json>]
  [--audience executive|analyst] [--lang en]` → `{title, subtitle, caption}`
  where: `title` is the primary measure + dimension axis ("Revenue by Region");
  `subtitle` is the time/filter context ("Q3 2025, Snowflake catalog");
  `caption` is the headline finding as a declarative sentence.
- `--insights` is optional; without it, caption extracts the chart's headline
  from the Vega spec encoding (highest-value bar/line named in the legend).
- With `--insights`, caption uses the top-ranked finding from `insight-extract`
  output as its statement, attributing polarity and magnitude.
- `serve` subprocess mode exposing `caption` as an `mqo-mcp-server` tool,
  composable after `build_vega_spec`.

Deps: `serde`/`serde_json`, `clap`. Parses Vega-Lite encoding to derive axis
labels; no rendering, no browser, no API.

## Acceptance criteria

1. `caption --spec S` (no insights) produces a `title` that names the primary
   measure and encoding dimension, and a `caption` that names the highest-value
   mark from the Vega encoding.
2. `caption --spec S --insights I` uses the top-ranked insight finding as the
   `caption` sentence; magnitude and polarity are reflected in the wording.
3. `subtitle` captures the active filters/time context from the Vega spec metadata
   or a `--context` CLI flag.
4. `--audience executive` produces a plain-English caption ("Revenue in South Korea
   leads with a 28% share"); `--audience analyst` includes the magnitude number and
   comparison basis ("South Korea: $1.4M, 28% of total, 12% above EMEA average").
5. Missing or null measure names in the Vega spec degrade gracefully — a clear
   "unknown measure" caption rather than a panic.
6. `serve` mode answers a `caption` tool call in `mqo-mcp-server` stdin/stdout JSON.
7. `cargo test` green offline; with-insights and without-insights paths both covered
   on Tasty-Bytes-shaped fixtures (geography dimension, ORDER_AMT measure).
8. No network call in any path; no external rendering dependency.
