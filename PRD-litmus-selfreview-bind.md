# PRD: litmus-selfreview-bind — surface probe-suspect findings in every review

Status: Draft v0.1
build_target: config
build_into: /home/jsy/.claude/skills/self-review/SKILL.md
Vision: visions/litmus.md
Depends: PRD-litmus-stuck-detector.md

## TL;DR

`litmus-stuck-detector` adds `docket stuck`, which names findings that
have been reported many times without resolving — the "the probe may be
lying" signal. But a query nobody runs changes nothing: the ctrace
false-negative survived ~10 reviews precisely because each review trusted
its probe's verdict and re-parked. This PRD wires `docket stuck` into
self-review Phase B.5 as a `litmus:` banner and adds a rule: a
probe-suspect finding routes to a fixture audit (via the
`litmus-probe-fixtures` harness) before it is allowed to re-park a
twelfth time. This mirrors the already-shipped `plumb-selfreview-bind`
pattern that bound `plumb` into the same phase.

## Why this exists (Phase 1 evidence, 2026-06-13)

- The ctrace finding was re-parked across `runs_seen: 10` (measured via
  `docket show ctrace-sessionend-flake`) because every review accepted
  the probe's "wiring absent" verdict at face value. The fix existed in
  prose in every journal ("add scribe backfill to the hook") yet the
  wiring was *already present* — what was missing was a step that
  *doubts the probe* when a finding won't die.
- Precedent exists and works: `plumb-selfreview-bind` (shipped
  2026-06-13 per CLAUDE_SELF changelog) added `plumb_gate` to Phase B.5.
  `docket digest` is likewise already consumed as a banner. Binding a new
  read-only signal into B.5 is a known, low-risk edit shape.
- SKILL.md already lists numbered Phase B.5 execution steps (e.g. steps
  14–15 run the ctrace backfill + sessionend-resolve playbooks) and a
  playbook/finding table — there is a clear, existing slot to add a
  litmus step and a `litmus_audit` playbook entry.

## What this builds

Edits to `~/.claude/skills/self-review/SKILL.md` (and, if used by the
fixtures PRD, a one-line reference to the probe snippets). No binary.

- **`litmus:` banner step.** Add a Phase B.5 step that runs
  `docket stuck --format json` near the digest/plumb banners and renders
  a one-line `litmus: <N> probe-suspect finding(s): <keys>` (or
  `litmus: clean` when empty). Captured into working memory for Phase E
  journaling alongside the existing docket digest line.
- **`litmus_audit` playbook.** Add a Phase B.5 playbook: when
  `docket stuck` returns a finding whose key maps to a probe with a
  litmus fixture, the review runs `litmus-probe-selftest.sh` for that
  probe. Two outcomes, both recorded to the apply-log:
  - *Probe passes its fixture* → the finding is genuinely world-stuck;
    leave it open and note "litmus: probe verified against fixture,
    finding is real."
  - *Probe fails its fixture* (or the finding has no fixture yet) → the
    probe is the suspect; do **not** silently re-park. Record evidence
    `"litmus: probe <name> suspect — <fixture result>; audit before
    trusting"` and, when the probe is missing a fixture, add building
    that fixture (via `litmus-probe-fixtures`) to Pending.
- **Journal integration.** Phase E's review section gains a one-line
  litmus summary (count of probe-suspect findings + any audited this
  run), so the signal is durable across runs the way the docket digest
  already is.
- **No auto-resolve.** litmus never resolves or acks a finding on its
  own — it only re-routes attention. Resolution stays with the existing
  per-finding playbooks (which already resolve on confirmed-good wiring).
  Visibility is the deliverable.

## Acceptance criteria

1. SKILL.md Phase B.5 gains a documented step that runs
   `docket stuck --format json` and renders a `litmus:` banner line
   (`<N> probe-suspect finding(s): <keys>` or `litmus: clean`), placed
   alongside the existing docket-digest / plumb banner steps.
2. SKILL.md gains a `litmus_audit` playbook entry in the Phase B.5
   playbook table with a trigger (`docket stuck` non-empty), the
   pass/fail branches above, and explicit apply-log `"action"` strings
   for each outcome.
3. The bound flow is non-destructive: the added steps run `docket stuck`
   (read-only) and `litmus-probe-selftest.sh` (isolated, per its own
   PRD); the playbook text states litmus never calls `docket resolve` or
   `docket ack`.
4. The pass branch (probe verified against fixture) leaves the finding
   open and journals it as real; the fail/no-fixture branch adds a
   fixture-audit item to Pending. Both branches are spelled out in the
   playbook prose with their journal/apply-log entries.
5. A dry walkthrough in the PRD-referenced playbook shows the ctrace case
   as the worked example: had litmus been bound, `docket stuck` would
   have surfaced `ctrace-sessionend-flake` by ~run 5–6 and the fixture
   audit would have exposed the grep false-negative then, not at run 10.
6. The edit is consistent with the existing `plumb-selfreview-bind`
   binding style (same banner placement conventions, same apply-log
   shape) so the three signals (docket digest, plumb, litmus) read
   uniformly in a review.
7. SKILL.md remains within its stated lint/line conventions and the
   existing Phase B.5 step numbering stays coherent after insertion.
