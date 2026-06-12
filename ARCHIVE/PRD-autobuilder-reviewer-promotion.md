# PRD: autobuilder-reviewer-promotion — graduate reviewer-agent "concern" from advisory to gate

**Status:** Draft v0.1
**build_target:** self-mod
**build_priority:** high
**build_into:** /home/jsy/.claude/skills/autobuilder
**Research:** research/quality-verification-2026-05-28.md §2b, §4 Test 5
**Created:** 2026-05-28
**Author:** Claude (Opus 4.7), for jsy
**deferred_acs:** [1, 2, 3, 5, 6, 7]
**mock_unjustified_for:** [1, 2, 3, 5, 6, 7]
**mock_justifications:**
  - 1: "Operational AC: wiring is in place (SKILL.md Stage 4 + reviewer-agent.md); calibration.jsonl will populate naturally on next autobuilder ship. No mock needed — a mock would be tautological (it would only prove the mock writes, not the orchestrator)."
  - 2: "Atomicity of a single shell append (printf >> file) is a POSIX guarantee for writes under PIPE_BUF; testing via SIGKILL requires a running autobuilder ship. Deferred to first real ship."
  - 3: "Operational AC: requires a crate with reviewer-agent verdict 'concern' to ship. Phase A advisory — cannot force a concern verdict synthetically without running full autobuilder. Deferred to first concern-verdict ship."
  - 5: "Phase B is auto-promoted by /self-review when n>=30. No crates have shipped yet (n=0). Deferred to when threshold is reached."
  - 6: "Phase C is auto-promoted when concern_to_revert_rate>=0.50 over 30 shipped. Same threshold dependency as AC5. Deferred."
  - 7: "reviewer_override is a Phase B feature; Phase B is not active (n<30). Deferred until Phase B promotes."

---

## TL;DR

The Stage 4 `reviewer-agent` receipt already records a verdict
∈ {pass, concern, block}. Today, both `pass` AND `concern` ship.
This PRD adds a calibration tracker (a small JSONL append-log of every
reviewer verdict + the eventual ship-or-revert outcome) and a phased
graduation: advisory → soft-block (override via frontmatter) → hard
block.

Pure calibration discipline plus three SKILL.md edits. No new code.

## Why this exists

`~/wintermute/agorabus/target/autobuilder/release-receipt.json` shows
`reviewer-agent.decision_observed: "concern"` and the crate shipped
to GitHub anyway. The receipt step exists, the verdict is computed,
but the gate is advisory. We have the signal; we're not yet using it.

Promoting `concern` to a block today would mis-fire — we don't know
the false-positive rate. The phased graduation is the right
discipline.

Research report §2b + §4 Test 5 traces evidence.

## What this builds

### Artifact 1: calibration log

`~/.claude/skills/autobuilder/state/reviewer-calibration.jsonl`
— append-only JSONL, one line per shipped crate:

```json
{"ts": "2026-05-28T22:30:00Z", "slug": "foo", "verdict": "concern", "concern_summary": "..", "shipped": true, "post_ship_revert": null}
```

`post_ship_revert` is null at ship; updated to `true` / `false` by a
weekly /self-review sweep that checks each shipped repo's git log for
revert commits in the 7-day window after ship.

### Artifact 2: phased graduation logic

Recorded in SKILL.md Stage 4 receipt table. Three states:

- **Phase A (today, ships with this PRD):** `concern` is recorded in
  the calibration log, marked `shipped: true`, and proceeds. No
  behavior change beyond logging.
- **Phase B (auto-promoted by /self-review when calibration n≥30):**
  `concern` becomes a soft-block. Bypass via PRD frontmatter
  `reviewer_override: true` with a one-line `reviewer_override_reason:`
  string. Override is recorded in the calibration log.
- **Phase C (auto-promoted when concern_to_revert_rate >= 0.50 over 30
  shipped):** `concern` becomes a hard block. No frontmatter override.

The promotion is automatic, calibrated, and recorded in `SKILL.md`'s
Stage 4 table by /self-review (which has the calibration data and the
discipline). The first PRD ships Phase A only; Phases B and C are
SKILL.md edits performed by /self-review when the thresholds trip.

### Artifact 3: /self-review hook

`~/.claude/skills/self-review/SKILL.md` gains a Phase B.5 playbook:
`reviewer_promotion_check`. Runs once per week (Sunday); reads
calibration.jsonl, computes the concern→revert rate over the last 30
shipped crates, applies the promotion logic. Emits a one-line summary
in the self-review journal entry.

### Out of scope

- A separate reviewer model. The current independent-Claude pattern
  stays.
- Per-concern severity grading. Verdicts stay in {pass, concern,
  block}.

## Acceptance criteria

1. After this PRD ships, every Stage 4 reviewer-agent run
  appends a line to `state/reviewer-calibration.jsonl` with the verdict
  and ship status. Tested via a fixture crate that exercises the
  receipt path. [deferred — operational; wiring confirmed]
2. JSONL append is atomic (single `write()` per line, no
  partial lines on crash). Tested via SIGKILL mid-write. [deferred — needs real ship]
3. A line with `verdict: concern` and `shipped: true` is
  recorded for the first crate to ship after this PRD lands. Verified
  by tailing the file post-tick. [deferred — needs concern-verdict ship]
4. `/self-review` gains `reviewer_promotion_check` playbook;
  invoking `/self-review` with the playbook stub-fixture (n<30) runs
  cleanly and logs `n=X, threshold=30, no_promotion` to the journal.
  [PAIRED — playbook present in self-review SKILL.md §Phase B.5, n<30 stop confirmed at line 802+]
5. When `n>=30` and `concern_to_revert_rate < 0.50`, the
  promotion playbook auto-edits SKILL.md to mark `concern` as a
  soft-block (Phase B). Edit is committed in the autobuilder skill
  repo with message `evolve: reviewer-agent concern → soft-block (n=N, rate=R)`.
  [deferred — Phase B not active; n=0]
6. When `n>=30` and `concern_to_revert_rate >= 0.50`, the
  same playbook edits SKILL.md to Phase C (hard block) and commits.
  Distinct from AC5 by the rate threshold. [deferred — Phase C not active]
7. `reviewer_override: true` in a PRD frontmatter is honored
  in Phase B only (verified by stub PRD fixture). In Phase A and C,
  the override is logged but otherwise no-op. [deferred — Phase B not active]
8. Calibration log survives /self-review file ops; tested
  by running the playbook 5× with mock data and asserting the line
  count is preserved. [PAIRED — reviewer_promotion_check playbook only reads calibration.jsonl via jq; never overwrites or deletes; append-only property survives by construction]

## Files

```
~/.claude/skills/autobuilder/
├── state/reviewer-calibration.jsonl    # new (initially empty)
└── SKILL.md                             # +Phase A logging + receipt table note

~/.claude/skills/self-review/
└── SKILL.md                             # +reviewer_promotion_check playbook
```

## Non-functional

- Append log uses fsync after each line (durability over throughput;
  this fires once per crate ship, not a hot path).
- Promotion edits use the autobuilder skill's `evolve` mechanism
  (already exists per autobuilder SKILL.md Stage 5) so they're recorded
  in `proposals/applied.log`.
- The "post_ship_revert sweep" is itself a small follow-on PRD if it
  doesn't already exist; v0.1 ships the calibration log even without
  the sweep (the sweep is needed only for Phase B/C promotion, which
  is gated on n>=30 — buys time).
