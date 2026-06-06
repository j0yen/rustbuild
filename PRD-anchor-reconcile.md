# PRD: anchor-reconcile — re-assert the lost watches, idempotently

**Status:** Draft v0.1
**build_target:** rust-extend
**build_into:** `~/wintermute/anchor`
**Vision:** visions/anchor.md
**deferred_acs:** [6]

## TL;DR

anchor-roots can *plan* the restore; anchor-reconcile *performs* it.
**anchor-reconcile** adds `anchor reconcile --apply`: it runs the same pure
diff anchor-roots produces, then executes the plan through the `WatchBackend` —
re-asserting any `Missing` watch and re-seeding the `wchg` cursor for `Stale`
roots — idempotently, so running it twice is a no-op on the second pass. This
is the machinery the boot oneshot and SessionStart hook (anchor-boot) will
call to retire the hand-re-watch dance for good.

## Why this exists

- **The manual fix repeats every reboot.** Journals 06-01/02/03 each record
  re-watching `~/.claude` + `~/brain` by hand (*"re-watched with fresh
  clocks"*, *"wchg roots RE-WATCHED"* — `recall:01KSTJ291SBXN9ZM890J21CEEA`,
  `01KSTRY6…`). A one-command idempotent reconcile is the obvious automation;
  the boot/session wiring (anchor-boot) needs an `--apply` to call.
- **Idempotence is the safety property.** A boot oneshot + a SessionStart hook
  may both fire close together (`recall:01KSNM4CP4…` — loss seen *between*
  runs, not just at boot); `--apply` must be safe to run repeatedly, asserting
  only what's actually missing and re-seeding only what's actually stale.
- **anchor-roots shipped the pure plan + the `WatchBackend` trait** with
  `watch` / `reseed_cursor` methods already declared but unused. This PRD is
  the first consumer of those mutating methods — small scope because the diff
  logic and the backend abstraction already exist.

## What this builds

Extends `~/wintermute/anchor` (no new repo). Adds `--apply` to a `reconcile`
subcommand and an `apply` module.

- **`anchor reconcile`** (no flag) = print-only: runs `reconcile` and prints
  the `ReconcilePlan` (same output family as `anchor plan`, exit non-zero if any
  `Missing`). Default posture is **print-only** — mirrors quicken-remedy /
  rollout `--dry-run` discipline.
- **`anchor reconcile --apply`** executes the plan: for each `Watch` action call
  `WatchBackend::watch(path)`; for each `ReseedCursor` call `reseed_cursor(path)`;
  `NoOp` / `NoteUndeclared` do nothing. It **never removes** an undeclared live
  root (anchor only adds/refreshes — removal is always the user's call).
- **Idempotence:** after a successful `--apply`, a second `--apply` produces a
  plan whose actions are all `NoOp`/`NoteUndeclared` (nothing left to do).
  Re-asserting an already-watched root through `watch` is itself idempotent
  (watchman `watch` on an existing root is a no-op), but anchor still recomputes
  the plan first so it only calls the backend for genuinely-missing roots.
- **`ApplyReport { attempted: Vec<ReconcileAction>, succeeded: …, failed:
  Vec<(ReconcileAction, String)> }`** — `--format json`; a per-action failure
  (e.g. a path that no longer exists) is reported, does not abort the rest, and
  sets a non-zero exit.
- **The one live AC** (a real `watch`/`reseed_cursor` against a running
  watchman that actually changes `watch-list`) is **deferred** — the cloud build
  box has no watchman daemon and no `wchg`. The pure plan→action mapping, the
  idempotence property, and the `ApplyReport` are all verified against the
  `FakeBackend` offline.
- `sigpipe::reset()` inherited; MSRV 1.85, no let-chains.

## Acceptance criteria

1. `cargo build` and `cargo test` succeed offline against a `FakeBackend`; no
   live watchman/network required for the non-deferred ACs.
2. `anchor reconcile` (no `--apply`) is print-only: it invokes **no** mutating
   backend method (asserted), prints the plan, and exits non-zero on any
   `Missing`.
3. `anchor reconcile --apply` against a `FakeBackend` with two `Missing` roots
   and one `Stale` root calls `watch` exactly twice and `reseed_cursor` exactly
   once, in plan order; an `Undeclared` live root triggers **no** removal call.
4. **Idempotence:** after one `--apply`, a re-`plan` against the
   `FakeBackend`'s updated state yields all `NoOp`/`NoteUndeclared` actions and
   a zero exit; covered by a two-pass integration test.
5. `ApplyReport` is `serde`-(de)serializable; a per-action backend failure is
   recorded in `failed`, does not abort remaining actions, and yields a
   non-zero exit. `--format json` round-trips.
6. *(deferred — cloud box has no watchman/wchg)* On a host with a live
   watchman, `anchor reconcile --apply` against a manifest containing a
   currently-unwatched real directory makes that path appear in `watchman
   watch-list`, and a subsequent `wchg since` on it returns a non-empty delta
   after a touch. Verified by the user on the laptop; gated behind an
   `#[ignore]`-style live test or a documented manual check.
7. The integration test entry file `tests/reconcile.rs` appears in `cargo test`
   output (`self_orphaned_mock_tests` guard).
8. `anchor reconcile --format json | head -1` does not panic on SIGPIPE.
9. README documents `--apply`, the print-only default, the never-remove
   guarantee, and the idempotence property so anchor-boot can call it safely
   from both a boot oneshot and a SessionStart hook.
