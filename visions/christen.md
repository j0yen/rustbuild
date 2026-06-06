# christen — give every session its true name at launch

## TL;DR

The wintermute kernel ships an eighth namespace, `CLONE_NEWAGENT`: every
task can carry an opaque 128-bit `agent_session_id`, an `intent_tag`,
per-namespace syscall counters, and kernel-enforced budgets. The whole
substrate is **built and booted** — `CONFIG_AGENT_NS=y`, the `/proc`
surface is live, `agent-wrap`/`agentns-claude` (the unshare launchers) and
`agentns-doctor` (the probe) are installed — but it is **inert at the
session level**. Every Claude session on this laptop sits in the *initial*
agent namespace (`/proc/self/ns/agent -> agent:[4026531996]`) with session
id `0…0`, because **no launch path routes through the wrapper**. christen is
the integration layer that lights the substrate up: it routes real session
launches through `agentns-claude`, grants the one capability the unshare
needs, detects when a session was born unwrapped, and ledgers each
session's true identity and resource footprint. A ship is christened at
launch; so is a session.

## Why now (live evidence, measured 2026-06-05)

- `/proc/self/ns/agent -> agent:[4026531996]` — the **init** agent NS. All
  three live Claude PIDs (454139, 866425, 899855) share it; every
  `agent_session` reads `00000000000000000000000000000000`; every
  `agent_counters` is all-zero.
- `CONFIG_AGENT_NS=y` in the booted `7.0.10-arch1-5-wintermute` kernel —
  the kernel side works; this is purely a userspace wiring gap.
- `agorabus-session-start.sh:51-60` already *wants* the real id: "The
  kernel writes a stable 32-char hex id into `/proc/self/agent_session`
  once `unshare(CLONE_NEWAGENT)` has been called (via agentns-claude)" —
  then falls back to PID synthesis because it never is.
- Docket item **`agentns-session-zeros`** (warn) has been open since
  2026-05-30 — 8 reports across 6 runs, `consecutive_runs: 0`, no
  playbook. It is the single genuinely *unaddressed* open docket item
  (keel took dead-cloud / `wm-anthropic-key-empty`, coda took the
  session-summary loop / `ctrace-sessionend-flake`).
- The continuity vision's own boot-validation note names this as the
  load-bearing blocker: "`/proc/self/agent_session` reads 32 zeros …
  confirms `claude-agentns-wrap` is the load-bearing PRD." That PRD
  *shipped* — yet the routing it promised was never actually placed on
  the launch path. The substrate has been waiting on its last wire.
- The root cause is structural, not a kernel fault: a **SessionStart hook
  cannot unshare its already-running parent.** The wrap must happen at
  *exec time* — `agentns-claude --intent … -- claude` — which means
  editing the launch sites (systemd units, shell rc) and granting
  `agent-wrap`/`agentns-claude` the `CAP_SYS_ADMIN` file cap the
  `unshare` needs (`agent-wrap.c`: "Needs CAP_SYS_ADMIN … `setcap
  cap_sys_admin+ep`"). Neither was ever done.

## End-state

When this is done:

- Every headless session (`claude-build.service`, `claude-dream.service`,
  `claude-self-review.service`) is launched through `agentns-claude` with a
  derived `intent_tag` and a default budget, so `/proc/$PID/agent_session`
  reads a real nonzero id from birth.
- The interactive shell launch has a documented, user-applied alias that
  does the same.
- `agent-wrap`/`agentns-claude` carry exactly the file-scoped capability
  the unshare needs — nothing broader, audited and verified.
- A `christen probe` correctly classifies any session's namespace state and
  resolves/keeps the `agentns-session-zeros` docket item honestly (live →
  resolve; init-while-wrapper-installed → actionable, not "registration
  failed").
- Each session leaves a ledger entry: birth `(sid, intent, budget)` →
  close `(final counters)`, keyed by the **true** session id that
  agorabus, memlog, and provfs have all been keying off `0…0` instead.

## Components (PRD-sized)

- **christen-plan** (rust-cli, NEW repo `~/wintermute/christen`) — the
  corpus: workspace + `christen` binary + the launch-site model
  (`LaunchSite` / `WrapState` / `RouteAction` / `RoutePlan`), a
  `LaunchSiteSource` trait + `FakeSource`, and a **pure** planner that turns
  discovered sites + current wrap state into a print-only route plan.
- **christen-detect** (rust-extend christen) — `christen probe`: read the
  `/proc` agent-namespace surface, classify init/live/absent/malformed
  (anti-regression: never say "registration failed" for init), and
  edge-trigger a `docket report`/`resolve` against `agentns-session-zeros`.
- **christen-route** (mixed: rust-extend + installer) — `christen route`:
  generate systemd drop-ins that rewrite each headless `ExecStart` to route
  through `agentns-claude --intent … --budget … --`; default print-only,
  `--apply` writes drop-ins but never enables/restarts.
- **christen-cap** (mixed: rust-extend + guarded installer) — `christen
  cap`: detect/grant the `cap_sys_admin+ep` file cap on the launchers,
  explain the precise scope, print the `setcap` line, verify post-grant by
  unsharing in a sandbox and reading a nonzero id.
- **christen-ledger** (rust-extend christen + SessionEnd hook) — capture
  each session's birth + close counters into `~/.claude/christen/ledger/`,
  keyed by the true session id; wires `agentns-doctor receipt`.

## Order

```
christen-plan  (FIRST — creates the repo; nothing rust-extend starts until it SHIPS)
      │
      ├── christen-detect  ┐
      ├── christen-route   ├── all depend on plan ONLY; build in PARALLEL
      └── christen-cap     ┘
                  │
            christen-ledger  (meaningful once route is live; builds vs FakeSource)
```

Strict hard dependency mirrors anchor/coda: do **not** start any
rust-extend until `christen-plan` has shipped and `~/wintermute/christen`
exists, or extend-validate fails (the rule that bit
relay/concord/quicken/keel/anchor).

## Open questions

- **Narrower privilege than CAP_SYS_ADMIN?** A tiny setuid-root helper that
  does only `unshare(CLONE_NEWAGENT)` then immediately drops would scope the
  privilege far tighter than a file-cap on the whole launcher. christen-cap
  should *explore* this; the first cut grants the file cap and documents the
  trade. (User decision — security-sensitive.)
- **Interactive shell routing** is user-typed (`claude`), so christen can
  only PRINT the alias; the systemd sites are the deterministic win and the
  fleet focuses there first.
- **Per-intent budget tuning** (a runaway `/build` SIGKILLed on fork/wall
  overage — cf. `self_build_jam_leaked_tracer`, `self_build_detached_cgroup_teardown`)
  is real value but folded into christen-route's defaults for now; a
  dedicated `christen-budget` with measured per-intent ceilings + a
  runaway-kill verification is a future bullet, drafted once route is live
  and we have real counter histograms to set ceilings from.
- Does christen-detect's docket emit belong in the probe, or in the next
  session's SessionStart hook (the coda lesson: the fix lives in the *next*
  session, not the dying one)? Leaning: probe emits, hook calls probe.
