# PRD: plumb-selfreview-bind — quarantine a finding whose probe disagrees with ground truth

Status: Draft v0.1
build_target: shell
build_into: /home/jsy/.claude/skills/self-review
Vision: visions/plumb.md

## TL;DR

`plumb check` (core) and `plumb trust` (ledger) can prove a self-review
probe untrustworthy — but nothing yet *consumes* that proof. This PRD
wires plumb into Phase B.5: before a probe's finding is promoted into
Auto-apply or Pending, it passes through `plumb check`. A finding whose
probe disagrees with its ground-truth oracle (or is `uncalibrated` per
`plumb trust`) is **quarantined** — withheld from the docket, logged as
`probe_uncalibrated`, surfaced in Pending as "fix the probe" — instead of
parked as if it were real. The live memlog probe (SKILL.md:186) is fixed
as the first proof.

## Why this exists (Phase 1 evidence, 2026-06-13)

- **B.5 already promotes findings on probe verdicts, with no gate.**
  `self-review/SKILL.md` Phase B.5 runs playbooks
  (`memlog_group_awaiting_activation` at SKILL.md:452,
  `ctrace_sessionend_resolve` near SKILL.md:638) that read a probe and
  either auto-apply, resolve a docket key, or write Pending. There is no
  step between "probe returned X" and "act on X."
- **The memlog probe parked a non-problem this run.** Journal 2026-06-13:
  the `memlog_group_awaiting_activation` playbook ran on a reading that
  *"breaks ACTIVE state check; real state IS active."* A plumb gate would
  have caught the verdict↔oracle disagreement and withheld it.
- **The ctrace probe carried a false finding across runs.** Journal
  2026-06-12 (*"probe was wrong — said wiring absent when it exists"*).
  A `plumb trust` check would have marked the probe uncalibrated after
  that disagreement and kept its finding quarantined until re-proven.
- **The fix half is concrete and small.** SKILL.md:186 is a one-line
  shell bug (`getent … && echo yes` multiline capture). Fixing it is the
  cheapest possible proof that the binding closed the loop end-to-end.
- **Precedent exists for self-review SKILL bindings.** `adopt-self-
  review-bind` (docket Fleet 2) is the sibling pattern: a shell PRD that
  threads a new CLI's verdict into B.5. This mirrors it, gated the same
  report-vs-apply way.

## What this builds

A self-review Phase B.5 integration (shell + SKILL.md edits), not a new
binary. It depends on `plumb` being installed (plumb-core +
plumb-ledger).

### The gate

Add a B.5 sub-step, **`plumb_gate`**, that runs before any playbook
promotes a probe-derived finding:

1. Map the playbook's finding to its `probe-id` (the playbooks whose
   probes are registered: `memlog-active`, `ctrace-backfill-wired`,
   `adopt-report-exists`; extensible as more probes are registered).
2. Run `plumb check <probe-id> --format json` and
   `plumb trust <probe-id> --format json`.
3. **Quarantine** when `result == "disagree"` OR `verdict ==
   "uncalibrated"`:
   - Do **not** park the finding to the docket / do not auto-apply.
   - Log `{"action":"plumb_gate","probe":"<id>","step":"probe_uncalibrated","result":"disagree"}`.
   - Write Pending: `probe <id> disagrees with ground-truth oracle
     (verdict=<v> oracle=<o>) — finding withheld; fix the probe.`
4. **Pass through** when `result == "agree"` and probe is trusted/unknown
   — the playbook proceeds exactly as today.
5. If `plumb` is **not on PATH**, skip the gate (fail-open — the gate
   must never *block* a self-review pass), and note `plumb absent` once in
   the journal so the absence is visible. (Mirrors the fallow-binary
   fail-open rule in /dream.)

### The first fix

As the proving change, correct the live memlog probe at
`self-review/SKILL.md:186` so verdict and oracle agree:

```sh
# before (multiline capture bug):
MEMLOG_GROUP=$(getent group memlog 2>/dev/null && echo yes || echo no)
# after:
getent group memlog >/dev/null 2>&1 && MEMLOG_GROUP=yes || MEMLOG_GROUP=no
```

After this fix, `plumb check memlog-active` returns `agree` — the
regression anchor from plumb-core AC8 flips, proving the loop end-to-end.

### Autonomy posture (drafted default)

Quarantine **+ report** only. The gate does **not** auto-rewrite a
proven-wrong probe in SKILL.md on its own (beyond the one explicit
memlog fix shipped with this PRD). Auto-fixing future probes is gated
behind jsy's call — same open question as `adopt-self-review-bind`
(report-only vs apply). See vision Open Questions.

## Acceptance criteria

1. A `plumb_gate` sub-step is documented in `self-review/SKILL.md` Phase
   B.5, ordered before the playbooks that consume registered probes.
2. The gate maps at least the three registered probes
   (`memlog-active`, `ctrace-backfill-wired`, `adopt-report-exists`) to
   their playbook findings.
3. Given a probe whose `plumb check` returns `disagree`, the bound
   playbook does **not** park the finding and instead writes a Pending
   "finding withheld; fix the probe" line and logs `step:probe_uncalibrated`.
4. Given a probe whose `plumb check` returns `agree` and trust is
   `trusted`/`unknown`, the playbook proceeds unchanged (no behavior
   regression).
5. With `plumb` absent from PATH, a self-review pass completes normally
   (gate skipped, fail-open) and records `plumb absent` once.
6. The memlog probe at `SKILL.md:186` is corrected so that, post-fix,
   `plumb check memlog-active` returns `agree` (verified on this laptop).
7. The change is documented so a future self-review reads the gate as a
   first-class B.5 step, not an optional aside.
