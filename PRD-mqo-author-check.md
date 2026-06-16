# PRD: mqo-author-check — pre-publish quality gate for AI-ready semantic models

Status: Draft v0.1
build_target: rust-cli
Vision: visions/atscale-ai-strategy.md
Fleet: 6 (AUTHOR)
Repo: j0yen/mqo-author-check (NOT AtScaleInc; consumes public mqo-spec shapes only)

## TL;DR

Fleet 6 ships four author tools that each answer one question about a model's
AI-readiness: lint finds structural defects, coverage scores each element,
synonym-seed generates description content, grounding-advisor suggests BFO
classes. Without an orchestrator, an author runs four separate commands, reads
four separate outputs, and decides what to do next. `mqo-author-check` is the
pre-publish playbook in runnable form: one command that chains all four, produces
a single prioritized action list, and exits non-zero when the model isn't ready
for AI consumers. The model-author equivalent of `cargo check`.

## Why this exists (Phase-1 evidence)

- Drafted directly from the conversation with jsy (2026-06-16): after describing
  the Fleet 6 tools and their workflow, jsy asked "is there a playbook for this?"
  The answer was no. This PRD makes the playbook executable.
- The four Fleet 6 tools (lint, ai-coverage, synonym-seed, grounding-advisor) each
  consume `describe_model.json` and each produce independent output. A human
  reading four JSON files and deciding priority is exactly the cognitive load a
  CLI should absorb.
- The `--ci` flag (non-zero exit on error-severity findings) makes this directly
  usable as a pre-merge gate in a model-publishing workflow, motivated by
  `mqo-semantic-regression`'s successful CI-gate pattern (Fleet 1, shipped).
- The composited report answers the one question an author actually asks: "what
  do I fix before I let an AI agent touch this model, in what order?"

## What this builds

A standalone Rust CLI `mqo-author-check`:

- `mqo-author-check run --model <describe_model.json>
  [--grounding <ground.json>] [--embed-index <index.bin>]
  [--sibling <sibling_model.json>]
  [--format text|json|html] [--ci]` →
  runs all four Fleet 6 tools as library calls (not subprocesses — statically
  linked) and emits a single composited report:

  ```
  mqo-author-check: Tasty Bytes — AI Readiness Report
  =====================================================
  Coverage score:  61%  [FAIL — below 75% threshold]

  ERRORS (block CI with --ci):
    M003  ORDER_AMT_V2_FINAL   version marker in name → rename to ORDER_AMT
    M002  COGS_USD             numeric filed as dimension → reclassify as fact

  WARNINGS:
    M001  ORDER_AMT            no description — synonym candidates: total sales,
                               revenue, order total, gross amount
    M001  ORDER_QTY            no description — synonym candidates: quantity,
                               units sold, order volume
    GRND  ROLLING_7D_AVG_SALES ungrounded — recommend BFO_0000033 (GDC);
                               add bfo_hint: 'BFO_0000033'

  DARK CORNERS (bottom quintile — agent will miss these):
    WEEK_OF_YEAR, FRANCHISE_EMAIL, ROW_NUM

  Next steps:
    1. Fix M003/M002 errors (model edit required)
    2. Run: mqo-synonym-seed apply --approved <approved.json> --out updated.json
    3. Add bfo_hint overrides, re-run: ousia-atscale annotate
    4. Re-check: mqo-author-check run --model updated.json --grounding new.json
  ```

- `--ci` exits 1 on any `error`-severity finding; exits 0 if only warnings/info.
- `--format json` emits the full structured report (lint findings + coverage score
  + synonym candidates + grounding suggestions) as one JSON document, consumable
  by upstream tooling or an MCP tool.
- `--format html` emits a self-contained HTML report with coverage score prominent.
- `serve` subprocess mode exposing `run` as an `mqo-mcp-server` tool — an
  author-facing MCP session can call it inline.
- The four Fleet 6 crates are **direct library dependencies**, not subprocess
  calls, so `mqo-author-check` has one binary, one install, one `cargo install`.

## Acceptance criteria

1. `run --model M` invokes all four Fleet 6 tools against M and produces a
   combined report with: coverage score, error/warning/info findings (from lint),
   top-5 dark corners (from coverage), synonym candidates for the lowest-coverage
   elements (from synonym-seed), and grounding suggestions for ungrounded elements
   (from grounding-advisor).
2. `--ci` exits 1 when any lint finding has `severity: error`; exits 0 on
   warnings-only. Exit code is documented in `--help`.
3. Findings are deduplicated: an element that triggers both M001 (no description)
   and a grounding suggestion appears once in the output with both issues listed,
   not twice as separate entries.
4. The "Next steps" block is always present and reflects only the finding classes
   that actually fired — no generic advice for problems that weren't found.
5. `--format json` includes all four tools' raw structured output nested under
   `{lint, coverage, synonyms, grounding}` plus the top-level `coverage_pct` and
   `ci_verdict`.
6. `--sibling S` passes through to `mqo-measure-lint`'s M006 rule (coverage
   inconsistency between model and sibling); exercised in tests with an
   `internet_sales` vs `internet_sales_no_pii` fixture.
7. `serve` mode answers a `run` tool call in `mqo-mcp-server` stdin/stdout JSON.
8. `cargo test` green offline; a fixture with two errors + two warnings + one
   ungrounded element exercises the full report format, the `--ci` exit codes,
   and the deduplication logic.
9. Single binary install: `cargo install --path .` produces one `mqo-author-check`
   binary with no subprocess dependencies on the other Fleet 6 tools.
