# PRD: mqo-grounding-advisor — suggest BFO grounding for ungrounded model elements

Status: Draft v0.1
build_target: rust-cli
Vision: visions/atscale-ai-strategy.md
Fleet: 6 (AUTHOR)
Repo: j0yen/mqo-grounding-advisor (NOT AtScaleInc; consumes public mqo-spec + ousia-atscale shapes)

## TL;DR

`ousia-atscale` grounds AtScale model elements to BFO 2020 — but requires a model
author to understand BFO to override or extend the mapping. An author who doesn't
know BFO looks at `ousia-atscale annotate` output and sees `BFO_0000033` — opaque.
`mqo-grounding-advisor` is the author-facing bridge: given an ungrounded or
questionable-grounded model element, it explains *why* a BFO class was chosen,
surfaces the closest alternatives (with their rationale), and for ungrounded
elements suggests the top candidate with a confidence score — all in plain business
language, not ontology jargon.

## Why this exists (Phase-1 evidence)

- `ousia-atscale` (queued/shipping) automates BFO grounding but its `bfo_hint`
  override mechanism (PRD-ousia-atscale-bfo-hint, shipped 2026-06-15 per changelog)
  expects the author to supply a BFO IRI. Nobody who isn't an ontologist knows
  which IRI to pick. `grounding-advisor` turns the IRI into a choice + rationale.
- The live catalog has heterogeneous elements: `COGS_USD` (a cost measure), `IS_HEALTHY_FLAG`
  (a boolean property), `FRANCHISE_ID` (an identifier), `ROLLING_7D_AVG_SALES`
  (a derived temporal aggregate). These map to completely different BFO categories
  (GDC, Quality, Role, Process, TemporalRegion) — the kind of non-obvious mapping
  where an advisor adds real value.
- `mqo-ai-coverage` (Fleet 6, this pass) gives bonus discoverability for grounded
  elements; `grounding-advisor` is the action that earns the bonus. Together they
  close the author loop: coverage flags "ungrounded," advisor says "ground it as X
  because Y," author adds the `bfo_hint`, grounding runs again, coverage improves.
- `ousia-mqo-bind` (shipped 2026-06-16 per changelog) resolves NL→BFO→measure;
  the quality of that resolution depends on how good the grounding is. The advisor
  makes the grounding better, which makes the bind better, which raises the bench
  accuracy score. Full feedback loop.

## What this builds

A standalone Rust CLI `mqo-grounding-advisor`:

- `mqo-grounding-advisor advise --model <describe_model.json>
  [--grounding <existing_ground.json>] [--top-k 3]` →
  for each element (or elements with `confidence < 0.7` if grounding supplied):
  `{element, current_class?, suggested_classes: [{iri, label, rationale,
  confidence, plain_english_why}], recommended_iri, recommended_hint}`.
- `plain_english_why` is the key field: "COGS_USD is a Generically Dependent
  Continuant because it is a quantity that depends on a production process, not
  an independent material object."
- `recommended_hint` is the exact string to add to the model's column annotation
  as a `bfo_hint` value — copy-pasteable by an author who doesn't know ontologies.
- Works without a live AtScale cluster: consumes `describe_model` JSON + an
  optional existing grounding JSON (from `ousia-atscale annotate --format json`).
- `serve` subprocess mode.

Deps: `serde`/`serde_json`, `clap`. The BFO class → pattern matching is a rule
table (element kind × name heuristics → candidate classes), not a model; the
rationale text is template-filled from the matching rules. Relies on the documented
BFO 2020 category set (35 classes), not the full OWL file.

## Acceptance criteria

1. `advise` on an ungrounded `describe_model` fixture produces a `suggested_classes`
   list for each element; `recommended_iri` is the top candidate; `recommended_hint`
   is a valid BFO IRI string.
2. `COGS_USD` (cost/quantity pattern) maps to `GDC` (Generically Dependent
   Continuant) as the top suggestion; `IS_HEALTHY_FLAG` maps to `Quality`.
3. `FRANCHISE_ID` maps to a Role or Object pattern (not GDC); the rationale
   distinguishes identifier vs quantity.
4. `ROLLING_7D_AVG_SALES` (temporal aggregate) maps to a temporal or process
   pattern; the rationale mentions the time-window derivation.
5. When `--grounding <g.json>` is supplied, elements with `confidence >= 0.7`
   in the existing grounding are not re-suggested (they are already well-grounded);
   only low-confidence or absent groundings are surfaced.
6. `plain_english_why` avoids BFO jargon: "Generically Dependent Continuant"
   is always followed by a plain-English parenthetical; `BFO_0000033` is never
   presented alone.
7. `serve` mode answers an `advise` tool call in `mqo-mcp-server` stdin/stdout
   JSON.
8. `cargo test` green offline; cost, boolean-property, identifier, and
   temporal-aggregate patterns each exercised with a positive rule hit and a
   negative (wrong-class) rejection test.
