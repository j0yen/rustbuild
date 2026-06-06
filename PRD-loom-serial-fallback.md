# PRD-loom-serial-fallback

Status: Draft v0.1

build_target: self-mod
build_priority: medium
build_into: /home/jsy/.claude/skills/build
Vision: visions/loom.md

## TL;DR

The other loom PRDs make most parallel same-target integrations clean
(append-only conventions) or self-healing (rebase-retry). But some collisions
are genuine: two branches really do edit the same source line incompatibly, and
no rebase resolves them. Today those branches re-fan **in parallel every tick**
and re-conflict every tick — an infinite stall. This PRD is the parent-side
backstop: when a target's branches conflict on the same pathset ≥2 consecutive
ticks, /build stops fanning that target in parallel and routes it **one branch
per tick** (serial) until the backlog drains, then resumes parallel.

## Why this exists

- gossip 2026-06-06T06:39:07Z: "a retry tick re-runs the same free-form edit and
  re-conflicts … No forward progress without manual resolution." The same note's
  proposed guardrail: "(or branches should land … not re-fan them in parallel)."
- This is the explicit complement to `loom-rebase-retry`: rebase-retry handles
  *resolvable* conflicts; serial-fallback handles the *unresolvable* residue so
  it still drains deterministically instead of livelocking.
- The signal already exists once the sibling PRDs land:
  `build-shared-cli-dispatch-merge-safe` AC4 and `loom-rebase-retry` AC4 both
  write `last_error=integrate-conflict:<files>` to the per-branch sidecar. This
  PRD is the **consumer** of that telemetry — it must not be built before at
  least one producer ships (otherwise it reads an absent key and never trips;
  see self_build_jq_escape_reads_absent — treat absent as "no streak," never as
  a reason to mis-route).

## What this builds

In the /build tick dispatcher (the Phase-1/scan + fan-out logic in the build
skill, `~/wintermute/build-skill/`):

1. **Conflict-streak ledger.** Maintain, per `(build_into repo, conflicting
   pathset)`, a count of consecutive ticks that ended in
   `last_error=integrate-conflict:*` (or `rebase-broke-build`) for branches
   targeting that repo. Reset the streak to 0 on any clean integrate for that
   repo. Store in the build skill's `state/` (JSON), keyed by repo + sorted
   pathset.
2. **Serial routing.** When a repo's streak ≥2, the next tick selects **at most
   one** `in_progress` same-target branch for that repo to integrate (the oldest
   by branch creation / first-deferred time — deterministic, not arbitrary),
   leaving its siblings untouched (worktrees + branches preserved). Other repos
   are unaffected and keep fanning in parallel.
3. **Drain + resume.** Each serial tick lands one branch (rebase-retry gives it
   a clean shot against the now-updated `main`). When the repo has ≤1
   `in_progress` same-target branch left, clear the serial flag and resume
   normal parallel fan-out.
4. **Telemetry / visibility.** Log to gossip (or the tick journal) when a target
   flips to serial mode and when it resumes, with the pathset and streak count,
   so the pattern is observable rather than silent (no_silent_caps discipline —
   if throughput is being capped, say so).

Constraints / shape:
- Pure scheduling change: never edits code, never force-merges, never drops a
  branch. Worst case it's slower, never wrong.
- Absent/empty telemetry ⇒ streak 0 ⇒ normal parallel behaviour (fail-open to
  today's semantics). A malformed sidecar entry reads as "no streak," never as a
  reason to serialize a healthy repo (self_build_jq_escape_reads_absent).
- Deterministic selection (oldest-first) so reruns/resumes are reproducible.
- Bounded: serial mode auto-clears; it cannot wedge a repo permanently.

## Acceptance criteria

1. The dispatcher records a per-`(repo, pathset)` conflict streak from
   `last_error=integrate-conflict:*` / `rebase-broke-build` sidecar entries, and
   resets it to 0 on a clean integrate for that repo — unit-tested over a
   sequence of synthetic tick outcomes.
2. When a repo's streak reaches 2, the next tick integrates **at most one**
   same-target branch for that repo and preserves all sibling branches +
   worktrees — verified by a simulated tick with 3 conflicting same-target
   branches (1 advances, 2 preserved, none dropped).
3. Serial selection is deterministic (oldest deferred branch first); the same
   input state selects the same branch across reruns.
4. Once a repo has ≤1 in_progress same-target branch, the serial flag clears and
   the next tick resumes parallel fan-out for that repo.
5. Absent, empty, or malformed conflict telemetry yields streak 0 and unchanged
   parallel behaviour (fail-open); a healthy repo is never serialized by a bad
   read.
6. Flipping to and from serial mode is logged (gossip or tick journal) with the
   repo, pathset, and streak — never a silent throughput cap.
7. The change is scheduling-only: a test asserts no branch is force-merged,
   dropped, or code-edited by the fallback path.
