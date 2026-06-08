# PRD: scribe-startup-sweep — close the residual holes a single reap can't

Status: Draft v0.1
build_target: hooks
Vision: visions/scribe.md

## TL;DR

`scribe-reap-wire` makes a SessionStart reap the *one* tracer orphaned by
the immediately-prior session. But the trace directory can hold holes a
single reap never touches: logs from sessions whose marker was already
cleared by an earlier partial reap, several SIGKILLed sessions stacked
since the last successful render, or logs left when `claude-owns.json`
was never written at all. This PRD adds a bounded `scribe backfill` sweep
to the SessionStart hook, run right after the reap, so *any* un-summarized
log self-heals at the next session boundary — independent of the morning
self-review.

## Why this exists

- **Reap is single-target; the backlog is plural.** `ctrace-orphan-reap
  --apply` reconciles the *current* `claude-owns.json` / `tracer.pid`
  pair. It renders that one orphan's log. It does nothing for a
  `*.ndjson` whose owning session died two boots ago, or whose marker a
  prior reap already cleared without rendering. Those accumulate.
- **The backlog is real and large.** Journal 2026-06-06 shows the
  self-review backfill rendering **626** logs in one run (residual 0,
  1251 skipped); 2026-06-07 rendered 7; 2026-06-08 rendered 2. Each of
  those is a hole that existed because nothing closed it between reviews.
  The numbers prove a SessionStart reap alone would still leave a tail.
- **The engine already does exactly this, idempotently.**
  `scribe backfill <dir>` (confirmed via `scribe --help`: "Render all
  `*.ndjson` in a directory that lack a `*.summary.md`") renders only the
  gaps and is a no-op on already-summarized logs — safe to run every
  startup. It is the same renderer self-review calls, so output can't
  diverge.
- **This is component #4's actual promise.** The vision's
  `ctrace-session-end-resilient` was "a SessionStart sweep that backfills
  any prior un-summarized logs before starting the new tracer." It
  shipped a binary but never wired the sweep into the live hook
  (`ctrace-session-start.sh` has no `scribe` call — grep-confirmed). This
  PRD lands the sweep.

## What this builds

A `hooks` change extending `~/.claude/scripts/ctrace-session-start.sh`
(symlink into `~/wintermute/dotfiles/.claude/scripts/`). Depends on
`scribe-reap-wire` having already added the reap step; this sweep goes
immediately after it, still before `ctrace start`:

```sh
scribe=/home/jsy/.local/bin/scribe
if [ -x "$scribe" ]; then
    "$scribe" backfill /home/jsy/.cache/ctrace/sessions \
        >/dev/null 2>>"$err" || true
fi
```

Semantics:

1. **After the reap, before the new tracer.** Reap clears the immediate
   orphan and renders it; the sweep then closes any older residual holes.
   Both run under the same `claude-build` cgroup guard that already
   short-circuits the hook (no sweep inside a build unit's cgroup).
2. **Bounded and non-blocking.** `scribe backfill` only touches
   `*.ndjson` lacking a `*.summary.md`; on a clean directory it does
   near-zero work. Wrapped in `|| true`, stderr to the existing `$err`,
   the hook still always `exit 0` and never delays startup. If the
   typical-case backlog is ever large enough to risk a slow startup, the
   sweep degrades gracefully — a slow render still doesn't block because
   the hook does not wait on its result for the `ctrace start` path.
3. **Degrades when scribe is absent.** Guarded by `-x "$scribe"`; if the
   binary isn't installed the sweep is skipped and behavior is identical
   to today.
4. **Idempotent.** Running the sweep every startup never re-renders an
   already-summarized log and never double-writes.

Constraints: pure shell wiring, no new binary, the `scribe` engine
unchanged. Do not touch `ctrace-session-end.sh` (owned by
`mend-ctrace-render`).

## Acceptance criteria

1. `ctrace-session-start.sh` invokes `scribe backfill
   /home/jsy/.cache/ctrace/sessions` after the reap step from
   `scribe-reap-wire` and before `ctrace start`.
2. With a temp sessions dir containing two `*.ndjson` files and no
   matching `*.summary.md`, running the wired `scribe backfill <dir>`
   renders **both** summaries; a second run renders **zero** (idempotent,
   residual 0) — verified directly.
3. The sweep is guarded by `-x` on the `scribe` binary; with `scribe`
   absent the hook skips the sweep and still starts a tracer and exits 0.
4. The `claude-build` cgroup guard still returns before the sweep line —
   no `scribe backfill` runs inside a `claude-build*` cgroup.
5. On a fully-summarized sessions dir the sweep is a no-op and adds no
   observable startup delay versus the pre-sweep hook.
6. Sweep stderr goes to `~/.cache/ctrace/claude-start.err`, never to
   stdout; the hook exits 0 in every branch.
