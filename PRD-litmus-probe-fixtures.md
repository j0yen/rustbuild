# PRD: litmus-probe-fixtures — fixture-backed self-tests for B.5 probes

Status: Draft v0.1
build_target: shell
Vision: visions/litmus.md

## TL;DR

Self-review's Phase B.5 playbooks classify system state with shell
probes — grep a hook file, `getent` a group, compare an apply-log count.
A probe whose pattern doesn't match the artifact it inspects produces a
false verdict that the `docket` ledger faithfully re-parks forever. There
is no test that a probe actually matches reality. This PRD adds
`litmus-probe-selftest.sh`: a fixture-backed harness, modelled on the
existing `docket-bind-selftest.sh`, that runs each probe's logic against
a checked-in copy of the real artifact and asserts the expected verdict.
A probe that drifts from the artifact it checks fails on commit, not
after twelve false reports.

## Why this exists (Phase 1 evidence, 2026-06-13)

The `ctrace-sessionend-flake` docket finding is a two-week worked example
of a false-by-construction probe. Measured live this session:

- `docket show ctrace-sessionend-flake` → `report_count: 12`,
  `runs_seen: 10`, first reported `2026-05-30`, finally resolved today
  with reason: *"scribe backfill already wired at line 35 of
  ctrace-session-start.sh as '"$scribe" backfill'; grep pattern 'scribe
  backfill' is a false negative — hook is correct."*
- Ground truth of `~/.claude/scripts/ctrace-session-start.sh`: line 27
  `reap=…ctrace-orphan-reap`, lines 33–35 `scribe=…/scribe` then
  `"$scribe" backfill /home/jsy/.cache/ctrace/sessions \`. The wiring was
  present the entire time.
- The probe (SKILL.md ~line 686) greps
  `'scribe[[:space:]]backfill|"\$scribe"[[:space:]]backfill'`. The first
  alternative never matches `"$scribe" backfill`; the second was added
  only after the manual catch. Across ~10 reviews nobody re-ran the probe
  against the actual file to notice the mismatch — there was no harness
  that would.
- The sibling `memlog` probe had the same class of bug (a multiline
  `getent group memlog && echo yes` capture misclassifying ACTIVE as
  staged), flagged in journals 2026-06-10..13. It has since been
  corrected inline (SKILL.md line 186 now uses
  `getent group memlog >/dev/null 2>&1 && MEMLOG_GROUP=yes`) — but again,
  only after repeated false signals, with no regression test to keep it
  fixed.

`~/.claude/skills/self-review/scripts/docket-bind-selftest.sh` already
proves the *binding* contract in an isolated env (XDG override + temp
db + assert helpers). The same pattern applied to *probes* would have
caught both bugs on day one. That harness is the template.

## What this builds

A new POSIX-sh harness at
`~/.claude/skills/self-review/scripts/litmus-probe-selftest.sh`, plus a
`fixtures/` directory beside it holding checked-in artifact copies.

- **Fixture format.** Each fixture is a real-artifact copy plus an
  expected-verdict line. Concretely, a directory
  `fixtures/<probe-name>/` containing the input artifact(s) (e.g. a copy
  of a hook script, a captured `getent`/`stat` output, a sample
  apply-log) and an `expect.env` of `KEY=value` assertions
  (e.g. `HAS_REAP=yes`, `HAS_BACKFILL=yes`, `MEMLOG_STATE=active`).
- **Probe-under-test extraction.** The probe logic must be exercised
  exactly as Phase B.5 runs it. The harness sources the probe snippet
  against the fixture's inputs (paths pointed at the fixture dir, not the
  live system) and compares emitted KEY=value pairs to `expect.env`. The
  probe text is the single source of truth — the harness does not
  reimplement the grep; it runs the same pattern. (If a probe is only
  embedded in SKILL.md prose, this PRD extracts it to a sourceable
  snippet file under `scripts/probes/<name>.sh` and the harness sources
  that; SKILL.md then references the snippet so prose and tested code
  cannot diverge.)
- **Seed fixtures (ship at least these two):**
  1. `ctrace-wiring` — input is a byte copy of the real
     `ctrace-session-start.sh`; expect `HAS_REAP=yes`, `HAS_BACKFILL=yes`.
     A second negative fixture `ctrace-wiring-unwired` (a hook copy with
     the scribe block deleted) expects `HAS_BACKFILL=no`, proving the
     probe distinguishes the two — not just that it always says yes.
  2. `memlog-state` — inputs are captured `getent`/`stat`/`id` outputs
     for the active case; expect `MEMLOG_STATE=active`. A second fixture
     for the staged case expects `staged-awaiting-install(...)`.
- **Isolation.** Like `docket-bind-selftest.sh`: a `mktemp -d` workdir,
  `trap … EXIT` cleanup, no writes outside the temp dir, no mutation of
  live `~/.claude`, `~/.cache`, or the docket ledger. Idempotent and
  re-runnable.
- **Output + exit.** Per-assertion `PASS:`/`FAIL:` lines (reuse the
  `assert_*` helper shape from `docket-bind-selftest.sh`), a summary
  count, exit 0 iff all pass, exit 1 on any failure. `SKIP:` + exit 0 if
  a required tool is missing (matching the existing selftest's
  `command -v docket` guard).

## Acceptance criteria

1. `bash scripts/litmus-probe-selftest.sh` exits 0 with the seed fixtures
   present and all probe snippets matching their fixtures; output lists a
   `PASS:` line per assertion and a final summary count.
2. The `ctrace-wiring` fixture is a byte-for-byte copy of the live
   `~/.claude/scripts/ctrace-session-start.sh` and the harness asserts
   `HAS_BACKFILL=yes` against it — i.e. the harness would have caught the
   original `scribe backfill` false-negative grep (the negative-control
   `ctrace-wiring-unwired` fixture asserts `HAS_BACKFILL=no`).
3. Introducing a known-bad probe pattern (e.g. reverting the ctrace grep
   to only `scribe[[:space:]]backfill`) makes the harness exit 1 with a
   `FAIL:` naming the `ctrace-wiring` assertion. Demonstrated by a test
   that runs the harness against a deliberately-broken probe snippet copy
   and asserts non-zero exit.
4. The `memlog-state` fixtures assert `MEMLOG_STATE=active` for the
   active inputs and the staged state for the staged inputs, exercising
   the corrected `getent group memlog >/dev/null` logic.
5. The harness writes nothing outside its `mktemp -d` workdir and does
   not touch the live docket ledger, `~/.cache/ctrace`, or `~/.claude`
   scripts. Verified by a scope check (snapshot the harness's parent dirs
   before/after; assert unchanged).
6. The harness exits 0 with a `SKIP:` line when a required external tool
   is absent, never a false failure (mirrors `docket-bind-selftest.sh`).
7. Each tested probe's logic lives in exactly one place: either sourced
   from a `scripts/probes/<name>.sh` snippet that SKILL.md references, or
   asserted to match the SKILL.md block verbatim — so prose and tested
   code cannot drift. A test asserts the snippet and the SKILL.md
   reference are in sync (or that the verbatim block matches).
