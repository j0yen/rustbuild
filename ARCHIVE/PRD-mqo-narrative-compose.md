# PRD: mqo-narrative-compose — compose extracted insights into audience-targeted prose

Status: Draft v0.1
build_target: rust-cli
Vision: visions/atscale-ai-strategy.md
Fleet: 5 (NARRATE)
Repo: j0yen/mqo-narrative-compose (NOT AtScaleInc; consumes public mqo-spec shapes only)

## TL;DR

`mqo-insight-extract` produces ranked `{finding, evidence, magnitude, polarity}`
records — the *what*. `mqo-narrative-compose` turns those into prose — the *story*.
Given an insight set and an audience (`executive|analyst|technical`), it fills
a prose template (or generates one) that names the headline finding in the first
sentence, supports it with evidence in the second, and optionally calls out the
outlier or parity gap. The output is copy-pasteable text, not another JSON blob,
that a colleague can drop into a slide or email.

## Why this exists (Phase-1 evidence)

- `mqo-insight-extract` (Fleet 5, drafted alongside this PRD) produces the
  structured signal. The gap between "here are ranked findings" and "here is a
  sentence a VP can read" is the last mile that makes the semantic-layer strategy
  legible to a non-technical stakeholder.
- The Tasty Bytes model (live MCP, 2026-06-16) is a food-truck business scenario
  designed for executive storytelling ("Summit 2026" in the catalog name). The
  whole product demo ends with a data answer nobody has rendered as prose.
- AtScale's competitive context (Cortex Analyst, from the `.pbit` import in the
  live catalog) is explicitly about NL-to-insight. The answer leg is matched by
  `mqo-mcp`; the insight-communication leg is currently unaddressed.
- Audience targeting is a feature: an executive gets "EMEA revenue fell 15% in Q3."
  An analyst gets "EMEA revenue fell 15% QoQ in Q3 (from $2.1M to $1.8M), driven
  by South Korea (-28%) and Australia (-19%); the anomaly scan flagged week 32
  at 2.8σ." Same findings; different prose depth.

## What this builds

A standalone Rust CLI `mqo-narrative-compose`:

- `mqo-narrative-compose compose --insights <insights.json>
  [--audience executive|analyst|technical] [--template <tmpl.md>]
  [--metric-label <label>] [--max-sentences 4]` → prose paragraph as plain text
  (and optionally Markdown with bold emphasis on numbers).
- Built-in template library (no LLM in the default path): `executive` = 2-sentence
  headline + one supporting fact; `analyst` = headline + dimension breakdown +
  anomaly callout; `technical` = full structured evidence chain. Template is a
  simple `{{headline}}` / `{{dimension_detail}}` / `{{anomaly_callout}}` mustache
  form so a project can swap the wording without changing the logic.
- `--template <tmpl.md>` override for project-specific voice/style.
- `--planner-brain` flag (opt-in) routes composition to a Claude API call for
  free-form prose when the built-in templates don't fit; the deterministic
  template path is always the tested default (no API key required).
- `serve` subprocess mode exposing `compose` as an `mqo-mcp-server` tool.

Deps: `serde`/`serde_json`, `clap`, a small mustache template crate. Default path:
zero LLM dependency.

## Acceptance criteria

1. `compose --insights I --audience executive` produces ≤2 sentences naming the
   headline finding and one supporting fact; no dimension breakdown unless a
   single dimension is the sole driver.
2. `compose --audience analyst` adds a dimension breakdown and anomaly callout when
   those evidence types are present in the insight set.
3. `compose --audience technical` renders the full evidence chain (evidence_type,
   magnitude, confidence) in prose form, retaining all top-k findings.
4. `--template <tmpl.md>` replaces the built-in template; a fixture with custom
   `{{headline}}` / `{{anomaly_callout}}` slots is exercised in tests.
5. Numbers are formatted consistently (e.g. percentages as "15%", currency as
   "$1.8M") according to the measure's declared unit from the insight record.
6. `serve` mode answers a `compose` tool call in `mqo-mcp-server` stdin/stdout JSON.
7. `cargo test` green offline; all three audience modes covered on a fixture
   insight set. No network call in the default path (verifiable by test).
8. `--planner-brain` is documented but its absence (no API key) produces a clear
   fallback to the deterministic template, not an error.
