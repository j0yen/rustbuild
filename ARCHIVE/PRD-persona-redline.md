# PRD: persona-redline — guarantee the forbidden word never reaches her ears

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/wintermute-brain
Vision: visions/persona.md

## TL;DR

`persona-forbidden-vocab` (shipped, `wintermute-brain` v0.20.0) composes a
`forbidden_terms` list into the **system prompt** as an instruction: *"never
say AI, computer, algorithm…"*. That is advice to the model, not a guarantee.
A language model — especially the **local-3b default tier** this box runs
(`qwen2.5:3b` via the brain tier ladder) — routinely emits a word it was told
to avoid. For Jocelyn, an artist who actively dislikes technology, a single
leaked "computer" in a spoken reply breaks the companion illusion the entire
persona vision exists to protect. `persona-redline` is the **output-side
enforcement** that sits between the brain's generated reply and the TTS path:
it scans the final reply text for any forbidden term, and on a hit either
**regenerates once** with a hardened instruction or **falls back to a safe
fixed phrase**, so the forbidden word is never published to `wm-tts`. The
advisory prompt becomes a runtime guarantee.

## Why this exists (Phase 1 evidence, 2026-06-13)

Measured live this session against the brain source and config:

- **`forbidden_terms` is prompt-only — there is no output check.**
  `wintermute-brain/src/lib.rs:237-238` is the *only* non-test use of the
  field: it joins the terms into a system-prompt instruction
  (`self.forbidden_terms.join(", ")`). A grep of `src/daemon.rs` for the
  reply/TTS publish path (`speak_payload`, `wm.brain.reply…`) shows the
  generated reply is published with **no scan against `forbidden_terms`**.
  Nothing enforces the instruction on the actual generated text.
- **The default tier is the most likely to leak.** Memory
  `[[project_brain_local_first_ladder]]`: the brain ladder defaults to
  `local-3b` (`qwen2.5:3b`) and skips `local-8b` on this CPU-only box. A 3B
  model honors a negative vocabulary constraint far less reliably than a
  cloud model — exactly the tier Jocelyn's device would run for latency/cost.
- **The principal cannot self-correct the slip.** Unlike a developer who
  reads "AI" and shrugs, Jocelyn hears it spoken and the friend-illusion
  collapses. The vision's end-state #1 is explicit: *"never 'I'm an AI', never
  'the computer says', never 'algorithm.'"* Prompt advice cannot promise that.
- **No existing layer covers spoken output.** `answerable-redline` (shipped)
  is a declarative policy over `CLAUDE_SELF.md` edits; `values-drift` watches
  the self-file. Neither inspects a runtime brain reply before it is spoken.

## What this builds

A reply-path filter inside `wintermute-brain`, behind a persona setting so
existing non-Jocelyn deployments are byte-for-byte unchanged.

- **New module `src/redline.rs`.**
  - `fn scan(reply: &str, forbidden: &[String]) -> Vec<Hit>` — case-insensitive,
    Unicode-word-boundary matching (so "AI" matches "AI" / "ai" but **not**
    "rain" or "said"). Multi-word terms ("neural network") matched as phrases.
    Returns each `Hit { term, byte_range }`.
  - `enum RedlineAction { Off, Regenerate { max_attempts: u8 }, SafePhrase(String) }`
    — added to `PersonaConfig` as `redline: RedlineAction` (serde default
    `Off`, so empty `[persona]` tables are unaffected — AC7).
- **Reply-path integration in `src/daemon.rs`.** After the model returns a
  reply and **before** it is published to the TTS/dialog reply topic, call
  `redline::scan`. On a non-empty hit set:
  - `Regenerate`: re-issue the same prompt with an appended hardened directive
    naming the exact leaked term(s); if still dirty after `max_attempts`, fall
    through to `SafePhrase` (or a built-in default if none set). Never publish
    a dirty reply.
  - `SafePhrase`: publish the configured fallback verbatim (e.g. "Let me put
    that a different way — everything's fine.").
- **Observability.** Each enforced leak publishes a `wm.persona.redline`
  event `{term, action, attempts}` (reusing the existing agorabus publish
  helper) and increments an in-process counter exposed in the daemon's status
  snapshot — so a leak rate is measurable, not silent.
- **`jocelyn` defaults.** When the `jocelyn` forbidden preset is active,
  `redline` defaults to `Regenerate { max_attempts: 1 }` with the SafePhrase
  fallback, matching the preset's intent.

Deps: none beyond the crate's existing set (serde, the agorabus client,
tokio). MSRV 1.85, no let-chains. `sigpipe::reset()` is already established
crate-wide for any CLI surface.

## Acceptance criteria

1. `redline::scan("the computer is fine", &["computer".into()])` returns one
   hit with the correct byte range; `scan("it will rain", &["AI".into()])`
   returns **zero** hits (word-boundary, no substring false-positive).
2. Case-insensitive: `scan("My A.I. brain", &["ai".into()])` matches the "AI"
   token but a bare "said" never matches "ai".
3. Multi-word term "neural network" matches as a phrase and not when only one
   word is present.
4. `RedlineAction::Off` (the default) is a no-op: a reply containing a
   forbidden term is published unchanged, byte-identical to pre-PRD behavior.
5. With `Regenerate { max_attempts: 1 }`, a stubbed model that returns a dirty
   reply then a clean reply causes the **clean** reply to be published; the
   dirty one never reaches the TTS topic (assert on the captured publish).
6. With `Regenerate` whose retries all stay dirty, the configured `SafePhrase`
   (or the built-in default) is published instead — a dirty reply is **never**
   published under any path.
7. An existing `brain.toml` with a `[persona]` table that omits `redline`
   deserializes to `RedlineAction::Off`; the full test suite that asserts
   `Plain`/`WarmElder` compose output is unchanged.
8. Every enforced leak publishes exactly one `wm.persona.redline` event with
   the leaked term and the action taken, and the daemon status snapshot
   reports a monotonic leak counter.
9. `cargo test` green; `cargo build` clean; no new clippy `-D warnings`
   regressions beyond the documented recall/brain baseline.
