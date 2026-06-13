# Vision: litmus — make self-review's own probes prove they tell the truth

## TL;DR

Self-review's Phase B.5 playbooks classify system state with shell
probes — a grep against a hook file, a `getent` against a group, a
verdict over an apply-log count — and feed the result to the `docket`
ledger. When a probe is *wrong*, the ledger faithfully records a false
finding that re-parks run after run, burning review cycles and burying
genuinely-open work. We have a two-week proof of exactly this: the
`ctrace-sessionend-flake` finding was reported **12 times across 10
runs** as "scribe backfill NOT wired" while the wiring was present the
whole time — the probe's grep `scribe backfill` simply never matched the
hook's `"$scribe" backfill`. litmus gives self-review's probes the same
rigor `docket-bind-selftest.sh` already gives the binding: golden-fixture
self-tests so a probe must match the real artifact it checks, a
stuck-finding detector that flags "this has cried wolf N times — audit
the probe," and a binding that surfaces both in every review.

## End-state

When this vision is done:

- Every Phase B.5 probe that greps or classifies a real file is backed
  by a fixture test asserting it produces the right verdict against a
  copy of the real artifact. A probe whose pattern doesn't match reality
  fails CI/self-test on commit — not two weeks and twelve false reports
  later.
- `docket stuck` surfaces findings that have been reported many times
  without ever resolving — the signal "the world may be fine and the
  probe may be lying" — instead of that suspicion living only in a human
  noticing a pattern across journal entries.
- Self-review's Phase B.5 emits a `litmus:` banner listing probe-suspect
  findings, and a probe-suspect finding triggers a fixture audit rather
  than a silent twelfth re-park.

## Components

- **litmus-probe-fixtures** — a fixture-backed self-test harness for B.5
  probes. Each probe that reads a real artifact gets a checked-in fixture
  + expected verdict; the harness runs them in an isolated env (the
  `docket-bind-selftest.sh` pattern) and asserts. Canonical regression
  case: the ctrace wiring grep against the real `ctrace-session-start.sh`
  must yield `HAS_BACKFILL=yes`.
- **litmus-stuck-detector** — `docket stuck` (rust-extend docket): flag
  open findings whose `report_count` / `runs_seen` exceed a threshold
  without resolving, as "probe-suspect — audit the probe, not the world."
- **litmus-selfreview-bind** — wire `docket stuck` into self-review Phase
  B.5 as a `litmus:` banner and a rule: a probe-suspect finding routes to
  a fixture audit (add/repair its litmus fixture) before it re-parks.

## Order

- `litmus-probe-fixtures` — independent; ship anytime.
- `litmus-stuck-detector` — independent of fixtures; extends docket.
- `litmus-selfreview-bind` — depends on `litmus-stuck-detector` (needs the
  `docket stuck` subcommand) and references the fixtures harness.

So: (fixtures ∥ stuck-detector) → bind.

## Open questions

- Should `docket stuck`'s threshold be absolute (`report_count >= 6`) or
  relative (reported in ≥ K of the last N runs while never resolving)?
  The ctrace case was 12 reports / 10 runs — both would have caught it.
- Should a probe-suspect finding *auto-ack* (hide from default views)
  once a fixture audit confirms the probe is the defect and the world is
  fine, or stay visible until the probe is fixed? Likely the latter —
  visibility is the point.
- Does litmus belong as scripts under `~/.claude/skills/self-review/` (a
  harness next to `docket-bind-selftest.sh`) or as its own
  `~/wintermute/litmus` crate? Start as scripts; promote only if probe
  count grows past what a shell harness can hold.
- Distinct from `plumb` (which *calibrates trust* in probe verdicts
  statistically) and `scion` (which fixes `adopt`'s freshness verdict):
  litmus is about whether self-review's *playbook probes* match the
  artifacts they inspect — correctness of the grep, not calibration of
  belief.
