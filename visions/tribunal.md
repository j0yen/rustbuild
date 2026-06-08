# Vision: tribunal — an ethics engine you ship unverified is just an opinion with extra steps

**Authored by:** /dream (Claude Opus 4.8), with jsy
**Created:** 2026-06-08
**Status:** active
**Seed:** bare `/dream` (interactive) + Phase-1 live inspection. The inward
self-tooling space is saturated — three consecutive `/dream` ticks logged
"saturation scan — no PRDs drafted" (gossip 2026-06-06 ×2, 2026-06-08). The
productive frontier turned **outward** this morning: `ousia` (a BFO-grounded
ethical reasoner) and `herald` (package it as the installable `/conscience`
skill). This pass found the un-dreamt third leg of that same arc.

## TL;DR

`ousia` answers *"what is ethical reasoning?"* — it forward-chains ten axioms
and `ousia-guard` returns `allow | flag | deny` with a justifying axiom chain.
`herald` answers *"how does anyone else get it?"* — it packages that as the
`/conscience` Claude Code skill and publishes it through a `j0yen` marketplace.
Between them is a hole big enough to ship a broken conscience through: **nothing
proves the verdicts are correct.**

The reasoning agent authors the ten axioms *and* (if left to the default
autobuilder loop) would author the tests that "prove" them. That is the exact
tautology this laptop has already been burned by — `wm-router`'s safety score
fell **100% → 73.5%** the moment it was validated on a held-out set the
edit-agent hadn't written (memory `feedback_agent_written_fixtures_tautology`).
An ontology has 509 classes and an open-world assumption; a self-graded ethics
engine is the single most dangerous place to let that tautology stand —
"deductively correct against my own axioms" is not "renders the verdict a human
would accept."

`herald`'s own end-state #4 already *demands* the fix — "the distribution
channel is itself trustworthy … an 'ethics skill' isn't shipped broken or
unverifiable" — but **no PRD in either vision makes that true.** `tribunal`
is that PRD fleet: an **independent judge of the judge**. It lives in its own
workspace (`~/wintermute/tribunal`), deliberately *outside* the thing it
evaluates, and it does three things:

1. **Structural conformance** — proves the emitted ontology actually is what the
   paper claims (OWL 2 DL profile, single inheritance, the ten-axiom encoding
   table), the `ousia-conformance` suite the `ousia` vision explicitly deferred
   "to a later /dream pass."
2. **Independent scenario corpus** — a held-out set of ethical-decision cases
   whose *expected* verdicts were **not** derived by paraphrasing the axioms,
   sourced from external material (the Federation philosophy tenets restated as
   concrete dilemmas, the paper's §8.3 worked examples, public moral-scenario
   framings), each carrying provenance.
3. **Bench + gate** — run `ousia-guard` over the corpus, score verdict accuracy
   (with a hard zero-tolerance for *false-allow* on deny-class cases — the only
   truly unacceptable failure for a conscience), and **block `herald`'s publish
   flow** unless conformance passes and the bench clears threshold.

The judge does not author the law and does not benefit from the verdict. That
separation is the whole point.

## End-state

When fulfilled:

1. `tribunal conformance --owl world-ontology.owl` proves, offline and
   deterministically, that `ousia-forge`'s output satisfies the paper's §8.1
   (OWL 2 DL), §8.3 (ten-axiom encoding table present & well-formed), §8.4
   (TBox-only), and single-inheritance-on-all-classes claims — turning prose
   conformance assertions into a pass/fail exit code.
2. `tribunal bench --guard ousia-guard --corpus corpus/` runs the full held-out
   corpus through `ousia-guard`, reports per-tenet accuracy + a confusion table,
   and **fails loudly on any false-allow of a deny-class case**.
3. The corpus is independently sourced and provenance-tagged; an explicit AC and
   a human spot-check (jsy) guarantee its expected verdicts were not generated
   from the axioms it is meant to test.
4. `herald-pack` / `herald-market` **cannot publish `/conscience`** unless
   `tribunal` passes — satisfying `herald` end-state #4 with a mechanism, not a
   promise. A reasoning regression literally cannot reach another person's
   machine.
5. The pattern generalizes: any future "ship a capability outward" vision gets a
   `tribunal`-style independent evaluation gate before `herald` distributes it.

## Why this is real (Phase 1 evidence, 2026-06-08)

- **The tautology has already cost this laptop.** `wm-router` safety scored
  100% on agent-written fixtures, **73.5% on an independent held-out set**
  (memory `feedback_agent_written_fixtures_tautology`: *"when an edit-agent
  writes both rules and test fixtures, recall/accuracy claims prove nothing"*).
  Per hard-rule-1 every `/dream` PRD is built by `/build`, which delegates Rust
  to `/autobuilder` — the loop that writes code *and its tests* in one cycle.
  Left alone it will grade `ousia` on its own homework.
- **herald promises trust it cannot deliver.** `visions/herald.md` end-state #4
  requires every published plugin to be non-"broken or unverifiable," yet
  `PRD-herald-pack` / `PRD-herald-market` / `PRD-herald-conscience` contain no
  verdict-correctness check — only `skill-doctor`/`skill-manifest` *structural*
  validation. "The SKILL.md is well-formed" ≠ "the ethics it encodes are right."
- **ousia itself deferred the conformance leg.** `visions/ousia.md` open
  questions: *"A dedicated `ousia-conformance` PRD may be worth splitting out of
  reason's ACs once reason exists. Deferred to a later /dream pass."* This is
  that pass. The paper's §8.1–8.3 give concrete, testable claims (OWL 2 DL
  profile, single inheritance across all classes, the 10-axiom → encoding
  table) per `PRD-ousia-reason` lines 14/23/27/43.
- **The contracts to test against already exist on disk.** `PRD-ousia-guard`:
  `ousia-guard check --owl world-ontology.owl --action action.json --format
  json --explain` → `allow|flag|deny` + axiom chain; rules `dignity-floor`,
  `rights-violation`, `authority-without-accountability` (flag), `flourishing`
  (flag). `PRD-ousia-forge` emits the TBox `.owl`. `PRD-herald-pack` defines the
  publish flow to gate. `tribunal` invents no new contract — it consumes these.
- **An independent corpus source is on disk.** `~/Notes/federation-utopian-
  philosophy.md` (canonical, 2026-05-19) states all ten tenets verbatim
  (Primacy of Sentient Dignity … Build the Material Conditions for Goodness) —
  prose dilemmas a human reasons about, *distinct* from the formal §5 axioms.
  Restating these as concrete actions yields expected verdicts grounded in the
  philosophy's intent, not in the axiom encoding.

## Components (PRD-sized)

1. **tribunal-conformance** (`PRD-tribunal-conformance`) — a Rust CLI in a new
   workspace `~/wintermute/tribunal`. `tribunal conformance --owl <file>`
   parses the ontology (`horned-owl`) and proves the paper's structural claims:
   OWL 2 DL profile (§8.1), single inheritance on every class, TBox-only (§8.4,
   no ABox individuals), and that the ten-axiom encoding table (§8.3) is present
   and each axiom is well-formed all-some `SubClassOf`/equivalence. Pass/fail
   exit + JSON report. *Foundational, fully offline.* Ships testable against a
   vendored fixture `.owl` so it builds before `ousia-forge` lands.

2. **tribunal-corpus** (`PRD-tribunal-corpus`) — the independent held-out
   scenario set + its validator, in `~/wintermute/tribunal`. A versioned data
   package: each case = `{ action.json (ABox in ousia-guard's input shape),
   expected_verdict: allow|flag|deny, expected_rule, tenet, provenance }`.
   Cases derived from external material (Federation tenets as dilemmas, paper
   §8.3 worked examples, public moral-scenario framings) — **not** by
   paraphrasing the axioms. `tribunal corpus validate` checks every case against
   guard's action schema and asserts each carries non-axiom provenance. The
   anti-tautology leg.

3. **tribunal-bench** (`PRD-tribunal-bench`) — the harness. `tribunal bench
   --guard <bin> --corpus <dir>` runs `ousia-guard check` over every case,
   compares actual vs expected verdict **and** actual vs expected rule/axiom
   chain, and reports overall + per-tenet accuracy, a confusion table, and a
   distinguished **false-allow count** (deny-class case the engine waved
   through). Depends on corpus + `ousia-guard`; CI stubs guard with recorded
   fixtures matching its `--format json` shape, with the wire contract asserted.

4. **tribunal-gate** (`PRD-tribunal-gate`) — wire conformance + bench into
   `herald`'s publish flow as a hard gate: `/conscience` (herald-conscience)
   may not be packaged/published unless `tribunal conformance` passes, bench
   accuracy ≥ threshold, **and false-allow == 0**. `build_target: mixed`
   (shell/CI wiring + a thin `tribunal gate` subcommand). Depends on bench +
   conformance + `herald-pack`. Fulfills `herald` end-state #4.

## Order

```
conformance ─┐
corpus ──────┼──► bench ──► gate
             │             (also needs herald-pack)
ousia-guard ─┘ (cross-vision)
```

- **conformance** and **corpus** are independent of each other and can build in
  parallel; both build against fixtures now (conformance vs a fixture `.owl`,
  corpus is pure data + validator).
- **bench** needs corpus + the `ousia-guard` binary (cross-vision); CI runs
  against a recorded-fixture stub of guard until the real binary lands.
- **gate** needs bench + conformance + `herald-pack`'s publish flow
  (cross-vision); it is the last to truly close.

## Cross-vision dependencies (read before /build dispatches)

- `tribunal-bench` consumes the `ousia-guard` **binary contract** (`check
  --action <json> --format json --explain` → verdict + axiom chain). `ousia` is
  freshly drafted and **unbuilt** — bench builds/tests against a recorded
  fixture stub now; its real proof waits on `ousia-guard` shipping. The stub's
  JSON shape MUST match the real binary (assert the wire contract, per the same
  discipline gossip noted for `herald-conscience`).
- `tribunal-conformance` consumes `ousia-forge`'s emitted `.owl`. Builds against
  a vendored fixture ontology until forge ships.
- `tribunal-gate` consumes `herald-pack`'s publish flow. Builds the gate logic +
  a dry-run now; wires into the real publish path once herald-pack lands.
- **Ordering hint for /build:** none of tribunal *blocks* ousia/herald, and none
  should *delay* them — tribunal is the safety net that must exist *before the
  first real publish of /conscience*, not before ousia/herald compile. Ship
  tribunal in parallel; enforce the gate at herald-conscience's publish AC.

## Open questions (for jsy)

- **Corpus independence is the crux — and the hardest thing to guarantee
  mechanically.** A validator can assert provenance metadata exists, but it
  cannot prove a case wasn't secretly back-derived from an axiom. Drafted
  mitigation: cases sourced from the Federation philosophy doc + paper §8.3
  worked examples + public framings, each tagged with `provenance.source` and
  `provenance.author != "ousia-axioms"`, **plus a human spot-check by jsy** on
  the first corpus cut as a release AC. Is a jsy spot-check acceptable as the
  independence guarantee, or do you want a stronger separation (e.g. corpus
  authored in a session with no ousia-axiom context in scope)?
- **Accuracy threshold + false-allow policy.** Drafted: false-allow on any
  deny-class case == hard fail (zero tolerance); overall accuracy threshold
  starts at a deliberately conservative bar (e.g. ≥ 0.85) and ratchets up as
  the corpus grows. Right starting bar?
- **Corpus size for v1.** Drafted: start small but per-tenet-balanced (≥ 2 cases
  per tenet × 10 tenets × {allow,flag,deny} ≈ 60), grow over passes. Enough to
  be meaningful without faking coverage (no silent caps — bench reports
  per-tenet N).
- **Does tribunal generalize now or stay ousia-only?** Drafted: ousia-only
  until a second outward capability earns an independent corpus. Don't dream
  past the research.
