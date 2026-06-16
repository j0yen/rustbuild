# PRD: mqo-insight-extract — extract the "so what" from a set of metric answers

Status: Draft v0.1
build_target: rust-cli
Vision: visions/atscale-ai-strategy.md
Fleet: 5 (NARRATE)
Repo: j0yen/mqo-insight-extract (NOT AtScaleInc; consumes public mqo-spec shapes only)

## TL;DR

`mqo-agent` now returns structured answers — a bound MQO, execution result rows,
an anomaly scan, a parity verdict. None of those become a sentence. An analyst
asking "how is EMEA performing?" gets a JSON blob, not "EMEA revenue is down 15%
YoY, driven by Q3; the anomaly scan flags week 32 as 2.8σ below baseline."
`mqo-insight-extract` is the signal extractor: given metric answers + optional
anomaly/parity/confidence results, it identifies the top-N findings that carry
actual news — comparisons that moved, anomalies that crossed a threshold, parity
gaps that widened — and emits them as structured `{finding, evidence, magnitude,
polarity}` records ready for narrative composition.

## Why this exists (Phase-1 evidence)

- The Fleet 3 `mqo-agent` (shipped 2026-06-16) and Fleet 4 `mqo-decision-log`
  (shipped 2026-06-16) now produce a rich structured answer stream. Fleet 2's
  `mqo-scorecard` renders that as a dashboard summary. But the gap between
  "dashboard row" and "insight sentence" is entirely unaddressed — no existing
  fleet (confirmed: grep over all shipped + queued PRDs, 2026-06-16) extracts
  semantic signal from a result rowset.
- The live `Tasty Bytes` model (observed 2026-06-16 via `list_models`) has
  `ROLLING_7D_AVG_SALES`, `ORDER_COUNT`, `ORDER_AMT` and a rich geography
  dimension — exactly the data surface where "sales are up in South Korea but
  down in the US" is a real insight waiting to be stated. Nothing states it.
- Fleet 4's `mqo-drift-watch` (shipped) catches SLO regressions but doesn't
  name the winner/loser dimension — `insight-extract` is the complement: not
  "the SLO broke" but "revenue in TRUCK_BRAND_NAME=Guiltless Thai fell 28% WoW."
- Composing with `mqo-anomaly-scan` (queued, mqo-tools fleet): the anomaly output
  names outlier rows; insight-extract *interprets* them — 2.8σ is news, 0.4σ is
  not. That prioritization logic belongs here, not in the scan.

## What this builds

A standalone Rust CLI `mqo-insight-extract`:

- `mqo-insight-extract extract --result <rows.json> --model <describe_model.json>
  [--anomalies <scan.json>] [--parity <parity.json>] [--compare-prior <rows.json>]
  [--top-k 5] [--min-magnitude 0.05]` → `{insights: [{finding, evidence_type,
  dimension_path, magnitude, polarity: up|down|anomaly|parity_gap, confidence}]}`
  ranked by magnitude × confidence.
- Evidence types it reasons over: period-over-period delta (if two result sets
  supplied), anomaly scores (from anomaly-scan output), parity gaps (from
  engine-parity output), binding-confidence drops (from mqo-trust's confidence
  output). Any combination; each maps to its ranked finding.
- `--min-magnitude` filters noise: a 0.3% change is not an insight; a 28% drop
  is. Threshold is user-configurable and documented.
- `serve` subprocess mode exposing `extract` as an `mqo-mcp-server` tool.

Deps: `serde`/`serde_json`, `clap`. Pure statistical ranking over documented JSON
shapes; offline, cluster-free.

## Acceptance criteria

1. `extract --result R` ranks top-k rows by magnitude of deviation from the
   rowset mean for each numeric measure; `polarity` is `up` or `down`.
2. `--compare-prior P` surfaces period-over-period deltas; a dimension cell that
   moved more than `--min-magnitude` relative to the prior is a finding.
3. `--anomalies A` incorporates `mqo-anomaly-scan` outlier scores; an anomaly
   row above the scan's threshold becomes a finding with `evidence_type:anomaly`.
4. `--parity P` incorporates `mqo-engine-parity` drift cells as findings with
   `evidence_type:parity_gap`.
5. `--top-k` limits total findings; within the top-k, findings are ranked by
   `magnitude × confidence`, not by evidence type.
6. A `0.3%` delta below `--min-magnitude` is filtered; a `28%` delta above it
   is not. Threshold default and override documented.
7. `serve` mode answers an `extract` tool call in `mqo-mcp-server` stdin/stdout
   JSON.
8. `cargo test` green offline; period-over-period, anomaly, and parity-gap
   finding types each covered by fixture.
