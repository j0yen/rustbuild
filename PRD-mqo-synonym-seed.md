# PRD: mqo-synonym-seed — generate NL synonym sets for measures to improve AI retrieval

Status: Draft v0.1
build_target: rust-cli
Vision: visions/atscale-ai-strategy.md
Fleet: 6 (AUTHOR)
Repo: j0yen/mqo-synonym-seed (NOT AtScaleInc; consumes public mqo-spec shapes only)

## TL;DR

`mqo-catalog-embed` (Fleet 1, shipped) finds a measure semantically — but its
embedding is only as good as the text it indexes. A measure named `ORDER_AMT`
with no description is indexed as the token "ORDER_AMT" and will never match
"total sales" or "revenue" in a semantic search. The fix is adding synonyms to
the description *before* indexing. `mqo-synonym-seed` generates a candidate
synonym set for each measure — paraphrases, business-domain aliases, common
question phrases — that an author reviews and adds to the model's description.
The author-facing companion to `mqo-catalog-embed`.

## Why this exists (Phase-1 evidence)

- `ORDER_AMT`, `ORDER_QTY`, `AVG_ORDER_VALUE` in the live Tasty Bytes catalog
  (2026-06-16) have no descriptions. An NL query "what is total revenue?" will not
  match `ORDER_AMT` semantically if its embedding is just the 8-character token.
  After a synonym seed (`total sales`, `gross revenue`, `order value`) is added to
  the description, the embedding improves immediately on the next `mqo-catalog-embed
  index` run.
- `mqo-ai-coverage` (Fleet 6, this pass) will flag these as low-discoverability.
  `mqo-synonym-seed` is the action the author takes to fix the discoverability score:
  coverage reports the problem, synonym-seed generates the solution.
- `mqo-measure-lint`'s M001 rule flags empty descriptions; synonym-seed produces
  the content that resolves the lint finding. The three Fleet 6 tools compose:
  lint → coverage → synonym-seed → re-index → coverage improves.
- Synonym generation is deterministic rule-based in the default path (no API key):
  camelCase/snake_case expansion ("AVG_ORDER_VALUE" → "average order value"),
  unit-aware aliases ("_USD" suffix → adds "in dollars", "cost", "spend"),
  domain-pattern aliases ("_AMT"→ "amount, total, sum"; "_QTY" → "quantity,
  count, volume"; "_PCT" → "percentage, rate, share"). These patterns cover
  the majority of BI measure naming conventions without any ML.

## What this builds

A standalone Rust CLI `mqo-synonym-seed`:

- `mqo-synonym-seed generate --model <describe_model.json> [--rules <rules.toml>]
  [--planner-brain] [--top-k 5]` → for each measure/dimension:
  `{element, existing_description, candidate_synonyms: [str], rationale: [str]}`
- Default rules engine: snake_case tokenization, suffix/prefix tables (`_AMT`,
  `_QTY`, `_USD`, `_PCT`, `_ID`, `_FLAG`, `AVG_`, `ROLLING_`), domain-pattern
  expansion, common BI phrasings. Produces clean, human-reviewable candidates.
- `--planner-brain` (opt-in): routes each element through a Claude API prompt for
  richer free-form synonyms; deterministic rule path is always the default.
- `mqo-synonym-seed apply --model <m.json> --approved <approved.json> --out <m2.json>`
  → write approved synonyms into the description fields of a copy of the model
  JSON, ready to re-import or diff against the live model.
- `serve` subprocess mode.

Deps: `serde`/`serde_json`, `clap`, `toml`. Default path: zero API dependency.

## Acceptance criteria

1. `generate` on the live Tasty Bytes `describe_model` fixture produces synonym
   candidates for `ORDER_AMT` that include at least two of: "total sales",
   "revenue", "order total", "gross amount" (derived from the `_AMT` suffix rule).
2. `AVG_ORDER_VALUE` candidates include "average order size", "mean order value"
   (derived from `AVG_` prefix + `_VALUE` suffix rules).
3. `ROLLING_7D_AVG_SALES` candidates include a rolling-window paraphrase (e.g.
   "7-day rolling average", "weekly average sales").
4. Elements with existing non-empty descriptions have their existing description
   *preserved* in the output and new candidates are *additive*, not replacing.
5. `apply --approved A` writes only the approved synonym sets into description
   fields; elements not in `approved` are unchanged.
6. `--planner-brain` is documented; absent API key degrades to the deterministic
   rule path with a logged warning, not an error exit.
7. `serve` mode answers a `generate` tool call in `mqo-mcp-server` stdin/stdout
   JSON.
8. `cargo test` green offline; `_AMT`, `AVG_`, `_PCT`, `_FLAG` suffix/prefix rules
   each exercised on a fixture; `apply` round-trip verified.
