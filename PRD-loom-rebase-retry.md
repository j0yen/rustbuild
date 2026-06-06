# PRD-loom-rebase-retry

Status: Draft v0.1

build_target: self-mod
build_priority: high
build_into: /home/jsy/.claude/skills/build
Vision: visions/loom.md

## TL;DR

When a `/build` tick fans out ≥2 same-target rust-extend branches, the serial
`worktree-extend.sh integrate` merges them one by one. The first lands clean;
the next conflicts on the shared lines, and today `integrate` immediately
`git merge --abort`s and `die 4`s, leaving the branch deferred. A retry tick
re-runs the same edit against the same `main` and **re-conflicts forever** — no
forward progress without a human. This PRD makes the loser **self-heal**: on
merge conflict, rebase the branch onto the just-merged `main` HEAD, re-check it
compiles, and retry the merge once. "Stalls forever" becomes "lands one tick
later."

## Why this exists

- `~/wintermute/build-skill/scripts/worktree-extend.sh:81-84` — `cmd_integrate`
  runs `git merge --no-ff --no-edit "$branch"`; on failure it does
  `git merge --abort` then `die 4 "merge conflict integrating … (branch kept for
  next tick)"`. There is no rebase and no retry. The deferred branch is based on
  the **old** `main` HEAD (set at `add` time, line 59), so the next tick's
  integrate re-attempts the identical merge and conflicts identically.
- gossip 2026-06-06T06:39:07Z names this exactly: `quicken-attest` and
  `anchor-probe` "kept for next tick rebase" — the rebase was assumed but never
  implemented. journal 2026-06-05 logs both deferrals.
- The gossip "build reflect candidate" proposes the fix verbatim: "integrate
  should auto-rebase the loser onto the just-merged HEAD and retry."

## What this builds

A self-healing branch in `cmd_integrate` (`worktree-extend.sh`). After the
`git merge --no-ff` conflict on line 81, instead of aborting straight to exit 4:

1. `git merge --abort` to restore `main` (as today).
2. Rebase the branch onto current `main`:
   `git -C "$repo" rebase main "$branch"` **run inside the branch's worktree**
   (the worktree at `$WT_ROOT/<repo>-<slug>` is where the branch is checked out;
   `main` is checked out in the primary tree, so the rebase must target the
   worktree's checkout, not the locked primary tree).
3. If the rebase reports conflicts → `git rebase --abort`, then `die 4` exactly
   as today (genuine semantic collision; defer for the serial-fallback PRD to
   route). Telemetry: write `last_error=integrate-conflict:<files>` (shared
   sidecar key with `build-shared-cli-dispatch-merge-safe`).
4. If the rebase is clean → run a cheap **post-rebase guard**:
   `cargo check --offline` (or `--quiet`) in the worktree. The branch already
   passed the full gate pre-rebase; `check` only catches a rebase that
   auto-resolved into non-compiling code (e.g. two append-only edits that are
   individually fine but reference each other). If `check` fails →
   `git rebase --abort`, `die 4` with `last_error=rebase-broke-build`.
5. If the guard passes → re-attempt `git merge --no-ff --no-edit "$branch"`. It
   is now a fast-forward / clean merge (branch sits on top of `main`). Continue
   to the existing version-bump + changelog path unchanged.

Constraints / shape:
- One retry only (rebase → check → merge). If the second merge still conflicts
  (should be impossible after a clean rebase, but guard anyway) → `die 4`.
- The whole sequence stays **inside the existing per-repo integration flock**
  (line 72-73) — no concurrency change; serial integrate semantics preserved.
- Identity for any rebase commits stays Joe Yen (`GIT_ID`, line 41).
- Add a `--no-rebase` integrate flag that preserves today's abort-immediately
  behaviour, for back-compat / debugging.
- sigpipe/exit-code discipline unchanged; this is bash, exits stay 0/2/3/4/5/6.

## Acceptance criteria

1. `worktree-extend.sh integrate`, on a merge conflict that a rebase onto `main`
   resolves cleanly, rebases the branch, passes `cargo check`, completes the
   merge, bumps the version, and exits 0 — verified by a test harness with two
   throwaway branches whose conflict is rebase-resolvable (e.g. each appends a
   distinct line at the same end-of-file anchor).
2. On a conflict the rebase **cannot** resolve (both branches edit the same line
   incompatibly), integrate `git rebase --abort`s, leaves the branch intact
   (still `autobuilder/<slug>`), and exits 4 — no partial merge, `main` clean.
3. When the rebase succeeds but `cargo check` fails, integrate aborts the rebase,
   leaves the branch intact, exits 4, and the sidecar records
   `last_error=rebase-broke-build`.
4. The rebase-conflict path writes `last_error=integrate-conflict:<comma-sep
   paths>` to the sidecar (same key/format the dispatch-merge-safe PRD reads).
5. `integrate --no-rebase` reproduces today's behaviour exactly (abort + exit 4
   on first conflict, no rebase attempt).
6. The rebase is performed against the branch's **worktree checkout**, never the
   primary `main` working tree, and the primary tree is left on `main` and clean
   regardless of outcome (no detached HEAD, no leftover MERGE_HEAD/rebase state).
7. The /build SKILL "Worktree isolation / integrate" section documents the
   rebase-retry behaviour and the `--no-rebase` escape hatch.
