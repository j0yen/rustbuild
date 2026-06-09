# Vision: fallow — dream rests when the field is exhausted

> A field left unplanted recovers. `/dream` does the opposite: when the
> inward signal is exhausted it plants anyway, burning a full Claude
> invocation to re-derive "nothing new" and append yet another saturation
> note to gossip. `fallow` gives dream a cheap, deterministic way to know
> the field is spent — so it rests instead of churning, and escalates to
> the user instead of re-noticing in prose.

**Authored by:** /dream (Claude Sonnet 4.6), with jsy
**Created:** 2026-06-08
**Status:** active
**Seed:** bare `/dream` (9th pass of 2026-06-08) + Phase-1 live inspection
(gossip tail showing 8 documented no-op passes today; `docket` vision as
the proven analog).

---

## TL;DR

On 2026-06-08, `/dream` ran **nine times**. The 1st–4th passes drafted real
fleets; the 5th–9th each walked Phases 0–1 in full — recall seeds, gossip
tail, journal, 51 vision docs — concluded "no new evidence-backed signal,
drafting again = duplication," and appended a prose saturation note to
gossip. `grep -cE "no PRDs drafted|saturation scan"` over
`notes/gossip.md` returns **30**. Each no-op pass is a full Claude
invocation (the active session burned 79 min / 239k events / 97k `sed`
calls per today's journal) spent to re-decide something a prior pass
already decided.

This is the **same pathology `docket` solved for self-review**: a process
that re-discovers the same finding every run, writes it in prose, parks it,
and re-discovers it next run — with no structured memory of "I already
concluded this." Docket gave self-review findings an identity, a streak,
and an escalation threshold. `fallow` is the dream-side equivalent: a tiny
ledger + fingerprint that lets dream answer **"has the inward signal
actually changed since my last productive pass?"** deterministically,
*before* spending a research pass — and that converts a fifth consecutive
"nothing new" from a silent re-note into an explicit outward-steer request
to the user.

The relationship to docket is sibling, not duplicate: docket tracks
*findings* surfaced by self-review; fallow tracks *dream's own drafting
cadence and saturation state*. Different producer, different surface,
different lifecycle. They cross-link but do not overlap. See [[docket]].

## Why this is real (Phase 1 evidence, 2026-06-08)

Measured live this session:

- **Saturation is the norm now, not the exception.** Gossip carries
  documented no-op passes labelled "saturation scan #1" through "#7" plus
  the 8th-pass note, all dated 2026-06-08. The 5th–8th notes are nearly
  verbatim restatements of each other ("Drafting again = duplication.
  Bottleneck is implementation, not ideation").
- **The cost is a full Claude invocation per no-op.** Today's journal
  records the active dream/build session at **79m36s, 51,373 writes,
  `sed`×97,812, 239k events**. Even a pure-research no-op pass walks
  recall (5+ queries), reads 51 vision docs, the journal, and the manifest
  before concluding nothing.
- **The queue proves the bottleneck has moved.** `ls PRD-*.md | wc -l` =
  **104 open PRDs**; 51 vision docs. The 5th–8th gossip notes all say the
  same thing in prose: *"the inward queue outruns /build throughput; the
  bottleneck has shifted from ideation to implementation."* That judgment
  is recomputed by eyeball every pass with no stored state.
- **The escalation never fires deterministically.** Every saturated pass
  says "the honest move is to surface the outward choice to the user" but
  none of them *do* it as a built-in step — it depends on a human running
  `/dream` interactively and a model choosing to ask. The 9th pass (this
  one) finally asked; passes 5–8 only wrote that they should.
- **`docket` is the proof the pattern works.** docket's own vision (2026-05-29)
  documents the identical anti-pattern for self-review ("agorabus daemon
  stale binary appears 7× in the 2026-05-28 journal") and the fix —
  streak + threshold + auto-close — is shipped and cited as effective.

## End-state

When `fallow` is done:

- `/dream` Phase 0 begins with `fallow check`. If the inward-signal
  fingerprint is unchanged since the last *productive* pass, dream
  short-circuits: it does **not** walk full Phases 0–1, does **not**
  re-read 51 vision docs, and emits a single one-line gossip note instead
  of a multi-paragraph saturation essay.
- A consecutive-saturation **streak** is tracked. When it crosses a
  threshold (default 3), dream's response to a manual invocation flips from
  "draft inward" to **"ask the user for an outward steer"** — deterministically,
  not at model whim.
- A timer-fired (non-interactive) dream pass that hits saturation logs the
  streak and exits cheaply rather than producing a no-op essay; the cost of
  a saturated overnight pass drops from a full research walk to a fingerprint
  compare.
- The fingerprint is honest: it hashes the *actual inward corpus* dream
  reads (open-docket slugs, the journal "Pending your call" block, the open
  PRD slug set, the gossip last-drafted marker), so "unchanged" means the
  evidence genuinely didn't move — not that the model felt unimaginative.

## Components (PRD-sized)

1. **fallow-fingerprint** — new `fallow` rust-cli at `~/wintermute/fallow/`.
   `fallow record <outcome>` appends a pass record (timestamp, drafted-count,
   fingerprint) to a ledger under `~/.claude/fallow/`. `fallow fingerprint`
   computes a deterministic digest of the inward-signal corpus and prints it.
   This is the substrate; everything else reads it.

2. **fallow-check** — rust-extend `fallow`. `fallow check` compares the
   current fingerprint against the last record whose drafted-count > 0
   (the last *productive* pass). Exit 0 + "fresh" when changed; exit 1 +
   "fallow" when unchanged, printing the consecutive-saturation streak.
   Mirrors docket's streak/escalation counter. Depends on fallow-fingerprint.

3. **fallow-dream-wire** — config / skill-edit into `~/.claude/skills/dream/SKILL.md`.
   Add a "Phase 0.5 — Check the field" that runs `fallow check` before the
   full research walk; on `fallow`, short-circuit to a one-line gossip note;
   when the streak ≥ threshold and the invocation is interactive, escalate
   the outward-steer `AskUserQuestion` as a built-in step. Calls `fallow
   record` at the end of every pass. Depends on fallow-check.

## Order

```
fallow-fingerprint  →  fallow-check  →  fallow-dream-wire
```

fallow-check needs the ledger + fingerprint from fingerprint; the wire needs
`fallow check`'s exit-code contract from check.

## Open questions

- **Threshold value.** Default 3 (mirrors docket's "3+ runs"). Should an
  *interactive* `/dream` escalate sooner (streak ≥ 2) since a human is
  present to answer, while the overnight timer waits for 3? Leave configurable.
- **Should fallow also gate the timer itself?** A further step (Fleet 2)
  could have `claude-dream.timer`'s `ExecStart` consult `fallow check` and
  *skip the Claude invocation entirely* on a saturated field — turning a
  no-op pass from "cheap research walk" into "zero cost." Captured here as a
  bullet, not yet drafted; it depends on the wire landing first and on
  deciding whether skipping overnight passes risks missing a genuinely-new
  fresh signal (a new incident that lands between passes).
- **Fingerprint corpus membership.** Open-docket slugs + journal pending
  block + open-PRD slug set is the v1 corpus. If it proves too coarse (a
  trivial journal edit flips the fingerprint and defeats the short-circuit)
  or too fine, the corpus is the tuning knob — revisit after observing
  real streaks.
