# PRD: muster-reap

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/muster
Vision: visions/muster.md

## TL;DR

Once `muster verdict` can defensibly mark a Claude session `orphan` or `stale`,
the natural next step is to clean it up. `muster reap` does that — but only as a
**reviewer-gated proposal**: dry-run by default, a hard refusal to ever target
an interactive or a live session, and `--confirm` required before any signal is
sent. It is the safe hand on the kill switch that self-review's "do not kill"
rule has been waiting for.

## Why this exists

self-review's SKILL is explicit that it must *not* kill duplicate/leaked
sessions (`SKILL.md:70`, *"note duplicates, do not kill"*) precisely because it
had no defensible verdict to act on — so leaked headless runs accumulate and get
re-listed under "Pending your call" every review (2026-06-06, -07). With
`muster verdict` supplying an auditable `orphan`/`stale` call plus its evidence,
a *narrow, guarded* reaper becomes possible. The risk is obvious and asymmetric:
killing a real interactive session destroys the user's work. So this PRD adopts
the fleet's settled safety stance verbatim — the same posture
`recourse-contest` and `mend-bridge` take: **propose, never auto-act.**

## What this builds

A `reap` subcommand extending the `muster` crate (`~/wintermute/muster`),
consuming `muster verdict` — preserve census + verdict modules. SIGPIPE reset
already in `main()`; rustc 1.85, no let-chains.

Hard safety invariants, baked into ACs (mirroring
[[feedback]] / the recourse/mend safety stance):

- **Dry-run by default.** Bare `muster reap` emits the *proposed* signals (the
  exact `kill` it would send, per target, with the verdict + evidence that
  justifies it) and exits without signalling anything. This is the only behavior
  without `--confirm`.
- **Never targets a protected class.** `reap` refuses — with a non-zero exit and
  a printed reason — to propose killing any entry whose verdict is `live` or
  `duplicate`, or whose origin is `interactive-tty`, even with `--confirm`.
  Only `orphan` and `stale` are eligible. (Duplicates are a human call, not a
  reap target — killing the wrong one of two real interactive sessions is
  exactly the catastrophe.)
- **Grace-gated.** Refuses to propose reaping any session younger than its
  grace window, re-reading the verdict evidence (never trusts a stale roster).
- **Subtree-aware, root-only action.** Reports the full subtree (agorabus
  subscriber + worker + tracer children) for each proposed target, but signals
  **only the session root** when confirmed, leaving the established
  orphan-subscriber / `ctrace-orphan-reap` sweepers to collect the children
  (per the vision's open-question default). No subtree `kill -9` storms.
- **SIGTERM-first.** When `--confirm` is given for an eligible target, send
  SIGTERM, wait a bounded grace, and only escalate to SIGKILL if the process
  survives — never SIGKILL-first.
- **Auditable.** `--format json` emits, per target, `{pid, verdict, evidence,
  proposed_signal, would_act (bool)}` so a wrapper / self-review can log exactly
  what was (or would be) done.

Out of scope: any automatic invocation (no timer, no auto-`--confirm`); the
self-review wiring (muster-selfreview-bridge surfaces proposals, it does not
auto-confirm them).

## Acceptance criteria

1. Bare `muster reap` (no `--confirm`) sends **zero** signals and prints, for
   each eligible target, the exact `kill` command it *would* run plus the
   verdict and evidence — verified by asserting no target process changes state.
2. `muster reap --confirm` against an entry whose verdict is `live`,
   `duplicate`, or whose origin is `interactive-tty` refuses (non-zero exit,
   printed reason) and sends no signal.
3. Only `orphan` and `stale` entries are ever eligible; a roster with none
   yields "nothing to reap" and exit 0.
4. An eligible target younger than its grace window is refused with a
   grace-window reason, even with `--confirm`.
5. For each proposed target, the full subtree (subscriber/worker/tracer PIDs) is
   listed, but a confirmed reap signals only the root PID.
6. A confirmed reap of an eligible target sends SIGTERM first and only SIGKILLs
   after a bounded grace if the process survives (verified against a stub
   process that ignores SIGTERM vs one that handles it).
7. `muster reap --format json` emits `would_act`, `proposed_signal`, `verdict`,
   and `evidence` per target.
8. `cargo test` green, including a test proving an interactive-classified entry
   can never be reaped under any flag combination; rustc 1.85, no let-chains;
   census + verdict tests still pass.
