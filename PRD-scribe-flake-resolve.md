# PRD: scribe-flake-resolve — prove the hole-closer is live, then close the docket finding

Status: Draft v0.1
build_target: shell
Vision: visions/scribe.md

## TL;DR

`docket` has carried `ctrace-sessionend-flake` at **runs_seen: 5** for at
least three nights, and the only thing that ever drives its daily
`residual` to 0 is the self-review's own backfill pass — the band-aid,
not a fix. Once `scribe-reap-wire` + `scribe-startup-sweep` wire recovery
into the SessionStart hook (and `mend-ctrace-render` lands graceful
render-on-exit), the band-aid should stop being load-bearing. This PRD is
the verify-and-close step: teach self-review to confirm the hook wiring
is actually live and that the day's residual was reached *by the hook,
not by the review's own backfill*, and only then `docket resolve` the
finding so it stops re-escalating.

## Why this exists

- **A shipped fix that doesn't kill its symptom is the failure
  `assay`/`mend` exist to catch.** All five scribe v0.1 components are
  marked shipped, yet the flake still fires (runs_seen: 5). The vision
  update 2026-06-08 traces this to integration that never reached the
  live hook. Resolving the docket entry *without* verifying the wiring
  would just hide the gap; this PRD resolves it only on proof.
- **Self-review currently can't tell "fixed" from "band-aided."** Its
  Phase B.5 `ctrace_scribe_backfill` playbook renders the gaps and
  reports residual 0 — but residual 0 reached by the review's own
  backfill looks identical to residual 0 reached by the hook. The
  finding therefore re-counts forever. The signal that the fix holds is
  **the review's backfill rendering 0** (the hook already closed
  everything) — exactly the success condition `mend-ctrace-render` AC5
  defines for the exit path.
- **The wiring is greppable, so verification is cheap and deterministic.**
  After the two hook PRDs land, `ctrace-session-start.sh` contains
  literal `ctrace-orphan-reap` and `scribe backfill` calls. Self-review
  can assert their presence in one grep — a concrete, non-flaky
  verification, not a vibe.

## What this builds

A `shell` change to the self-review skill
(`~/.claude/skills/self-review/SKILL.md` + its Phase B.5 scripts), adding
a `ctrace_sessionend_resolve` verification that runs after the existing
`ctrace_scribe_backfill` playbook:

1. **Assert the hook wiring is live.** Grep
   `~/.claude/scripts/ctrace-session-start.sh` for both an
   `orphan-reap` invocation and a `scribe backfill` invocation. If either
   is missing, the recovery path is not actually wired — do **not**
   resolve; leave the finding open with evidence "wiring absent".
2. **Distinguish hook-closed from review-closed.** Record the count of
   logs the review's own backfill rendered this run. If the hook is doing
   its job, that count trends to 0 (the SessionStart sweep already closed
   everything before the review ran). Resolve only when wiring is present
   **and** the review's backfill rendered 0 this run (nothing left for
   the band-aid to do) — i.e. the success signal from `mend-ctrace-render`
   AC5 observed end to end.
3. **Resolve the docket finding.** On both conditions met, run
   `docket resolve ctrace-sessionend-flake` (with evidence: the grep hits
   + the 0-backfill count) so it stops re-escalating. If a later run
   regresses (review backfill renders >0 again), the existing
   `ctrace_scribe_backfill` playbook re-reports the finding and the
   streak restarts — self-healing in both directions.
4. **Coordinate, don't duplicate.** This consumes the outcomes of
   `scribe-reap-wire`, `scribe-startup-sweep`, and `mend-ctrace-render`;
   it adds no new render or reap logic of its own. It is purely the
   assay/verify handoff that closes the loop.

Constraints: shell + skill-doc edit only. No new binary. The resolve must
be conditional and reversible — never blanket-resolve a finding whose
symptom could recur.

## Acceptance criteria

1. Self-review Phase B.5 gains a `ctrace_sessionend_resolve` step that
   runs after `ctrace_scribe_backfill`.
2. The step greps `ctrace-session-start.sh` for both an `orphan-reap`
   call and a `scribe backfill` call; with either absent it does **not**
   resolve and records evidence "wiring absent" — verified against a
   fixture hook script missing each call.
3. With wiring present **and** the run's own backfill having rendered 0
   logs, the step runs `docket resolve ctrace-sessionend-flake` exactly
   once with evidence attached — verified against a fixture where both
   conditions hold.
4. With wiring present but the run's backfill rendering >0, the step does
   **not** resolve (the hook didn't fully close the gap this run) and the
   finding stays open.
5. The step is read-mostly: its only mutation is the conditional
   `docket resolve`; it never edits hook scripts or renders summaries
   itself.
6. A regression after resolution (backfill renders >0 on a later run)
   re-opens the finding via the existing `ctrace_scribe_backfill`
   playbook — confirmed by walking the two-run sequence
   (resolve → regress → re-report).
