# PRD: homeward-eval-harness — run the eval, publish the number, no tautology

Status: Draft v0.1
build_target: mixed
build_into: /home/jsy/wintermute/homeward
Vision: visions/homeward.md

## TL;DR

`eval.py` is a complete, honest held-out retrieval harness — Rank-1/5/20 and mAP
over a gallery/query split — and it has **never been run** and **published no
number**. The vision calls honest held-out accuracy "non-negotiable," rooted in
[[feedback_agent_written_fixtures_tautology]] (a wm-router safety claim of 100%
collapsed to 73.5% on a truly held-out set). A matcher with an unrun eval is
exactly that unproven claim. This PRD makes the harness a real, runnable CLI with
a disjoint-individual guardrail enforced in code, a permissively-licensed
correctness fixture that proves the *harness* computes Rank-k correctly, and a
committed `EVAL.md` that records the real number plus an explicit, documented path
to the research-gated datasets that produce the headline figure.

## Why this exists

Phase-1 live inspection (2026-06-13, `homeward/embed/homeward_embed/eval.py`):

- The harness is real and well-shaped: `eval.py:1` "Honest held-out evaluation
  harness," computes "Rank-1 / Rank-5 / Rank-20 retrieval accuracy and mAP," and
  even warns (`eval.py:113`) to "Use a truly held-out split (no shared individuals
  between gallery and queries)." But that warning is **prose, not an assertion** —
  nothing in code *enforces* gallery/query individual-disjointness.
- `eval.py:35` is explicit: "PetFace and Flickr-Dog datasets are research-gated /
  not bundled." So the headline accuracy figure depends on a dataset `/build`
  cannot legally fetch unattended. Pretending otherwise would itself be the
  tautology the vision forbids.
- Net: the harness exists, can't currently be *run* by anyone (no CLI invocation
  is wired into the workspace tooling, no fixture to run it against, no published
  output), and has no in-code guard against the precise inflation trap the user
  named.

## What this builds

- **A disjoint-individual guard in code**: `eval.py` raises (not warns) if any
  individual id appears in both the gallery and the query split. The harness
  *refuses* to score a tautological split. This is the load-bearing honesty fix.
- **A runnable CLI**: `homeward-embed eval --dataset-dir <path> [--ks 1,5,20]`
  loads the documented `EvalSample` dataset contract, runs retrieval through the
  real embedder, and writes a JSON result + a human-readable summary.
- **A bundled correctness fixture** (`embed/fixtures/eval-smoke/`): a tiny,
  permissively-licensed multi-photo-per-individual set (a few CC/public-domain
  animals, ≥2 photos each, organized as disjoint gallery/query) whose ONLY purpose
  is to prove the harness *arithmetic* is correct (e.g. an identical-image gallery
  entry must score Rank-1=1.0; a known-different individual must not). `EVAL.md`
  states plainly that this fixture validates the harness, **not** real-world
  re-ID accuracy.
- **`EVAL.md`**: records (a) the harness-correctness fixture result, (b) the
  measured number on this box's embedder if any obtainable held-out set is run,
  and (c) the explicit manual/partnership step to drop in PetFace
  (`--dataset-dir`) for the headline figure — so the honest gap is documented, not
  hidden behind a green check.

### Non-goals

- No v2 ArcFace fine-tune (a separate, research-licensed effort).
- No claim of production accuracy from the smoke fixture — it proves the harness,
  explicitly and in writing.
- No automated download of research-gated datasets.

## Acceptance criteria

1. `eval.py` raises a clear error if any individual id is shared between the
   gallery and query splits; a unit test feeds a deliberately-overlapping split
   and asserts the raise.
2. `homeward-embed eval --dataset-dir embed/fixtures/eval-smoke` runs end-to-end
   through the real embedder and writes both a JSON result and a printed summary.
3. On the correctness fixture, an identical gallery/query image yields Rank-1 =
   1.0, and a known-different individual is ranked below it — asserted in a test.
4. `EVAL.md` is committed and states, in plain language, that the bundled fixture
   validates the harness arithmetic and is NOT a real-world accuracy claim, and
   documents the exact `--dataset-dir` step to run PetFace held-out.
5. Every fixture image's source URL and license is recorded in
   `embed/fixtures/eval-smoke/SOURCES.md`; nothing without a documented permissive
   license is committed.
6. `uv run pytest` for the embed subtree passes, including the disjointness-guard
   test and the fixture-correctness test.
