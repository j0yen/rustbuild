# PRD: mqo-eval — LLM-free MCP eval harness driven by mqo-agent

Status: Draft v0.1
build_target: rust-cli
Vision: visions/atscale-ai-strategy.md
Fleet: MEASURE (companion to Fleet 1)
Repo: joeyen-atscale/mqo-eval (NOT AtScaleInc; personal fork namespace)

## TL;DR

`joeyen-atscale/mcp-eval` evaluates NL→MCP query accuracy but requires an LLM
API key (Anthropic/OpenAI/Gemini via LiteLLM) to drive the MCP tool calls. This
blocks eval runs on machines with no API key configured (e.g. this laptop: no
`ANTHROPIC_API_KEY`, `api.anthropic.com` is inaccessible in some contexts).
`mqo-eval` is an LLM-free alternative: it drives the same question YAML through
`mqo-agent` (the deterministic rule planner shipped in Fleet 3) instead of an
LLM, grades results via the same PGWire oracle (`ATSCALE_PG_PASS`) or an
offline fixture oracle, and emits scores in a compatible format. Zero LLM API
dependency.

## Why this exists (Phase-1 evidence)

- `joeyen-atscale/mcp-eval` README (read 2026-06-16): "LiteLLM reads each
  provider's API key from its standard env var (GEMINI_API_KEY,
  ANTHROPIC_API_KEY, OPENAI_API_KEY)." No key → no run.
- Recall memory `01KS9ZZD7WM7MJ421G78FRE33R` (2026-06-16): "Phase A network
  probe to api.anthropic.com is blocked." A laptop-local eval loop that works
  without cloud LLM calls is needed.
- `mqo-agent` (joeyen-atscale, shipped 2026-06-16 per git log) is already a
  deterministic rule planner that takes an NL question, runs the pillar chain
  (bind → time-intel → parity check → sensitivity scan → confidence), and returns
  a structured result. It is exactly the driver mcp-eval's LLM slot needs.
- `mqo-bench` (Fleet 1, queued) measures binding accuracy on a golden fixture set
  without a live cluster. `mqo-eval` is the companion that measures accuracy
  *with* a live MQO pipeline (PGWire oracle) but *without* a live LLM — the two
  cover offline and online eval respectively.
- `mcp-eval` v0.2.0 already does MQO-aware grading (PGWire oracle, not run_query).
  This PRD reuses that grading logic; only the driver changes.

## What this builds

A standalone Rust CLI `mqo-eval`:

- `mqo-eval run --questions <questions.yaml> --model <catalog>/<model>
  [--agent '<mqo-agent-cmd>'] [--oracle pgwire|fixture]
  [--pg-host H] [--pg-pass-env ATSCALE_PG_PASS]
  [--out results.json] [--format text|json]`

  For each question entry:
  1. Call `mqo-agent ask "<question>" --model <model>` as a subprocess → get a
     structured `{bound_mqo, result_rows, confidence, pillars_fired}` response.
  2. Grade: with `--oracle pgwire`, execute the bound MQO over a direct PGWire
     connection and compare the answer against the `expected_answer` field; with
     `--oracle fixture`, compare against a pre-computed fixture answer (cluster-free).
  3. Emit per-question `{question, verdict: correct|wrong|no_bind, confidence,
     pillars_fired, latency_ms}`.

- `mqo-eval summary --results <results.json>` → aggregate scores: accuracy,
  no-bind rate, mean confidence, mean latency, per-pillar fire rate. Compatible
  field names with `mcp-eval`'s output so dashboards can consume both.

- `mqo-eval compare --a <results_a.json> --b <results_b.json>` → paired diff of
  two runs (e.g. mqo-eval vs mcp-eval, or before/after a planner change): which
  questions flipped correct→wrong or wrong→correct.

- Questions YAML format (subset of mcp-eval's existing format):
  ```yaml
  questions:
    - id: q001
      question: "What is total revenue by region this year?"
      expected_answer: "..."   # string match or numeric tolerance
      model: "Tasty Bytes"
  ```

- `--agent '<cmd>'` is a subprocess override so any binder (including
  `mqo-textsql-baseline` for the control arm) can be swapped in without
  recompiling. Default: `mqo-agent`.

Deps: `serde`/`serde_json`, `serde_yaml`, `clap`, `tokio` (for PGWire async).
PGWire connection via `tokio-postgres`. No LiteLLM, no Anthropic SDK, no API key.

## Acceptance criteria

1. `run --questions Q --oracle fixture` completes with no network calls and no
   API key set; a fixture questions YAML with 5 entries + fixture oracle answers
   exercises the full pipeline and produces a `results.json`.
2. Per-question `verdict` is `correct` when the agent's answer matches the
   expected answer within tolerance, `wrong` when it doesn't, `no_bind` when
   `mqo-agent` returns no bound MQO.
3. `--agent '<cmd>'` substitutes the binder subprocess; a stub binder fixture
   (echoes a fixed bound MQO) is used in CI to avoid requiring a real `mqo-agent`
   binary.
4. `summary` emits accuracy (% correct), no-bind rate, mean confidence, and
   mean latency over a results file; verified on a fixture results.json with
   known counts.
5. `compare --a A --b B` reports questions that flipped verdict between the two
   runs, with both verdicts shown.
6. `--oracle pgwire` connects via `tokio-postgres` using `ATSCALE_PG_PASS` env
   var; absent env var produces a clear error (not a panic); this path is not
   exercised in CI (skipped when `ATSCALE_PG_PASS` unset).
7. Output field names for `summary` are documented as compatible with `mcp-eval`'s
   score output (accuracy, no_bind_rate, mean_latency_ms).
8. `cargo test` green offline; no `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, or
   `GEMINI_API_KEY` referenced anywhere in the source tree (CI-verifiable by grep).
