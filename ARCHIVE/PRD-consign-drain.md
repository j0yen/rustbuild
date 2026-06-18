# PRD: consign-drain — push every eligible repo, safely

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/consign
Vision: visions/consign.md

## TL;DR

With survey enumerating push-debt and policy gating eligibility, consign-drain
is the autonomous reconcile loop: for every `auto-ok` repo that is `ahead` or
`no-upstream`, push to its remote — setting the upstream when absent —
serialized, one receipt per repo, never `--force`. It is to un-mirrored commits
what `adopt apply` is to un-installed artifacts. `diverged` and `manual-only`
repos are skipped and surfaced, never auto-resolved.

## Why this exists

- Live evidence (2026-06-18): 7 repos `ahead` of upstream + 5 `no-upstream`
  with unpushed commits = **12** repos that consign-policy would mark `auto-ok`
  for pushing, sitting un-mirrored with no automated drainer. The human runs
  `git push` by hand, repo by repo, which is why the count never reaches zero.
- The reconcile-on-a-loop idiom already works for two sibling problems:
  `adopt apply` drains shipped-but-uninstalled artifacts; `headway` drains
  behind-head daemons. There is no equivalent for git push-debt — drain fills
  that hole.
- Safety precedent: `rollout` serializes restarts and never force-pushes
  state; consign-drain serializes pushes and never force-pushes commits. A
  `diverged` repo (`mqo-narrative-compose`, ahead 2 / behind 2) must be left
  for a human — auto force-push would destroy the 2 behind commits.

## What this builds

Extend `~/wintermute/consign` with a `drain` module + subcommand.

- `consign drain [--root <dir>]... [--dry-run] [--only <name>]...` — runs
  survey+policy, then for each `auto-ok` repo classified `ahead` or
  `no-upstream`:
  - `no-upstream` ⇒ `git push --set-upstream origin <branch>`.
  - `ahead` ⇒ `git push` (fast-forward only; never `--force`/`--force-with-lease`).
  - Serialized (one repo at a time) so a transient auth/network failure is
    attributable and the receipt is ordered.
- `--dry-run` is the **default** (print the push plan, push nothing);
  `--no-dry-run` performs the pushes. Mirrors `adopt apply` / consign-publish.
- `no-remote` repos are delegated to consign-publish (or skipped with a
  pointer note if `--only` didn't include publish scope) — drain does not mint
  remotes. `diverged` and `manual-only` are skipped and listed under a
  "needs human" section.
- Per-repo receipt: name, branch, action (`push | set-upstream | skip`),
  result (`ok | error`), commits pushed, error detail. `--format json` emits
  the receipt array; a clean (zero-debt) run emits an empty/`"nothing to do"`
  result, not noise.
- Never `--force`, never `--force-with-lease`, never rebase, never touch
  working-tree state. Read-then-push only.

## Acceptance criteria

1. `consign drain` (default) is a dry-run: prints the per-repo push plan
   (`push` vs `set-upstream`) for each `auto-ok` `ahead`/`no-upstream` repo and
   performs no pushes. (Assert no `git push` invoked — injected runner.)
2. `consign drain --no-dry-run` pushes an `ahead` fixture repo with plain
   `git push` and a `no-upstream` fixture with `push --set-upstream origin
   <branch>`; receipts record the action and `ok`. (Test against local
   bare-remote fixtures, not GitHub.)
3. No drain code path ever passes `--force` or `--force-with-lease` to
   `git push`. (Assert by inspecting the constructed argv in tests.)
4. `diverged` and `manual-only` repos are skipped and appear in a "needs human"
   section of the output; they are never pushed.
5. A push that fails (auth/network/non-fast-forward) records a structured
   per-repo error and `error` status; the batch continues to the next repo.
6. A clean fleet (no `auto-ok` debt) yields an explicit "nothing to do" result
   with exit 0 and no spurious output.
7. `cargo test` green (bare-remote fixtures + injected runner); build via
   /cloudbuild; `consign drain --help` documents the dry-run default and the
   no-force guarantee.
