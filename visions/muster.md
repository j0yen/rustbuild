# Vision: muster — a definitive roster of the live Claude session population

*To muster is to assemble a body and account for everyone present.* This
vision gives the laptop a single source of truth for the question every
self-review currently answers with a guess: **which `claude` processes are
running right now, where did each come from, and which of them shouldn't be.**

## TL;DR

The seed: bare `/dream` (jsy, 2026-06-08), but the signal is chronic and
laptop-native. Three consecutive self-review journals (2026-06-06, -07, and
the recall reflective for -07) flag *"duplicate Claude sessions … may both be
real sessions"* and decline to act because the playbook cannot tell a real
concurrent session from a leaked one. The self-review SKILL itself only does
`pgrep -af claude` with the explicit note *"note duplicates, do not kill"*
(SKILL.md:70) — it has no way to attribute a PID to its origin.

This was captured **live this run**: at dream time the process table held
three `claude` processes — pid 33958 interactive (parent 4279, a tty),
pid 402723 `claude -p /self-review`, and pid 402724 `claude -p /dream` (this
very session). `/self-review` and `/dream` were running concurrently, each
about to report the *other* as a "duplicate Claude session." Yet the process
tree already encodes the full answer: the ppid chain names the launcher
(`claude-self-review-headless.sh` vs a login tty), the argv names the role
(`-p /self-review` vs `-p /dream` vs interactive), the cgroup/scope names the
systemd unit, and the agorabus `claude-<pid>-jsy` peer + worker subtree ties
each to the bus. Nothing reads that structure and renders a verdict.

The existing `session-*` fleet does **not** close this gap: `session-index`,
`session-postmortem`, and `session-trace-receipt` all key on a *session-id read
from transcript logs* (`~/.claude/projects/*.jsonl`) — post-hoc forensics on
sessions that already ran, not a census of the live process population.
`agorabus peers` reconciles the *bus* side (one peer per session) and `pevent`
reconciles *supervised jobs*, but neither attributes a live `claude` process to
its origin or judges whether it should still be alive. `ctrace-orphan-reap`
reaps leaked **bpftrace tracers**, not Claude sessions.

`muster` is the missing layer: enumerate the live `claude` population, attribute
each process to an origin class, reconcile it against its bus peer / worker
subtree / tracer / (when non-zero) `agentns` session-id, render a per-session
verdict (`live | duplicate | orphan | stale`), and feed self-review a definitive
roster instead of `pgrep | grep` plus a shrug. A separate, proposal-only reaper
acts on the verdict for the genuinely dead — never on an interactive or a live
timer run.

## End-state

When this vision is fulfilled:

1. `muster census` enumerates every live `claude` process and emits a
   structured roster: for each, its root PID, origin class
   (`interactive-tty | timer:<unit> | headless:<script> | unknown`), launching
   command, cwd/project, uptime, RSS, its agorabus peer session-id + worker
   subtree PIDs, its ctrace tracer (if any), and its `agentns` session-id when
   the kernel reports a non-zero one (gracefully degrading to the heuristic
   while `agentns-session-zeros` is open — see [[assay]]).
2. `muster verdict` classifies each roster entry with a reason string:
   `live` (healthy parent, expected for its origin), `duplicate` (two
   interactive sessions in the same project/cwd — the exact thing self-review
   keeps flagging), `orphan` (parent dead / reparented to init **and** idle past
   a grace window), `stale` (a headless `-p` run that outlived its expected
   timeout). The verdict replaces "may both be real" with a defensible call.
3. `self-review` consumes `muster verdict --format selfreview` and journals a
   definitive session census — no ad-hoc `pgrep` parsing, no guess.
4. `muster reap` can act on the `orphan`/`stale` verdicts, but only as a
   **reviewer-gated proposal**: dry-run by default, never proposes killing an
   `interactive-tty` or a `live` timer run, never touches a session younger than
   its grace window, and requires `--confirm` to send a signal.

## Components (PRD-sized)

1. **muster-census** (`PRD-muster-census`) — the core CLI, a new repo at
   `~/wintermute/muster`. Enumerate live `claude` processes; attribute origin
   via ppid chain + argv + cgroup/scope; reconcile each against its agorabus
   peer, worker subtree, ctrace tracer, and `agentns` session-id; emit the
   roster as text + `--format json`. *Foundational.*
2. **muster-verdict** (`PRD-muster-verdict`) — `rust-extend` into
   `~/wintermute/muster`: per-entry classifier
   (`live | duplicate | orphan | stale`) with an explicit reason and the
   evidence it rested on. Depends on census's roster.
3. **muster-reap** (`PRD-muster-reap`) — `rust-extend`: proposal-only,
   reviewer-gated termination of `orphan`/`stale` entries. Mirrors the settled
   safety stance of `recourse-contest` / `mend-bridge` — dry-run default, hard
   refusal to target interactive/live sessions, `--confirm` to act. Depends on
   verdict.
4. **muster-selfreview-bridge** (`PRD-muster-selfreview-bridge`) — `mixed`:
   a `--format selfreview` emit + the self-review playbook edit that swaps the
   `pgrep -af claude` step for `muster verdict`. Depends on verdict.

## Order

```
muster-census ──► muster-verdict ──┬──► muster-reap
                                    └──► muster-selfreview-bridge
```

census is the gate. verdict consumes the roster. reap and the self-review
bridge both consume the verdict and can proceed in parallel once it lands.

## Open questions

- **Origin truth source.** While `agentns-session-zeros` is open (see
  [[assay]]), origin must be inferred heuristically from ppid/argv/cgroup. Once
  `agentns-clone-flag-fix` lands a non-zero `/proc/<pid>/agent_session`, muster
  should prefer the kernel session-id and fall back to the heuristic. Census
  must be written so that swap is additive, not a rewrite.
- **What counts as a real concurrent duplicate.** Two interactive sessions in
  *different* projects are normal; two in the *same* cwd/project is the
  flagged case. Is cwd the right key, or the resolved `~/.claude/projects/<slug>`?
  Leaning project-slug; confirm against a real duplicate when one recurs.
- **Grace windows.** What uptime makes a parentless `claude` an `orphan`
  vs a just-launched run whose parent script already exec-replaced itself?
  Headless `-p` runs have a known timeout (e.g. dream/self-review tick budgets);
  the `stale` threshold should derive from that, not a magic constant.
- **reap signal discipline.** SIGTERM-then-wait-then-SIGKILL vs propose-only-
  never-signal. Starting proposal-only (emit the kill command, never run it
  unless `--confirm`), matching `mend-bridge`'s never-auto-act stance.
- **Reaping the subtree.** Killing a session root should also account for its
  agorabus subscriber + worker + tracer children; does muster reap the subtree
  or leave the existing orphan-subscriber / orphan-reap playbooks to sweep them?
  Leaning: muster reports the full subtree, reaps only the root, lets the
  established sweepers collect the rest.

---

## Extension 2026-06-16 — the live verdict must grade the bus subtree (Fleet 2.0)

*Seed: bare `/dream`, Phase-1 live inspection. The vision's four components
(census/verdict/reap/selfreview-bridge) all shipped — `muster` is installed and
`muster verdict` runs. But running it against the live process table this run
exposed two gaps the original PRDs didn't reach, both backed by hard evidence.*

### What the live run showed

`muster verdict` (2026-06-16, this box) returned:

```
1054     interactive-tty   duplicate   296277s   duplicate interactive-tty session for project slug 'jsy'
949440   interactive-tty   duplicate    41771s   duplicate interactive-tty session for project slug 'jsy'
1147375  interactive-tty   duplicate    34435s   duplicate interactive-tty session for project slug 'jsy'
1195852  interactive-tty   duplicate    33669s   duplicate interactive-tty session for project slug 'jsy'
```

Two problems, both verified:

1. **The roster's `worker_pids` is a lie.** `muster census --format json` for
   pid 1054 returns `"worker_pids": []` — yet `pgrep -fc "agorabus-worker.sh
   claude-1054-jsy"` = **21**, and 3 of that session's `agorabus subscribe`
   children are **deleted-exe** (`readlink /proc/<pid>/exe` ends ` (deleted)` —
   pids 1906, 2201, 579203). The census field promised in End-state §1
   ("its agorabus peer session-id + worker subtree PIDs") is declared but never
   populated. So muster cannot even *see* the subtree it claims to reconcile.

2. **Every $HOME session collapses to `duplicate`.** All four interactive
   sessions run from `/home/jsy`, so all resolve to project slug `jsy` and all
   four are flagged `duplicate` with the same reason. The verdict cannot say
   *which* of the four is the genuine zombie (1054, 3.4 days old, idle, dragging
   21 leaked workers) versus three legitimately-active concurrent shells. The
   open question "is cwd the right key … confirm against a real duplicate when
   one recurs" has now recurred **four-fold, live** — it's research, not
   speculation.

### Root cause of the rot (verified)

The subtree rot is not random churn — it has a one-line cause.
`~/.claude/scripts/agorabus-worker.sh:51` (live symlink →
`~/wintermute/dotfiles/.claude/scripts/agorabus-worker.sh`) guards against
double-spawn with:

```sh
if pgrep -f "agorabus-worker.sh $sid\$" | grep -v "^${self_pid}\$" ...
```

The `\$` anchors end-of-string immediately after the session id — but the real
argv is `agorabus-worker.sh claude-1054-jsy /home/jsy` (a trailing cwd arg the
script itself passes). The anchor therefore **never matches**, the idempotency
guard is a no-op, and every re-trigger (each agorabus reconnect under the
`Restart=always` drop-ins) spawns a fresh worker that believes it is the only
one. 21 workers for one session is the result; the deleted-exe subscribers are
the old binaries those stale workers keep alive across reloads. This is the
exact "fleet-binary-staleness: 3 deleted-exe agorabus subs" finding self-review
has carried open for 3+ consecutive runs.

### New components (PRD-sized)

5. **muster-subtree-census** (`PRD-muster-subtree-census`) — `rust-extend`
   into `~/wintermute/muster` (`census.rs`). Make `worker_pids` actually
   populate, and add a `subtree` block per session: subscriber PIDs, worker
   PIDs, `deleted_exe` count, and distinct worker generations. Fixes the empty
   field and gives every later component its data. *Foundational for this
   extension.*
6. **muster-verdict-subtree-rot** (`PRD-muster-verdict-subtree-rot`) —
   `rust-extend` (`verdict.rs`). Add a `subtree-rot` annotation orthogonal to
   the live/duplicate/orphan/stale axis: any session whose subtree carries ≥1
   deleted-exe member or more than a threshold of worker generations is flagged
   with the counts and a reason, *even when the session itself is `live`*.
   Depends on #5.
7. **muster-duplicate-rank** (`PRD-muster-duplicate-rank`) — `rust-extend`
   (`verdict.rs`). Disambiguate same-slug duplicates by activity recency
   (transcript mtime / ctrace last-event / uptime): exactly the freshest stays
   `live`; the idle-older ones become `duplicate (idle <N>s)`. Settles the
   "which duplicate is the zombie" open question. Depends on #5; parallels #6.
8. **agorabus-worker-idempotency-fix** (`PRD-agorabus-worker-idempotency-fix`)
   — `shell`. Fix the `:51` anchor so the guard matches the real argv (drop the
   `\$` or match `"agorabus-worker.sh $sid( |\$)"`). Plugs the leak at source.
   Ships to `proposals/agorabus-worker.draft.sh` per the script's own
   held-out-until-smoke-tested discipline. Independent of the muster PRDs.
9. **muster-subtree-reap** (`PRD-muster-subtree-reap`) — `rust-extend`
   (`reap.rs`). Extend `muster reap` to optionally collect just the **dead
   children** (deleted-exe subscribers/workers) of an otherwise-`live` session,
   proposal-only and `--confirm`-gated, never signalling the session root.
   Clears the recurring self-review finding without killing the live session.
   Depends on #6.

### Order (extension)

```
agorabus-worker-idempotency-fix  (independent — the leak source)

muster-subtree-census ──► muster-verdict-subtree-rot ──► muster-subtree-reap
                      └──► muster-duplicate-rank
```

### Open questions (extension)

- **Generation threshold.** How many worker generations is "rot" vs normal
  reconnect churn? Lean: any deleted-exe member is rot regardless of count;
  worker-count rot needs a threshold (start at >3, tune against a clean
  freshly-booted session).
- **Does the worker fix make #9 mostly moot?** Once #8 lands, new sessions stop
  accumulating workers — but already-leaked subtrees (1054's 21) persist until
  the session dies. #9 is the cleanup arm for the backlog the fix can't
  retroactively undo. Both earn their place.
