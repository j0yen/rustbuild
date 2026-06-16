# PRD: mqo-agent — the adaptive reference agent that plans which pillars to call

Status: Draft v0.1
build_target: rust-cli
Vision: visions/atscale-ai-strategy.md
Repo: j0yen/mqo-agent (NOT AtScaleInc; orchestrates existing mqo-*/ousia/rosetta CLIs)

## TL;DR

`mqo-demo-runner` proves the pillars compose — but only for a *fixed, scripted*
question walking a *hard-coded* pillar order. A real AI agent over the semantic
layer is handed an *arbitrary* NL question and must **decide**: does this need a
clarify round? does it mention YoY so time-intelligence fires? is the bind
confident enough to skip disambiguation? `mqo-agent` is that decision-maker — a
deterministic planner that, given one NL question, selects which pillar tools to
invoke and in what order, loops on clarify, and stops with a defensible answer.
It is the difference between a *demo* (one rehearsed path) and an *agent* (any
path the question demands).

## Why this exists (Phase-1 evidence)

- `mqo-demo-runner` (Fleet 2, `PRD-mqo-demo-runner.md`) is explicitly
  single-scenario and deterministic: it executes a *declared* ordered pipeline
  from a scenario file. Its own non-goals state "one scenario, one transcript,
  exit" and "no scheduling/serving loop." Nothing on the box *chooses* the
  pipeline from the question — the order is authored by a human, not derived.
- The vision's end-state #1–#2 require an agent whose "default data interface is
  the semantic layer" and that "grounds every answer before execution." A demo
  runner with a fixed chain does not satisfy "any question"; it satisfies "the
  rehearsed question." The planner is the missing piece between them.
- Live MCP (`list_models`, 2026-06-16) shows real questions need *different*
  pillar subsets: a Tasty Bytes "menu items by category" query needs no
  time-intelligence; an `internet_sales` "revenue YoY EMEA vs AMER" query needs
  time-intelligence *and* engine-parity (the model is on both BigQuery and
  Snowflake). One fixed chain cannot serve both without wasting or skipping
  pillars — the agent must branch.
- The pillar tools already speak the `mqo-mcp-server` flag-CLI + `serve`
  subprocess JSON contract (verified: `mqo-catalog-embed`,
  `mqo-binding-confidence`, `mqo-clarify`, `mqo-time-intelligence`,
  `mqo-engine-parity`, `mqo-sensitivity-scan`, `rosetta-credential` under
  `~/wintermute/`). The conductor (`mqo-demo-runner`) exists; the *planner* does
  not.

## What this builds

A standalone Rust CLI `mqo-agent`:

- `mqo-agent ask "<nl question>" --model <model>` → runs a deterministic
  planning loop and emits an `answer.json`: the chosen plan (ordered pillar
  steps with the *reason* each was selected), each step's verdict, and the final
  signed answer (or a `clarify_needed` prompt when the bind is ambiguous).
- A **deterministic rule-based planner** (NOT an LLM in tests): a small,
  inspectable policy that maps question features → pillar selection. E.g.
  always run catalog-embed → binding-confidence; if confidence < threshold →
  emit a clarify question and pause (the loop resumes with the user's answer via
  `--answer`); if the question contains period-over-period tokens
  (YoY/QoQ/MoM/"year over year") → insert time-intelligence; if the model is
  registered on >1 engine → insert engine-parity; always run sensitivity-scan
  before returning; always finish with rosetta-credential. The rule table is a
  bundled, documented config, not hidden code.
- Optional `--planner brain` flag to consult the box's local brain for plan
  ordering, with the deterministic planner as the default and the test path
  (keeps CI offline; cf. local-first ladder).
- Each selected step is invoked as the same subprocess tool-JSON contract
  `mqo-demo-runner` uses, so the two share the pillar-invocation convention;
  `mqo-agent` reuses (does not reimplement) that calling layer where practical.
- `--mock` fixture mode: each pillar step reads a canned response so the full
  planning loop is testable cluster-free and binary-free.
- A clarify loop: an ambiguous bind returns `clarify_needed` + a question;
  `mqo-agent ask … --answer "<disambiguation>"` resumes deterministically from
  the recorded state rather than restarting.
- `mqo-agent plan "<question>"` → prints the plan *without executing* (the
  selected pillars + per-step reason), so the planner's decisions are auditable
  before any query fires.
- `serve` subprocess mode exposing `ask`/`plan` as `mqo-mcp-server` tools.

## Acceptance criteria

1. `plan "<question with YoY>"` includes a time-intelligence step with a recorded
   reason; `plan "<question without period tokens>"` omits it — proving the plan
   is derived from the question, not fixed.
2. `plan` against a model the fixtures mark as multi-engine includes an
   engine-parity step; against a single-engine model it does not.
3. `ask … --mock` executes exactly the planned steps in planned order and emits
   `answer.json` with per-step verdicts and a final answer; no unplanned pillar
   runs.
4. A low-confidence bind yields `clarify_needed` + a question and does NOT
   fabricate a final answer; `ask … --answer <x>` resumes and completes
   deterministically.
5. sensitivity-scan always appears before the final answer in any non-clarify
   plan; rosetta-credential is always the terminal step of a completed answer.
6. The planner rule table is a readable bundled config; `--help` documents every
   subcommand/flag.
7. Determinism: identical `plan` and `--mock ask` output across runs (timings
   excluded from equality).
8. `serve` answers `mqo-mcp-server` `ask` and `plan` tool calls over
   stdin/stdout; the full mock test runs with no network and no sibling binaries
   installed.

## Non-goals

- Not an LLM. The default planner is deterministic rules; `--planner brain` is
  opt-in and not the tested path. If the *binding* logic is wrong, that is
  `mqo-binding-confidence`'s PRD, not this one.
- Does not reimplement any pillar — it selects and calls them.
- Not a server loop or daemon — one question, one answer, exit. A standing agent
  service is a possible follow-on.
- Access control, budget enforcement, and decision logging are *consulted/emitted*
  by the agent but owned by sibling Fleet-3 PRDs (`mqo-access-policy`,
  `mqo-session-budget`, `mqo-decision-log`).
