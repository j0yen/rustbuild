# PRD: mqo-clarify — ask the disambiguating question instead of guessing

Status: Draft v0.1
build_target: rust-cli
build_into: /home/jsy/wintermute/mqo-clarify
Vision: visions/mqo-trust.md
Depends-on: (soft) mqo-binding-confidence — consumes its margins; runs standalone too

## TL;DR

When a user asks for "margin" and the model has `gross_margin`, `net_margin`, and
`operating_margin` all scoring within a hair of each other, `mqo-mcp` binds one and
moves on — and may answer the wrong question with total fluency. This PRD ships
`mqo-clarify`, whose `ask` subcommand detects when a field's binding is genuinely
ambiguous (≥2 candidates within a confidence margin) and emits a natural-language
disambiguation question plus the option set, so the agent can ask the user rather
than silently choose.

## Why this exists

Verified 2026-06-15: nothing in the 50-crate workspace matches `clarif|ambig|
disambig`. The pipeline is built to *commit* to a bind (the README: "resolve every
ref to an exact unique_name") — which is correct for execution but wrong for the
moment *before* execution when the choice is a coin-flip. The honest behavior for an
agent on production data is to surface the ambiguity, not resolve it by luck. This
tool is the missing "stop and ask" gate, and it composes directly with
`mqo-binding-confidence` (the margin it already computes) — or with any supplied
candidate set so it can run without the confidence tool present.

## What this builds

New repo `joeyen-atscale/mqo-clarify` (binary `mqo-clarify`):

- **`mqo-clarify ask --candidates <file> [--margin <f>] [--max-options <N>]
  [--format json]`**:
  - `--candidates` is a per-field candidate set: `{field: [{unique_name, label,
    score, description?}]}` (the shape `mqo-binding-confidence` emits, or a hand
    supplied set).
  - For each field, if the top candidate's lead over the next is **within**
    `--margin` (default 0.1), mark the field ambiguous.
  - For each ambiguous field, generate a disambiguation question from the
    candidates' labels/descriptions — e.g. `"By 'margin' did you mean: (1) Gross
    Margin, (2) Net Margin, (3) Operating Margin?"` — capped at `--max-options`
    (default 4, most-likely first).
  - Emit `{ambiguous: bool, questions: [{field, question, options:[{unique_name,
    label}]}]}`. If no field is ambiguous, `{ambiguous:false, questions:[]}`.
  - Exit 0 always; `--fail-if-ambiguous` makes it exit non-zero when any field is
    ambiguous (so a pipeline can pause for human input).
- **`mqo-clarify serve`** — `{"tool":"clarify_binding","args":{"candidates":…,
  "margin":…}}` → `{"ok":true,"data":{ambiguous,questions}}`.
- Ships `fixtures/`: an ambiguous candidate set (three close margins) and an
  unambiguous one (clear winner) for tests.

MSRV 1.85. Deps: clap, anyhow, serde/serde_json. Question generation is template-
based and deterministic (no LLM call) so it is testable and offline.
`#![forbid(unsafe_code)]`.

## Acceptance criteria

1. `cargo test --release` passes; `cargo clippy -- -D warnings` passes.
2. `ask --candidates fixtures/ambiguous.json` returns `ambiguous:true` with a
   question naming all close candidates for the ambiguous field — a test asserts
   the option set matches the within-margin candidates.
3. `ask --candidates fixtures/clear.json` returns `ambiguous:false` with no
   questions — a test asserts this.
4. `--margin 0.0` treats only exact-tie candidates as ambiguous; `--margin 0.3`
   widens the ambiguity band — a test asserts a candidate set flips from
   unambiguous to ambiguous as the margin widens.
5. `--max-options 2` caps each question to the top 2 options, most-likely first —
   a test asserts the cap and ordering.
6. `--fail-if-ambiguous` exits non-zero on an ambiguous set and 0 on a clear one —
   tests assert both.
7. `--format json` emits the documented `{ambiguous, questions}` shape and
   round-trips; `serve` handles `clarify_binding` and returns `{ok,data}`, unknown
   tool → `{ok:false,error}` non-zero — tests assert all three.
8. A field with a single candidate is never ambiguous (no question generated) — a
   test asserts the single-candidate path.
