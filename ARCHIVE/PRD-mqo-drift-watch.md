# PRD: mqo-drift-watch — continuous binding-accuracy & parity SLO monitor

Status: Draft v0.1
build_target: rust-cli
Vision: visions/atscale-ai-strategy.md
Repo: j0yen/mqo-drift-watch (NOT AtScaleInc; Fleet 4 — the LEARN pillar)

## TL;DR

`mqo-scorecard` (shipped) renders the thesis as a one-glance artifact — accuracy,
cross-engine parity, PII count, cost savings — at a single point in time. But a
strategy that's proven once and never watched silently rots: a model variant
ships, a binder regresses, an engine drifts, and the headline number quietly
falls without anyone being told. `mqo-drift-watch` turns the static scorecard
into a continuous SLO gate. On each scheduled run it recomputes the headline
metrics, compares them against the last accepted scorecard plus a declared
tolerance, and *alerts* (non-zero exit + structured event) when any metric
regresses past its bound — the difference between "we proved it works" and "we
keep proving it works."

## Why this exists (Phase-1 evidence)

- `mqo-scorecard` shipped today (binary on PATH, `~/.local/bin/mqo-scorecard`;
  `PRD-mqo-scorecard.md`) and renders trend deltas *when handed a prior
  scorecard* — but nothing schedules that comparison or acts on a regression;
  the trend is computed only if a human happens to run it twice.
- `mqo-bench`, `mqo-engine-parity`, and `mqo-sensitivity-scan` (all binaries on
  PATH) each emit the underlying numbers; the scorecard already aggregates them.
  drift-watch consumes the *scorecard's* JSON, not the raw tools — one source of
  truth, no recomputation.
- `mqo-semantic-regression` (binary on PATH) gates the *model contract* in CI;
  drift-watch is its runtime twin for *agent accuracy* — the contract can be
  unchanged while the live binding accuracy still falls (e.g. catalog grew and
  retrieval-`k` is now too small), which a contract snapshot cannot catch.
- The vision's end-state #4 says "the delta is published"; publishing once is a
  brochure, publishing on a watch is an SLO — the operational form the
  DEPLOY/OPERATE fleet (Fleet 3) implies but does not yet provide.

## What this builds

A standalone Rust CLI `mqo-drift-watch`:

- `mqo-drift-watch check --current <scorecard.json> --baseline <prior.json>
  --slo <slo.toml>` → compares each headline metric (binding accuracy, parity
  drift, PII leak count, est. scan cost) against the baseline and the SLO
  tolerance in `slo.toml`, emitting a `{metric, baseline, current, delta,
  status: ok|warn|breach}` row per metric.
- Exit code encodes severity: `0` all ok, `2` any `warn`, `3` any `breach` — so
  a scheduler/CI step gates on breach while letting warnings through to a log.
- `mqo-drift-watch event --check <result.json>` → emits one structured alert
  event per `warn`/`breach` (JSON, the shape a notifier or `mqo-decision-log`-
  style sink ingests) naming the metric, the magnitude, and the breached SLO —
  so the watch composes with whatever alerting the box grows, without hardcoding
  a transport.
- `mqo-drift-watch baseline --accept <scorecard.json>` → promotes a scorecard to
  the new baseline (after a human confirms the change is intended, not a
  regression to alert on) — the controlled "this is the new normal" step that
  stops a real regression from silently becoming the baseline.
- SLO direction is per-metric: accuracy breaches *downward*, parity-drift and
  PII count breach *upward*; the `slo.toml` declares direction + tolerance so the
  comparison is never ambiguous.
- `--mock`/fixture mode with bundled current/baseline scorecards and an SLO file
  exercising ok, warn, and breach so all three exit codes are testable offline.
- `serve` subprocess mode exposing `check`/`event`/`baseline` as `mqo-mcp-server`
  tools.

## Acceptance criteria

1. `check` over a fixture where every metric is within tolerance emits all `ok`
   rows and exits `0`.
2. `check` where accuracy fell past its downward SLO emits a `breach` row for
   accuracy and exits `3`, naming the metric and delta.
3. `check` where a metric moved past the warn band but not the breach band emits
   `warn` and exits `2` (boundary tested at both band edges).
4. Per-metric direction is honored: a parity-drift *increase* and a PII-count
   *increase* breach, while an accuracy *increase* never does.
5. `event` emits one structured alert per non-ok metric with magnitude and the
   breached SLO; an all-ok check emits no events.
6. `baseline --accept` writes the supplied scorecard as the new baseline and a
   subsequent `check` against it reports `ok` (the controlled promotion works).
7. Determinism: `check`/`event` output and exit codes are stable across runs for
   fixed inputs (no wall-clock in the comparison).
8. `serve` answers the watch tool calls; `--help` documents every flag; all
   tests run cluster-free against bundled fixtures.

## Non-goals

- Does not compute the metrics itself — it consumes `mqo-scorecard`'s JSON;
  recomputing would fork the source of truth.
- Does not deliver alerts (no email/Slack/webhook transport) — it emits a
  structured event a notifier consumes; transport is out of scope.
- Does not auto-promote the baseline; accepting a new normal is a human gate so a
  real regression can't silently become the reference.
