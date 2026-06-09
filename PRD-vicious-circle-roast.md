# PRD: vicious-circle-roast — personas roast each other's verdicts

**Status:** Draft v0.1
**build_target:** rust-extend
**build_into:** `~/wintermute/vicious-circle`
**Vision:** visions/vicious-circle.md

## TL;DR

The real Vicious Circle's signature wasn't reviewing art — it was reviewing each
other. Parker would skewer Woollcott's enthusiasm; Kaufman would needle
everyone's structure. This PRD builds the mutual roasting: given the verdicts
from a `review` round, each persona reacts to the *other* personas' verdicts and
emits a `CrossVerdict` — the table arguing with itself, which is where the wit
actually lived.

## Why this exists

`visions/roundtable.md` describes the Round Table as voices that "critiqued each
other's work mercilessly and wittily." The single-pass `review`
(`PRD-vicious-circle-review.md`) answers the artifact; it does not yet make the
table *converse*. Without the roast, vicious-circle is five parallel monologues,
not a circle. The roast is what makes it social — the missing layer the umbrella
vision says turns write-only output into a conversation.

Reactions reuse the deterministic, tone-templated approach established in
`PRD-vicious-circle-review.md` (in the spirit of `~/wintermute/concord`'s
`concord-deescalate/src/prompt.rs` tone machinery): a persona's reaction is keyed
on its own stance crossed with the *target verdict's* stance (Parker reacting to
Woollcott reads differently than Parker reacting to Ferber).

## What this builds

Extends `~/wintermute/vicious-circle`.

Modules:
- `src/cross.rs` — `CrossVerdict { from: String, about_persona: String,
  about_verdict_target: String, line: String, agreement: f32 }` (serde).
  `agreement` in `-1.0..=1.0` (−1 savage, +1 celebratory).
- `src/roast.rs` — `fn roast(personas: &Registry, verdicts: &[Verdict]) ->
  Vec<CrossVerdict>`: for each ordered pair (reactor != author), produce one
  `CrossVerdict` reacting to that author's verdict. Line template keyed on
  `(reactor.stance, author.stance)`; `agreement` derived deterministically from
  the stance pairing and the author's score.
- `src/main.rs` — add the `roast` subcommand.

CLI:
- `vicious-circle roast <verdicts.json>` — read a `Vec<Verdict>` (the JSON that
  `review --json` emits), produce cross-verdicts, print as a table
  (from → about → line, agreement).
- `vicious-circle roast <verdicts.json> --json` — emit `Vec<CrossVerdict>`.
- `vicious-circle roast <artifact-path> --review` — convenience: run `review`
  internally to get verdicts, then roast them in one pass.

## Acceptance criteria

1. `cargo build` and `cargo test` succeed in `~/wintermute/vicious-circle`.
2. Given a `Vec<Verdict>` with one verdict from each of the 5 default personas,
   `roast` produces exactly `5 * 4 = 20` `CrossVerdict`s (every ordered pair of
   distinct personas).
3. No persona roasts its own verdict: a test asserts `from != about_persona` for
   every emitted `CrossVerdict`.
4. `agreement` is always within `-1.0..=1.0`; a test asserts the bound on every
   emitted cross-verdict.
5. `roast` is deterministic: running it twice on the same verdict slice yields
   byte-identical output.
6. Reaction voice depends on both stances: a test shows Parker→Woollcott and
   Parker→Ferber produce different `line`s for the same author score.
7. `roast --json` emits a JSON array re-parsing to `Vec<CrossVerdict>`.
8. `vicious-circle roast <file> --review` runs end-to-end from an artifact text
   file to printed cross-verdicts without an intermediate file.
