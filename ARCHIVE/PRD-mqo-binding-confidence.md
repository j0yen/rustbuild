# PRD: mqo-binding-confidence — a calibrated trust signal on every bound field

Status: Draft v0.1
build_target: rust-cli
build_into: /home/jsy/wintermute/mqo-binding-confidence
Vision: visions/mqo-trust.md

## TL;DR

`mqo-mcp` binds every MQO field to an exact `unique_name` and then answers — but it
never tells the agent *how sure* the bind was. A field that matched one obvious
candidate and a field that barely edged out three lookalikes are reported
identically, so the agent can't know which numbers to trust or double-check. This
PRD ships `mqo-binding-confidence`, whose `score` subcommand attaches a calibrated
confidence (0–1) and a ranked alternative list to each bound field, turning a
silent bind into an inspectable one.

## Why this exists

Verified 2026-06-15: `mqo-param-validator` rejects *unmapped* fields
("lookalike_measure, wrong_hierarchy_level, …") — a binary accept/reject — and
`mqo-catalog-binder` resolves refs to `unique_name`s. Neither emits a *graded*
confidence. Grepping all 50 crates for `confidence|calibrat` returns nothing. The
README's whole thesis is that the dangerous failures are *silent and coherent* —
a lookalike measure that binds cleanly but answers the wrong question. A confidence
score is the signal that surfaces exactly that risk: "this bound, but only just."

## What this builds

New repo `joeyen-atscale/mqo-binding-confidence` (binary `mqo-binding-confidence`):

- **`mqo-binding-confidence score --bound <file> [--candidates <file>] [--catalog
  <file>] [--format json]`**:
  - For each bound field, compute a confidence from deterministic features:
    - **Name match strength**: similarity between the requested phrase/label and
      the chosen `unique_name`/label (exact > token-subset > fuzzy).
    - **Margin**: gap between the chosen candidate's match score and the
      runner-up's (a large gap = high confidence; a thin gap = low).
    - **Uniqueness**: how many catalog entries are plausible lookalikes (more
      lookalikes = lower confidence).
    - **Structural fit**: the field's role matches its position (a measure bound
      to a measure slot, a level to a hierarchy slot).
  - Combine into a 0–1 confidence with a documented, deterministic formula
    (weighted, monotonic in each feature). Bucket into `high|medium|low`.
  - Emit per field: `{field, bound_unique_name, confidence, bucket, alternatives:
    [{unique_name, score}]}` and an overall `min_confidence` for the MQO.
  - Exit 0 always (scoring is informational); a `--fail-below <f>` flag makes it
    exit non-zero if any field's confidence is below the threshold (CI gate).
- **`mqo-binding-confidence serve`** — `{"tool":"score_binding","args":{"bound":…,
  "candidates":…}}` → `{"ok":true,"data":{fields:[…],min_confidence}}`.
- Ships `fixtures/`: a confident bind (clear winner) and a shaky bind (thin margin,
  many lookalikes) for tests.

MSRV 1.85. Deps: clap, anyhow, serde/serde_json, a small string-similarity helper
(e.g. strsim). `#![forbid(unsafe_code)]`.

## Acceptance criteria

1. `cargo test --release` passes; `cargo clippy -- -D warnings` passes.
2. `score --bound fixtures/confident.json` returns every field with
   `bucket:"high"` and `min_confidence ≥ 0.8` — a test asserts the buckets.
3. `score --bound fixtures/shaky.json` returns at least one field with
   `bucket:"low"` and a non-empty `alternatives` list — a test asserts the low
   bucket and that runners-up are listed.
4. The confidence formula is monotonic: increasing the margin (widening the gap to
   the runner-up) never decreases confidence — a property-style test asserts this
   on synthetic inputs.
5. `--fail-below 0.5` exits non-zero when a field scores below 0.5 and exits 0
   otherwise — tests assert both branches.
6. `--format json` emits the documented `{fields, min_confidence}` shape and
   round-trips through `serde_json`.
7. `serve` handles `score_binding` and emits `{ok,data}`; unknown tool →
   `{ok:false,error}` non-zero — tests assert both.
8. A bound field with no candidates/catalog context degrades to a documented
   default confidence (not a panic, not a fabricated 1.0) — a test asserts the
   degraded path.
