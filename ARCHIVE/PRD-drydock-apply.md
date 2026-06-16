# PRD: drydock-apply — drain the auto lane, and only the auto lane

**Status:** Draft v0.1
**build_target:** rust-extend
**build_into:** /home/jsy/wintermute/drydock-survey
**Vision:** visions/drydock.md
**Repo:** j0yen/drydock-survey (NEVER AtScaleInc)

## TL;DR

Classification and digest make the safe work *visible*; nothing yet *does* it.
drydock-apply is the narrowly-scoped executor that runs the remediation commands
for lane `auto` and refuses everything else. Dry-run by default, `--apply`
required to act. It never restarts a guarded voice daemon, never installs a
kernel package, never touches an `approval` item — so by construction it stays
inside the immutable self-review guardrail. It writes each action to the
drydock-ledger so convergence is provable. This is the piece that finally lets
the recurring "adopt N/N not-current" line shrink without a human reading it.

## Why this exists

Verified 2026-06-16:

- The whole drydock fleet is worthless if the safe lane still needs a human to
  run it. The recurring symptom — "adopt … installed-stale (many)" every
  self-review for a week — is *exactly* the auto-safe lane: idempotent plain-CLI
  installs that no policy reason keeps parked, yet nobody runs them.
- The Explore pass over the rollout ecosystem confirmed the live blocker:
  `SKILL.md:830` declares self-review *"never runs `rollout apply`
  autonomously … immutable,"* and changeover/vigil both await jsy's approval to
  lift it. drydock-apply does **not** fight that guardrail — it is scoped to a
  strictly-narrower set (plain-CLI `adopt apply` and non-voice idempotent
  installs) that the guardrail's intent (don't drop live voice subscribers) does
  not cover. Voice daemons, kernel, and approval items are out of scope by
  construction.
- `adopt apply` already ships dry-run-by-default and idempotent (per the `vest`
  vision's correctness work) and `rollout install` already restarts the owning
  daemon — drydock-apply orchestrates these existing, safe primitives for the
  auto lane only; it re-implements neither.

## What this builds

Extends the `drydock-survey` crate with an `apply` module and a `drydock-apply`
subcommand/binary.

**Behavior**
- Consume classify JSON; select **only** items with `lane == "auto"`. Refuse
  (skip + log) every `window`/`reboot`/`approval` item — these are never
  actionable by drydock-apply, even with `--apply`.
- Dry-run by default: print the exact commands that would run. `--apply` required
  to execute. Each command is run serially; a failure on one item is recorded and
  does not abort the rest.
- **In-flight guard:** before acting, refuse to run if a `/build` or
  `/autobuilder` cargo build is active (check for an active claude-build cgroup /
  known build markers) — never install over a compile in progress.
- After each action, append an outcome to the drydock-ledger
  (`item, command, result: ok|failed, ts`).
- **Guardrail assertion:** a unit test asserts that for any classify input, no
  command is ever emitted/run for an item whose lane is not `auto` — the guarded
  set cannot leak into execution.

**Deps:** inherits survey's; `std::process` to invoke adopt/rollout. No network
beyond what those tools do. Time supplied by caller for ledger writes.

**UX**
```
drydock-classify --json | drydock-apply               # dry-run: shows auto-lane plan
drydock-classify --json | drydock-apply --apply       # executes auto lane only
drydock-apply --apply --now <ts>                       # runs full pipeline + ledger
```

**Posture:** ships dry-run-default and is NOT wired into the self-review cron.
Wiring the auto-drain into the cron is jsy's opt-in (see vision Open Questions);
this PRD delivers the safe executor, not the schedule.

## Acceptance criteria

1. `drydock-apply` (no `--apply`) prints the auto-lane plan and executes nothing
   (verified: filesystem and installed binaries unchanged).
2. Only `lane == "auto"` items are ever selected; a classify input containing
   `window`/`reboot`/`approval` items runs/None of their commands even with
   `--apply` (guardrail unit test asserts no non-auto command is emitted).
3. `--apply` executes auto-lane commands serially; one item's failure is recorded
   and does not abort remaining items.
4. The in-flight build guard refuses to act (clear stderr, exit non-zero) when a
   build is active; acts normally when none is.
5. Each executed action appends an outcome record to the drydock-ledger
   (`item, command, result, ts`), time caller-supplied.
6. Re-running after a successful apply is a no-op: items it fixed now survey as
   `fresh` and are absent from the auto lane (idempotent convergence verified
   against a stubbed adopt/rollout).
7. `--help` documents the dry-run/`--apply` distinction and the
   auto-lane-only scope; `cargo test` green; `cargo clippy` clean on the crate's
   own code.
