# PRD: persona-redline-eval — a real number, not a self-graded test

Status: Draft v0.1
build_target: mixed
Vision: visions/persona.md

## TL;DR

`persona-redline` shipped a forbidden-vocabulary guard and the `redline.rs`
unit tests are green — but those tests are author-written exact-match cases
(`scan_exact_match_returns_hit`, `scan_case_insensitive`, …). They prove the
matcher matches the strings it was handed; they say nothing about whether the
*running local-3b model* actually stops leaking "AI", "computer", or
"algorithm" to Jocelyn in natural conversation. `persona-redline-eval` builds an
**independent held-out corpus** of naturalistic technophobe-trigger prompts —
written without reference to `redline.rs`'s own test strings — drives the live
model through the deployed pipeline, and reports a real pre/post leak rate.
The number is the deliverable.

## Why this exists (Phase 1 evidence, 2026-06-13)

- **The existing tests are tautological.** `wintermute-brain/src/redline.rs`
  tests feed the scanner the exact terms it then asserts are found. Per
  [[feedback_agent_written_fixtures_tautology]]: when the same author writes
  the rule and the fixture, recall/accuracy claims prove nothing. The held-out
  set must be independent of `redline.rs`.
- **No leak rate exists for the model.** `redline` enforces on the *reply
  text*, but the relevant safety question is end-to-end: given a Jocelyn-style
  prompt ("how does this thing know my name?", "is there a computer in here?"),
  how often does local-3b emit a forbidden term, and does enforcement catch all
  of them before TTS? Nobody has measured this. Per
  [[feedback_verify_before_concluding]]: instrument the actual failing path and
  prove the value at the failing step — don't assert safety from green unit
  tests.
- **The pipeline is real and reachable.** `redline::enforce()` runs at
  `src/daemon.rs:2277` against the generated reply; the model tier defaults to
  `local-3b` (the leakiest tier, per `[[project_brain_local_first_ladder]]`).
  This is exactly the surface to measure.
- **No-model honesty.** On a box without the local model (or without network
  for a cloud tier), the harness must **skip with a clear message**, not
  fabricate a pass — same discipline as the homeward eval-harness and the voice
  detectors. A green run on zero samples is a lie.

## What this builds

A new directory `~/wintermute/persona-redline-eval/` (mixed: a small Python
harness + a shell entrypoint), published as `j0yen/persona-redline-eval`:

- **`corpus/triggers.jsonl`** — ≥ 40 hand-authored held-out prompts, each a
  natural utterance from a technophobe elder that *tempts* a technology-framed
  answer, with the forbidden terms it must never surface. Authored fresh for
  this PRD; a header comment asserts independence from `redline.rs` test
  strings. Categories: self-reference ("what are you?"), mechanism ("how do you
  hear me?"), failure ("why didn't you understand?"), capability ("can you
  remember things?").
- **`run_eval.py`** — for each prompt: (a) query the live model through the
  deployed config (forbidden_terms + redline as configured), capturing both the
  raw reply and the post-`enforce` reply; (b) scan both for any forbidden term
  using the *same* `redline::scan` semantics (call the `wmd` binary, do not
  reimplement matching); (c) tally `raw_leaks`, `enforced_leaks`,
  `n_samples`. Reports `raw_leak_rate`, `enforced_leak_rate`, and the list of
  any prompt where an *enforced* reply still leaked (the only real failures).
- **`eval.sh`** — entrypoint: detects model availability; if absent, prints
  `SKIP: no local model (set WM_BRAIN_* / start ollama)` and exits 0 with a
  clearly-marked skip status (not a pass). If present, runs `run_eval.py`,
  writes `results/<n>-summary.json`, and prints the headline numbers.
- **`README.md`** — how to run, how to read the number, and the explicit claim
  that the corpus is held-out.

Dependencies: `wmd` (≥ v0.22.0) for model queries + scan parity; `uv` for the
Python env; a configured local/cloud tier at runtime.

## Acceptance criteria

1. `corpus/triggers.jsonl` contains ≥ 40 well-formed records, each with a
   prompt and a non-empty `must_not_say` list; a test asserts none of the
   prompt strings appear verbatim in `wintermute-brain/src/redline.rs`.
2. `eval.sh` on a box with **no** model exits 0 and prints a line beginning
   `SKIP:` — and writes no `results/*-summary.json` claiming a pass.
3. `eval.sh` with a model present produces `results/<n>-summary.json`
   containing `n_samples`, `raw_leak_rate`, `enforced_leak_rate`, and a
   `residual_failures` array.
4. The scan step shells out to `wmd` (or the shipped `redline` matcher) for
   term detection — `run_eval.py` does not contain its own forbidden-term
   regex, so eval and production agree by construction.
5. `enforced_leak_rate` and `residual_failures` are computed against the
   post-`enforce` reply, not the raw reply; both raw and enforced rates are
   reported so the value redline adds is visible.
6. Running twice with a fixed `--seed`/temperature config produces the same
   `n_samples` and a deterministic corpus pass (model sampling may vary; the
   harness records the config used).
7. README states plainly that the corpus is held-out and why that matters
   (cites the tautology hazard).

## Out of scope

- Improving the model or the forbidden list based on findings (that's a
  follow-on PRD once a number exists).
- The PetFace-style external dataset path — this corpus is hand-authored and
  self-contained.
- Changing `redline.rs` (consumed, not modified).
