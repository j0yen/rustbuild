# PRD: mqo-replay — behavioral regression replay of the agent against its own history

Status: Draft v0.1
build_target: rust-cli
Vision: visions/atscale-ai-strategy.md
Repo: j0yen/mqo-replay (NOT AtScaleInc; Fleet 4 — the LEARN pillar)

## TL;DR

`mqo-semantic-regression` (shipped) gates changes to the *model contract* — it
fails the build when a measure's grain or PII tag changes under the agent. But
nothing gates the *agent's own behavior*: when the planner, a binder, or a
pillar tool changes, a question that yesterday returned a clean signed answer
might today loop on clarify, bind to a different measure, or return a different
number — and no one would notice. `mqo-decision-log` now persists exactly the
series needed to catch this. `mqo-replay` re-runs the NL questions from a
decision log through the *current* `mqo-agent` and diffs the new plan/bind/
outcome against what was logged, surfacing behavioral drift before it reaches a
user.

## Why this exists (Phase-1 evidence)

- `mqo-decision-log` shipped today (binary on PATH; `PRD-mqo-decision-log.md`)
  and `query` emits a durable JSONL series of `{question, plan[], pillars_fired,
  outcome, credential_id}` records — a ready-made regression corpus with no
  consumer that replays it.
- `mqo-agent` shipped today (CLAUDE_SELF changelog 2026-06-16: "adaptive
  reference agent that plans which MQO pillars to call") — its planner is the
  exact surface most likely to drift as pillar tools evolve, and it has no
  behavioral-regression guard.
- `mqo-semantic-regression` (binary on PATH) proves the box already values
  regression gating — but it snapshots the *model's* contract, not the agent's
  *conduct*. The two are complementary: one watches the data definition, the
  other watches the agent's response to it.
- The vision's end-state #3 ("the same question yields the same number") is a
  cross-engine claim today; replay extends it across *time* — the same question
  yields the same answer release-over-release, or the diff says why.

## What this builds

A standalone Rust CLI `mqo-replay`:

- `mqo-replay run --log <decisions.jsonl> --agent <cmd> [--since <ts>]` →
  extracts each logged question, re-invokes the current agent (default
  `mqo-agent`, overridable for testing) as a subprocess on that question, and
  records the fresh `answer.json` beside the logged record.
- `mqo-replay diff --baseline <decisions.jsonl> --replay <fresh.jsonl>` →
  per question, classifies the delta: `unchanged` (same plan, same bound
  measure, same outcome), `plan_drift` (different pillars fired),
  `bind_drift` (different measure/grain bound), `outcome_drift`
  (answered↔clarify↔blocked changed), or `value_drift` (numeric answer moved
  beyond tolerance). Emits a `{question, class, baseline, replay}` row per item.
- `mqo-replay report --diff <diff.json> [--fail-on <classes>]` → renders a
  summary (counts per class, the worst offenders) and exits non-zero when any
  class in `--fail-on` is present — so CI can block a release on `bind_drift` or
  `value_drift` while tolerating benign `plan_drift`.
- Tolerance is explicit: `--value-tol <pct>` controls `value_drift`; numeric
  comparison reuses the same tolerance convention as `mqo-engine-parity` so the
  two read consistently.
- Replay is read-only against the warehouse path in tests: the agent subprocess
  is mocked with a fixture responder, so the whole flow runs cluster-free.
- `--mock`/fixture mode with a bundled baseline log, a "no-change" fixture agent,
  and a "drifted" fixture agent so every diff class is exercised offline.
- `serve` subprocess mode exposing `run`/`diff`/`report` as `mqo-mcp-server`
  tools.

## Acceptance criteria

1. `run` extracts every question from a fixture log and produces one fresh
   record per question via the (mocked) agent subprocess.
2. `diff` against a no-change fixture agent classifies every item `unchanged`.
3. `diff` against a drifted fixture agent correctly labels at least one each of
   `plan_drift`, `bind_drift`, and `outcome_drift`, naming the question.
4. `value_drift` fires only when the numeric answer moves beyond `--value-tol`
   and not within it (boundary tested at the tolerance edge).
5. `report --fail-on bind_drift,value_drift` exits non-zero when those classes
   are present and zero when only `plan_drift` is present.
6. Numeric comparison matches `mqo-engine-parity`'s tolerance convention (same
   inputs ⇒ same verdict).
7. Determinism: `diff`/`report` output is stable across runs for fixed inputs;
   no wall-clock leaks into the classification.
8. `serve` answers the replay tool calls; `--help` documents every flag; all
   tests run cluster-free against bundled fixtures.

## Non-goals

- Not a golden-accuracy benchmark — `mqo-bench` scores against ground truth;
  replay compares the agent against its *own past behavior*, which may also have
  been wrong (a drift is a flag for review, not a verdict on correctness).
- Does not re-run against live warehouses in tests; production replay is the
  operator's call, gated by `mqo-session-budget` and `mqo-access-policy`.
- Does not auto-fix drift; it reports and gates, a human decides.
