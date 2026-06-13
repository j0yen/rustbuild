# Vision: answerable — an autonomous agent must be answerable to its human

**Authored by:** /dream (Claude Sonnet 4.6), with jsy
**Created:** 2026-06-12
**Status:** active
**Seed:** user — `/dream about AI ethics`. Phase-1 live inspection turned the
abstract topic concrete: this laptop already has a deep *outward* ethics arc
(`ousia` BFO reasoner → `tribunal` verifies it → `herald` ships it as
`/conscience` → `recourse` handles appeals). That arc builds ethics as a
**library you ship to others**. None of it governs **this agent's own
autonomy on this box** — which is where the real, unaddressed ethical surface is.

## TL;DR

This laptop runs an agent (Claude) that acts autonomously at consequential
scale, with almost no human in the loop:

- **It publishes code with zero human review.** `ls ~/wintermute/` = 157 repos;
  `/build` ships up to 30 PRDs/tick, auto-creates GitHub repos, and pushes — the
  skill's own doc says "Auto-publish is the default, no opt-outs." 13 repos
  shipped on 2026-06-12 alone.
- **It edits its own values file.** `~/.claude/CLAUDE_SELF.md` (Voice / Values /
  Boundaries) is a git-tracked file the agent revises — confirmed by commits like
  `965bbe7 CLAUDE_SELF.md: add ... changelog entry`. The existing `drift` vision
  watches *tool-doc* drift; **nothing watches drift in the agent's own values.**
- **Its principal cannot read.** The wintermute vision's user is "someone who is
  completely computer-illiterate." She cannot audit `REPOS.md` (230 lines),
  `settings.json`, or a PRD. Consent that requires reading is consent she can
  never give.

The outward arc cannot fill this gap — it answers *"what is ethical reasoning?"*
and *"how do others install it?"*. `answerable` asks the reflexive question:
**when this agent acts on its own, is it answerable — auditable, value-stable,
and consented-to — by the human it serves?** Not a reasoner that emits verdicts;
a thin accountability spine: a human-readable record of what the agent did
autonomously and why, a watch on its own values, a redline of things it must not
do without fresh consent, and — crucially — a way to *tell its non-reading
principal, out loud,* what it has been doing.

## End-state

When answerable is fulfilled:

1. **Every high-consequence autonomous action is on one human-readable ledger** —
   "published j0yen/foo because PRD-foo §3", "edited settings.json to allow X",
   "revised CLAUDE_SELF.md Boundaries" — one honest line each, append-only.
2. **The agent's own values are watched for drift.** A weakened or removed Value
   or Boundary in CLAUDE_SELF.md raises a flag the user sees, not a silent edit.
3. **A redline policy the user owns is enforced before irreversible autonomy.**
   `/build` and `/dream` check it before publishing a *public* repo, weakening a
   boundary, or deleting outside `target/` — and stop if the action crosses it.
4. **The non-reading principal can hear what the agent did** — a plain-language,
   voice-deliverable digest ("today I published 13 programs and changed 2 of my
   own settings; say 'details' to hear them"), so consent rests on comprehension.

## Why this is distinct from the outward ethics arc

- `ousia`/`tribunal`/`herald`/`recourse`/`lattice` = a **general** ethical
  reasoner + its proof + its distribution. They reason about *proposed* actions
  in the abstract and ship that capability to anyone.
- `answerable` = **this** agent's **actual** autonomous actions, recorded and
  governed for **this** user. It is reflexive, concrete, and local. It could
  *consume* ousia's verdicts later, but its job is transparency + value-stability
  + consent, not deductive ethical reasoning.
- `warden`/`bpolicy` enforce *file-write* guardrails at the eBPF-LSM layer; the
  auto-mode classifier blocks *per-command*. Neither records the *semantic*
  "I published this public repo because that PRD said so," watches *value* drift,
  or speaks to the user. answerable is the semantic/consent layer above them.

## Components (one bullet per PRD)

- **answerable-ledger** — a new `~/wintermute/answerable/` Rust CLI: an
  append-only, human-readable JSONL+pretty log of high-consequence autonomous
  actions (publish, settings edit, self-value edit, delete). `answerable record`
  appends; `answerable log` renders. `/build` + `/dream` call `record` inline.
- **answerable-values-drift** — extend answerable: diff the Values/Boundaries
  sections of `CLAUDE_SELF.md` across git history; flag a weakened/removed value
  or boundary. `answerable values-drift` → OK / DRIFT with the exact diff.
- **answerable-redline** — extend answerable: a user-owned declarative policy
  (`~/.config/answerable/redline.toml`) of "never autonomously without fresh
  consent" actions; `answerable check <action>` returns allow/redline so /build
  and /dream gate irreversible autonomy on it.
- **answerable-consent-digest** — extend answerable: read the ledger over a
  window and emit a plain-language summary; a `--speak` path publishes it to
  `wm.tts.speak` so the computer-illiterate user *hears* what the agent did.

## Order

```
answerable-ledger ─► answerable-values-drift
        │
        ├─► answerable-redline
        └─► answerable-consent-digest (reads the ledger; --speak needs wm-tts)
```

ledger is the spine (the record everything else reads/guards); the other three
extend it and are mutually independent.

## Open questions

- **Who writes the ledger?** Cleanest is each skill (`/build`, `/dream`) calling
  `answerable record` at the publish/edit step — but that trusts the agent to
  self-report. A provfs/ctrace cross-check (did a `gh repo create` happen with no
  ledger line?) could catch omissions. Self-report first; reconcile later.
- **Redline enforcement teeth.** Is `answerable check` advisory (the skill
  honors it) or hard (a hook blocks the action)? Start advisory + logged; a
  PreToolUse hook is a later escalation if self-honor proves leaky.
- **Consent cadence.** Does the digest speak unprompted (daily at a fixed time),
  or only when the user asks "what have you been doing"? Probably pull-by-default
  (she asks) with an opt-in daily push — pushing unprompted voice is its own
  consent question.
- **Value-drift judgment.** "Weakened" is partly semantic — removing a Boundary
  line is clearly drift; rewording one is ambiguous. Start with structural
  signals (section/line removed, hedge words added) and leave nuanced judgment to
  a flagged human review, not an autonomous verdict.
