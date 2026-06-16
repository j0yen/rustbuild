# PRD: ousia-mqo-bind — resolve NL phrases to AtScale measures via BFO class

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/ousia-mqo
Vision: visions/ousia-mqo.md
Depends-on: ousia-mqo-ground (GroundedOverlay); lattice-ground CLI (NL→BFO resolution)

## TL;DR

When a user asks "what is gross margin?" and the AtScale model has `gp_rate`,
`net_margin_pct`, and `gross_profit_ratio`, the current `mqo-catalog-binder`
picks one by string distance from the question — a fragile heuristic. With BFO
grounding, "gross margin" resolves through `lattice-ground` to `BFO:GDC /
FinancialProcess`, and the grounded model overlay exposes exactly which elements
carry that class. This PRD ships `ousia-mqo bind` which resolves a natural-
language phrase to a BFO class via `lattice-ground resolve` and then ranks all
model elements grounded to that class as binding candidates for `mqo-catalog-
binder` to prefer.

## Why this exists

Verified 2026-06-15: `lattice-ground` is installed at `~/.local/bin/lattice-ground`
(v0.1.0). Its `resolve` subcommand takes a natural-language phrase and emits the
best-matching BFO class IRI (e.g., `"gross margin" → BFO_0000033`). The
`ousia-mqo-ground` overlay maps every model element to a `philosophicalGrounding
.iri`. The join is mechanical: resolve the phrase to an IRI, filter annotations by
that IRI, rank by `domainModule` specificity. This is the grounded NL binding the
user named as the second primary value from the federation layer.

The `mqo-catalog-binder` in `mqo-mcp` works from the raw `describe_model` catalog
and has no BFO awareness (verified via `joeyen-atscale/mqo-mcp` README + ARCHITECTURE
section on binder). Adding `ousia-mqo bind` as a pre-binding enrichment step that
narrows the candidate set before the binder runs upgrades binding quality without
rewriting the binder.

## What this builds

Extend `~/wintermute/ousia-mqo`:

- **`src/bind.rs` — `NlBinder` lib**:
  - `BindCandidate { element_name: String, iri: String, domain_module: String,
    label: String, score: f32 }` — one candidate per matching element.
  - `NlBinder::new(lattice_ground_bin: PathBuf, grounder: Grounder)`.
  - `NlBinder::resolve(phrase: &str, model: &GroundedOverlay) ->
    Result<Vec<BindCandidate>>`:
    1. Shell out to `lattice-ground resolve "<phrase>"` → get best BFO IRI (or
       top-3 with scores).
    2. Filter `model.annotations` to elements whose `philosophical_grounding.iri`
       matches any returned IRI.
    3. Rank by: exact IRI match score × `domainModule` specificity (FinancialProcess
       > GDC). Return `Vec<BindCandidate>` sorted descending.
  - `NlBinder::resolve_json(phrase: &str, model: &serde_json::Value) ->
    Result<Vec<BindCandidate>>` — ground the model first, then resolve.
- **`ousia-mqo bind --phrase "<text>" --model <path> [--top N] [--format text|json]`** CLI:
  - Run `NlBinder::resolve`, print top-N candidates (default 5).
  - Text: ranked table (rank, element_name, BFO label, domainModule, score).
  - JSON: `Vec<BindCandidate>` — this is the shape `mqo-catalog-binder` can
    consume as a pre-binding hint when called as a subprocess tool.
  - Exit 0 always (empty candidates = "no grounded match" is informational, not
    an error).
- Tool resolution: resolve `lattice-ground` from `$PATH`; error message names it
  if absent (separate from `ousia-atscale` error).

MSRV 1.85. No new deps beyond ousia-mqo-ground's; `lattice-ground` is a subprocess.

## Acceptance criteria

1. `cargo test --release` passes; `cargo clippy -- -D warnings` passes.
2. `ousia-mqo bind --phrase "revenue" --model fixtures/sales_model.json` with
   both `ousia-atscale` and `lattice-ground` on `$PATH` returns ≥1 candidate
   whose `iri` is `BFO_0000033` — a test asserts this with a stub
   `lattice-ground` that emits a fixed IRI for "revenue".
3. A phrase that resolves to `BFO:temporal_region` returns only elements grounded
   to that IRI (e.g. `order_date`, `ship_time` from the sales fixture) and
   excludes measures — a test asserts the candidate set excludes `revenue`.
4. A phrase with no BFO match (stub `lattice-ground` returns empty) returns an
   empty candidate list and exits 0 — a test asserts the empty path.
5. `--format json` emits the documented `Vec<BindCandidate>` shape; a test
   round-trips it.
6. `--top 3` limits output to 3 candidates; `--top 0` returns all (no cap).
7. A missing `lattice-ground` binary exits non-zero naming the tool; a missing
   `ousia-atscale` binary is also caught before grounding (inherits from Grounder).
