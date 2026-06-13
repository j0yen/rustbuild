# Vision: persona — a companion can be anyone her principal needs

**Authored by:** /dream (Claude Sonnet 4.6), with jsy
**Created:** 2026-06-12
**Status:** active
**Seed:** user — "please remember my mother as the primary user in a secondary
sister project... We will need to create a sub-identity of yours for the elderly.
And I will need another for my work laptop which will need its own persona."
**Phase-1 evidence date:** 2026-06-12

---

## TL;DR

The companion/hearth/kin fleet shipped a voice assistant for "a non-technical elder."
But Jocelyn is not just non-technical — she's an artist who actively dislikes
technology. The gap is not tone (WarmElder register exists and is live), it is
**framing**: nothing prevents the assistant from saying "AI", "algorithm", "device",
or "the computer" in a sentence. For Jocelyn those words are a wall. A companion that
describes itself as technology to a technophobe is a failed companion.

This vision ships the identity layer that sits *above* the tone layer:

1. **What the assistant may never say** — a structured forbidden-vocabulary list,
   composed into the system prompt, so the assistant speaks as a warm friend rather
   than a piece of software. Jocelyn ships with a default preset; the field is
   open for any deployment.
2. **How the assistant introduces itself** — Jocelyn didn't buy this device; Joe
   did. She meets the assistant cold. The name ceremony is the moment she learns
   what to call it and why — a spoken introduction, not a setup screen.
3. **How the non-reading principal consents** — answerable's consent-digest speaks
   a plain-language summary. But speaking is not consenting. This vision adds a
   voice-acknowledgment gate: the digest waits for "okay" or "yes" before the
   ledger records consent as *received*, not merely *delivered*.
4. **The work-laptop persona** — Joe's AtScale machine is not wintermute. It needs
   its own CLAUDE_SELF.md: professional register, narrower autonomy, `joeyen-atscale`
   scope, no voice features, no family reach.

## Why this is distinct from hearth

hearth.md — the shipped personality vision — addressed the *register* question:
short sentences, warm tone, addresses the user by name, no jargon in the *manner*
of speaking. It didn't address the *vocabulary* question: whether a word like
"algorithm" or "artificial intelligence" appears in a sentence at all. hearth
ships WarmElder phrasing; persona ships what words are off-limits. They compose.

## End-state

When persona is fulfilled:

1. Jocelyn's device never says a word that signals "technology" to her. The
   assistant speaks as a warm, unhurried friend — never "I'm an AI", never
   "the computer says", never "algorithm." She can forget what is underneath.
2. The first time Jocelyn speaks to the device, it introduces itself by the
   name Joe chose — not "wintermute," not "assistant" — and waits for her to
   acknowledge before going further.
3. When the device tells Jocelyn what it has been doing (the consent digest),
   it waits for her spoken "okay" before the ledger records the conversation
   as complete. Consent for a non-reading principal is auditory and confirmed.
4. Joe's work Claude has its own identity: professional, scoped to AtScale
   work, never auto-publishing to personal repos, never activating voice
   features or family connections. Two boxes, two selves, no bleed.
5. The personas are not merely *buildable* — they are *live and proven*. The
   Jocelyn identity is assembled into the running `brain.toml` with a warm name
   Joe chose, redline enforcement actually active, and an independent held-out
   number showing the running model does not leak technology vocabulary to her.
   A deployed persona that nobody has activated or measured is a drawer of parts,
   not a companion.

## Components (one bullet per PRD)

**Shipped (verified live 2026-06-13):**

- **persona-forbidden-vocab** — ✅ SHIPPED (`wintermute-brain` v0.20.0).
  `forbidden_terms: Vec<String>` on `PersonaConfig`, composed into the system
  prompt (`src/lib.rs:237`). Ships a `jocelyn` preset.
- **persona-name-ceremony** — ✅ SHIPPED (`src/introduction.rs`).
  `IntroductionMode::{Off,FirstEverBoot,Explicit}`, `compose_introduction_text`,
  ack timeout; Explicit responds to `wm.persona.introduce` (wired in
  `src/daemon.rs`).
- **persona-consent-voice-ack** — ✅ SHIPPED (`answerable` v0.5.0).
  `digest --speak --wait-ack`, records `consent-voice-ack` /
  `consent-unacknowledged`.

**Drafted 2026-06-13 (this dream pass):**

- **persona-redline** — extend `wintermute-brain`: output-side enforcement of
  `forbidden_terms`. Today the list is *prompt-only advice* (`src/lib.rs:237`);
  nothing scans the generated reply before TTS, and the default tier is
  `local-3b` (most likely to leak). `src/redline.rs` scans the reply, and on a
  hit either regenerates once or substitutes a safe phrase — the advisory
  prompt becomes a runtime guarantee.
- **persona-profile** — extend `wintermute-brain`: a named profile registry +
  `wm-brain persona {list,show,diff,apply}`. Today persona is scattered knobs in
  `brain.toml`; the live config has no `forbidden_terms` and no intro mode at
  all. One named declaration (`jocelyn`, `default`) materializes a complete,
  consistent `[persona]` block — `persona apply jocelyn` instead of a dozen
  manual TOML edits.
- **persona-work** — a shell target that writes Joe's work-laptop identity:
  `CLAUDE_WORK.md` with professional register, `joeyen-atscale` scope, no
  auto-publish, no voice, no family reach. An idempotent, reversible install
  script that drops it as `~/.claude/CLAUDE_SELF.md` on the work machine.
  (The last of the original four components; still entirely unbuilt.)

## Order

```
[shipped] forbidden-vocab ──► persona-redline      (redline enforces the shipped list)
                          └─► persona-profile       (profile composes the shipped fields)
[shipped] name-ceremony ─────► persona-profile      (profile binds the shipped intro mode)

persona-work                                        (shell, fully independent)
```

persona-redline and persona-profile both extend `wintermute-brain` and both
build on the shipped forbidden-vocab/name-ceremony surfaces; they are
independent of each other (profile's `redline` field is optional/defaulted so it
builds with or without redline). persona-work is a shell/config target with no
Rust and no dependency on the others.

## Open questions

1. **What name does Jocelyn call the assistant?** Joe must decide. The PRDs
   ship the *mechanism* (self_name config field + name ceremony); the actual
   name is a deployment choice. Something warm, human-register, not a brand.
   "Clara," "Rose," "Nora," "Wren" — Joe's call, not the agent's.

2. **Whose consent cadence?** The voice-ack digest is pull (Jocelyn asks,
   or the daily digest fires, and she says "okay"). Should it fire daily at
   a configured time, or only when explicitly requested? Probably a configurable
   schedule — `WM_FAMILY_DIGEST_TIME` already exists in the family-enroll
   config; the voice-ack PRD should read from it.

3. **Work-machine detection.** The work-laptop persona ships as a file Joe
   copies manually, or as a `chezmoi` template (if constellation-appearance
   ships first and the work machine is in the constellation). For now: manual
   copy + install script; chezmoi integration is a future extension.

4. **Persona bleed prevention.** Joe uses both machines. If he asks the
   wintermute laptop about AtScale work, what identity does it use? The current
   answer is: wintermute's identity — there's no context switching on the
   personal machine. That's probably correct; leave it unaddressed unless Joe
   raises it.

## Deployment — the second frontier (drafted 2026-06-13)

The mechanism layer is complete and *verified live this session*: every
component above is shipped, and `redline::enforce()` is genuinely wired into
the daemon reply→TTS path (`wintermute-brain/src/daemon.rs:2277`), not merely
present. But the running deployment has none of it. Checked 2026-06-13 on the
live box:

- `~/.config/wintermute/brain.toml` `[persona]` still reads
  `self_name = "wintermute"`, `register = "warm-elder"`, **no
  `forbidden_terms`, no `[persona.introduction]`, no `redline`**. The
  `wmd persona profile apply jocelyn --write` mechanism (which would write the
  whole jocelyn block, backing up to `brain.toml.bak`) has never been run.
- `redline` therefore defaults to `RedlineAction::Off` — the wired guarantee is
  dormant. The forbidden-vocab list reaches the model only as prompt advice,
  exactly the gap `persona-redline` was built to close, because nothing
  activated it.
- No independent number exists for leak rate. The `redline.rs` tests are
  author-written exact-match unit cases (`scan_exact_match_returns_hit`, …) —
  see [[feedback_agent_written_fixtures_tautology]]. Nobody has driven the
  *running local-3b model* on Jocelyn-style prompts and counted leaks.

This frontier turns the parts into a companion. Four PRDs:

- **persona-deploy-jocelyn** (shell) — idempotent installer that assembles the
  live elder deployment: applies the jocelyn profile, sets the warm `self_name`
  Joe chose (the preset placeholder `"jocelyn"` names the assistant after the
  *principal* — wrong; the installer parameterizes the real name), flips
  `redline` from `Off` to active `SafePhrase`, restarts wm-brain, and verifies
  `persona profile diff jocelyn` reconciles. Reversible via `brain.toml.bak`.
- **persona-redline-eval** (mixed) — an independent held-out corpus of
  naturalistic technophobe-trigger prompts (NOT drawn from `redline.rs` tests),
  driven through the live model, reporting a real pre/post leak rate. Honest
  skip-with-message when no model is present. Closes [[feedback_verify_before_concluding]].
- **persona-redline-regenerate** (rust-extend `wintermute-brain`) — adds the
  `Regenerate` variant that `redline.rs:39` documents as deferred "future
  work": re-issue the model request with a hardened addendum naming the leaked
  term, falling back to `SafePhrase` only if regeneration also leaks. Makes the
  guarantee graceful instead of a jarring canned phrase.
- **persona-deploy-doctor** (shell) — periodic drift check: live `[persona]`
  still matches the jocelyn profile, redline still active, self_name still the
  chosen warm name. Surfaces persona drift the way self-review surfaces fleet
  staleness; ties to [[freshness]].

Order: `persona-deploy-jocelyn` is the foundation (activating redline is what
makes the eval meaningful). `persona-redline-eval` and `persona-deploy-doctor`
both depend on the deployment existing. `persona-redline-regenerate` is an
independent `wintermute-brain` extend; the eval should re-run against it once
shipped. Both brain-extends (regenerate, and the earlier redline/profile) must
serialize / worktree-isolate within a /build tick — shared build target.

Open: the assistant's actual name remains Joe's call (Open question #1 above);
`persona-deploy-jocelyn` ships the mechanism and a documented placeholder, not
the decision.
