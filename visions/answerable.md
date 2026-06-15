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

---

## Fleet 2 — the audit trail must audit itself (2026-06-14, `/dream about ethical AI`)

**Authored by:** /dream (Claude Sonnet 4.6), with jsy
**Seed:** user — `/dream about ethical AI` (again). Fleet 1 shipped completely
(ledger, values-drift, redline, consent-digest — all 5 PRDs archived, repo at
`~/wintermute/answerable` v0.5.0). Phase-1 live inspection found that the
accountability spine **is not load-bearing**: it was built and never wired.

### The gap Fleet 1 left open, now concrete

Fleet 1's first open question was *"Who writes the ledger? … Self-report first;
reconcile later."* Today's evidence shows **neither half happened**:

- `~/.local/state/answerable/ledger.jsonl` has **49 lines, every one a
  self-test** (the last five are duplicate `values-drift` entries from
  2026-06-13). Zero real actions.
- `grep -c answerable ~/.claude/skills/build/SKILL.md` = **0**. `/build` never
  calls `answerable record`. On 2026-06-14 this agent pushed `homeward` **22
  times** (v0.21.0 + v0.22.0), committed, and pushed to GitHub — **none of it on
  the ledger.** `/dream` is the same: it commits + pushes PRDs every pass and
  records nothing.
- Every ledger line carries `session:00000000000000000000000000000000` — the
  agentns-dark all-zeros (installed `linux-wintermute` is pkgrel-1, not ≥12).
  So even the self-tests are unattributable.

An accountability ledger that the agent forgets to write to is worse than none:
it *looks* like a complete record while silently omitting everything that
matters. The ethical core of Fleet 2 is **the audit trail must audit itself** —
self-report must actually fire, and an independent check must catch the
omissions self-report will inevitably have.

### Fleet 2 components (one bullet per PRD)

- **answerable-reconcile** [rust-extend] — `answerable reconcile` cross-checks
  the ledger against ground truth the agent *cannot* edit out of existence:
  git push history across `~/wintermute/*` (151 GitHub remotes), `gh repo list`
  creation dates, and `~/dotfiles` git log for `CLAUDE_SELF.md` edits. Reports
  **omissions** (a push/repo-create/value-edit with no matching ledger line) and
  **phantoms** (a ledger line with no matching real action). This is the
  trust-but-verify spine; everything else in Fleet 2 leans on it.
- **answerable-wire-build** [shell/config — edits build SKILL.md] — wire
  `answerable record` into `/build`'s publish, push, bump-commit, and self-mod
  steps so the self-report path the Fleet-1 vision *assumed* actually fires.
  Idempotent helper `answerable-emit.sh` skills call; non-fatal if the binary
  is absent.
- **answerable-wire-dream** [shell/config — edits dream SKILL.md] — same wiring
  for `/dream`'s Phase 5 (commit + push of PRDs/visions) and Phase 3 (PRD
  drafted). Records the generative half of autonomy, not just the building half.
- **answerable-session-truth** [rust-extend] — when agentns is dark
  (`/proc/self/agent_session` all-zeros), fall back to a real, stable session id
  for ledger lines: the provfs `user.prov.session` `comm:pid:uid` form, or the
  agorabus peer id of the writing session. A ledger you cannot attribute to a
  run is a ledger you cannot dispute.
- **answerable-digest-reconcile-bind** [rust-extend] — fold reconcile's verdict
  into the consent digest so the non-reading principal *hears the agent's own
  honesty gap*: "today I recorded 14 actions; an independent check found 3 more I
  did and did not tell you about." Honesty about one's own dishonesty is the
  whole point of an accountability spine.

### Fleet 2 order

```
answerable-reconcile ─┬─► answerable-digest-reconcile-bind
                      │
answerable-wire-build │  (independent — make self-report real)
answerable-wire-dream │
answerable-session-truth  (independent — make lines attributable)
```

reconcile is the new spine (digest-bind reads its verdict). The two wire-* PRDs
and session-truth are mutually independent and can ship in any order; they make
the self-report path real and attributable so reconcile has fewer omissions to
report over time.

### Fleet 2 open questions

- **Reconcile's matching tolerance.** A push at 17:08 and a ledger line at
  17:09 are the same action; how wide is the time window before "no match"
  becomes a false omission? Start with a generous ±10 min and a same-target
  string match; tighten only if false positives appear.
- **Does reconcile run autonomously?** It could be a self-review Phase-B.5
  playbook (daily, surfaces omissions to docket) or purely on-demand. Start
  on-demand + a self-review hook later — an autonomous integrity check that
  itself runs silently has the same problem one level up.
- **Phantom severity.** An omission (did, didn't say) is the serious ethical
  failure. A phantom (said, didn't do) is usually a dry-run or a reverted
  action — report it, but lower-severity than an omission.
