# Vision: mqo-tools — new analytical tools for the AtScale mqo-mcp server

> Drafted 2026-06-15. Seed: user prompt — "of tools for the atscale mqo-mcp server."

## TL;DR

`mqo-mcp` is a 50-crate workspace whose server exposes ~12–23 read-only tools:
catalog (list/describe/search), the pipeline (`query_multidimensional`,
`next_page`), federation (list/health/diff clusters), and visualization
(recommend_chart, build_vega_spec, build_bi_asset, compose_dashboard), plus
server-side handle ops. Verified 2026-06-15 by reading the live tool registry in
`mqo-mcp-server/src/mcp.rs` and the full crate list via `gh api`.

Five capabilities an AI analyst working a governed semantic layer still cannot
do are **absent** from the workspace (confirmed: no crate matches
time/unit/anomaly/template/lineage):

1. **Time intelligence** — derive period-over-period / YoY / QoQ / rolling-N /
   MTD-QTD-YTD MQOs from a base MQO. The single most common BI ask, with no tool.
2. **Unit/format compatibility** — measures carry units (currency, %, ratio,
   count); nothing stops an agent composing a query that adds revenue to a ratio.
3. **Anomaly scan** — statistical outlier detection over a result handle's rows,
   server-side, so the model gets "row 47 is 3σ high" without seeing all rows.
4. **Query lineage** — a human-readable "how was this number computed" tree from
   the bind→route→compile chain (the existing `mcp-causal-tracer` explains
   *deltas*, not a single result's derivation).
5. **MQO templates** — parameterized, reusable saved MQOs so an agent re-deriving
   the same query shape doesn't rebuild it from scratch each session.

Each ships as a standalone `joeyen-atscale/<slug>` repo (matching the existing
`mcp-*` tool pattern) with a flag-based CLI for composition + a `serve` mode
speaking mqo-mcp-server's stdin/stdout subprocess-tool JSON protocol. All are
fixture-driven and cluster-free in tests.

## End-state

With these on `$PATH` and registered, an agent can: ask "show revenue YoY" and
get a correct derived MQO; be blocked before composing unit-incompatible
measures; surface the outliers in a 50k-row result without paging through it;
explain exactly how a number was computed; and reuse a parameterized query shape
across sessions.

## Components

- **mqo-time-intelligence** (`rust-cli`, new repo) — `derive` subcommand: base
  MQO + time-intel spec (`yoy|qoq|mom|rolling:N|mtd|qtd|ytd`) → derived MQO that
  the existing pipeline binds/routes/executes unchanged.
- **mqo-unit-guard** (`rust-cli`, new repo) — `check` subcommand: an MQO + the
  model's per-measure unit metadata → `{ok, violations}` naming each
  semantically-invalid measure combination. A firewall like `mqo-param-validator`
  but for value semantics, not grounding.
- **mqo-anomaly-scan** (`rust-cli`, new repo) — `scan` subcommand over a result
  rowset (handle export or inline rows) + measure + method (`zscore|iqr|mad`) →
  ranked outlier rows with scores; rows stay server-side.
- **mqo-lineage** (`rust-cli`, new repo) — `explain` subcommand: a BoundMqo (or a
  `query_multidimensional` response) → a provenance tree (measure definition →
  filters applied → backend chosen → compiled query) as text/JSON.
- **mqo-template** (`rust-cli`, new repo) — `list`/`instantiate` subcommands: a
  template dir of parameterized MQOs (slots like `{{date_range}}`, `{{measure}}`)
  → a concrete MQO when given param bindings.

## Order

All five are independent (each consumes the documented `mqo-spec` MQO/BoundMqo
shapes; none depends on another). They can ship in any order / in parallel.
Suggested build priority: time-intelligence → unit-guard → anomaly-scan →
lineage → template (value-descending).

## Open questions

- Should these be standalone repos (this vision's default, matching `mcp-*`) or
  new workspace members in `joeyen-atscale/mqo-mcp`? Standalone keeps them
  independently deployable; the user can pull any into the workspace later.
- `mqo-spec` is the source of truth for MQO/BoundMqo JSON shapes but isn't cloned
  locally — each PRD ships a minimal fixture matching the documented shape and
  notes that the real schema must be confirmed against `mqo-spec` at build time.
- Where does per-measure unit metadata come from — `describe_model` output, or a
  side-table? Investigate in mqo-unit-guard; default to a `format`/`unit` field
  on the measure if present, else a supplied units map.
