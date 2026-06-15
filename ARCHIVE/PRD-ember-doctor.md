# PRD: ember-doctor — one idempotent command that diagnoses and cures the tracer

Status: Draft v0.1
build_target: shell
build_into: /home/jsy/.local/bin/ctrace
Vision: visions/ember.md

## TL;DR

Even after ownership and liveness are fixed, the docket and SessionStart
hook need a *single, guarded, idempotent* command that says what's wrong
with the tracer and — on request — fixes it. Today the docket pre-fills
a two-line manual recipe (`sudo chown … ; ctrace start`) that a human
must run. This PRD adds `ctrace doctor [--fix]`: it reports each failure
mode as JSON (down / stale pidfile / root-owned trap / wrong-identity
pid / healthy) and, with `--fix`, reaps any trap and restarts the
tracer. `doctor` is what self-review calls instead of pre-filling a
manual recipe.

## Why this exists (Phase 1 evidence, 2026-06-15)

- **The cure is currently a manual recipe, not a command.** Every
  self-review journal 2026-06-09 → 2026-06-14 pre-fills the same
  *"Fix: sudo chown jsy ~/.cache/ctrace/tracer.pid"* under "Pending your
  call," and it never runs. A one-shot command removes the human step.
- **The diagnosis is already implicit but scattered.** `ctrace status`
  (lines 108-119) prints `running` + `current.json`, but it does *not*
  distinguish *why* a tracer is down (dead vs trapped pidfile vs
  foreign PID). The information needed to tell those apart now exists
  once [[PRD-ember-tracer-ownership]] (reaper) and
  [[PRD-ember-liveness-truth]] (`is_our_tracer`) land — `doctor` just
  composes them into a labelled verdict.
- **Self-review wants exactly this shape.** Compare
  [[PRD-rollout-selfreview-apply]] (vigil): the pattern there is "the
  recurring finding never gets cured because the pre-filled command is
  manual/errors." `doctor --fix` is the ctrace analogue — a curative
  command the playbook can pre-fill (and, for ctrace, actually call,
  since reaping our own state file is lower-risk than restarting a voice
  daemon).

## What this builds

A `doctor` subcommand in `~/.local/bin/ctrace` (Python; reuses the
helpers from the two prior PRDs). No new deps.

- **`ctrace doctor` (no flag, non-mutating).** Emit JSON:
  `{"state": "<healthy|down|stale-pidfile|root-owned-trap|foreign-pid>",
  "tracer_pid": <int|null>, "pidfile_owner": "<user>",
  "pidfile_present": <bool>, "would_fix": [<step strings>]}`. Touches
  nothing. The `state` is derived from `running_pid()` (now
  identity-true) plus a stat of the pidfile:
  - live + ours → `healthy`
  - pidfile absent, no tracer → `down`
  - pidfile present, PID dead → `stale-pidfile`
  - pidfile present + root-owned + PID dead → `root-owned-trap`
  - pidfile present, PID alive but not our bpftrace → `foreign-pid`
- **`ctrace doctor --fix`.** For any non-`healthy` state: reap the
  pidfile (ownership reaper) if present, then run the existing
  `cmd_start` path; re-diagnose and emit the final JSON. Idempotent —
  running it twice on a healthy tracer is a no-op that reports
  `healthy`. Exit 0 if final state is `healthy`, non-zero otherwise (so
  a hook/playbook can branch).
- **`--root`/`--log` passthrough** to the underlying start, matching
  `cmd_start`'s existing flags, so `doctor --fix` can target the right
  PID tree.

## Acceptance criteria

1. `ctrace doctor` (no flag) prints valid JSON with a `state` field that
   is one of `healthy|down|stale-pidfile|root-owned-trap|foreign-pid`,
   and **mutates nothing** (pidfile mtime/owner unchanged after the
   call).
2. The five states are correctly distinguished by fixtures: healthy
   (live our-tracer), down (no pidfile), stale-pidfile (dead PID,
   user-owned), root-owned-trap (dead PID, root-owned), foreign-pid
   (live non-bpftrace PID).
3. `ctrace doctor --fix` on a `root-owned-trap` state reaps the pidfile
   and starts a fresh tracer, ending in `state: healthy` with exit 0 and
   **no** manual `chown`.
4. `ctrace doctor --fix` on an already-`healthy` tracer is a no-op:
   reports `healthy`, exit 0, does not restart (the live tracer PID is
   unchanged).
5. `ctrace doctor --fix` exits non-zero if it cannot reach `healthy`
   (e.g. `sudo -n` unavailable / bpftrace fails to attach), with the
   failing `state` and the bpftrace error surfaced in the JSON.
6. `--root`/`--log` are accepted and forwarded to the start path; help
   text (`ctrace doctor -h`) documents the subcommand. `status|query|
   tail|start|stop` behavior unchanged; no new Python deps.
