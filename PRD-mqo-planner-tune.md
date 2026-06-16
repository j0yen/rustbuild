# PRD: mqo-planner-tune — outcome-weighted calibration advisory for the agent's planner

Status: Draft v0.1
build_target: rust-cli
Vision: visions/atscale-ai-strategy.md
Repo: j0yen/mqo-planner-tune (NOT AtScaleInc; Fleet 4 — the LEARN pillar)

## TL;DR

`mqo-agent`'s deterministic rule planner fires pillars based on fixed
thresholds — a binding-confidence cutoff below which it clarifies, a
retrieval-`k` for catalog-embed, a period-over-period trigger for time
intelligence. Those constants were chosen by hand and never revisited against
what actually happened. `mqo-decision-log` and `mqo-goldgrow` now make the
outcomes observable: which plans ended signed on the first pass, which looped on
clarify, which a human later corrected. `mqo-planner-tune` reads that outcome
stream and *recommends* threshold adjustments — emitting a proposed planner
config diff with the evidence behind each change. It never auto-applies: the
planner stays deterministic and tested; tuning is an advisory a human merges.

## Why this exists (Phase-1 evidence)

- `mqo-agent` shipped today with a deterministic rule planner (CLAUDE_SELF
  changelog 2026-06-16) whose thresholds are hardcoded and have no calibration
  path — the planner can only be tuned by hand-editing constants with no data.
- `mqo-decision-log` shipped and its `summary` already computes "clarify rate,
  block rate, pillar-fire frequency" — the precise signals a calibrator needs,
  produced but unconsumed for tuning.
- `mqo-goldgrow` (Fleet 4, this fleet) records human accept/reject verdicts on
  harvested binds — a ground-truth signal of when the agent bound *correctly*,
  which lets tuning weight outcomes by correctness, not just by "did it stop."
- The box already has an outcome-feedback lineage to lean on: recall shipped a
  v0.6.0 `outcome-feedback` capability ([[reference_local_llm_setup]] era work,
  recall hit 01KSKFE74…) — the pattern of "observe outcomes, propose a weighted
  adjustment, gate on a human" is established here and reused, not invented.
- Determinism is non-negotiable for the planner (the vision sells the rule
  planner as "the tested default; `--planner brain` is opt-in"), so tuning must
  produce a *reviewable diff*, never silently mutate behavior.

## What this builds

A standalone Rust CLI `mqo-planner-tune`:

- `mqo-planner-tune analyze --log <decisions.jsonl> [--golden-verdicts <ledger>]`
  → joins each decision's `{plan, bound_measure, outcome}` with the goldgrow
  verdict for that bind where available, and computes per-threshold evidence:
  e.g. "of binds with confidence in [0.55,0.65] the agent did NOT clarify, 40%
  were later rejected by review" — i.e. the clarify cutoff is too low. Emits a
  `{threshold, current, observed_evidence, suggested}` table (JSON).
- `mqo-planner-tune propose --analysis <a.json> --config <planner.toml>` →
  renders a unified diff against the agent's planner config applying the
  suggested thresholds, annotated with the evidence line for each change. Writes
  nothing to the live config — stdout/`--out` only.
- `mqo-planner-tune explain --threshold <name> --analysis <a.json>` → prints the
  full evidence trail for one threshold (the binned outcomes the suggestion
  rests on) so a reviewer can judge the recommendation, not just accept a number.
- Every suggestion carries a minimum-support guard: a threshold with fewer than
  `--min-support` observations is reported as `insufficient_data` and never
  given a suggested value (no tuning on noise).
- Advisory only: there is no `apply` subcommand. The output is a diff a human
  reviews and merges, keeping the planner deterministic and the change auditable.
- `--mock`/fixture mode with a bundled decision log, goldgrow verdict ledger, and
  planner config so `analyze`/`propose`/`explain` are testable offline.
- `serve` subprocess mode exposing `analyze`/`propose`/`explain` as
  `mqo-mcp-server` tools.

## Acceptance criteria

1. `analyze` over a fixture log emits one evidence row per known planner
   threshold, each with `{current, observed_evidence, suggested}` or
   `insufficient_data`.
2. A threshold with fewer than `--min-support` observations is reported
   `insufficient_data` and carries no suggested value.
3. On a fixture where mid-confidence un-clarified binds are frequently rejected,
   `analyze` suggests *raising* the clarify cutoff, and `explain` shows the
   binned outcomes that justify it.
4. `propose` renders a valid unified diff against the fixture planner config that
   applies exactly the suggested thresholds and changes nothing else.
5. The tool writes nothing to any live config — `propose` output goes only to
   stdout or `--out` (no `apply` subcommand exists; verified by `--help`).
6. Joining decisions with goldgrow verdicts weights outcomes by correctness:
   a bind that "stopped signed" but was later rejected counts as a miss, not a
   success (verified on a fixture pairing the two).
7. Determinism: `analyze`/`propose` output is stable across runs for fixed
   inputs (binning and ordering are deterministic).
8. `serve` answers the tuning tool calls; `--help` documents every flag; all
   tests run cluster-free against bundled fixtures.

## Non-goals

- Does not train a model or learn online — it computes binned outcome statistics
  and proposes constants; the planner stays a deterministic rule engine.
- Does not apply changes. Auto-tuning a production planner from its own
  unreviewed outcomes is exactly the tautology
  ([[feedback_agent_written_fixtures_tautology]]) this fleet exists to avoid.
- Does not tune the `--planner brain` LLM path; it calibrates only the
  deterministic rule thresholds.
