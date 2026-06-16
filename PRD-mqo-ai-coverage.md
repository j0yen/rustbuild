# PRD: mqo-ai-coverage — score a model's AI-queryability and reveal the dark corners

Status: Draft v0.1
build_target: rust-cli
Vision: visions/atscale-ai-strategy.md
Fleet: 6 (AUTHOR)
Repo: j0yen/mqo-ai-coverage (NOT AtScaleInc; consumes public mqo-spec shapes only)

## TL;DR

A model author wants to know: "how much of my model can an AI agent actually
find and use?" Not just "are the measures defined" but "are they findable
(good descriptions for embed), bindable (clean names), and queryable at useful
grain?" `mqo-ai-coverage` scores every model element on three axes —
*discoverability*, *bindability*, and *queryability* — and surfaces the bottom
quintile as "dark corners": the parts of the model an AI will reliably miss or
misuse. The aggregate coverage percentage is the author-facing metric that
`mqo-bench`'s accuracy score validates.

## Why this exists (Phase-1 evidence)

- The live TPC-DS model (2026-06-16) has 30+ dimension groups and multiple
  fact tables. An author shipping this model has no way to know whether an
  AI will reach `Net Profit Tier` or `Web Site` — both exist, both have
  non-trivial names, neither has a description in the live catalog.
- `mqo-catalog-embed` (Fleet 1, shipped) retrieves better for measures with
  rich descriptions — but doesn't tell an author *which* measures are retrieval-
  poor. AI-coverage does.
- `mqo-measure-lint` (Fleet 6, this pass) finds structural defects; `ai-coverage`
  asks the composite readiness question — a measure with a perfect name but
  a bad grain is still a dark corner.
- `ousia-atscale` grounding (queued/shipping) improves formal coverage; AI-coverage
  should report a *higher* coverage score on a BFO-grounded model than an
  ungrounded one, making the grounding investment visible to an author.
- `mqo-bench` (Fleet 1, queued) measures accuracy externally; `ai-coverage` is
  the internal leading indicator — an author can improve it without running the
  benchmark, then validate with bench.

## What this builds

A standalone Rust CLI `mqo-ai-coverage`:

- `mqo-ai-coverage score --model <describe_model.json> [--grounding <ground.json>]
  [--embed-index <index.bin>] [--format text|json|html]` →
  `{coverage_pct, by_element: [{name, discoverability, bindability, queryability,
  composite, dark_corner: bool, top_issue}], dark_corners: [...]}`
- Scoring axes (each 0–1, weighted in composite):
  - **Discoverability** (0.4 weight): description length + richness, name
    readability (snake_case + noun phrase), BFO grounding presence (bonus).
  - **Bindability** (0.35 weight): name matches common NL patterns (no version
    markers, no abbreviations); if `--embed-index` supplied, embed retrieval
    rank for the element's own name (a measure that doesn't retrieve for its
    own name is poor).
  - **Queryability** (0.25 weight): queryable grain (not a key-only dimension),
    not a sensitivity-flagged column that would be blocked pre-query.
- `--html` emits a sortable table — the author-facing dashboard.
- `serve` subprocess mode exposing `score` as an `mqo-mcp-server` tool.

Deps: `serde`/`serde_json`, `clap`. Optional integration with catalog-embed index
(reads the same `.bin` format). Grounding JSON follows the `ousia-atscale annotate`
output shape.

## Acceptance criteria

1. `score` on the live TPC-DS fixture reports a model-level `coverage_pct` in
   `[0, 100]`; elements with empty descriptions score lower on discoverability
   than elements with rich descriptions.
2. Elements named with version markers (per `mqo-measure-lint M003`) score lower
   on bindability than clean names; documented in the scoring rationale field.
3. `--grounding <ground.json>` raises discoverability score for grounded elements
   vs ungrounded ones on the same model fixture (testable: fixture with 50%
   grounded elements should score higher than the same fixture with 0%).
4. `--embed-index` lowers bindability for a measure whose own name ranks below
   top-3 in its own index; a measure that retrieves for its own name is not
   penalized.
5. `dark_corner: true` is set for the bottom quintile of composite-score elements;
   `top_issue` names the weakest axis (e.g., "missing description").
6. `--format html` emits a sortable HTML table; tested by checking the output
   contains a `<table>` with one row per element.
7. `serve` mode answers a `score` tool call in `mqo-mcp-server` stdin/stdout JSON.
8. `cargo test` green offline; grounded vs ungrounded comparison and dark-corner
   threshold both covered on fixtures.
