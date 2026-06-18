# PRD: consign-verify — prove the debt actually drained

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/consign
Vision: visions/consign.md

## TL;DR

A drainer that reports "pushed 12 repos" but leaves three still `ahead` is worse
than no drainer — it tells self-review the durability gap is closed when it
isn't. consign-verify is the convergence check: after a drain, re-run the
survey and prove every `auto-ok` repo's push-debt fell to zero. If a repo that
drain reported pushed is still `ahead`, emit a *contradicted* verdict with a
non-zero exit — never a silent green.

## Why this exists

- The undercount this whole vision exists to fix (self-review's "8 unpushed"
  hiding 14–29 real repos) is itself a false-green failure: a check that
  *under-reports* debt reads as "mostly fine." consign-verify must not repeat
  that failure mode in the opposite direction by *over-reporting* success.
- Direct precedent on this box: `headway-verify` refuses to false-close —
  a still-`behind-head` daemon after reload is `contradicted` (exit 1), never
  success (gossip 2026-06-18, vision headway). `answerable`/`threshold` share
  the same contradicted-verdict idiom. consign-verify reuses it.
- Memory `feedback_verify_before_concluding`: never assert an outcome from
  indirect signals; prove the value at the actual point that matters. Here the
  point that matters is the post-push survey, not the push command's exit code.

## What this builds

Extend `~/wintermute/consign` with a `verify` module + subcommand.

- `consign verify [--root <dir>]... [--against <drain-receipt.json>]
  [--format json|table]`:
  - Re-runs `consign survey` (+policy) to get the *current* debt.
  - If `--against` is given, cross-checks: every repo the drain receipt marked
    `pushed=ok` must now be `clean` (or at least not `ahead`/`no-upstream`).
  - Produces a verdict per repo and an overall verdict:
    - `converged` — no `auto-ok` push-debt remains (exit 0).
    - `contradicted` — a repo drain claimed pushed is still `ahead`/`no-upstream`,
      OR `auto-ok` debt remains after a drain that should have cleared it
      (exit 1).
    - `residual` — `auto-ok` debt remains but no drain receipt was supplied to
      contradict (informational; exit 0 with a warning, since we can't prove a
      drain was supposed to clear it).
  - `manual-only`/`private-hold`/`diverged` debt does NOT count against
    convergence (it was never in scope to push).
- A receipt: overall verdict, per-repo before/after class, list of
  contradictions. `--format json` emits it.
- Standalone-useful: `consign verify` with no `--against` is a plain "is the
  fleet mirrored?" check self-review can call to replace its undercounting
  `@{u}` line.

## Acceptance criteria

1. `consign verify` with no `auto-ok` debt remaining returns overall
   `converged` and exit 0. (Fixture: all repos clean.)
2. `consign verify --against <receipt>` where the receipt marks a repo pushed
   but the live survey still shows it `ahead` returns `contradicted` and exit 1;
   the contradiction lists that repo. (Fixture: drain receipt + un-pushed repo.)
3. `auto-ok` debt remaining with no `--against` receipt returns `residual`,
   exit 0, with a warning — it does not claim `contradicted` without evidence a
   drain should have cleared it.
4. `manual-only`, `private-hold`, and `diverged` debt never trigger
   `contradicted` — they are out of convergence scope by design.
5. The verdict is computed from a fresh survey, not from the drain receipt
   alone (assert the survey runs even when `--against` is supplied).
6. `consign verify | head` does not panic (SIGPIPE reset); `--format json`
   emits a stable, parseable verdict object.
7. `cargo test` green (fixtures for converged / contradicted / residual); build
   via /cloudbuild; `consign verify --help` documents the three verdicts and
   their exit codes.
