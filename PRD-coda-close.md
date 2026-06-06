# PRD: coda-close — the idempotent actuator that closes summary debt

**Status:** Draft v0.1
**build_target:** rust-extend
**build_into:** `~/wintermute/coda`
**Vision:** visions/coda.md
**deferred_acs:** [8]

## TL;DR

coda-audit can see the debt; nothing yet closes it without a human invoking
`scribe backfill`. **coda-close** is the actuator: it takes the `SweepPlan`,
and for every `Render` action shells `scribe render <log>` to write the missing
`<log>.summary.md`. It is **print-only by default** — `coda close` lists what
it *would* render; `coda close --apply` is the single live-side-effect path.
Idempotent: a second `--apply` with no new orphans renders nothing. This is
the piece that turns "self-review remembers to backfill" into "the debt closes
itself."

## Why this exists

- **The repair exists but is unreliably triggered.** `scribe backfill` works
  (`~/wintermute/ctrace-scribe`), but it only runs when a self-review pass
  invokes it — 50 rendered 06-03, 2 on 06-02, **0 on 06-01** (journals). The
  engine is fine; the *actuation* is missing.
- **The debt only grows otherwise.** 623 orphaned logs as of 2026-06-05; every
  headless tick adds more (SIGKILL beats the SessionEnd hook —
  `ctrace-scribe/README.md`). An automatic, idempotent closer is the only way
  the gap trends to zero instead of up.
- **Print-only-by-default is the proven safety contract.** anchor-reconcile,
  quicken-remedy, and keel-cordon all default to a plan and gate the live path
  behind `--apply`. coda-close follows the same discipline so a hook or timer
  can run `coda close` (dry) safely and only `--apply` writes.
- coda-sweep has shipped (the rust-extend rule); coda-close consumes its
  `SweepPlan` and `LogStore::render`.

## What this builds

Extends the `coda` crate; no new repo.

**`LogStore::render` — the real implementation.** Shells `scribe render
<log.ndjson>` (the engine already writes `<log>.summary.md` beside the log;
coda does not re-implement rendering). `scribe` is resolved from `PATH`
(`~/.local/bin/scribe`); the binary name is a `CodaConfig` knob
(`render_cmd = "scribe"`) so a fork/rename doesn't break coda.

**`coda close`** — the actuator command:

- runs the full pipeline: `FsStore` scan → active-log resolve → `sweep` →
  `SweepPlan`, then acts on every `Render` action.
- **default (no flag):** print-only. Lists each log it *would* render and a
  summary line (`would render N, M already settled`). Exit non-zero if ≥1
  orphan remains unrendered (it's a dry run — debt still exists).
- **`--apply`:** invokes `LogStore::render` for each `Render` action;
  on success the log becomes `Settled`. Prints `rendered N, skipped M, failed
  K`. A per-log render failure is logged and counted (`failed`) but does **not**
  abort the run — one corrupt ndjson must not block closing the other 600.
  Exit zero if all orphans rendered (or none existed); non-zero if any failed.
- **`--limit N`:** cap renders per invocation (the timer/hook can chip away at
  a 600-log backlog without a multi-minute stall); logged when the cap drops
  work (`self_orphaned_mock_tests`-style no-silent-cap discipline — print
  `capped at N, P orphans remain`).
- **`--json`:** machine-readable outcome (`rendered`/`skipped`/`failed`/
  `remaining`).

**Idempotency.** After a successful `--apply`, re-running `coda close` finds
the just-rendered logs `Settled` and renders nothing. An integration test
asserts the second run's `rendered` count is 0.

**No new writes beyond summaries.** coda-close writes only via
`LogStore::render` (i.e. only `scribe` writing `*.summary.md` beside the log).
A scope test asserts coda itself opens no other file for write.

**Deps:** reuse coda-sweep/coda-audit's; `std::process::Command` for the shell.
MSRV 1.85, no let-chains. `sigpipe::reset()` in `main` (`coda close --json |
head`).

## Acceptance criteria

1. `cargo build` / `cargo test` succeed offline; a `tests/close.rs` entry file
   appears in cargo test output (`self_orphaned_mock_tests` guard).
2. `coda close` (no flag) over a `FakeStore` with orphans is **print-only**:
   a `render`-counting fake records **zero** render calls; output lists the
   would-render logs; exit non-zero (debt remains).
3. `coda close --apply` invokes `render` exactly once per `Orphaned` log and
   not for `Settled`/`Fresh`/`Active` logs; output reports `rendered N`.
4. **Idempotent:** after a successful `--apply` against a fake that flips
   rendered logs to `has_summary=true`, a second `coda close` run renders 0 and
   exits zero.
5. A `render` failure on one log (fake returns `Err` for a chosen path) is
   counted in `failed`, does not abort the run, the other logs still render,
   and the command exits non-zero.
6. `--limit N` renders at most N logs, prints `capped at N, P orphans remain`
   when it drops work, and exits non-zero while orphans remain.
7. `coda close --json` emits `rendered`/`skipped`/`failed`/`remaining`
   matching the run; `coda close --json | head -1` does not panic (SIGPIPE).
8. **[deferred — laptop-only]** `coda close --apply` against the real
   `~/.cache/ctrace/sessions/` closes the live debt (orphan count drops to 0,
   or to only the active log) and a second run renders 0. (deferred_acs:[8] —
   needs the real `scribe` binary + the real session backlog; the cloud box
   has neither.)
