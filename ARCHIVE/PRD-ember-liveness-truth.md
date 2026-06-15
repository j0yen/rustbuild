# PRD: ember-liveness-truth — a pid is "our tracer" only if it really is bpftrace running our script

Status: Draft v0.1
build_target: shell
build_into: /home/jsy/.local/bin/ctrace
Vision: visions/ember.md

## TL;DR

`ctrace`'s `running_pid()` decides "is the tracer alive?" by mere
PID-existence, and — fatally — treats a `PermissionError` from
`os.kill(pid, 0)` as **"still running."** So a dead tracer whose
root-owned PID got recycled by an unrelated root process reads as
*alive*, and `ctrace start` bails with "already running" forever. This
PRD replaces existence-only liveness with **identity-true** liveness: a
pid counts only if `/proc/<pid>/comm` is `bpftrace` and its cmdline
references our `session.bt`. Anything else is a stale/foreign pidfile →
reap it (the reaper from [[PRD-ember-tracer-ownership]]).

## Why this exists (Phase 1 evidence, 2026-06-15)

- **The liveness lie, in source (`~/.local/bin/ctrace` lines 36-43):**
  ```python
  try:
      os.kill(pid, 0)
      return pid
  except ProcessLookupError:
      return None
  except PermissionError:
      # exists but we can't signal it (root-owned via sudo) — still running
      return pid
  ```
  `PermissionError` means *"a process with this PID exists and I can't
  signal it"* — which is true of **any** root process that recycled the
  PID, not just our bpftrace. The comment encodes the bug as
  intent.
- **Why it bites here.** Tracers run for hours/days and the box reboots;
  PIDs recycle. A root-owned `tracer.pid` pointing at a dead tracer,
  once its number is reused by *any* root process, makes `running_pid()`
  return that PID → `cmd_start` exits `"already running"` (line 47-48)
  → the tracer is "up" forever while `ctrace status` and the journals
  show `"running": false` and "tracer NOT running." This is the
  false-positive twin of the ownership trap.
- **bpftrace is identifiable.** It is launched as `bpftrace -q
  --no-warnings <session.bt> <root>` (line 60) and the real pid is
  found by walking sudo's children (lines 74-82). `/proc/<pid>/comm`
  and `/proc/<pid>/cmdline` make "is this *our* tracer?" a cheap,
  unprivileged, unambiguous check (both are world-readable).

## What this builds

A patch to `running_pid()` (and a tiny shared identity helper) in
`~/.local/bin/ctrace`. No new deps.

- **`is_our_tracer(pid)` helper.** Read `/proc/<pid>/comm` (== `bpftrace`)
  and `/proc/<pid>/cmdline` (NUL-split; contains the basename of
  `SCRIPT`, i.e. `session.bt`). Return `True` only if both match. Any
  read error (process gone, race) → `False`.
- **`running_pid()` becomes identity-true.** Parse the pidfile; if the
  PID is absent from `/proc` → stale. If present but
  `not is_our_tracer(pid)` → stale (foreign/recycled). On stale, reap
  the pidfile (via the helper from [[PRD-ember-tracer-ownership]]) and
  return `None`. Only a present pid that *is* our tracer returns the
  pid. The `PermissionError ⇒ alive` branch is **deleted** —
  `/proc/<pid>/comm` is readable without signalling, so we no longer
  need to infer liveness from `os.kill`.

This composes with ownership: ownership makes a stale pidfile
*removable*; liveness-truth makes a stale pidfile *recognizable*.
Together, `ctrace start` heals both the can't-remove and the
looks-alive failure modes.

## Acceptance criteria

1. An `is_our_tracer(pid)` helper returns `True` only when
   `/proc/<pid>/comm` == `bpftrace` **and** `/proc/<pid>/cmdline`
   contains the `session.bt` basename; returns `False` on any read
   error or mismatch.
2. `running_pid()` returns `None` when `tracer.pid` points at a PID that
   exists but is **not** our tracer (e.g. a foreign/recycled root PID).
   (Test: write a `tracer.pid` containing the PID of some non-bpftrace
   process — e.g. PID 1 — assert `ctrace status` reports
   `"running": false`.)
3. `running_pid()` still returns the PID when a genuine `bpftrace
   <session.bt>` is running. (Test: stub a process whose `comm`/cmdline
   match, or assert against a real `ctrace start` in the harness.)
4. The `except PermissionError: return pid` branch is removed; liveness
   no longer depends on `os.kill` permission.
5. When `running_pid()` finds a stale/foreign pidfile it reaps it (uses
   the [[PRD-ember-tracer-ownership]] reaper), so a subsequent
   `ctrace start` proceeds cleanly.
6. `ctrace status|query|tail` output schema unchanged; no new Python
   dependencies.
