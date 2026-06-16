# PRD: mqo-scorecard — publish the thesis as one stakeholder-readable artifact

Status: Draft v0.1
build_target: rust-cli
Vision: visions/atscale-ai-strategy.md
Repo: j0yen/mqo-scorecard (NOT AtScaleInc; consumes mqo-* tool JSON only)

## TL;DR

The atscale-ai-strategy end-state says *"the semantic layer's advantage over
text-to-SQL is a number… and the delta is published."* The Fleet-1 tools each
emit that evidence — but as isolated JSON blobs no stakeholder will ever open:
`mqo-bench` accuracy here, `mqo-engine-parity` drift there, `mqo-sensitivity-scan`
PII counts somewhere else. `mqo-scorecard` aggregates those outputs into a single
rendered scorecard (markdown + JSON) — the thesis as a one-glance dashboard, with
trend deltas against a prior scorecard. It computes nothing itself; it is the
publication layer.

## Why this exists (Phase-1 evidence)

- End-state item #4 of the vision ("the delta is published") has no owner: Fleet 1
  *produces* the numbers, but nothing *presents* them
  (visions/atscale-ai-strategy.md, End-state).
- The Fleet-1 PRDs each define a JSON output with no shared home: `mqo-bench`
  reports accuracy + paired delta; `mqo-engine-parity` reports
  `{parity, max_abs_delta, offending_cells}`; `mqo-sensitivity-scan` (mqo-trust)
  reports PII findings; `mqo-aggregate-advisor` reports scan-cost advice. A
  stakeholder asking "is the thesis true this week?" must today open four files.
- Live MCP (`list_models`, 2026-06-16) shows the proliferation a scorecard must
  track over time: `internet_sales` vs `internet_sales_no_pii` on *two* engines,
  three distinct models. A point-in-time number is not enough; the value is the
  *trend* as models change — which only a published, re-runnable scorecard gives.

## What this builds

A standalone Rust CLI `mqo-scorecard`:

- `mqo-scorecard render --inputs <dir-or-glob>` → ingests the documented JSON
  outputs of the Fleet-1 tools (each tagged by a `source` field), composes a
  single `Scorecard` model (binding accuracy + N, cross-engine parity status,
  PII-leak count, aggregate-cost savings, vs-text-to-SQL delta), and emits
  `scorecard.json`.
- `mqo-scorecard render … --format md` → renders a one-page markdown scorecard: a
  headline pass/fail per pillar, the key numbers, and a "thesis verdict" line.
- `mqo-scorecard render … --baseline <prior_scorecard.json>` → annotates each
  metric with its delta vs a prior run (↑/↓/→), so regressions in the thesis are
  visible at a glance.
- Strictly presentational: it never recomputes accuracy or parity; it only reads,
  validates, and lays out the upstream tools' verdicts. Missing inputs render as
  an explicit `n/a` cell, never a fabricated value (cf.
  [[feedback_verify_before_concluding]]).
- `serve` subprocess mode exposing `render` as an `mqo-mcp-server` tool.
- Fixtures under `fixtures/inputs/` with canned Fleet-1 tool outputs + a golden
  rendered scorecard for snapshot testing.

## Acceptance criteria

1. `render --inputs <fixtures>` ingests ≥3 distinct Fleet-1 tool JSON shapes
   (bench, engine-parity, sensitivity-scan) and emits one `scorecard.json`
   containing each pillar's headline metric.
2. `--format md` produces a single-page scorecard with a per-pillar pass/fail and
   a thesis-verdict line, matching a golden snapshot.
3. `--baseline <prior>` annotates every shared metric with a signed delta and an
   up/down/flat direction.
4. A missing input source renders that pillar's cell as `n/a` and is excluded from
   the thesis verdict — never defaulted to a passing value.
5. The tool performs no computation of accuracy/parity itself: given deliberately
   wrong-but-well-formed upstream numbers, it reproduces them verbatim (it is a
   presenter, not a judge).
6. `serve` answers an `mqo-mcp-server` `render` tool call over stdin/stdout.
7. Determinism: identical inputs → byte-identical `scorecard.json` and markdown.
8. `--help` documents every subcommand/flag; tests run cluster-free on fixtures.

## Non-goals

- No metric computation — that lives in the Fleet-1 tools. Garbage in is
  faithfully rendered, by design.
- No web server / hosted dashboard — it emits files. (A served HTML dashboard is
  a possible follow-on.)
- No historical store — trend is computed against one supplied baseline file, not
  a database. (A time-series of scorecards is a follow-on.)
