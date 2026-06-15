# PRD: ember-selfheal-hook — the SessionStart hook relights a trapped tracer instead of logging PermissionError

Status: Draft v0.1
build_target: hooks
build_into: /home/jsy/.claude/scripts/ctrace-session-start.sh
Vision: visions/ember.md

## TL;DR

The SessionStart hook tries to `ctrace start` on every session, but when
`tracer.pid` is root-owned-and-dead the bare start raises
`PermissionError`, the error lands silently in `claude-start.err`, and
no tracer comes up — for ~6 days running. This PRD swaps the hook's
launch step from the trap-prone `ctrace start` to the self-healing
`ctrace doctor --fix` (from [[PRD-ember-doctor]]), so a trapped pidfile
is reaped and the tracer relit automatically at session start, within
the existing guardrails. This is the capstone that closes the recurring
`ctrace-tracer-down` docket item for good.

## Why this exists (Phase 1 evidence, 2026-06-15)

- **The hook's launch step is exactly where the trap bites.**
  `~/.claude/scripts/ctrace-session-start.sh:75`:
  ```bash
  if "$ctrace" start --root "$root" --log "$log" >/dev/null 2>"$err"; then
  ```
  When `tracer.pid` is root-owned and dead, this `start` raises
  `PermissionError: Permission denied:
  '/home/jsy/.cache/ctrace/tracer.pid'` (the exact string journaled in
  `claude-start.err` 2026-06-12 and 2026-06-14), the `if` fails, the
  marker is not written, and the session runs untraced.
- **The hook already tries to self-heal — but not this failure.** It
  reaps orphaned tracers (`ctrace-orphan-reap`, line 27-30), reaps a
  stale marker (line 53-57), and stops+replaces an orphaned-root tracer
  (line 62-71). None of these clears a root-owned **pidfile** trap;
  that path was missing because the cure (`doctor --fix`) didn't exist
  yet. This PRD adds the one missing branch by routing the launch
  through `doctor`.
- **Untraced sessions degrade every downstream consumer.** `ctrace
  query`, the scribe rollups in every self-review journal's "Claude
  session activity" section, `ctrace-orphan-reap`, and
  `ctrace-scribe` all read the session logs the hook is failing to
  create. Six days of gaps is six days of blind self-reviews.

## What this builds

A minimal edit to `~/.claude/scripts/ctrace-session-start.sh`. No new
scripts; the heavy lifting is in [[PRD-ember-doctor]].

- **Route the launch through `doctor --fix`.** Replace the line-75
  `"$ctrace" start --root "$root" --log "$log"` with `"$ctrace" doctor
  --fix --root "$root" --log "$log"`, keeping the same
  `>/dev/null 2>"$err"` capture and the same marker-on-success write.
  Because `doctor --fix` is idempotent and exits 0 only on `healthy`,
  the surrounding `if` semantics are preserved — but now a root-owned
  trap is reaped instead of fatal.
- **Preserve every existing guard.** The `claude-build` cgroup skip
  (lines 14-16), orphan-reap, scribe backfill, marker reap, and
  orphaned-root stop/replace logic are untouched. The only behavioral
  change is: a *trapped* pidfile now heals instead of blocking.
- **No new autonomy surface.** `doctor --fix` here runs in the same
  unprivileged SessionStart context that already calls `ctrace start`
  and `ctrace stop`; its single privileged action is the `sudo -n rm`
  reap of a state file ctrace owns the semantics of — strictly less than
  the `sudo -n bpftrace`/`sudo -n kill` the hook already triggers.

## Acceptance criteria

1. `ctrace-session-start.sh` invokes `ctrace doctor --fix` (not bare
   `ctrace start`) for the launch step, preserving the `--root`/`--log`
   arguments and the `>/dev/null 2>"$err"` redirection.
2. Simulating a session start with a **root-owned, dead** `tracer.pid`
   results in a **running** tracer and a written `claude-owns.json`
   marker — no `PermissionError` in `claude-start.err`. (Test: seed the
   trap, run the hook with a known root PID, assert `ctrace status` →
   `running: true` and marker present.)
3. On a session where a tracer is **already healthy/our-own**, the hook
   still early-exits without restarting (existing lines 62-71 behavior
   preserved; the live tracer PID is unchanged).
4. The `claude-build` cgroup skip (lines 14-16), orphan-reap, scribe
   backfill, and stale-marker reap all remain present and functional.
5. The hook still always exits 0 and never blocks Claude startup
   (existing contract, lines 3-4).
6. After this PRD ships, a fresh self-review run finds the tracer
   **running** with no `ctrace-tracer-down` "pending your call" item —
   the recurring finding is closed. (Verification note for /build: this
   AC is observational; confirm against the next self-review journal,
   not a unit fixture.)
