# PRD: concord-bridge

Status: Draft v0.1
build_priority: high
build_target: rust-extend
build_into: /home/jsy/wintermute/concord
Vision: visions/concord.md

## TL;DR

A disagreement map is a tool for an analyst; most people just want one honest
page that tells them what's going on. `concord-bridge` is the capstone of the
corpus chain: it takes the `DisagreementMap` (from `concord-cruxes`) and
synthesizes a single balanced, cited brief — what each side actually wants, where
the real disagreement is located, the common ground both sides already share, and
what evidence or concession would move each side. It is the artifact a person
facing a polarized question reads to *understand* it instead of picking a team.
It passes a balance check before it's allowed to emit.

## Why this exists

- **Evidence — it's the vision's stated end-state, item by item.**
  `visions/concord.md` End-state lists exactly four deliverables: strongest case
  per side, a map of where disagreement actually is, the concrete shared values,
  and what would change minds. `concord-bridge` is the stage that renders all four
  into one consumable brief. The earlier stages produce the inputs; this produces
  the output a human reads.
- **Evidence — inputs exist by construction.** Consumes `map.json` from
  `concord-cruxes`, which consumes `concord-steelman`, which consumes
  `concord-corpus` — the final link in the chain.
- **Evidence — balance is checkable, not vibes.** Because the corpus carries
  stance tags and the map carries shared/crux/misunderstanding buckets, a
  deterministic balance check (roughly symmetric coverage per side, every cited
  claim traceable to a `Source` id) can gate the brief before it emits — turning
  "is this fair?" into a test, not a judgement call.
- **Evidence — offline synthesis.** Runs on the local ladder (`qwen3:8b`
  verified 2026-06-05); the whole pipeline stays on-device.

## What this builds

- A `concord-bridge` lib crate in `~/wintermute/concord` + a
  `concord bridge <map.json> [--out brief.md] [--model <name>]` subcommand.
- Reuses the `ConcordModel` trait (`LadderModel` real, `MockModel` test).
- **Bridge engine**: from the map, generate a Markdown brief with fixed sections —
  *What each side wants* (one steelmanned paragraph per stance, cited),
  *Where the disagreement actually is* (the real cruxes, empirical vs value),
  *Common ground* (the shared values), *What would change minds* (per crux). Every
  factual sentence carries a citation to a `Source` id; the renderer appends a
  references list resolving ids to urls.
- **Balance gate** (deterministic, pre-emit): (a) every cited id resolves to a
  `Source` in the corpus; (b) per-stance coverage is within a tunable ratio (no
  side gets <N% of the "what each side wants" section while another dominates);
  (c) the common-ground section is non-empty when the map has ≥1 shared value.
  A brief failing the gate is regenerated once, then the failure is reported
  rather than emitting an unbalanced brief.
- **CLI UX**: `concord bridge <map.json> --out brief.md`; and a convenience
  `concord run "<claim>" --fixtures <dir>` that chains corpus→steelman→cruxes→bridge
  end to end (using whatever model is configured; mock in tests).

## Acceptance criteria

1. `cargo build` / `cargo test` green with the new crate; `concord bridge --help`
   and `concord run --help` work. MSRV 1.85.
2. Given a fixture `map.json` + scripted `MockModel`, `concord bridge` emits a
   `brief.md` containing all four named sections and a resolved references list.
3. **Citation integrity:** a test with a `MockModel` that cites a non-existent
   source id is caught by the balance gate (clause a) and does not emit.
4. **Balance gate:** a `MockModel` response that covers only one stance trips the
   coverage clause (b) → regenerate-then-report path (asserted); a balanced
   response passes and emits.
5. `concord run "<claim>" --fixtures tests/fixtures/<case>` with `MockModel`
   produces a `brief.md` end-to-end, exercising all four stages in one command
   (integration test). Verify the `tests/<entry>.rs` actually runs (per
   `self_orphaned_mock_tests` — assert the integration file appears in cargo output).
6. Full suite passes with no ollama / no network; MockModel only.
7. **Deferred AC (live, manual):** on this laptop, `concord run` on a real
   contested claim with `LadderModel` on `qwen3:8b` produces a brief a neutral
   reader would judge fair to both sides (hand-evaluated).

deferred_acs: [7]

## Depends on

`concord-cruxes` (and transitively steelman + corpus). The last PRD in the chain;
build only after all three have shipped.
