# PRD: scribe-reap-wire — the orphan-reaper is built but nothing ever calls it

Status: Draft v0.1
build_target: hooks
Vision: visions/scribe.md

## TL;DR

`ctrace-orphan-reap` was shipped on 2026-06-03 to do exactly the right
thing — `--apply` stops an orphaned tracer (alive, but its owning Claude
session is dead), renders the abandoned NDJSON log to a summary, and
clears the stale `claude-owns.json` marker. It works. It is also **dead
code**: a grep across `~/.claude/scripts` and every skill finds **zero**
callers. The only path that ever recovers a SIGKILLed session's trace is
the human-run self-review the next morning. This PRD wires
`ctrace-orphan-reap --apply` into the SessionStart hook, *before* a new
tracer starts, so a session that died ungracefully gets reaped and
rendered at the very next session boundary — no self-review required.

## Why this exists

- **The binary has no callers.** Verified this pass:
  `grep -rl 'orphan-reap' ~/.claude/scripts ~/.claude/skills` → empty.
  `ctrace-orphan-reap --help` confirms the capability is real (verdict
  taxonomy: `healthy` / `orphaned-tracer` / `stale-marker` / `no-tracer`;
  flags `--apply`, `--dry-run`, `--json`, `--state-dir`). It was built
  to be wired and never was.
- **The flake it was built to fix is still open.** `docket` carries
  `ctrace-sessionend-flake` at **runs_seen: 5** across journals
  2026-06-06/-07/-08, each "Pending your call" listing it unresolved with
  the note "the hook itself is the root cause; backfill is a band-aid."
- **SIGKILL is the unrecoverable case.** `ctrace-session-end.sh` only
  runs on a graceful SessionEnd. A heavy /build or /dream session is
  routinely SIGKILLed by cgroup teardown (see
  [[self_build_detached_cgroup_teardown]],
  [[self_build_worktree_reset_wipes_state]]) — no exit hook fires, so
  no render and no marker cleanup ever happen for that session. The
  *companion* PRD `mend-ctrace-render` (visions/mend.md) hardens the
  graceful-exit render but, by construction, cannot help a process that
  never runs its exit hook. The only recovery is the next SessionStart.
- **The state to reconcile is sitting right there.** Live this pass:
  `~/.cache/ctrace/claude-owns.json` names `claude_pid` + `log`;
  `~/.cache/ctrace/tracer.pid` holds the tracer PID. `orphan-reap`
  already reconciles exactly these against live PIDs — the logic exists,
  it just needs an invocation.

## What this builds

A `hooks` change to `~/.claude/scripts/ctrace-session-start.sh` (a
symlink into `~/wintermute/dotfiles/.claude/scripts/`; edit lands in the
dotfiles repo).

Insert, **before** the existing "start a new tracer" logic and after the
`claude-build` cgroup guard (which must still short-circuit first — never
run a root reaper inside a build unit's cgroup), a reap step:

```sh
reap=/home/jsy/.local/bin/ctrace-orphan-reap
if [ -x "$reap" ]; then
    "$reap" --apply >/dev/null 2>>"$err" || true
fi
```

Semantics this must preserve:

1. **Run only when safe.** The existing `grep -q 'claude-build'
   /proc/self/cgroup` guard at the top of the hook still returns early —
   the reaper (which may `ctrace stop` a root-owned `sudo bpftrace`) must
   never execute inside a `claude-build*` cgroup. Place the reap after
   that guard.
2. **Never block startup.** Wrapped in `|| true`, stderr to the existing
   `$err` file, bounded — the hook still always `exit 0`, and a slow or
   failing reap never delays Claude coming up.
3. **Idempotent + order-correct.** The reap runs before the hook's own
   stale-marker reconciliation and before `ctrace start`. After a clean
   reap (orphan stopped, marker cleared) the existing start logic sees a
   clean slate and starts the new tracer normally. When the verdict is
   `healthy` (a live sibling session owns the tracer) the reaper changes
   nothing and the existing "honor a running tracer" branch still wins.
4. **No behavior change on the happy path.** A session that started with
   no prior orphan (`no-tracer` verdict) sees a no-op reap and identical
   downstream behavior to today.

Constraints: pure shell wiring, no new binary. The reaper itself is
unchanged (already shipped). No edit to `ctrace-session-end.sh` — that
file is owned by `mend-ctrace-render`.

## Acceptance criteria

1. `ctrace-session-start.sh` invokes `ctrace-orphan-reap --apply` after
   the `claude-build` cgroup guard and before `ctrace start`.
2. With a synthesized orphan state (a `claude-owns.json` naming a dead
   PID + a stale `tracer.pid`, in a temp `--state-dir`),
   `ctrace-orphan-reap --apply --state-dir <tmp>` renders the named log's
   summary and clears the marker — verified directly (proves the wired
   command does the recovery the hook relies on).
3. The hook still exits 0 in every branch: no marker, healthy sibling
   tracer, orphaned tracer, and reaper-absent (binary not installed).
4. The `claude-build` cgroup guard still returns **before** the reap line
   — confirmed by exercising the hook with a faked `claude-build` entry
   in a stubbed `/proc/self/cgroup`; the reaper is not called.
5. On the happy path (no prior orphan), the reap is a no-op
   (`no-tracer`/`healthy` verdict) and a new tracer still starts exactly
   as before — no regression to normal session startup.
6. The reap's stderr is redirected to the hook's existing `$err`
   (`~/.cache/ctrace/claude-start.err`); it never writes to stdout (which
   would corrupt hook output).
