# PRD: ousia-mqo-diff — semantic cross-cluster diff by BFO class, not column name

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/ousia-mqo
Vision: visions/ousia-mqo.md
Depends-on: ousia-mqo-ground (the Grounder lib and GroundedOverlay type)

## TL;DR

`mcp-cross-cluster-diff` in `mqo-mcp` compares two `describe_model` catalogs by
column name and type — it finds that cluster A has `revenue` and cluster B has
`rev_usd` and calls them different. A formal diff compares their BFO grounding:
both are `BFO_0000033 / FinancialProcess` → semantically equivalent even if named
differently; they diverge only if their `domainModule` or genus IRI disagrees.
This PRD ships `ousia-mqo diff` which grounds two models and compares element-by-
element at the BFO class level, producing a verdict that names real semantic
divergences and avoids false positives from naming conventions.

## Why this exists

Verified 2026-06-15 by reading `joeyen-atscale/mcp-cross-cluster-diff`'s
description ("Diff two AtScale cluster describe_model catalogs and classify
divergences") and ARCHITECTURE.md. It is a structural differ — column-name
and type matching. The user confirmed semantic cross-cluster diff as the primary
value they want from the federation layer. The ousia-atscale overlay
(`BFO IRI + domainModule`) provides exactly the class-level key for element
identity that a semantic diff needs: two elements with the same IRI + domainModule
are *the same kind of thing*, regardless of their name in the catalog.

The `ousia-mqo-ground` lib (prior PRD) provides `GroundedOverlay` with
`annotations: HashMap<String, Annotation>` keyed by element name. A semantic diff
joins on `(iri, domainModule)` as the semantic key, not the element name, and
produces a richer classification.

## What this builds

Extend `~/wintermute/ousia-mqo`:

- **`src/diff.rs` — `SemanticDiff` lib**:
  - `ElementVerdict` enum: `Agree { name_a, name_b, iri, domain_module }`,
    `Diverge { name_a, name_b, iri_a, iri_b, domain_module_a, domain_module_b }`,
    `OnlyA { name, iri, domain_module }`, `OnlyB { name, iri, domain_module }`.
  - `SemanticDiff::compare(a: &GroundedOverlay, b: &GroundedOverlay) ->
    Vec<ElementVerdict>` — for each element, match on `(iri, domainModule)`:
    - Both models have an element grounding to the same IRI+module → `Agree`
      (even if the column names differ — this is the key semantic insight).
    - Both have an element of the *same name* but different IRI/module →
      `Diverge` (same name, different meaning — a semantic collision, the
      dangerous case).
    - IRI+module only in A or only in B → `OnlyA`/`OnlyB`.
  - `SemanticDiff::summary(verdicts: &[ElementVerdict]) -> DiffSummary`: counts
    by verdict type, list of divergences.
- **`ousia-mqo diff --model-a <path> --model-b <path> [--format text|json]`** CLI:
  - Ground both models via `Grounder`, run `SemanticDiff::compare`, print the
    verdict table or JSON.
  - Text: one row per element pair, verdict label, IRIs. JSON: the documented
    `Vec<ElementVerdict>` shape.
  - Exit 0 = no divergences; non-zero = ≥1 `Diverge` verdict (enables CI gate).
  - `--dry-ground` skip re-grounding if cache already has both overlays (fast
    re-diff on unchanged models).

MSRV 1.85. No new deps beyond ousia-mqo-ground's.

## Acceptance criteria

1. `cargo test --release` passes; `cargo clippy -- -D warnings` passes.
2. `ousia-mqo diff --model-a fixtures/sales_model.json --model-b
   fixtures/sales_model.json` reports all `Agree` verdicts (same model → no
   divergences) and exits 0 — a test asserts `diverge_count == 0`.
3. A synthetic fixture where `revenue` in model A is `BFO:GDC/FinancialProcess`
   and `revenue` in model B is `BFO:Quality/FinancialProcess` produces a
   `Diverge` verdict naming both elements, and the command exits non-zero — a test
   asserts this.
4. Two models with `revenue` (A) and `rev_usd` (B) both grounding to
   `BFO:GDC/FinancialProcess` produce an `Agree` verdict naming both column names
   — a test asserts this semantic-equivalence-despite-name-difference case.
5. `--format json` emits the documented `Vec<ElementVerdict>` shape; a test
   round-trips it through `serde_json`.
6. A missing `ousia-atscale` makes `diff` exit non-zero naming the missing tool
   (inherits from Grounder).
7. `ousia-mqo diff --help` documents `--model-a`, `--model-b`, `--format`,
   `--dry-ground`.
