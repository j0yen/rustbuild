# PRD: vicious-circle-crown — crown the bon mot of the round

**Status:** Draft v0.1
**build_target:** rust-extend
**build_into:** `~/wintermute/vicious-circle`
**Vision:** visions/vicious-circle.md

## TL;DR

A round produces dozens of lines — verdicts and cross-verdicts — and almost all
of them are noise. The Round Table's whole social engine was that one line a day
*landed*, got repeated, got printed. This PRD crowns it: given a round's
verdicts and roasts, it surfaces the single best line — the day's `bon mot` —
ranked by score and by how the peers reacted to it, attributed to its author.

## Why this exists

The umbrella vision `visions/roundtable.md` makes the bon mot the payload of the
whole system: "The single best line of the day is crowned, attributed, and
published into a running column." That column (`conning-tower`) and the
periodical (`new-yorker`) downstream both consume exactly one crowned line per
round. Without `crown`, the circle critiques but never *publishes* — it stays as
write-only as the `~/wintermute/day-haiku` / `~/wintermute/letters-we-never-sent`
artifacts it was built to answer. Crown is the moment the conversation produces
something worth keeping.

## What this builds

Extends `~/wintermute/vicious-circle`.

Modules:
- `src/crown.rs`:
  - `BonMot { line: String, author: String, target: String, score: f32,
    peer_reaction: f32, rank: f32 }` (serde).
  - `fn crown(verdicts: &[Verdict], crosses: &[CrossVerdict]) -> Option<BonMot>`:
    for each verdict, compute `peer_reaction` = mean `agreement` of all
    `CrossVerdict`s whose `about_persona`/`about_verdict_target` match that
    verdict, then `rank = score + W_PEER * peer_reaction_scaled`. Pin the
    formula (`W_PEER`, the peer-reaction scaling to the score range) as a
    documented const so the crown is reproducible. Return the highest-ranked
    verdict as the `BonMot`; `None` only on empty input.
  - Deterministic tie-break: on equal `rank`, prefer the verdict with the
    higher `peer_reaction`, then lexicographically by `author` id.
- `src/main.rs` — add the `crown` subcommand.

CLI:
- `vicious-circle crown <verdicts.json> <crosses.json>` — print the crowned
  bon mot: line, author, score, peer reaction, rank.
- `vicious-circle crown <artifact-path> --round` — convenience: run `review`
  then `roast` internally, then crown, in one pass (the full daily round).
- `--json` on either form emits the `BonMot`.

## Acceptance criteria

1. `cargo build` and `cargo test` succeed in `~/wintermute/vicious-circle`.
2. `crown` over a hand-built set where one verdict has a strictly higher score
   and strictly more celebratory peer reactions returns that verdict's line as
   the `BonMot`.
3. The ranking formula is a documented `const` (`W_PEER` and the peer scaling);
   a test reproduces the exact `rank` value for a fixed input.
4. Tie-break is deterministic: a test with two equal-`rank` verdicts asserts the
   documented tie-break (higher `peer_reaction`, then author id) picks one
   stably across repeated runs.
5. `peer_reaction` for a verdict equals the mean `agreement` of the
   cross-verdicts about it; a unit test checks the average against a known set.
6. `crown` on empty verdicts returns `None`; the CLI reports "no quorum" and
   exits non-zero.
7. `vicious-circle crown <file> --round` runs review → roast → crown end-to-end
   from an artifact text file and prints exactly one bon mot.
8. `crown --json` emits a `BonMot` that re-parses through serde.
