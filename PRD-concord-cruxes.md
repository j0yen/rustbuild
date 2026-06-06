# PRD: concord-cruxes

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/concord
Vision: visions/concord.md

## TL;DR

Most "disagreements" are three different things wearing one coat: a few real
value differences, a pile of factual questions that have answers, and a much
larger pile of people simply talking past each other. When you can't tell them
apart, every conversation feels equally hopeless. `concord-cruxes` takes the
steelmanned positions (from `concord-steelman`) and separates them into three
buckets — **shared values** both sides already hold, **real cruxes** where
reasonable people genuinely diverge, and **misunderstandings** (apparent
conflicts that dissolve on clarification). The output is a structured
disagreement map: the single most useful artifact for anyone trying to make a
conversation productive.

## Why this exists

- **Evidence — this is the vision's load-bearing insight.** `visions/concord.md`
  names "mistake misunderstanding for disagreement" as a core driver of
  breakdown and makes crux-separation the stage that converts a pile of arguments
  into a map of *where the disagreement actually is*. Without it, steelmans are
  just two essays; with it, they become navigable.
- **Evidence — the inputs exist by construction.** This stage consumes
  `steelmanned.json` (from `concord-steelman`), which consumes a `Corpus` (from
  `concord-corpus`). It is the third link in the straight dependency chain the
  vision specifies.
- **Evidence — runs offline on the local ladder.** Classification of a pair of
  premises into shared/crux/misunderstanding is a bounded judgement task suited to
  `qwen3:8b`/`qwen2.5:3b` (verified live 2026-06-05); no cloud, no leak.
- **Why a held-out eval matters here specifically.** "Crux vs misunderstanding"
  is exactly the kind of label an edit-agent could write tests to confirm
  tautologically. Per `feedback_agent_written_fixtures_tautology`, the accuracy
  AC validates against an independently authored labelled set.

## What this builds

- A `concord-cruxes` lib crate in the `~/wintermute/concord` workspace + a
  `concord cruxes <steelmanned.json> [--out map.json] [--model <name>]` subcommand.
- Reuses the `ConcordModel` trait from `concord-steelman` (real `LadderModel`,
  test `MockModel`).
- **Crux engine**: pairwise over the steelmanned positions, classify each point
  of apparent contention into:
  - `SharedValue` — a value both steelmans assert or presuppose.
  - `Crux` — a genuine divergence, tagged `Empirical` (resolvable by evidence) or
    `Value` (a difference in what to weight), with a one-line statement of *what
    would change each side's mind*.
  - `Misunderstanding` — the two sides use a term differently or are answering
    different questions; includes the clarifying distinction that dissolves it.
- Output `DisagreementMap { claim, shared_values[], cruxes[], misunderstandings[] }`
  serialized as `map.json` — the input contract for `concord-bridge`.
- **CLI UX**: `concord cruxes <file>` writes the map; `concord cruxes show <map.json>`
  renders a human-readable three-column view (shared / cruxes / misunderstandings).

## Acceptance criteria

1. `cargo build` / `cargo test` green with the new crate; `concord cruxes --help`
   works. MSRV 1.85.
2. Given a fixture `steelmanned.json` and a scripted `MockModel`, `concord cruxes`
   emits a `DisagreementMap` validating against the schema, with every entry
   assigned to exactly one of the three buckets and every `Crux` tagged
   `Empirical | Value` with a non-empty "what would change minds" field.
3. **Independent-eval accuracy:** on a hand-labelled, independently authored
   fixture of ≥12 contention points (labels in a checked-in `expected.json`
   written before the classifier), the engine's bucket assignment matches the
   gold label on ≥70% (the honest bar from `self_recall_baseline_gate_red`-style
   realism; not a tautological 100%). The held-out set is *not* agent-generated
   to pass.
4. A `Misunderstanding` entry always carries the clarifying distinction; a
   `Crux` never lacks the change-minds field (deterministic post-validation,
   tested).
5. Full suite passes with no ollama / no network (cloud-build-safe); MockModel
   only.
6. **Deferred AC (live, manual):** on this laptop with `LadderModel` on
   `qwen3:8b`, the map on a real contested claim correctly separates at least one
   genuine value-crux from at least one terminological misunderstanding (hand-judged).

deferred_acs: [6]

## Depends on

`concord-steelman` (and transitively `concord-corpus`). Build only after both
have shipped and the workspace contains the `Steelman` schema + `ConcordModel`
trait.
