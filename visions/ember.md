# Vision: ember — the session tracer keeps its own flame

**Authored by:** /dream (Claude Sonnet 4.6), with jsy
**Created:** 2026-06-15
**Status:** active
**Seed:** reflection — the self-review docket has carried
`ctrace-tracer-down` as an unresolved "pending your call" item in
*every* journal for a week+ (2026-06-09 → 2026-06-14). Caught live in
this dream's Phase 1: `ctrace status` → `"running": false`,
`~/.cache/ctrace/tracer.pid` is `root:root` (6 bytes, Jun 13 09:34),
and the fix the docket keeps pre-filling — `sudo chown jsy
~/.cache/ctrace/tracer.pid` — has never been run because it requires
a human at a sudo prompt.

> `vigil` watches for stale *binaries*. `rouse` woke the deaf *voice*
> loop. `ember` keeps the *eye* lit — the eBPF tracer that records
> what actually ran. When the eye goes dark it must be able to relight
> itself, without a human at a sudo prompt.

## TL;DR

`ctrace` is the laptop's record of what actually executed each session
(`ctrace query`, the scribe rollups in every journal, the
`ctrace-orphan-reap` and `ctrace-scribe` sidecars all consume it). It
is the single most-cited source in self-review's "Claude session
activity" section. And it has been **down for ~6 days straight**,
because the tool cannot survive its own privilege model.

The mechanism, read out of `~/.local/bin/ctrace` (115 lines, Python)
this Phase 1:

- The tracer runs as root: `subprocess.Popen(["sudo", "-n",
  "bpftrace", ...])` (line 60). Anything that touches the state dir
  *from inside that root context* lands root-owned.
- Liveness is mere PID-existence. `running_pid()` does `os.kill(pid,
  0)` and — critically — treats `PermissionError` as **"still
  running"** (lines 41-43, comment: *"exists but we can't signal it
  (root-owned via sudo) — still running"*). So a recycled root PID, or
  a dead tracer whose pidfile jsy can't signal, both read as *alive*.
- `cmd_stop` ends with `sf("tracer.pid").unlink(missing_ok=True)`
  (line 104) — which **raises** `PermissionError`, uncaught, when the
  pidfile is root-owned. So `stop` cannot clean up the very trap that
  blocks `start`.

The result is a self-locking failure: a sudo-context start leaves a
root-owned `tracer.pid`; the tracer later dies; the unprivileged
`ctrace start` (run by the SessionStart hook and by self-review) hits
`PermissionError: Permission denied:
'/home/jsy/.cache/ctrace/tracer.pid'` and gives up. The docket dutifully
pre-fills `sudo chown jsy …` and waits for a human who, six days
running, hasn't been at the prompt at the right moment.

ember gives the tracer an **honest lifecycle**: state files owned by the
invoking user even though the process is root; liveness that verifies
*identity* (is the pid actually our bpftrace?) instead of mere
existence; and an unprivileged, guarded relight path so self-review can
cure "tracer down" inside its existing autonomy guardrails — no manual
chown, no human at a sudo prompt.

## End-state

When ember is done:

- A dead tracer + a root-owned `tracer.pid` no longer blocks anything:
  `ctrace start` reaps the trap (via `sudo -n rm` fallback — the one
  privileged primitive, the same `sudo -n` the tool already relies on)
  and proceeds. No manual `chown`.
- `ctrace status`/`running_pid()` cannot be fooled: a recycled or
  foreign root PID reads as **down**, not "running"; only a live
  bpftrace executing *our* `session.bt` reads as up.
- `ctrace stop` never crashes on a root-owned pidfile.
- `ctrace doctor [--fix]` exists: one idempotent command that diagnoses
  every failure mode and, with `--fix`, cures a down+trapped tracer —
  exactly the steps the docket pre-fills today.
- The SessionStart path self-relights the tracer via `ctrace doctor
  --fix`, inside the existing guardrails, so `ctrace-tracer-down` stops
  being a standing manual item.

## Components (one bullet per future PRD)

- **ember-tracer-ownership** — `ctrace start`/`stop` never leave (or
  get trapped by) a root-owned state file. Reap a dead-but-root-owned
  `tracer.pid` via `sudo -n rm` fallback; make `stop`'s unlink
  PermissionError-safe; ensure the state dir + pidfile are
  invoking-user-owned after a normal start. Foundation. **shell**
  (patch `~/.local/bin/ctrace`).
- **ember-liveness-truth** — replace existence-only liveness with
  identity-true liveness: a pid counts as "our tracer" only if
  `/proc/<pid>/comm` is `bpftrace` and its cmdline references our
  `session.bt`; otherwise the pidfile is stale → reap. Kills the
  recycled-PID false positive and the `PermissionError ⇒ alive` lie.
  Depends on ownership (shares the reaper). **shell**.
- **ember-doctor** — a `ctrace doctor [--fix]` subcommand that reports
  each failure mode (down / stale pidfile / root-owned trap /
  wrong-identity pid) as JSON and, with `--fix`, applies reap+restart.
  The one guarded, idempotent command self-review can call. Depends on
  ownership + liveness-truth. **shell**.
- **ember-selfheal-hook** — wire `ctrace doctor --fix` into the
  SessionStart hook (`ctrace-session-start.sh`) so the tracer
  self-relights on session start within guardrails, closing the
  recurring docket item. Depends on doctor. **hooks**.

## Order

```
ember-tracer-ownership ──► ember-liveness-truth ──► ember-doctor ──► ember-selfheal-hook
```

Linear. ownership defines the reaper primitive; liveness-truth reuses
it; doctor composes both into one command; the hook calls doctor. Each
is independently shippable in order — after ownership alone, the
week-long recurring finding is already cured for the manual path.

## Open questions (for the next /dream pass or jsy)

- **Rust rewrite?** ctrace is Python at `~/.local/bin/ctrace` with no
  `~/wintermute/ctrace/` source repo (only the `ctrace-orphan-reap` and
  `ctrace-scribe` sidecars are Rust). A full Rust rewrite is *not*
  motivated by this evidence — the bug is a ~10-line privilege/liveness
  fix. Left as a question, not a PRD. If it ever happens it's a separate
  vision.
- **Should `--fix` ever run fully autonomously** (self-review B.5
  playbook), or only on the human-initiated SessionStart path? The
  privileged primitive is a single `sudo -n rm -f` of a state file we
  own the semantics of — lower-risk than a daemon restart, but still a
  sudo action. Default proposed: SessionStart auto-relight is fine
  (it's reaping our own trap); leave a heavier autonomous loop gated.
- **Pidfile under a user-only dir?** An alternative to reaping is to
  never let root write there — e.g. have bpftrace's wrapper drop a
  user-owned pidfile via a tee that runs as jsy. Heavier than the reap
  fallback; noted but not drafted.
