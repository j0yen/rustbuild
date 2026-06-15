# PRD: ember-tracer-ownership — the tracer never leaves (or stays trapped by) a root-owned state file

Status: Draft v0.1
build_target: shell
build_into: /home/jsy/.local/bin/ctrace
Vision: visions/ember.md

## TL;DR

`ctrace` runs `bpftrace` under `sudo` but is *managed* by an
unprivileged user. When a tracer is started from a sudo/root context its
`tracer.pid` ends up `root:root`; when that tracer later dies, the
unprivileged `ctrace start` (run by the SessionStart hook and by
self-review) hits `PermissionError: Permission denied:
'/home/jsy/.cache/ctrace/tracer.pid'` and refuses to start. This PRD
makes `ctrace`'s pidfile lifecycle privilege-safe: a dead-but-root-owned
pidfile is *reaped* (via the same `sudo -n` the tool already uses)
instead of trapping the user, and `stop` never crashes unlinking a
root-owned file. After this PRD the recurring `ctrace-tracer-down`
finding is curable without a human at a sudo prompt.

## Why this exists (Phase 1 evidence, 2026-06-15)

- **Live state.** `ctrace status` → `{"running": false, "tracer_pid":
  259895, ... "started_at": 1781368450}`. `ls -l
  ~/.cache/ctrace/tracer.pid` → `-rw-r--r-- 1 root root 6 Jun 13 09:34`.
  The tracer has been down since Jun 13.
- **Recurring docket item, verbatim across journals 2026-06-09 →
  2026-06-14:** *"ctrace tracer down (auto-fix blocked): `tracer.pid`
  is root-owned from a prior `sudo ctrace start`. Non-sudo start fails
  with `PermissionError`. Fix: sudo chown jsy
  ~/.cache/ctrace/tracer.pid"* and *"start.err: `PermissionError:
  Permission denied: '/home/jsy/.cache/ctrace/tracer.pid'`"*. The fix
  has never run — it needs a human at a sudo prompt.
- **Root cause in source (`~/.local/bin/ctrace`, read this Phase 1):**
  - line 60: `subprocess.Popen(["sudo", "-n", "bpftrace", ...])` — the
    tracer is a root process; state touched from that context is
    root-owned.
  - line 104: `sf("tracer.pid").unlink(missing_ok=True)` in `cmd_stop`
    — `missing_ok` does nothing for a *present but root-owned* file; the
    unlink raises an uncaught `PermissionError`, so `stop` cannot clear
    the trap that blocks `start`.
  - line 47-48: `if running_pid(): sys.exit("ctrace: already
    running ...")` — combined with the liveness lie (handled in
    [[PRD-ember-liveness-truth]]) the start path bails instead of
    healing.

## What this builds

A patch to `~/.local/bin/ctrace` (Python; no new deps). One small
helper plus three call-site changes.

- **`reap_pidfile()` helper.** Remove `tracer.pid` even when root-owned:
  try a plain `unlink`; on `PermissionError`, fall back to
  `subprocess.run(["sudo", "-n", "rm", "-f", str(p)])`. This reuses the
  *exact* privilege primitive the tool already depends on (`sudo -n`
  for bpftrace/kill at lines 60/97/103) — no new trust surface, no
  password prompt (if `sudo -n` is unavailable the command fails
  cleanly and the existing manual fix still applies).
- **`cmd_start` heals instead of trapping.** Before the
  `already running` exit: if `running_pid()` is `None` (dead) but
  `sf("tracer.pid")` still exists, `reap_pidfile()` it and continue.
  After a successful start, ensure the state dir and the freshly-written
  `tracer.pid` are owned by the invoking user (best-effort `os.chown`
  to `os.getuid()/os.getgid()`; ignore failure when already correct).
- **`cmd_stop` is unlink-safe.** Replace the bare
  `sf("tracer.pid").unlink(missing_ok=True)` with `reap_pidfile()` so a
  root-owned pidfile is cleared via the sudo fallback rather than
  crashing.

Out of scope (other PRDs): identity-true liveness
([[PRD-ember-liveness-truth]]); the `doctor` subcommand
([[PRD-ember-doctor]]); SessionStart wiring
([[PRD-ember-selfheal-hook]]).

## Acceptance criteria

1. A `reap_pidfile()` helper exists that removes `tracer.pid` via plain
   `unlink`, and on `PermissionError` falls back to `sudo -n rm -f`,
   returning a boolean success without raising.
2. Given a **dead** tracer and a **root-owned** `tracer.pid`, a plain
   (non-sudo) `ctrace start` succeeds — it reaps the trapped pidfile and
   launches a new tracer — with **no** manual `chown`. (Test: create a
   root-owned `tracer.pid` containing a dead PID, run `ctrace start`,
   assert exit 0 and a fresh user-owned pidfile.)
3. After a normal `ctrace start`, `tracer.pid` is owned by the invoking
   user (`stat -c %U` == `$USER`).
4. `ctrace stop` on a state where `tracer.pid` is root-owned completes
   without raising `PermissionError` and leaves no `tracer.pid` behind.
5. `ctrace start` while a *genuinely live* tracer is running is
   unchanged — it still refuses with a non-zero exit and does **not**
   reap the live pidfile. (Liveness identity itself is
   [[PRD-ember-liveness-truth]]; here, only assert the live case is not
   reaped.)
6. No new Python dependencies; `ctrace status|query|tail` output schema
   unchanged.
