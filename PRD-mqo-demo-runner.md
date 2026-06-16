# PRD: mqo-demo-runner — run the five-pillar story end-to-end as one provable artifact

Status: Draft v0.1
build_target: rust-cli
Vision: visions/atscale-ai-strategy.md
Repo: j0yen/mqo-demo-runner (NOT AtScaleInc; orchestrates existing mqo-*/ousia/rosetta CLIs)

## TL;DR

The atscale-ai-strategy vision's payoff is one end-to-end story: an agent asked
"gross margin by region, year over year, EMEA vs AMER" is answered by chaining
every pillar — semantic retrieval, confidence, clarify, time-intelligence,
grounding, engine-parity, PII scan, signed credential. Today that story is a
**paragraph of prose**. Nobody can run it; nobody can show it to a colleague and
watch the pillars fire in order. `mqo-demo-runner` turns the prose into a runnable
orchestrator that emits one ordered, inspectable transcript ending in a defensible,
signed answer.

## Why this exists (Phase-1 evidence)

- The vision's "How the pillars compose (the demo)" section is explicitly the
  roadmap's payoff and is written as narrative only — there is no executable that
  walks it (visions/atscale-ai-strategy.md).
- The pillar tools already exist as standalone CLIs on this box (verified in
  `~/wintermute/`): `mqo-catalog-embed`, `mqo-binding-confidence`, `mqo-clarify`,
  `mqo-time-intelligence`, `ousia-mqo`/`ousia-mqo-diff`, `mqo-engine-parity`,
  `mqo-sensitivity-scan`, and `rosetta-credential`. Each already speaks flag-CLI
  + `serve` subprocess JSON. What is missing is the *conductor* that calls them in
  order and threads each one's output into the next's input.
- Live MCP (`list_models`, 2026-06-16) confirms the demo's premises are real, not
  staged: the *same* `internet_sales` model is registered on both
  `internet_sales_catalog_BigQuery` and `internet_sales_catalog_Snowflake` (so
  the engine-parity step has two real engines), and `internet_sales_no_pii`
  variants exist (so the sensitivity step has a real PII surface to guard).
- The TPC-DS model's region/geography dimensions and Tasty Bytes' `LOCATION_REGION`
  give the "by region, EMEA vs AMER" slice a concrete, runnable target.

## What this builds

A standalone Rust CLI `mqo-demo-runner`:

- `mqo-demo-runner run --scenario <scenario.json>` → executes an ordered pipeline
  of pillar steps. Each step is a declared subprocess invocation (command +
  args), its stdout JSON captured, validated against the step's expected shape,
  and passed forward. Emits a single `transcript.json`: per-step
  `{pillar, tool, input, verdict, ms}` plus the final signed answer.
- `mqo-demo-runner run … --format md` → renders the transcript as a human demo
  script (one section per pillar, the verdict quoted) — the artifact a colleague
  reads.
- A step that is unavailable (binary missing) or returns a non-ok verdict
  short-circuits with a clear `blocked_at` marker rather than fabricating a
  downstream result — the transcript never claims a pillar fired when it did not
  (cf. [[feedback_verify_before_concluding]]).
- `mqo-demo-runner steps` → lists the canonical pillar order and which binary each
  step needs, so missing dependencies are visible before a run.
- A bundled `fixtures/scenarios/gross-margin-by-region-yoy.json` encoding the
  vision's flagship question, plus a fully-mocked fixture mode (`--mock`) where
  each step reads a canned response, so the full pipeline is testable cluster-free
  and binary-free in CI.
- `serve` subprocess mode exposing `run` as an `mqo-mcp-server` tool.

## Acceptance criteria

1. `run --scenario <flagship> --mock` executes all pillar steps in declared order
   and emits a `transcript.json` with one entry per pillar plus a final answer.
2. The transcript records, for each step, the exact subprocess invoked and the
   verdict it returned — no step output is synthesized by the runner itself.
3. A failing/non-ok step sets `blocked_at: <pillar>` and stops; downstream steps
   are reported `skipped`, never given fabricated inputs.
4. A missing step binary is reported by `steps` and produces `blocked_at` on
   `run` (not a panic), so the demo degrades legibly on a partial box.
5. `--format md` renders a readable per-pillar demo script from the same
   transcript JSON.
6. `serve` answers an `mqo-mcp-server` `run` tool call over stdin/stdout.
7. Determinism in `--mock` mode: identical transcript across runs (timings may be
   the only varying field and are excluded from the equality check).
8. `--help` documents every subcommand/flag; the mock pipeline test runs with no
   network and no sibling binaries installed.

## Non-goals

- Not a reimplementation of any pillar — it *calls* them; if a pillar's logic is
  wrong, that is the pillar's PRD, not this one.
- No scheduling/serving loop — one scenario, one transcript, exit. (A standing
  demo service is a possible follow-on.)
- Does not require all pillar binaries to be installed to build or test (mock
  mode covers CI); a real cross-binary run is a manual/integration check.
