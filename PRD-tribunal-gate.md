# PRD: tribunal-gate — a broken conscience cannot reach another machine

Status: Draft v0.1
build_target: mixed
build_into: /home/jsy/wintermute/tribunal
Vision: visions/tribunal.md

## TL;DR

`herald` promises that the `/conscience` skill it publishes is trustworthy
(end-state #4: "an 'ethics skill' isn't shipped broken or unverifiable") but
contains no mechanism to make that true — only structural `skill-doctor` /
`skill-manifest` checks. This PRD is the mechanism: `tribunal gate` composes the
conformance suite and the bench into a single publish precondition, and wires it
into `herald`'s pack/publish flow so `/conscience` (herald-conscience) **cannot
be published unless the ontology conforms and the reasoner clears the held-out
corpus with zero false-allows.**

## Why this exists

- `visions/herald.md` end-state #4 requires every published plugin to be
  non-"broken or unverifiable," yet `PRD-herald-pack.md` / `PRD-herald-market.md`
  / `PRD-herald-conscience.md` ship only structural validation. "The SKILL.md is
  well-formed" is not "the ethics it encodes are correct." This PRD closes the
  gap herald opened.
- Shipping an *ethics* engine outward is the highest-stakes place on this laptop
  to let the agent-graded-on-its-own-homework tautology stand
  (`feedback_agent_written_fixtures_tautology`). A publish gate that consumes an
  *independent* corpus (tribunal-corpus) is the structural answer.
- Composing rather than duplicating: `tribunal-conformance` and `tribunal-bench`
  already produce machine-readable verdicts; the gate is the thin policy layer
  that turns two JSON reports into one go/no-go and binds it to herald's flow.

## What this builds

- **`tribunal gate` subcommand** (in `~/wintermute/tribunal`):
  ```
  tribunal gate --owl world.owl --guard ousia-guard --corpus corpus/ \
                --min-accuracy 0.85 --policy gate-policy.toml
  ```
  Runs `conformance` then `bench`, applies the policy, prints a one-screen
  verdict, and exits 0 (publish allowed) / non-zero (blocked) with the reason.
- **Policy (`gate-policy.toml`, checked in):**
  - `conformance` must be all-`pass`.
  - `false_allow == 0` — **hard, non-overridable** (a deny-class case waved
    through blocks publish unconditionally).
  - `overall_accuracy >= min_accuracy` (default 0.85, ratchets up over passes per
    the vision's open question).
  - `per_tenet_min_n >= 2` (refuse to bless a corpus that silently under-covers a
    tenet — no false confidence from thin coverage).
- **herald wiring (the "mixed" half):** an `install`/`publish` hook for
  `herald-pack` / `herald-market` such that packaging or publishing
  `/conscience` invokes `tribunal gate` against the bundled `ousia-guard` +
  ontology + corpus and **aborts the publish on non-zero exit**. Delivered as a
  small shell shim + the documented integration point (herald-pack's publish
  step calls `tribunal gate` before writing the marketplace entry). Because
  herald is also unbuilt, this PRD ships the gate binary + a `--dry-run` that
  proves the abort path, plus the documented hook; the live wire-in lands at
  herald-conscience's publish AC (cross-vision, noted in the vision).
- **Deps:** reuses the tribunal crate; `clap`, `serde`, `toml`. SIGPIPE reset.

## Acceptance criteria

1. `tribunal gate` with a passing fixture set (conformant `.owl`, stub guard
   scoring above threshold, zero false-allow, balanced corpus) exits 0 and
   prints `PUBLISH ALLOWED`.
2. `tribunal gate` exits non-zero and prints the blocking reason when (a)
   conformance fails, (b) `false_allow > 0`, (c) accuracy < `min_accuracy`, or
   (d) any tenet has `N < per_tenet_min_n` — one AC sub-case each, driven by
   fixtures.
3. The `false_allow == 0` rule is non-overridable: a config attempting to set a
   non-zero false-allow tolerance is rejected with an explicit error (it is not
   a tunable).
4. A `tribunal gate --dry-run` against a stub herald publish flow demonstrates
   the publish is aborted on a non-zero gate and proceeds on a zero gate
   (proving the integration contract before herald-pack lands).
5. The documented herald hook (publish step → `tribunal gate` → abort-on-fail)
   is committed as a runnable shim + README section that herald-conscience's
   publish AC can reference.
6. `cargo test` green on rustc 1.85; gate composes the real `conformance` and
   `bench` code paths (not reimplemented); no panic when piped to `head`.
