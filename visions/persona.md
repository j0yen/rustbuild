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

## Components (one bullet per PRD)

- **persona-forbidden-vocab** — extend `PersonaConfig` in `wintermute-brain`
  with `forbidden_terms: Vec<String>`, composed into the system prompt as a
  firm instruction. Ships a `jocelyn` preset (defaults populated, fully
  overridable). Foundation for the Jocelyn deployment.
- **persona-name-ceremony** — extend `wintermute-brain`: on first-ever boot,
  the assistant introduces itself by `self_name` and waits for a voice
  acknowledgment before transitioning to the regular `FirstEver` greeting.
  `introduction_mode: Off | FirstEverBoot | Explicit`. Explicit mode also
  responds to `wm.persona.introduce` bus topic.
- **persona-consent-voice-ack** — extend `answerable`: `answerable digest
  --speak --wait-ack` subscribes to `wm.stt.final` after speaking, waits for
  an affirmative word, records `consent-voice-ack` or `consent-unacknowledged`
  in the ledger. The voice substitute for a signature.
- **persona-work** — a shell target that writes Joe's work-laptop identity:
  `CLAUDE_WORK.md` with professional register, `joeyen-atscale` scope, no
  auto-publish, no voice, no family reach. An install script that drops it
  as `~/.claude/CLAUDE_SELF.md` on the work machine.

## Order

```
persona-forbidden-vocab ──► persona-name-ceremony    (both extend wintermute-brain)

persona-consent-voice-ack                             (extends answerable, independent)

persona-work                                          (shell, fully independent)
```

persona-forbidden-vocab is the foundation for name-ceremony because the
ceremony speaks using the composed persona (which includes the forbidden list).
The other two are independent — answerable-consent-voice-ack extends a different
repo; persona-work is a shell/config target with no Rust.

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
