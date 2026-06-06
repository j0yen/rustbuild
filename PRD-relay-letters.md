# PRD: relay-letters

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/relay
Vision: visions/relay.md

## TL;DR

The last mile of helping is paperwork: the referral letter, the benefits-appeal
letter, the intake summary for another agency. Helpers do this by hand, from
scratch, over and over. `relay-letters` drafts these from a `relay-intake`
`CaseRecord` + a template — a first draft a human edits and sends. It **never
auto-sends** and **never gives legal or medical advice**; it fills in the
boilerplate a person's situation implies, so the helper spends their judgment on
the substance, not the formatting.

## Why this exists

- **Evidence:** the `relay` arc ends here — `directory` finds help, `match`
  ranks it, `intake` structures the case; the case still has to become a document
  someone acts on. Letter-writing is explicitly named as the dreaded final
  overhead (visions/relay.md, end-state).
- **Evidence — privacy + local LLM:** these letters contain the same sensitive
  facts as intake; drafting them on-device (qwen, installed) is the only safe
  path (visions/relay.md design note).
- **Safety boundary (from the vision's open question):** a tool that helps
  vulnerable people must not impersonate a lawyer or clinician. `relay-letters`
  draws the line at *navigation and boilerplate*, with required human review.

## What this builds

- New crate `relay-letters` in the workspace, wired as `relay letter`.
- **Template engine (deterministic core):** typed templates (referral,
  benefits-appeal, intake-summary, support-letter) with named slots filled from
  a `CaseRecord` — pure, testable, no LLM. Missing required slots produce a
  listed `[[NEEDS: ...]]` placeholder, never a fabricated value.
- **`Prose` trait** (`LocalLlmProse` + `MockProse`): optionally smooths the
  filled template into fluent prose via the local model, constrained to *not*
  introduce new facts (a post-check diffs entities in vs. out and flags
  additions). Tests use `MockProse`.
- **Guardrails (deterministic):** every output carries a `DRAFT — review before
  sending` header and a "not legal/medical advice" footer; an advice-phrase
  linter flags imperative legal/medical claims ("you should sue", "you have a
  case") for human attention. No send capability exists in the crate at all.
- **CLI**: `relay letter --type referral --case <case.json> [--smooth]
  [--out letter.md]`.

## Acceptance criteria

1. `relay letter --help` listed under `relay`; crate builds in the workspace.
2. Template fill is deterministic: a fixture `CaseRecord` + `--type referral`
   produces a known letter (golden test), with no LLM.
3. A missing required slot yields a visible `[[NEEDS: <slot>]]` placeholder —
   never a fabricated or blank-but-plausible value (tested).
4. Every generated letter contains the `DRAFT — review before sending` header and
   the not-advice footer (tested on all template types).
5. The advice-phrase linter flags a seeded "you should sue them" line and does
   not flag neutral navigation text (tested fixtures).
6. The crate exposes NO network/send function; a test asserts no outbound
   connection is made during generation under `MockProse`.
7. `--smooth` with `MockProse` preserves all `CaseRecord` entities; the
   fact-addition check flags an injected new entity (tested with a MockProse that
   deliberately adds one).
8. **Deferred / manually-verified AC:** prose quality + faithfulness vs. real
   qwen on the four template types — hand-verified, not a cloud test
   (`deferred_acs`).

ACs 1–7 deterministic / cloud-build-safe; AC8 is the only live-model check,
deferred.
