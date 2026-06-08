# Vision: recourse — the verdict comes back

**Authored by:** /dream (Claude Sonnet 4.6), with jsy
**Created:** 2026-06-08
**Status:** active
**Seed:** bare `/dream` (interactive) + Phase-1 live inspection. The inward
self-tooling space is saturated (three `/dream` ticks logged "saturation scan —
no PRDs"; gossip 2026-06-06 ×2, 2026-06-08). The productive frontier is the
**outward arc** dreamed this morning: `ousia` (a BFO-grounded ethical reasoner),
`tribunal` (proves the verdicts before shipping), `herald` (ships it as the
installable `/conscience` skill), `lattice` (federates it across every BFO
ontology). This pass found the arc's un-dreamt **last leg**: what happens *after*
the verdict ships.

## TL;DR

The outward arc is a one-way street. `ousia` reasons, `tribunal` proves against
a held-out corpus jsy authored, `herald` packages `/conscience` and publishes it
to other people's machines. Then the verdicts happen **in the world** — on
machines this laptop will never see — and *nothing comes back*. There is no
receipt of what `/conscience` decided, no way for a downstream user to **contest**
a verdict they believe is wrong, no path for a real-world disagreement to flow
back into the axioms or the corpus, and no honest measure of whether the shipped
engine behaves the way the corpus predicted it would.

That is a hole the size of the whole point. An ethics engine under an **open-world
assumption** (509 classes, deliberately incomplete) will hit cases the corpus
never imagined the moment it leaves the laptop. `tribunal` proves correctness
*before* shipping against a **static, jsy-authored** corpus; the world is the
ultimate held-out set, and right now its answers never reach the answer key.

`recourse` is the reception-and-amendment layer that closes the arc into a
**cycle**: every verdict emits a **receipt** → a downstream user can **contest**
it (reviewer-gated, never auto-applied) → an upheld contest becomes a new
**held-out corpus case** that re-runs `tribunal` → a **pulse** measures whether
the field matches the corpus → a warranted change is **fed back** through
`herald` as a new ontology version, so shipped `/conscience` instances improve.
reason → prove → distribute → *receive → amend → re-prove → re-ship*.

The elegant hinge: a field contest's provenance author is a **downstream human**,
which is `≠ "ousia-axioms"` — so a contested case **mechanically satisfies
`tribunal-corpus`'s independence guarantee** (PRD-tribunal-corpus AC: every case
must carry `provenance.author != "ousia-axioms"`). A field contest is the most
honest held-out case there is: the axioms could not possibly have written it.

## Why this exists (evidence from Phase 1, 2026-06-08)

- **The arc has no return path.** `herald.md` end-state ships `/conscience` to
  *other people's machines* via a `j0yen` marketplace. Once installed, every
  `allow|flag|deny` verdict happens off-laptop with **zero telemetry back**.
  No PRD in `ousia`, `herald`, `tribunal`, or `lattice` defines a receipt,
  a contest, or a feedback channel. The outward arc is write-only.
- **`herald` end-state #4 demands trust, but only `tribunal` partly delivers
  it — pre-ship.** `tribunal` validates against a held-out corpus that is
  *finite and authored on this laptop*. The open-world assumption guarantees the
  field will produce verdicts the corpus never covered. Nothing keeps the engine
  honest *after* the corpus runs out.
- **The tautology is the laptop's own scar.** `feedback_agent_written_fixtures_tautology`:
  `wm-router` safety fell **100% → 73.5%** the instant it met a held-out set the
  edit-agent hadn't written. `tribunal` answers this with a jsy-authored corpus;
  `recourse` extends the same defense to its limit — **let the world write the
  held-out cases.** A contested-and-upheld field verdict is a test the engine's
  authors provably did not write.
- **The review-gate pattern already exists to make contests safe.** The fleet
  consistently routes low-trust input to a **reviewer-gated proposals file**, never
  auto-merging (gossip: lattice-bridge "NO silent auto-merge"; tribunal-gate
  "false-allow == 0, non-overridable"; recall-observe / skill-doctor proposals).
  `recourse-contest` reuses it verbatim: a contest is a *proposal*, not a change.
- **Receipt prior art is on disk.** `coda` and `daily-receipt` already model the
  "every event emits a durable, renderable receipt" shape. `recourse-receipt`
  applies it to verdicts: a signed, append-only, PII-free record per decision.
- **The corpus wire contract is already specified.** PRD-tribunal-corpus fixes the
  exact case shape — `action.json` (ousia-guard ABox) + `expected.toml`
  (`{verdict, rule, tenet, rationale}`) + `provenance.toml`
  (`{source, source_ref, author, spot_checked_by}`). `recourse-amend` emits
  *exactly* this shape with `source = "field-contest"`, so it drops into the
  corpus with no schema negotiation.
- **Outcome-feedback prior art.** This laptop already drafted
  `recall-outcome-feedback` (MEMORY.md) — the "did the thing we shipped actually
  help?" loop. `recourse` is that loop for ethical verdicts specifically.

## End-state

When fulfilled:

1. **Every `/conscience` verdict emits a receipt.** A signed, append-only,
   PII-free record — `{receipt_id, ts, action_digest, verdict, fired_rule, tenet,
   axiom_chain, ontology_version, guard_version}`. The raw action stays local
   (opt-in); only a content **digest** ever identifies a case. A user (or the
   shipping author) can reconstruct *what was decided, by which axioms, under which
   ontology version* — months later, on any machine.
2. **A downstream user can contest a verdict** they believe is wrong:
   `recourse contest <receipt-id> --expected allow --reason "…"`. The contest
   lands in a **reviewer-gated proposals file** and changes **nothing** until a
   human upholds it. No field input ever silently mutates the corpus or ontology.
3. **An upheld contest becomes a held-out corpus case.** `recourse amend` renders
   the contest into tribunal-corpus's exact `action.json`/`expected.toml`/
   `provenance.toml` shape (`author = downstream user ⇒ independence guarantee
   satisfied mechanically`), then re-runs `tribunal corpus validate` + `tribunal
   gate` so the new world-case is proven against the engine before anything ships.
4. **A pulse measures field behavior against the corpus's prediction.** `recourse
   pulse` aggregates receipts into an honest, opt-in, **aggregate-only** view:
   verdict distribution, contest/override rate, which axioms actually fire in the
   wild, drift across ontology versions. It answers "is the shipped conscience
   behaving the way `tribunal` said it would?" with no per-action data leaving.
5. **Warranted change feeds back through `herald`.** When amended corpus + pulse
   justify it, `recourse feedback` cuts a new versioned ontology changeset (tied
   to the contests that drove it), reviewer-gated, and publishes it through
   `herald-market` so installed `/conscience` instances upgrade. The arc is a
   cycle: the world's disagreement becomes the next version's wisdom.

## Components (PRD-sized)

- **recourse-receipt** (rust-cli, standalone — buildable now) — the verdict-receipt
  format + append-only local sink + `emit`/`show`/`ls`. PII-free by construction;
  raw action local-and-opt-in only. Builds against a checked-in verdict JSON Schema
  + a recorded guard-stub (tribunal-bench's fixture approach), so it does not wait
  on ousia.
- **recourse-contest** (rust-cli — depends on receipt format) — `contest`/`ls
  --pending`; writes a structured contest to a reviewer-gated proposals file.
  Hard rule baked into ACs: a contest is a proposal, never a mutation.
- **recourse-amend** (mixed — depends on contest + cross-vision tribunal-corpus) —
  renders an upheld contest into a tribunal-corpus case (`source="field-contest"`,
  `author=downstream ⇒ independence holds`) and re-runs `tribunal corpus validate`
  + `tribunal gate`. Stub tribunal now; real wire at AC.
- **recourse-pulse** (rust-cli — depends on receipt) — aggregate-only, opt-in field
  telemetry: verdict distribution, contest rate, per-axiom fire counts, version
  drift. No silent caps; surfaces under-fired tenets and verdict-class skew.
- **recourse-feedback** (mixed — depends on amend + pulse + cross-vision herald) —
  cut a versioned ontology changeset from upheld contests, reviewer-gated, publish
  through `herald-market`. The cycle's closing edge.

## Order

```
recourse-receipt  ─┬─→ recourse-contest ──→ recourse-amend ──┐
                   └─→ recourse-pulse ──────────────────────┴─→ recourse-feedback
```

- `recourse-receipt` is the **only** standalone piece — buildable now against
  fixtures. Everything else keys on its receipt format.
- `recourse-amend` has a cross-vision dep on `tribunal` (corpus + gate); stub now,
  wire at AC. `recourse-feedback` has a cross-vision dep on `herald` (market);
  stub now, wire at AC. Neither blocks or delays tribunal/herald.

## Open questions (for jsy)

- **Receipt transport.** v1 is local-and-opt-in (no receipt leaves the machine
  unless the user runs `recourse pulse --export`). Is *any* default-on telemetry
  acceptable, or must the field stay fully air-gapped until the user explicitly
  exports? (Drafted: air-gapped by default — the user mails a pulse, nothing
  phones home.)
- **Contest identity.** A contest's `author` must be `≠ ousia-axioms` to satisfy
  the independence guarantee — but how much identity do we record? (Drafted: an
  opaque per-installation id, no PII; enough to dedup, not enough to deanonymize.)
- **Who is the reviewer for a field contest?** On this laptop it is jsy. On a
  *third party's* installation, the upheld-contest → corpus path runs against
  *their* local corpus fork, or proposes upstream to `j0yen`? (Drafted: local fork
  by default; upstream proposal is an explicit `recourse feedback propose
  --upstream`.)
- **Amendment cadence vs. ontology stability.** Re-versioning the ontology on every
  upheld contest would churn the marketplace. (Drafted: `recourse feedback` batches;
  a version cut is a deliberate reviewer action, not per-contest.)
