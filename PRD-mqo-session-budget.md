# PRD: mqo-session-budget — bound an agent session's queries, scan cost, and wall-time

Status: Draft v0.1
build_target: rust-cli
Vision: visions/atscale-ai-strategy.md
Repo: j0yen/mqo-session-budget (NOT AtScaleInc; consumes mqo-aggregate-advisor estimates + agentns surface)

## TL;DR

An agent loop over governed data can run away: a clarify cycle that never
converges, a fan-out of expensive table scans, an unbounded wall-clock. An
enterprise will not point an LLM at production warehouse spend without a ceiling.
`mqo-session-budget` is the per-session governor: it tracks queries issued,
estimated scan cost, and elapsed wall-time against a declared budget, refuses the
next step when a ceiling is crossed, and (where the kernel supports it) binds the
budget to the agent namespace so a runaway process is killed, not merely warned.

## Why this exists (Phase-1 evidence)

- `mqo-agent` (this fleet's keystone) introduces a *loop* (clarify can repeat,
  multiple pillars each issue queries). A loop without a budget is the obvious
  production hazard; nothing on the box bounds an agent session's cost today.
- The kernel already exposes the right primitive: agent namespaces support
  `PR_SET_AGENT_BUDGET_LIMITS` → SIGTERM/SIGKILL on overage, with per-NS counters
  (`/proc/$PID/agent_counters`: syscalls/openat/write_bytes/connect) — documented
  in the dream Phase-1.5 kernel surface (`~/wintermute/agentns/`). This PRD
  *consumes* that primitive rather than inventing userspace plumbing.
- That kernel path is **currently blocked**, which this PRD must honestly handle:
  the SessionStart hook reports `agentns ACTIVATION BLOCKED at kernel-prctl`, and
  self-review has flagged `agentns all-zeros` (EINVAL from a CLONE_NEWAGENT flag
  collision) across 16+ runs ([[self_agentns_einval_flag_collision]]). Therefore
  v1 enforces in **userspace** (an accounting ledger + a refuse-next-step gate),
  detects whether the agentns budget primitive is live, and uses it when present
  — degrading legibly, never pretending kernel enforcement it cannot deliver
  ([[feedback_verify_before_concluding]]).
- `mqo-aggregate-advisor` (Fleet 1, MEASURE) already estimates scan cost for an
  MQO. This PRD consumes that estimate as the "cost" axis rather than
  reimplementing cost modeling — the advisor measures, the budget enforces.

## What this builds

A standalone Rust CLI `mqo-session-budget`:

- `mqo-session-budget open --session <id> --budget <budget.toml>` → starts a
  session ledger (queries, estimated cost units, wall-time) under a declared
  ceiling; emits a session handle.
- `mqo-session-budget charge --session <id> --queries 1 --cost <units>
  [--mqo <bound.json>]` → debits the ledger. When `--mqo` is given and
  `mqo-aggregate-advisor` is installed, it derives `cost` from the advisor's
  estimate; otherwise it takes explicit `--cost`. Returns
  `{ok: bool, remaining, exceeded_axis?}`.
- `mqo-session-budget gate --session <id>` → the call `mqo-agent` makes *before*
  each step: returns `allow`/`deny` with the limiting axis, so the loop stops
  cleanly at the ceiling instead of overrunning.
- `mqo-session-budget status --session <id>` → current consumption vs ceiling on
  every axis.
- **Kernel binding (opt-in, capability-detected):** `open --enforce-kernel`
  attempts `PR_SET_AGENT_BUDGET_LIMITS` via the agentns surface. If the primitive
  is unavailable (the current box state), it logs `kernel_enforcement: false` and
  falls back to userspace accounting — the documented, tested default. A
  `mqo-session-budget kernel-probe` subcommand reports whether the primitive is
  live (reads `/proc/self/agent_session`; all-zeros ⇒ not live).
- Ledger persists to a session file so `charge`/`gate` work across the separate
  CLI invocations an agent loop makes.
- `serve` subprocess mode exposing `open`/`charge`/`gate`/`status` as
  `mqo-mcp-server` tools.

## Acceptance criteria

1. `open` then a sequence of `charge` calls that stays under ceiling: each
   returns `ok:true` with decreasing `remaining`; `gate` returns `allow`.
2. A `charge` that crosses any axis (queries OR cost OR wall-time) returns
   `ok:false` with the correct `exceeded_axis`; the subsequent `gate` returns
   `deny`.
3. `charge --mqo <bound>` with `mqo-aggregate-advisor` present derives cost from
   the advisor; with it absent, `--cost` is required and used — both paths tested
   (advisor mocked).
4. `kernel-probe` reports `live:false` on an all-zeros agent_session (the current
   box) and the code path does not panic or block when the primitive is missing.
5. `open --enforce-kernel` on a box without the primitive logs
   `kernel_enforcement:false` and still enforces in userspace — never fails open.
6. Ledger state survives across separate CLI invocations of the same `--session`.
7. Determinism: identical charge sequences yield identical ledgers (wall-time
   axis uses an injectable clock in tests, not the real clock).
8. `serve` answers the budget tool calls; `--help` documents every flag; all
   tests run with no kernel primitive and no live cluster.

## Non-goals

- Does not model query cost itself — it consumes `mqo-aggregate-advisor`'s
  estimate or an explicit value. Cost modeling is the advisor's PRD.
- Does not *fix* the agentns EINVAL kernel bug — it detects and degrades. The
  kernel fix is the agentns/vision-assay arc ([[self_agentns_einval_flag_collision]]).
- Not a billing system — units are abstract budget tokens, not currency.
- Not a scheduler/quota service across many sessions — one session, one ledger.
  A fleet-wide quota broker is a possible follow-on.
