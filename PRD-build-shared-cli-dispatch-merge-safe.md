# PRD-build-shared-cli-dispatch-merge-safe

Status: Draft v0.1

build_target: self-mod
build_priority: high
build_into: /home/jsy/.claude/skills/build

## TL;DR

When a single `/build` tick fans out ≥2 rust-extend branches that share one
`build_into` repo, each branch independently appends its new subcommand to the
**same** CLI dispatcher file (`src/main.rs` — the `match`/`clap` subcommand
enum). The serial `worktree-extend.sh integrate` then `--no-ff` merges these
branches one by one, and every branch after the first hits a **merge conflict
on the identical dispatcher lines**. Observed 2026-06-06: of 6 shared-target
branches (concord ×3, quicken ×3), 3 deferred on integrate conflict
(`concord-bridge`, `quicken-remedy`, `quicken-attest`) — they will re-conflict
on every retry tick until the dispatcher edits stop colliding. This PRD adds a
**merge-safe subcommand registration convention** so parallel shared-target
branches never touch the same lines of the dispatcher.

## Problem (observed)

- Each new `concord <sub>` / `quicken <sub>` subcommand is wired by editing the
  central `enum Command { ... }` and the `match cmd { ... }` arms in
  `src/main.rs`. All branches edit the same hunk.
- `worktree-extend.sh integrate` merges `autobuilder/<slug>` into `main`
  serially under the integration lock. First branch lands clean; subsequent
  branches conflict because `main` now has a different edit at the same lines.
- The branch agents correctly defer (exit 4 → keep branch, status
  `in_progress`), but the conflict is **structural**: a retry tick re-runs the
  same free-form edit and re-conflicts. No forward progress without manual
  resolution.
- Secondary finding: the `concord` repo had no `main` branch at all — a prior
  tick's `autobuilder/concord-corpus` was never landed. The first shared-target
  branch had to synthesize `main`. Integration assumes a landed `main` exists.

## Proposed solution

Adopt an **append-only, per-subcommand registration** pattern for any repo that
is a recurring shared rust-extend target, plus a small integrate-time guard:

1. **Dispatcher uses a registry, not a hand-edited match.** Convert the shared
   CLI to a pattern where each subcommand lives in its own module that
   self-registers (e.g. an `inventory`/`linkme` slice, or a generated
   `mod.rs` whose `pub use` lines are appended — never inserted mid-block).
   New subcommands append a single new line at the **end** of a sorted region,
   so two branches adding different subcommands produce non-overlapping diffs
   that git auto-merges.
2. **Anchored-append helper.** Add `scripts/cli-register.sh <repo> <subcmd>
   <module>` that appends (idempotently, at a unique `// @build:subcommands`
   anchor at end-of-list) rather than editing the match body. Branch agents for
   shared targets call this instead of free-form editing `main.rs`.
3. **Integrate-time precheck.** `worktree-extend.sh integrate` gains a
   `--ensure-main` step: if the target repo has no `main` branch, create it from
   the default branch HEAD before the first merge, so a never-landed prior
   branch doesn't force each agent to reinvent `main`.
4. **Conflict-class telemetry.** On exit 4, integrate writes the conflicting
   pathset to the sidecar (`last_error=integrate-conflict:<files>`) so the
   parent can detect "same file conflicted ≥2 ticks" and route those branches
   serially (1-same-target-per-tick) instead of re-fanning them in parallel.

## Acceptance

1. `scripts/cli-register.sh` exists; appends a subcommand line at the
   end-of-list anchor idempotently (re-run = no-op) and is unit-tested.
2. Two simulated branches each registering a *distinct* subcommand into a test
   repo's dispatcher produce diffs that `git merge` auto-resolves with **zero
   conflicts** (regression test for the observed failure).
3. `worktree-extend.sh integrate --ensure-main` creates `main` from the default
   branch HEAD when absent, and is a no-op when `main` already exists.
4. On integrate conflict, the sidecar `last_error` records
   `integrate-conflict:<comma-separated-paths>`.
5. The /build SKILL "Worktree isolation" section documents the merge-safe
   registration convention and instructs shared-target branch prompts to use
   `cli-register.sh` instead of editing the dispatcher match body.
6. Back-compat: a shared target that does NOT adopt the registry (legacy
   hand-edited match) still integrates exactly as today — the helper is opt-in
   per repo, no behavioural change to non-adopting repos.

## Trigger

Phase-6 reflect, 2026-06-06 tick: 3/6 shared-target branches deferred on the
same structural `src/main.rs` integrate conflict; this is the
"repeated failure pattern across ≥2 branches → propose a guardrail" trigger.
