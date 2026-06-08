# PRD: concord-steelman

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/concord
Vision: visions/concord.md

## TL;DR

People argue against strawmen. The single most common way a disagreement becomes
unresolvable is that each side attacks a caricature of the other instead of the
position a thoughtful proponent would actually defend. `concord-steelman` takes a
`Corpus` (from `concord-corpus`) and, for each stance in it, produces the
*strongest good-faith version* of that argument — the case a proponent would
endorse, not the version that's easiest to knock down. It runs on the local LLM
ladder so no one's argument leaves the machine.

## Why this exists

- **Evidence — the local LLM ladder is live and offline-capable.** `ollama list`
  shows `qwen3:8b`, `qwen2.5:3b`, `llama3.2:3b` (verified 2026-06-05). Steelmanning
  is a bounded generation task that runs on these CPU-only models;
  `reference_local_llm_setup` makes `qwen2.5:3b` the practical fast tier.
- **Evidence — the routing abstraction already exists; don't hand-roll ollama.**
  `~/wintermute/wintermute-brain/src/ladder.rs:53` defines
  `pub trait LocalBackend { async fn generate(&self, model, prompt, sink) -> LocalOutcome }`
  and a `LadderClient` (ladder.rs:200) that already routes local→cloud. The real
  concord model impl wraps that ladder rather than reinventing an HTTP/ollama client.
- **Evidence — the anti-caricature need is the vision's core mechanic.**
  `visions/concord.md` names "people argue against strawmen" as the first
  tractable driver of breakdown; steelman is the stage that directly removes it.
- **Why it must inject the model behind a trait.** `/build` now runs cargo on the
  **cloud box, which has no ollama** (gossip 2026-06-05, `feedback_cloudbuild_over_build`).
  So the live model is never reachable in CI — the deterministic wiring must be
  tested against a mock.

## What this builds

- A `concord-steelman` lib crate added to the `~/wintermute/concord` workspace,
  plus a `concord steelman <corpus.json> [--out steelmanned.json] [--model <name>]`
  subcommand on the existing `concord` binary.
- **`ConcordModel` trait** (defined in a shared `concord-model` crate or in
  `concord-corpus`, reused by all LLM stages): one async method
  `complete(&self, prompt: &str) -> Result<String>`. Two impls:
  - `LadderModel` — wraps `wintermute-brain`'s `LadderClient` / `LocalBackend`
    (real, used at runtime).
  - `MockModel` — returns scripted responses keyed by prompt fingerprint (used in
    all tests; cloud-build-safe).
- **Steelman engine**: for each `StanceSummary` in the corpus, assemble a prompt
  from that stance's strongest sources, ask the model for the argument a proponent
  would endorse, and capture it as a `Steelman { stance, claim, premises[],
  conclusion, cited_source_ids[] }`. Every premise must cite a `Source` id present
  in the corpus — uncited assertions are rejected (citation integrity).
- **Refusal-to-caricature guardrail**: the prompt and a deterministic
  post-check forbid the steelman from containing contempt-lexicon terms or
  "obviously / any reasonable person" dismissals; a stance whose steelman fails
  the check is regenerated once, then flagged rather than emitted weak.
- Output is `Corpus` + `Vec<Steelman>` serialized as `steelmanned.json`, the
  input contract for `concord-cruxes`.

## Acceptance criteria

1. `cargo build` / `cargo test` green in the `concord` workspace with the new
   crate; `concord steelman --help` works. MSRV 1.85.
2. Given a fixture `corpus.json` with ≥2 stances and a `MockModel` scripted to
   return known arguments, `concord steelman` emits one `Steelman` per stance,
   each validating against the schema. (Deterministic — no live model.)
3. **Citation integrity:** a unit test feeds a `MockModel` response asserting a
   premise that cites a source id *not* in the corpus; the engine rejects/flags
   it rather than emitting it. A valid response with in-corpus citations passes.
4. **Anti-caricature check:** a `MockModel` response containing a
   contempt-lexicon term is caught by the deterministic post-check and triggers
   the regenerate-then-flag path (asserted in a test).
5. The entire test suite passes with **no ollama and no network** (runs on the
   cloud box). A test confirms no `ConcordModel::complete` reaches a real backend
   under the mock.
6. **Deferred AC (live, manual / not cloud-gated):** on this laptop, running
   `concord steelman` against a real corpus with `LadderModel` on `qwen2.5:3b`
   produces a coherent steelman a proponent would plausibly endorse. Marked
   `deferred_acs` — verified by hand, never gated in CI (per
   `self_deferred_acs_inline_only`, expressed as inline bare ints).

deferred_acs: [6]

## Depends on

`concord-corpus` must have shipped (the `Corpus` schema + workspace must exist),
per `visions/concord.md` ordering and the rust-extend `build_into` rule.
