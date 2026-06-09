# PRD: fallow-dream-wire — teach /dream to check the field before planting

Status: Draft v0.1
build_priority: normal
build_target: config
build_into: /home/jsy/.claude/skills/dream/SKILL.md
Vision: visions/fallow.md

## TL;DR

`fallow check` exists and answers, by exit code, whether dream's inward signal
has moved since the last productive pass. Nothing calls it yet. This PRD wires
it into the dream skill: a new **Phase 0.5 — Check the field** that runs
`fallow check` before the full research walk, short-circuits a saturated pass
to a single-line gossip note instead of a multi-paragraph essay, escalates the
outward-steer question to the user once the saturation streak crosses the
threshold, and records every pass's outcome via `fallow record`. This is the
step that actually stops the 9-no-op-passes-a-day waste the vision documents.

## Why this exists

Phase 1 evidence, 2026-06-08 (full set in `visions/fallow.md`):

- `/dream` ran nine times on 2026-06-08; passes 5–9 were no-ops. Each walked
  full Phases 0–1 (recall, gossip, journal, 51 vision docs) before concluding
  nothing. The gossip log carries 30 saturation/no-op markers total.
- Passes 5–8 each *wrote* "the honest move is to surface the outward choice to
  the user" but none *did* it — the escalation depended on a model choosing to
  ask. Only the 9th pass asked. A built-in skill step makes the escalation
  deterministic.
- The two prior fallow PRDs build the mechanism; without this wire it's an
  unused binary. The vision's end-state is specifically "dream Phase 0 begins
  with `fallow check`."

## What this builds

An edit to `~/.claude/skills/dream/SKILL.md` (no binary, no new repo). Insert
a phase between Phase 0 (Listen) and Phase 1 (Research):

**Phase 0.5 — Check the field.**

1. Run `fallow check --json`.
2. **Exit 0 (fresh):** proceed to Phase 1 as normal. The evidence moved;
   dreaming is warranted.
3. **Exit 1 (fallow) and `escalate=false`:** short-circuit. Do **not** walk
   full Phases 0–1 or re-read the vision corpus. Append a **one-line** gossip
   note (`/dream fallow — field unchanged (streak=k); rested`) and end the
   pass. This replaces the multi-paragraph saturation essays passes 5–8 wrote.
4. **Exit 1 (fallow) and `escalate=true`:** the streak has crossed the
   threshold. If the invocation is **interactive**, present the outward-steer
   `AskUserQuestion` (homeward / constellation / companion-kin / name-a-topic)
   as a built-in step rather than at model discretion. If **non-interactive**
   (timer-fired), append a one-line gossip note flagging that the field needs a
   user steer and end cheaply.
5. **At the end of every pass** (drafted or rested), call
   `fallow record --drafted <N> --seed <seed> --note <vision-or-"none">` so the
   ledger and streak stay current.

Also add a short subsection to the skill's "Phases" preamble documenting the
exit-code contract and the `escalate` semantics, and a one-line note in the
`## Hard rules` section: *"Rule 8 — Check the field first. A saturated field
(fallow exit 1) means rest, not a thinner fleet. Never draft past `fallow
check`."*

**Resilience.** If `fallow` is not yet installed (binary missing on `$PATH`),
Phase 0.5 treats the result as `fresh` and proceeds — the wire must never
*block* dreaming just because the tool isn't built yet. Document this fallback
explicitly in the phase text.

## Acceptance criteria

1. `~/.claude/skills/dream/SKILL.md` contains a "Phase 0.5 — Check the field"
   section, placed between Phase 0 and Phase 1, describing the `fallow check`
   gate.
2. The section specifies all four branches: fresh→proceed, fallow+no-escalate→
   one-line gossip + stop, fallow+escalate+interactive→`AskUserQuestion`,
   fallow+escalate+non-interactive→one-line gossip + stop.
3. The section instructs the skill to call `fallow record` at the end of
   **every** pass (drafted or rested), with the documented flags.
4. A new Hard Rule ("Check the field first / never draft past `fallow check`")
   is appended to the `## Hard rules` list without modifying or deleting any
   existing rule (per existing dream hard-rule #2 / #5 append-only ethos).
5. The fallback ("`fallow` missing → treat as fresh, proceed") is documented
   in the phase text.
6. The edit is additive: no existing phase text is removed or reworded beyond
   inserting the new phase and the one Hard Rule. (Verifiable by diff.)
7. The exit-code contract documented in the skill matches `fallow check`'s
   actual contract (0 fresh / 1 fallow / 2 error).

## Out of scope

- Modifying `claude-dream.timer` to skip the Claude invocation entirely on a
  fallow field — that's the Fleet 2 bullet in the vision (depends on this
  landing and on confirming overnight skips don't miss a fresh incident).
- Any change to `fallow` the binary (its contract is fixed by the two prior
  PRDs).
