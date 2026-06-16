# PRD: muster-subtree-reap

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/muster
Vision: visions/muster.md

## TL;DR

The worker-idempotency fix stops *new* sessions from leaking workers, but it
cannot retroactively undo a subtree that has already rotted — pid 1054's 21
workers and 3 deleted-exe subscribers persist until that session dies. This PRD
extends `muster reap` to collect just the **dead children** of an otherwise-live
session: proposal-only by default, `--confirm`-gated to act, and never
signalling the session root.

## Why this exists

Verified live (2026-06-16): even after the leak source is fixed
(`PRD-agorabus-worker-idempotency-fix`), session 1054 will keep its 21 leaked
workers and 3 deleted-exe subscribers until it exits — and it has been alive 3.4
days. `muster reap` today acts on `orphan`/`stale` *session roots*; it has no
way to clean the dead children of a session that is itself `live` and must not
be killed (it may be an interactive shell jsy is using). The recurring
self-review finding ("3 deleted-exe agorabus subs") is precisely this backlog.
Reaping the dead children — not the session — clears it safely.

## What this builds

Extends `~/wintermute/muster/src/reap.rs`:

1. **`muster reap --subtree`** mode: for sessions flagged `subtree_rot` (from
   `PRD-muster-verdict-subtree-rot`), propose terminating only the rotten
   subtree members — deleted-exe subscribers and the surplus leaked workers —
   leaving the session root and one current worker untouched.
2. **Safety, mirroring the settled stance** (`muster reap` root behavior /
   `mend-bridge` / `recourse-contest`):
   - dry-run by default: prints the exact `kill` commands, signals nothing;
   - `--confirm` required to send any signal;
   - hard refusal to target the session root PID or an `interactive-tty` /
     `live`-timer root, even with `--confirm`;
   - never targets a subtree member younger than a grace window
     (`--grace <secs>`, default conservative) to avoid racing a just-spawned
     worker.
3. **Selection rule:** target = subtree members with deleted-exe true, plus
   workers beyond the single newest generation; the newest worker and all
   present-exe subscribers needed for the live session stay.
4. Output names each target with its reason (`deleted-exe` / `surplus-worker
   gen K`) so the proposal is auditable.

Deps: none new; consumes the `subtree` block and `subtree_rot` annotation.

## Acceptance criteria

1. `muster reap --subtree` (no `--confirm`) prints the kill commands for a
   session's deleted-exe + surplus-worker members and sends **no** signal
   (test asserts a dry-run signal count of 0).
2. With `--confirm`, only the selected subtree members are targeted; the session
   root PID is never in the target set (unit test: a `live` root with 3
   deleted-exe + 20 surplus workers → 23 targets, root excluded).
3. Reap refuses (exits non-zero, signals nothing) if asked to target a session
   root or an `interactive-tty`/`live`-timer root, even with `--confirm`.
4. Subtree members younger than `--grace` are excluded from the target set
   (boundary test at the grace threshold).
5. The newest worker generation and present-exe subscribers required by a live
   session are retained (not targeted).
6. Existing `muster reap` root-level behavior and its tests are unchanged
   (additive `--subtree` path); `cargo test` green; `cargo build` clean.
