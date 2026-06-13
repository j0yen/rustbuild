# PRD: persona-work-eval — an independent number for the work-scope guard

Status: Draft v0.1
build_target: mixed
build_into: /home/jsy/wintermute/persona-work
Vision: visions/persona.md

## TL;DR

`persona-work-redline` builds the gate that blocks personal-scope actions on
the work box. But "the policy file has a rule" is not "the guard actually
blocks the action a real session would attempt." This PRD ships an independent,
held-out corpus of *naturalistically phrased* work-inappropriate actions —
deliberately NOT copied from the redline policy's own match expressions — drives
each through the live guard, and reports a real block rate. It is the work-side
analogue of `persona-redline-eval`, which produced the elder persona's honest
pre/post leak number instead of trusting author-written unit cases.

## Why this exists (Phase 1 evidence, 2026-06-13)

- **Author-written fixtures prove nothing about the running guard.** The elder
  persona's `redline.rs` tests are exact-match unit cases
  (`scan_exact_match_returns_hit`); the vision flagged them as tautological
  ([[feedback_agent_written_fixtures_tautology]]) and shipped
  `persona-redline-eval` to drive the *actual model* on independent prompts.
  `persona-work-redline`'s ACs (per-rule allow/redline checks) have the same
  weakness: they exercise the policy with the exact shapes the policy was
  written to match. A held-out set written by a different hand catches the
  classification gaps — a `git push` phrased as `git push origin HEAD` where
  the remote's j0yen-ness is only discoverable via `git remote get-url`, a repo
  creation via the API rather than `gh repo create`, a voice daemon launched by
  its binary name rather than the word "voice".
- **Verify the failing path, not a theory.** [[feedback_verify_before_concluding]]:
  the only honest claim about the guard is one backed by running the real
  classifier+policy over inputs it has not seen. A number is the deliverable.
- **The harness exists to produce it.** `answerable check --json` returns
  `{verdict, matched_rule, reason}` and the `PreToolUse` hook emits a
  deny/allow decision — both are scriptable, so a corpus runner can record
  per-item verdict and aggregate a block rate without new infrastructure.

## What this builds

Extends `~/wintermute/persona-work/` (published as `j0yen/persona-work`):

- **`eval/corpus.jsonl`** — a hand-authored held-out set (≥ 30 items) of work-
  inappropriate and benign-sibling actions, each as a realistic tool-call JSON
  (the shape the `PreToolUse` hook receives), labeled `expect: block|allow`.
  Authored to phrase forbidden actions *differently* from the redline policy's
  match expressions: indirect remotes, API-based repo creation, binary-name
  daemon launches, skill aliases. Benign siblings (`joeyen-atscale` push, a
  read-only `gh repo view`, a `cargo build`) labeled `allow`. A header comment
  asserts the corpus was NOT derived from `redline-work.toml`.
- **`eval/run-eval.sh`** — feeds each corpus item through the actual guard
  (`hooks/pretooluse-work-guard.sh` with the installed `redline-work.toml`),
  records the verdict, and prints a report: total, blocked, allowed,
  false-allow (expect=block got allow — the dangerous miss), false-block
  (expect=allow got block — the annoyance), and a single headline block-rate on
  the `expect=block` subset. Exit 0 if false-allow count is 0; exit 1
  otherwise.
- **Honest SKIP path.** If `persona-work-redline` is not installed (no
  `redline-work.toml`, or `validate.sh` does not report the `work` identity),
  `run-eval.sh` prints `SKIPPED: work guard not installed` and exits 0 with a
  clearly-marked non-result — never a fabricated pass. This box (the wintermute
  laptop) is the expected SKIP case; the work box is where the real number is
  produced.

### Acceptance criteria

1. `eval/corpus.jsonl` contains ≥ 30 items, each valid JSON with an `expect`
   field of `block` or `allow`, with at least 12 `block` and 8 `allow`.
2. The corpus header documents that items were written independently of
   `redline-work.toml`; a build-time check fails if any corpus item's raw text
   is a substring-copy of a policy match expression (tautology guard).
3. `run-eval.sh` against the installed guard prints total / blocked / allowed /
   false-allow / false-block counts and a headline block-rate, and exits 1 if
   false-allow > 0.
4. On a box where the work guard is NOT installed, `run-eval.sh` prints the
   SKIP message and exits 0 without emitting a numeric pass — verified on this
   wintermute box as the SKIP fixture case.
5. The eval covers every action class in CLAUDE_WORK.md (j0yen push, force
   push, repo creation, auto-publish, voice/agorabus/family launch,
   /build·/dream) with at least one independently-phrased block item each.
6. README documents how to run the eval, how to read the number, and states
   plainly that the meaningful run happens on the work machine.

## Out of scope

- Building the guard — depends on `persona-work-redline` being installed.
- Auto-remediation of a false-allow (the eval reports; fixing the policy is a
  follow-on edit to `persona-work-redline`'s `redline-work.toml`).
- Continuous/scheduled running — that overlaps `persona-work-doctor`.
