# Vision: plumb — a detection probe you never checked against ground truth is a guess in a verdict's clothes

**Authored by:** /dream (Claude Sonnet 4.6), with jsy
**Created:** 2026-06-13
**Status:** active
**Seed:** bare `/dream` (interactive, manual). `fallow check` = fresh
  (streak=0, fingerprint moved 06:35). Phase-1 live inspection found the
  strongest *unaddressed* signal was not a missing fix and not a false
  close-note — it was a **self-review detection probe that returns a
  reading provably contradicted by the system's actual state**, caught
  by reading the probe's own source against today's journal.

## TL;DR

There is an existing verification arc on this laptop, and it is good:
`assay` proves a *fix* outlived its symptom, `warrant` proves a *close
note* that names a new mechanism, `tribunal`/`recourse` prove a shipped
*verdict*. Every one of them assumes the **instrument that took the
reading was correct**. That assumption is false here. Self-review's
Phase B.5 runs deterministic shell probes; at least three of them have
returned readings that the system's real state contradicts — a
false-negative that *parks a non-problem* and a false-positive that
*hides a real one*. Nothing checks the probe.

`plumb` is the missing layer below `assay`: for each probe, an
**independent ground-truth oracle** measures the same condition by a
*different mechanism*. When the probe and the oracle disagree, the probe
is **uncalibrated** and its finding is **quarantined** — withheld from
the docket, flagged for repair — instead of silently parked. A
per-probe **calibration ledger** accumulates agreement history, so a
probe with a record of lying is distrusted until it is re-proven.

A plumb line does not trust the wall to be vertical; it hangs an
independent weight and reads true. plumb does that for every probe that
feeds a verdict.

## Why this exists (Phase 1 evidence, 2026-06-13)

Measured live this session, against the probe source and today's journal:

- **The memlog probe lies right now.** `self-review/SKILL.md:186` reads
  `MEMLOG_GROUP=$(getent group memlog 2>/dev/null && echo yes || echo no)`.
  `getent group memlog` prints the whole group line
  (`memlog:x:NNN:jsy`) *and then* `echo yes`, so `MEMLOG_GROUP`
  captures a multiline string, and the gate at `SKILL.md:200`
  (`[ "$MEMLOG_GROUP" = yes ]`) fails — the probe reports memlog
  **inactive while it is active**. Today's journal (2026-06-13,
  verbatim): *"memlog detection bug — getent+echo multiline capture
  breaks ACTIVE state check; real state IS active."* The B.5 playbook
  `memlog_group_awaiting_activation` ran on that false reading.
- **The ctrace-wiring probe has read false before.** `SKILL.md:644`
  gates resolution on `grep -q 'scribe backfill' "$HOOK"`. The 2026-06-12
  journal records: *"Phase A ctrace-sessionend-flake probe was wrong
  (said wiring absent when it exists)."* A false-absent that carried the
  finding across runs.
- **The adopt-report probe assumed a subcommand that doesn't exist.**
  2026-06-13 journal: *"adopt `report` subcommand not yet implemented;
  manually reported to docket."* The probe's verdict step depended on a
  capability the binary lacks; ground truth (`adopt report --help` exit
  code) would have caught it.
- **These are not one-offs; they are a class.** Three distinct B.5
  probes, three different failure modes (multiline capture, grep
  false-absent, missing-capability), each only caught by a human reading
  the source or the live state. No probe on this laptop checks its own
  verdict against an independent second measurement.
- **The existing verification arc sits one layer too high.** `assay`
  re-runs the *primitive a fix targeted* to see if the symptom outlived
  the fix — but it trusts the probe that *detects* the lingering
  symptom. `warrant` proves a *close note*. `tribunal`/`recourse` prove a
  shipped *verdict*. None proves the *detector*. plumb is the floor
  under all of them.

## End-state

When this is done:

- Every self-review B.5 probe that can be expressed as a verdict has a
  registered **independent oracle** that measures the same condition by
  a different mechanism.
- Before B.5 promotes a probe's finding into Auto-apply or Pending, the
  finding passes through `plumb check`. A probe whose verdict disagrees
  with its oracle is **uncalibrated**; its finding is **quarantined**
  (withheld + flagged for repair), not parked to the docket.
- A **calibration ledger** records every check; `plumb trust <probe-id>`
  reports a probe's agreement history, and an uncalibrated probe stays
  distrusted until it agrees again.
- The recurring hand-written false carries (memlog false-inactive,
  ctrace false-absent) stop reaching the journal as real findings.

## Components (PRD-sized)

1. **plumb-core** — `~/wintermute/plumb/` (rust-cli → `~/.local/bin/plumb`).
   `plumb check <probe-id>` / `plumb check --all`. Reads a declarative
   `probes.toml`: per probe a `verdict` command, an independent `oracle`
   command, a `normalize` rule, and the condition name. Runs both,
   normalizes, emits `agree | disagree | error` with both raw readings.
   `--format json`. Ships seeded with the three known-bad probes.
2. **plumb-ledger** — rust-extend plumb. Append-only calibration ledger
   (`~/.local/share/plumb/calibration.ndjson`, O_APPEND, SIGPIPE-safe).
   `plumb trust <probe-id>` → agreement rate, last-disagreement, and an
   `uncalibrated` verdict under threshold. `plumb trust --all`.
3. **plumb-selfreview-bind** — shell → self-review SKILL.md B.5. Gate
   each probe's finding through `plumb check` before promotion;
   quarantine disagreeing/uncalibrated findings. Fix the live memlog
   probe (SKILL.md:186) as the first proof.

## Order

```
plumb-core  (the oracle-pairing engine + probes.toml)
   ├─ plumb-ledger        (consumes check output → calibration history + trust)
   └─ plumb-selfreview-bind (consumes check + trust → gates B.5 promotion)
```

plumb-core ships first; everything consumes its `plumb check --format
json` contract. plumb-ledger and plumb-selfreview-bind both extend/consume
core but are independent of each other (bind can quarantine on a single
`check` even before `trust` history exists; trust makes the quarantine
sticky).

## Open questions (for jsy)

- **Quarantine scope.** Should an uncalibrated probe block its *single*
  finding (drafted default) or the whole B.5 step? Drafted per-finding —
  one bad probe shouldn't blind the rest of the pass.
- **Trust threshold.** Strict 1.0 (any single disagreement →
  uncalibrated until it agrees again, drafted default) vs a rolling
  agreement rate. Strict is safer for a watcher-of-watchers; configurable
  either way.
- **Autonomy posture.** Should plumb-selfreview-bind *auto-fix* a proven-
  wrong probe (rewrite SKILL.md:186 in place) or only *quarantine +
  report* and leave the rewrite to a human/`/build`? Drafted quarantine+
  report as default, auto-fix gated — mirrors the unresolved report-vs-
  apply question in `adopt-self-review-bind`. See [[docket]].

## Relationship to the verification arc

plumb is deliberately the lowest layer. `assay`, `warrant`, `tribunal`,
`recourse`, `vigil` all answer "is this conclusion true?" for a fix, a
close note, a verdict, or a running binary. plumb answers the question
underneath: **is the instrument that produced the reading itself
trustworthy?** It does not replace them; it removes the assumption they
silently rely on.

---

## Extend: a calibrator you never audited is itself an unchecked instrument (2026-06-13)

The first plumb fleet shipped (plumb-core + plumb-ledger +
plumb-selfreview-bind, all 2026-06-13). plumb now pairs each registered
probe with an oracle, gates B.5 promotion, and tracks trust. But the
recursion stops one level too soon: **nothing audits the registry
itself.** Three live gaps, each caught by reading the shipped
`~/.config/plumb/probes.toml` against the system:

- **The oracle can be a tautology.** The `ctrace-backfill-wired` probe's
  verdict is `grep -q "scribe backfill" <file>` and its oracle is
  `grep -qE "scribe.backfill|..." <file>` — *the same tool reading the
  same file*. The config comment even admits it: "use a slightly
  different grep pattern on the same file." A verdict and oracle that can
  only fail together prove nothing on agreement — this is
  [[feedback_agent_written_fixtures_tautology]] living inside the
  calibration layer that exists to prevent exactly that. plumb measured
  the probe but never measured whether its own oracle was independent.

- **Coverage is unknown and small.** `plumb list` returns 3 probes;
  self-review B.5 has many playbooks (memlog, ctrace-resolve,
  fleet-binary-staleness, warden, agorabus, autobuilder-gate-promote, …).
  The vision end-state says *every verdict-expressible B.5 probe* has a
  registered oracle. There is no command that reports the gap, so "are
  the watchers all registered?" is answered by hand each pass — exactly
  the manual audit plumb was built to retire.

- **The registered verdict drifts from the live probe.** Each
  `probes.toml` verdict is a hand-copy of a SKILL.md probe (the comments
  say so: "The BROKEN probe from SKILL.md:186"). Once
  plumb-selfreview-bind fixes SKILL.md:186, the registered verdict still
  holds the broken version — plumb then calibrates *stale logic* and its
  agreement verdict is meaningless. Nothing checks that the registered
  verdict still matches the live probe it claims to mirror.

### Extend components (PRD-sized)

4. **plumb-independence** — rust-extend plumb. `plumb lint [--all|<id>]`:
   for each probe, extract a coarse *mechanism signature* from verdict and
   oracle (binaries invoked + file paths touched) and flag a probe whose
   verdict and oracle share the same mechanism — same primary tool AND
   same target path — as `tautological`. JSON + human output, nonzero
   exit when any probe is tautological. Seeded proof: `ctrace-backfill-wired`
   must flag (both grep the same file); `memlog-active` must pass (getent
   vs id -nG/stat).

5. **plumb-coverage** — rust-extend plumb. `plumb coverage`: parse the
   self-review SKILL.md B.5 playbook IDs (and the `plumb_gate`
   probe→playbook table) and cross-reference against registered probe
   IDs; report `registered`, `unregistered` (playbook with a
   verdict-expressible condition but no probe), and `orphan` (probe with
   no playbook). JSON + summary count. This makes the end-state
   measurable: "N of M B.5 detectors are calibrated."

6. **plumb-sync** — rust-extend plumb. `plumb sync [--all|<id>]`: for
   each probe whose `probes.toml` entry carries a `source` pointer
   (new optional field: `source = "<file>:<anchor>"`), read the live
   probe text at that anchor and compare its normalized command against
   the registered `verdict`. Report `in-sync | drifted | source-missing`.
   Catches the post-fix staleness where SKILL.md is repaired but the
   registered verdict still mirrors the broken form.

### Extend order

```
plumb-independence ┐
plumb-coverage     ├─ all rust-extend plumb, independent subcommands, parallelizable
plumb-sync         ┘
```

All three add a separate subcommand over the existing `Registry` /
`probes.toml` contract; none depends on another. Ship in any order.
plumb-independence carries live proof (the ctrace probe) so it is the
highest-value of the three.

### Extend open questions (for jsy)

- **Mechanism-signature granularity.** Should `plumb lint` flag on
  *same binary OR same path* (stricter, more false-positives) or *same
  binary AND same path* (drafted default — the ctrace case trips it,
  memlog doesn't)? Drafted AND; a `--strict` flag could widen it.
- **Should `plumb lint` failures feed the B.5 quarantine** the way a
  `check` disagreement does, or only block at registration time? Drafted
  as a standalone audit (CI/self-review-reportable) for now; wiring lint
  into the gate is a follow-on once the heuristic earns trust.
- **`source` anchor format for plumb-sync.** `file:line` is brittle
  across edits; `file:#anchor-comment` is more durable but requires
  adding anchors to SKILL.md. Drafted to support both and prefer a
  named anchor when present.
