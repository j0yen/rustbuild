# Vision: concord — common ground across divides

## TL;DR

World peace is not a program you can `cargo install`, and any PRD that claimed
to "build peace" would be fiction — a violation of this skill's rule that every
component cite real evidence. But conflict at every scale — a comment thread, a
family, a parliament, a border — shares one tractable, *mechanical* driver: the
**breakdown of understanding**. People argue against strawmen, miss the values
underneath each other's positions, mistake misunderstanding for disagreement,
and escalate language until the substance is unreachable. `concord` is a toolkit
that attacks exactly that failure mode — and nothing grander. It gathers
perspective-diverse sources on a contested question, builds the strongest
good-faith version of each side, separates the real cruxes from the
misunderstandings, surfaces the genuine common ground, and helps individuals
speak across the divide without escalating. A humble, buildable sliver of the
north star, made entirely from primitives already on this laptop.

## End-state

When `concord` is done, a person facing a polarized question can run one command
and get back: (1) the strongest honest case for each side, sourced and cited;
(2) a map of where the disagreement is *actually* located — which parts are
empirical, which are value differences, which are just two groups talking past
each other; (3) the concrete shared values both sides already hold; and (4) for
their own heated message, a rephrase that preserves every substantive ask while
stripping the contempt that makes the other side stop listening. It runs offline
on the local LLM ladder, so it works without sending anyone's argument to a
cloud provider.

## Why this is buildable here (evidence)

- **Local LLM ladder** is live: `ollama list` shows `qwen3:8b`, `qwen2.5:3b`,
  `llama3.2:3b` (verified 2026-06-05). The steelman, crux-classification, and
  de-escalation engines run offline on these — no cloud, no leak of anyone's
  words. Per `reference_local_llm_setup`, `qwen2.5:3b` is the practical fast tier
  on this CPU-only box.
- **Deep-research fan-out harness** (`/deep-research` skill) already does
  multi-source web search → fetch → adversarial verification → cited synthesis.
  `concord-corpus` reuses that shape for perspective-diverse gathering rather
  than reinventing fetch/search.
- **`wintermute-brain`** (`~/wintermute/wintermute-brain`) ships the
  local→cloud tier ladder + LLM routing abstraction `concord` calls instead of
  hand-rolling an ollama client.
- **agorabus** gives the pipeline a bus surface (`wm.concord.*`) so a future UI
  or other tools can drive it — same pattern the voice stack uses.
- This is the **second outward-facing vision** after `homeward` (lost pets);
  recall confirms no prior conflict/dialogue work exists on this laptop, so the
  ground is clear (the only "conflict" memories are git merge conflicts).

## Components (PRD-sized, dependency order)

1. **concord-corpus** (new repo `~/wintermute/concord`, rust-cli + lib) —
   perspective-diverse source gathering. Given a contested claim, gather sources,
   tag each by stance, dedup near-identical framings, score credibility → a
   structured `Corpus`. Foundation crate + the `concord` binary that hosts the
   workspace.
2. **concord-steelman** (rust-extend → concord) — for each stance in a Corpus,
   the strongest good-faith argument a proponent would endorse, via the local
   LLM. Refuses to caricature.
3. **concord-cruxes** (rust-extend → concord) — from the steelmanned positions,
   separate shared values, real cruxes (where reasonable people diverge), and
   misunderstandings (apparent conflicts that dissolve on clarification). A
   structured disagreement map.
4. **concord-bridge** (rust-extend → concord) — synthesize a balanced, cited
   brief: what each side wants, where the real disagreement is, the common
   ground, what would change minds. Passes a balance check.
5. **concord-deescalate** (rust-extend → concord) — rephrase a heated message
   into a non-inflammatory, perspective-taking form (observation/feeling/need/
   request) preserving every substantive ask. Useful standalone.

## Order

`corpus → steelman → cruxes → bridge`, a straight dependency chain (each consumes
the prior's structured output). `deescalate` is independent — it needs only the
local LLM, so /build can ship it in parallel with the corpus chain.

## Design note for buildability (important)

`/build` now runs cargo on the **cloud box**, which has **no ollama**. So every
LLM-touching crate must inject its model client behind a trait and test the
deterministic logic against a **mock LLM + golden fixtures** (corpus structure,
dedup, citation integrity, contempt-lexicon stripping, crux-vs-misunderstanding
classification on a labelled set). Live-LLM behavior is a manually-verified /
deferred AC, never a cloud-gated test. This keeps the fleet buildable under the
cloud-routed pipeline and avoids the tautology trap (the mock tests the wiring,
the golden fixtures are an independent held-out set, not agent-written-to-pass).

## Open questions (next /dream pass — not yet drafted)

- **concord-serve** — expose the pipeline over agorabus (`wm.concord.*`) + a tiny
  HTTP endpoint. Needs a consumer design before it's real.
- **concord-web** — a browser UI for non-CLI users. Outward-facing means
  non-technical users; needs the homeward-style hosting decision first.
- Evaluation: is there a public labelled dataset of "crux vs misunderstanding"
  to validate concord-cruxes against, or do we hand-build the golden set?
- Ethics/abuse: a steelman engine can also steelman bad-faith positions. Should
  there be a refusal boundary, and where?
