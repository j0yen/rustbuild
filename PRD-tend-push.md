# PRD: tend-push

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/tend
Vision: visions/tend.md

## TL;DR

A handful of fleet repos carry **committed work that never reached the remote** —
self-review lists `rollout(1)`, `wintermute-desktop(2)`, `build-skill(1)` ahead
of upstream, run after run — and that work silently never lands. `tend push
--plan` lists the repos that are clean-tree **and** ahead of upstream, prints
the exact `git push` command for each, and marks each ff-safe vs diverged. It is
proposal-only — it never pushes (pushing touches shared remote state and stays
human-gated).

## Why this exists

Captured live during the dream that drafted this PRD (2026-06-08):

- The 2026-06-08 journal lists, under "Unpushed commits," `build-skill(1)`,
  `rollout(1)`, `wintermute-desktop(2)` — and the 2026-06-07 reflection lists
  `rollout(1)`, `wintermute-desktop(2)` too. The same committed-but-unpushed
  work is reported across runs; nothing surfaces it as actionable, so it sits.
- A live survey this pass confirmed `rollout` ahead 1 (clean tree),
  `wintermute-desktop` ahead 2, `build-skill` ahead 1 — the clean-and-ahead case
  is real and recurring.
- Push is a shared-state, human-visible action ([executing-actions-with-care]);
  the established pattern for it on this laptop is propose-don't-do — like
  `tide-window` printing the reboot command without running it. tend-push prints
  the push command and lets the human run it.

## What this builds

`tend-push` rust-extends `~/wintermute/tend` (created by [[PRD-tend-survey]],
classified by [[PRD-tend-classify]]). Do not start until both exist and build.
It is independent of [[PRD-tend-gitignore]] (both consume the classified survey;
neither depends on the other). Conventions inherited: rustc 1.85, no let-chains,
cloud-build-safe, `sigpipe::reset()` in `main()`.

### Modules

- `push.rs`:
  - `propose(report: &FleetReport) -> Vec<PushPlan>` — select repos where the
    working tree is **clean** (no `Source`/`Artifact`/`Generated` dirt; a repo
    with only artifact dirt is *not* clean for push purposes — surface it but
    mark `blocked_by_dirt`) **and** `ahead > 0`.
  - `PushSafety` = `FastForward` (ahead > 0, behind == 0) | `Diverged`
    (ahead > 0 && behind > 0) | `BlockedByDirt`.
  - `PushPlan { repo, branch, upstream, ahead, behind, safety, command:
    String }` where `command` is the literal
    `git -C <repo> push <remote> <branch>` (remote/branch parsed from the
    tracking ref; never `--force`).
  - **Never executes.** No `--apply`/`--confirm`/`--push` flag exists; the only
    output is the printed plan. (Vision open question resolved toward
    always-proposal, matching tide-window / muster-reap / recourse-contest.)
- extend `render.rs`: human groups by safety (ff-safe first, then diverged, then
  blocked-by-dirt) and prints each `command` verbatim so it can be copy-run;
  `--json` emits `Vec<PushPlan>`.
- extend `main.rs`: `tend push [--root <p>]... [--plan]` (`--plan` default/
  implicit; no apply path).

### UX

```
$ tend push
ff-safe (2):
  rollout             ahead 1   git -C ~/wintermute/rollout push origin main
  wintermute-desktop  ahead 2   git -C ~/wintermute/wintermute-desktop push origin main
diverged (0):
blocked by dirt (1):
  build-skill         ahead 1   (working tree dirty — commit/clean first)
2 ff-safe pushes proposed.  tend never pushes; run the commands yourself.
```

### Dependencies

No new crates (reuses survey/classify output + the read-only git readers). No
network — the tracking ref is read locally; no `git fetch` is run (ahead/behind
is reported against the last-known upstream, flagged as such).

## Acceptance criteria

1. `propose` is unit-tested against fixture `FleetReport`s: a clean repo ahead 1
   behind 0 → one `PushPlan` with `safety == FastForward`; a clean repo ahead 2
   behind 3 → `Diverged`; a repo ahead 1 with artifact dirt → `BlockedByDirt`;
   a clean repo ahead 0 → **no** plan.
2. The `command` string is exactly `git -C <abs-repo-path> push <remote>
   <branch>` with remote/branch taken from the upstream tracking ref, and
   **never** contains `--force`/`-f`/`+`; asserted by a test.
3. No execution path exists: a test asserts the binary's command set contains no
   `push` invocation via `std::process::Command` (push appears only as a printed
   string), and the read-only-git guard from [[PRD-tend-survey]] AC7 still
   passes.
4. A repo with no upstream (`upstream: None`) produces **no** `PushPlan` (can't
   propose a push target) and is reported as such, not silently dropped.
5. `tend push --json` emits a stable `PushPlan[]` schema
   (`repo,branch,upstream,ahead,behind,safety,command`) validated by a serde
   round-trip test.
6. Human output prints each ff-safe `command` verbatim on its own line so it is
   directly copy-runnable; diverged and blocked-by-dirt repos are listed but
   without a runnable command.
7. No `git fetch`/network call is made; ahead/behind is computed from local refs
   and the output notes it reflects the last-known upstream.
8. `README.md` documents that tend never pushes, the safety classes, and that
   ahead/behind is local-only (no implicit fetch).
