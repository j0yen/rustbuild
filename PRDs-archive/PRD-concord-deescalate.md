# PRD: concord-deescalate

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/concord
Vision: visions/concord.md

## TL;DR

The fastest way to make the other side stop listening is contempt. People send
messages that contain a legitimate ask wrapped in language that guarantees the
recipient defends instead of engages — and the substance never lands.
`concord-deescalate` takes a heated message and rephrases it into a
non-inflammatory, perspective-taking form (observation / feeling / need /
request) that **preserves every substantive ask** while stripping the contempt
that makes the other side stop reading. It is the one concord tool that's useful
entirely on its own — you don't need a corpus or a debate, just a message you're
about to send angry.

## Why this exists

- **Evidence — the vision marks it independent and standalone.**
  `visions/concord.md` Order section: "`deescalate` is independent — it needs only
  the local LLM, so /build can ship it in parallel with the corpus chain." It does
  not consume a `Corpus`, so it has no dependency on corpus/steelman/cruxes/bridge.
- **Evidence — local LLM ladder is live.** `ollama list` (verified 2026-06-05)
  shows `qwen2.5:3b` (the practical fast tier per `reference_local_llm_setup`) —
  rephrasing a short message is well within a CPU-only 3B model's reach, and it
  runs offline so a private/angry message never leaves the machine.
- **Evidence — reuses the workspace's model abstraction.** Once `concord-corpus`
  exists, the `ConcordModel` trait + `LadderModel`/`MockModel` are already in the
  workspace; this PRD reuses them rather than introducing a second model client.
- **Why "preserves the ask" is the hard part.** A naive "make it nicer" rewrite
  drops the request and produces mush. The substantive-ask-preservation check is
  what makes this tool trustworthy rather than a politeness filter.

## What this builds

- A `concord-deescalate` lib crate in `~/wintermute/concord` + a
  `concord deescalate [--in msg.txt | "<message>"] [--model <name>]` subcommand
  that prints the rephrased message (and, with `--explain`, what it changed and why).
- Reuses `ConcordModel` (`LadderModel` real, `MockModel` test).
- **Deescalate engine**: extract the substantive asks/claims from the input
  (deterministic-ish: imperative/question/demand detection + model-assisted
  extraction), then generate a rephrase in observation/feeling/need/request form
  that contains every extracted ask. A deterministic post-check verifies each
  extracted ask still appears (semantically) in the output and that
  contempt-lexicon terms are absent.
- **Contempt lexicon**: a checked-in, documented list of contempt/sarcasm/
  absolutist markers (extensible via a user file). Rule-based stripping is the
  floor; the model rephrase is the ceiling. The lexicon check is deterministic
  and cloud-build-safe on its own.
- **Safety boundary** (addresses the vision's open ethics question for this stage):
  deescalate refuses to launder threats or harassment — if the input contains a
  threat of harm, it does not "politely rephrase" it; it declines with a message.
  This boundary is a documented, tested rule.

## Acceptance criteria

1. `cargo build` / `cargo test` green with the new crate; `concord deescalate --help`
   works. MSRV 1.85. SIGPIPE-safe output.
2. Given an input message and a scripted `MockModel`, `concord deescalate` prints
   a rephrase; with `--explain` it lists the changes. (Deterministic, no live model.)
3. **Ask preservation:** a fixture message with N≥2 distinct substantive asks,
   run through a `MockModel` whose scripted rephrase drops one ask, is caught by
   the post-check (which flags the missing ask) rather than silently emitting a
   lossy rephrase. A complete rephrase passes.
4. **Contempt stripping:** the deterministic lexicon check removes/flags a known
   contempt term in a fixture message independent of the model (asserted without
   the model path).
5. **Safety boundary:** a fixture input containing a threat is declined by the
   documented rule (tested), not rephrased.
6. Full suite passes with no ollama / no network (cloud-build-safe); MockModel only.
   Confirm the test entry file runs in cargo output (per `self_orphaned_mock_tests`).
7. **Deferred AC (live, manual):** on this laptop, `concord deescalate` on a real
   heated message with `LadderModel` on `qwen2.5:3b` produces a rephrase that a
   reader agrees keeps the ask and drops the heat (hand-judged).

deferred_acs: [7]

## Depends on

`concord-corpus` only — for the workspace + `ConcordModel` trait. It does **not**
depend on steelman/cruxes/bridge and can build in parallel with them once the
corpus foundation has shipped.
